// Padma v0.1.0 — a small, dependency-free Bangla-English language MVP.
//
// This executable intentionally implements a narrow but complete vertical slice:
// UTF-8 source, Bangla/English keyword aliases, expressions, variables, print,
// conditionals, string interpolation, and localized diagnostics.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::env;
use std::fmt;
use std::fs;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(test)]
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::Value as JsonValue;

static RANDOM_COUNTER: AtomicU64 = AtomicU64::new(0);
#[cfg(test)]
static AI_WORKFLOW_CURL_TEST_PROGRAM: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();
#[cfg(test)]
static AI_WORKFLOW_CURL_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
const BRIDGE_MAX_BYTES: usize = 256 * 1024;
const BRIDGE_TIMEOUT: Duration = Duration::from_secs(10);
const SQLITE_MAX_BYTES: usize = 256 * 1024;
const SQLITE_TIMEOUT: Duration = Duration::from_secs(5);
const AI_WORKFLOW_MAX_JSON_DEPTH: usize = 16;
const AI_WORKFLOW_MAX_STDERR_BYTES: usize = 4 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Locale {
    Bangla,
    English,
}

impl Locale {
    fn from_source(source: &str) -> Self {
        if source.contains("padma:locale=en") {
            return Self::English;
        }
        if source.contains("padma:locale=bn") {
            return Self::Bangla;
        }

        let bangla = [
            "ধরি",
            "দেখাও",
            "যদি",
            "নইলে",
            "যতক্ষণ",
            "প্রতি",
            "মধ্যে",
            "ফাংশন",
            "ফেরত",
            "ইমপোর্ট",
            "সত্য",
            "মিথ্যা",
        ]
        .iter()
        .map(|word| source.matches(word).count())
        .sum::<usize>();
        let english = [
            "let", "print", "if", "else", "while", "for", "in", "function", "return", "import",
            "true", "false",
        ]
        .iter()
        .map(|word| source.matches(word).count())
        .sum::<usize>();

        if english > bangla {
            Self::English
        } else {
            Self::Bangla
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Position {
    line: usize,
    column: usize,
}

impl Position {
    const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Clone, Debug)]
struct PadmaError {
    code: &'static str,
    message: String,
    hint: Option<String>,
    position: Position,
    locale: Locale,
    source_path: Option<PathBuf>,
    source_text: Option<String>,
}

impl PadmaError {
    fn new(
        locale: Locale,
        code: &'static str,
        message: impl Into<String>,
        hint: Option<String>,
        position: Position,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            hint,
            position,
            locale,
            source_path: None,
            source_text: None,
        }
    }

    fn with_source_context(mut self, path: PathBuf, source: String, locale: Locale) -> Self {
        self.locale = locale;
        self.source_path = Some(path);
        self.source_text = Some(source);
        self
    }
}

fn error_for(locale: Locale, code: &'static str, position: Position, detail: &str) -> PadmaError {
    let (message, hint) = match (locale, code) {
        (Locale::Bangla, "P1001") => (
            format!("অচেনা চিহ্ন `{detail}`"),
            Some("শুধু অনুমোদিত Padma চিহ্ন ব্যবহার করুন।".into()),
        ),
        (Locale::English, "P1001") => (
            format!("Unexpected character `{detail}`"),
            Some("Use only supported Padma characters.".into()),
        ),
        (Locale::Bangla, "P1002") => (
            "string শেষ হওয়ার আগে file শেষ হয়ে গেছে".into(),
            Some("string-এর শেষে একটি closing quote (\") যোগ করুন।".into()),
        ),
        (Locale::English, "P1002") => (
            "Reached the end of the file before the string was closed".into(),
            Some("Add a closing double quote (\").".into()),
        ),
        (Locale::Bangla, "P1003") => (
            format!("এখানে `{detail}` প্রত্যাশিত ছিল"),
            Some("আগের statement ও বন্ধনীগুলো পরীক্ষা করুন।".into()),
        ),
        (Locale::English, "P1003") => (
            format!("Expected `{detail}` here"),
            Some("Check the preceding statement and delimiters.".into()),
        ),
        (Locale::Bangla, "P1004") => (
            format!("এই statement শুরু করতে `{detail}` ব্যবহার করা যাবে না"),
            Some("`ধরি`, `দেখাও`, অথবা `যদি` দিয়ে নতুন statement শুরু করুন।".into()),
        ),
        (Locale::English, "P1004") => (
            format!("`{detail}` cannot start a statement"),
            Some("Start a statement with `let`, `print`, or `if`.".into()),
        ),
        (Locale::Bangla, "P1007") => (
            format!("`{detail}` নামে কোনো variable পাওয়া যায়নি"),
            Some(format!("আগে এটি ঘোষণা করুন: `ধরি {detail} = ...`")),
        ),
        (Locale::English, "P1007") => (
            format!("Cannot find variable `{detail}`"),
            Some(format!("Declare it first: `let {detail} = ...`")),
        ),
        (Locale::Bangla, "P1010") => (
            format!("`{detail}` অপারেশনের জন্য মানগুলোর ধরন মিলছে না"),
            Some("সংখ্যার সঙ্গে সংখ্যা বা লেখার সঙ্গে লেখা ব্যবহার করুন।".into()),
        ),
        (Locale::English, "P1010") => (
            format!("Values do not have compatible types for `{detail}`"),
            Some("Use numbers with numbers or text with text.".into()),
        ),
        (Locale::Bangla, "P1011") => (
            "শূন্য দিয়ে ভাগ করা যাবে না".into(),
            Some("ভাজকটি শূন্য কি না আগে `যদি` দিয়ে পরীক্ষা করুন।".into()),
        ),
        (Locale::English, "P1011") => (
            "Cannot divide by zero".into(),
            Some("Use `if` to check that the divisor is not zero first.".into()),
        ),
        (Locale::Bangla, "P1012") => (
            "loop নির্ধারিত iteration সীমা অতিক্রম করেছে".into(),
            Some("condition ও loop-এর variable update পরীক্ষা করুন।".into()),
        ),
        (Locale::English, "P1012") => (
            "The loop exceeded the configured iteration limit".into(),
            Some("Check the condition and update the loop variable.".into()),
        ),
        (Locale::Bangla, "P1014") => (
            format!("নিরাপত্তার কারণে output path অনুমোদিত নয়: `{detail}`"),
            Some("বর্তমান folder-এর ভেতরের relative path ব্যবহার করুন; `..` বা absolute path ব্যবহার করবেন না।".into()),
        ),
        (Locale::English, "P1014") => (
            format!("Output path is not allowed for safety: `{detail}`"),
            Some("Use a relative path inside the current folder; do not use `..` or an absolute path.".into()),
        ),
        (Locale::Bangla, "P1015") => (format!("file লেখা যায়নি: `{detail}`"), Some("folder ও permission পরীক্ষা করুন।".into())),
        (Locale::English, "P1015") => (format!("Could not write file: `{detail}`"), Some("Check the folder and permissions.".into())),
        (Locale::Bangla, "P1016") => (format!("URL অনুমোদিত নয়: `{detail}`"), Some("http:// অথবা https:// URL ব্যবহার করুন।".into())),
        (Locale::English, "P1016") => (format!("URL is not allowed: `{detail}`"), Some("Use an http:// or https:// URL.".into())),
        (Locale::Bangla, "P1017") => (format!("এই process অনুমোদিত নয়: `{detail}`"), Some("শুধু অনুমোদিত downloader/tool ব্যবহার করুন।".into())),
        (Locale::English, "P1017") => (format!("This process is not allowed: `{detail}`"), Some("Use only an approved downloader/tool.".into())),
        (Locale::Bangla, "P1018") => (format!("process চালু করা যায়নি: `{detail}`"), Some("Termux-এ toolটি install আছে কি না পরীক্ষা করুন।".into())),
        (Locale::English, "P1018") => (format!("Could not start process: `{detail}`"), Some("Check that the tool is installed in Termux.".into())),
        (Locale::Bangla, "P1019") => (format!("process ব্যর্থ হয়েছে: `{detail}`"), Some("tool-এর output ও URL পরীক্ষা করুন।".into())),
        (Locale::English, "P1019") => (format!("Process failed: `{detail}`"), Some("Check the tool output and URL.".into())),
        (Locale::Bangla, "P1020") => (format!("map-এর key অবশ্যই text হতে হবে: `{detail}`"), Some("উদাহরণ: `map.get(\"নাম\")` ব্যবহার করুন।".into())),
        (Locale::English, "P1020") => (format!("Map keys must be text: `{detail}`"), Some("Use a string key, for example `map.get(\"name\")`.".into())),
        (Locale::Bangla, "P1021") => (format!("map-এ `{detail}` key পাওয়া যায়নি"), Some("আগে `map.set` দিয়ে key-টি যোগ করুন।".into())),
        (Locale::English, "P1021") => (format!("Map key `{detail}` was not found"), Some("Add the key first with `map.set`.".into())),
        (Locale::Bangla, "P1022") => (format!("নিরাপদ module path নয়: `{detail}`"), Some("বর্তমান folder-এর ভেতরের `.pd` file ব্যবহার করুন; `..` বা absolute path ব্যবহার করা যাবে না।".into())),
        (Locale::English, "P1022") => (format!("Unsafe module path: `{detail}`"), Some("Use a `.pd` file inside the current folder; `..` and absolute paths are not allowed.".into())),
        (Locale::Bangla, "P1023") => (format!("module পড়া যায়নি: `{detail}`"), Some("File name ও relative path পরীক্ষা করুন।".into())),
        (Locale::English, "P1023") => (format!("Could not read module: `{detail}`"), Some("Check the file name and relative path.".into())),
        (Locale::Bangla, "P1024") => (format!("circular module import পাওয়া গেছে: `{detail}`"), Some("একটি module-কে নিজের মাধ্যমে আবার import করবেন না।".into())),
        (Locale::English, "P1024") => (format!("Circular module import detected: `{detail}`"), Some("Do not import a module again through itself.".into())),
        (Locale::Bangla, "P1025") => (format!("module-এ error আছে: `{detail}`"), Some("module file-এর code ও syntax পরীক্ষা করুন।".into())),
        (Locale::English, "P1025") => (format!("Module contains an error: `{detail}`"), Some("Check the module code and syntax.".into())),
        (Locale::Bangla, "P1026") => (format!("collection index অবশ্যই শূন্য বা তার বেশি পূর্ণসংখ্যা হতে হবে: `{detail}`"), Some("উদাহরণ: `তালিকা[0]` অথবা `তালিকা.get(0)` ব্যবহার করুন।".into())),
        (Locale::English, "P1026") => (format!("Collection index must be a non-negative whole number: `{detail}`"), Some("Use an index such as `items[0]` or `items.get(0)`.".into())),
        (Locale::Bangla, "P1027") => (format!("collection index সীমার বাইরে: `{detail}`"), Some("তালিকার দৈর্ঘ্যের চেয়ে ছোট index ব্যবহার করুন।".into())),
        (Locale::English, "P1027") => (format!("Collection index is out of bounds: `{detail}`"), Some("Use an index smaller than the list length.".into())),
        (Locale::Bangla, "P1028") => (format!("file পড়া যায়নি: `{detail}`"), Some("File name, folder এবং permission পরীক্ষা করুন।".into())),
        (Locale::English, "P1028") => (format!("Could not read file: `{detail}`"), Some("Check the file name, folder, and permissions.".into())),
        (Locale::Bangla, "P1029") => (format!("JSON মান পড়া বা তৈরি করা যায়নি: `{detail}`"), Some("সঠিক JSON text এবং Padma-এর number, text, list, map, true/false, none value ব্যবহার করুন।".into())),
        (Locale::English, "P1029") => (format!("Could not parse or create JSON: `{detail}`"), Some("Use valid JSON text and Padma number, text, list, map, true/false, or none values.".into())),
        (Locale::Bangla, "P1030") => (format!("সঠিক URL নয়: `{detail}`"), Some("পূর্ণ URL দিন, যেমন `https://example.com/path`।".into())),
        (Locale::English, "P1030") => (format!("Invalid URL: `{detail}`"), Some("Provide a complete URL such as `https://example.com/path`.".into())),
        (Locale::Bangla, "P1031") => (format!("text.format-এর placeholder পাওয়া যায়নি: `{detail}`"), Some("map-এ একই নামের একটি text key যোগ করুন।".into())),
        (Locale::English, "P1031") => (format!("text.format placeholder was not found: `{detail}`"), Some("Add a text key with the same name to the map.".into())),
        (Locale::Bangla, "P1034") => (format!("এই project-এ `{detail}` capability অনুমোদিত নয়"), Some("padma.toml-এর [capabilities] অংশে প্রয়োজনীয় সীমিত permission যোগ করুন।".into())),
        (Locale::English, "P1034") => (format!("This project has not granted the `{detail}` capability"), Some("Add only the required limited permission under [capabilities] in padma.toml.".into())),
        (Locale::Bangla, "P1035") => (format!("এই bridge runtime সমর্থিত নয়: `{detail}`"), Some("শুধু `python` অথবা `javascript` ব্যবহার করুন।".into())),
        (Locale::English, "P1035") => (format!("This bridge runtime is not supported: `{detail}`"), Some("Use only `python` or `javascript`.".into())),
        (Locale::Bangla, "P1036") => (format!("bridge script path নিরাপদ বা সঠিক নয়: `{detail}`"), Some("project folder-এর ভেতরে `.py` বা `.js` relative file ব্যবহার করুন।".into())),
        (Locale::English, "P1036") => (format!("Bridge script path is unsafe or invalid: `{detail}`"), Some("Use a relative `.py` or `.js` file inside the project folder.".into())),
        (Locale::Bangla, "P1037") => ("bridge input বা output অনুমোদিত আকারের চেয়ে বড়".into(), Some("bridge JSON data 256 KiB-এর মধ্যে রাখুন।".into())),
        (Locale::English, "P1037") => ("Bridge input or output exceeds the permitted size".into(), Some("Keep bridge JSON data within 256 KiB.".into())),
        (Locale::Bangla, "P1038") => (format!("bridge process চালু বা সম্পন্ন করা যায়নি: `{detail}`"), Some("runtime install, script code, এবং capability পরীক্ষা করুন।".into())),
        (Locale::English, "P1038") => (format!("Bridge process could not start or complete: `{detail}`"), Some("Check the runtime installation, script code, and capability.".into())),
        (Locale::Bangla, "P1039") => ("bridge process সময়সীমা অতিক্রম করেছে".into(), Some("bridge কাজটি 10 সেকেন্ডের মধ্যে শেষ করুন।".into())),
        (Locale::English, "P1039") => ("Bridge process exceeded its time limit".into(), Some("Make the bridge finish within 10 seconds.".into())),
        (Locale::Bangla, "P1040") => ("bridge output সঠিক JSON data নয়".into(), Some("standard output-এ একটি মাত্র JSON value লিখুন।".into())),
        (Locale::English, "P1040") => ("Bridge output is not valid JSON data".into(), Some("Write exactly one JSON value to standard output.".into())),
        (Locale::Bangla, "P1041") => (format!("SQLite database tool চালানো যায়নি: `{detail}`"), Some("Termux-এ `pkg install sqlite -y` চালিয়ে আবার চেষ্টা করুন।".into())),
        (Locale::English, "P1041") => (format!("Could not run the SQLite database tool: `{detail}`"), Some("Install it in Termux with `pkg install sqlite -y`, then try again.".into())),
        (Locale::Bangla, "P1042") => ("SQLite থেকে সঠিক JSON row পাওয়া যায়নি".into(), Some("database file ও Padma-managed records পরীক্ষা করুন।".into())),
        (Locale::English, "P1042") => ("SQLite did not return valid JSON rows".into(), Some("Check the database file and Padma-managed records.".into())),
        (Locale::Bangla, "P1043") => (format!("SQLite কাজের নিরাপদ সীমা অতিক্রম করেছে: `{detail}`"), Some("request ছোট করুন এবং কাজটি 5 সেকেন্ডের মধ্যে শেষ করুন।".into())),
        (Locale::English, "P1043") => (format!("SQLite work exceeded a safety limit: `{detail}`"), Some("Reduce the request and make it complete within 5 seconds.".into())),
        (Locale::Bangla, "P1045") => (format!("identity data নিরাপদ বা সঠিক নয়: `{detail}`"), Some("password record, subject, অথবা environment secret name পরীক্ষা করুন; plaintext secret source code-এ লিখবেন না।".into())),
        (Locale::English, "P1045") => (format!("Identity data is unsafe or invalid: `{detail}`"), Some("Check the password record, subject, or environment secret name; do not place plaintext secrets in source code.".into())),
        (Locale::Bangla, "P1046") => ("session token সঠিক নয়, পরিবর্তিত হয়েছে, অথবা মেয়াদ শেষ হয়েছে".into(), Some("আবার sign in করে একটি নতুন session নিন।".into())),
        (Locale::English, "P1046") => ("The session token is invalid, modified, or expired".into(), Some("Sign in again to obtain a new session.".into())),
        (Locale::Bangla, "P1047") => ("নিরাপদ random data তৈরি করা যায়নি".into(), Some("Termux OS entropy source পরীক্ষা করে আবার চেষ্টা করুন।".into())),
        (Locale::English, "P1047") => ("Could not create secure random data".into(), Some("Check the Termux OS entropy source and try again.".into())),
        (Locale::Bangla, "P1050") => (format!("AI workflow manifest বা request descriptor নিরাপদ বা সঠিক নয়: `{detail}`"), Some("padma-ai.toml-এ reviewed HTTPS endpoint, environment variable-এর নাম এবং নির্ধারিত সীমা ব্যবহার করুন।".into())),
        (Locale::English, "P1050") => (format!("AI workflow manifest or request descriptor is unsafe or invalid: `{detail}`"), Some("Use a reviewed HTTPS endpoint, an environment-variable name, and the declared limits in padma-ai.toml.".into())),
        (Locale::Bangla, "P1051") => (format!("AI workflow transport ব্যর্থ হয়েছে: `{detail}`"), Some("endpoint, network availability, timeout এবং provider gateway পরীক্ষা করুন; secret value output-এ দেবেন না।".into())),
        (Locale::English, "P1051") => (format!("AI workflow transport failed: `{detail}`"), Some("Check the endpoint, network availability, timeout, and provider gateway; do not place a secret value in output.".into())),
        (Locale::Bangla, "P1052") => (format!("AI workflow response সঠিক বা নিরাপদ নয়: `{detail}`"), Some("provider gateway-কে Padma AI workflow v1 JSON protocol ও response limit মানতে হবে।".into())),
        (Locale::English, "P1052") => (format!("AI workflow response is invalid or unsafe: `{detail}`"), Some("The provider gateway must follow the Padma AI workflow v1 JSON protocol and response limit.".into())),
        (Locale::Bangla, "P1053") => (format!("browser planning manifest নিরাপদ বা সঠিক নয়: `{detail}`"), Some("padma-browser.toml-এ শুধুমাত্র reviewed lowercase HTTPS origin ও navigation-review policy ব্যবহার করুন।".into())),
        (Locale::English, "P1053") => (format!("Browser planning manifest is unsafe or invalid: `{detail}`"), Some("Use only reviewed lowercase HTTPS origins and the navigation-review policy in padma-browser.toml.".into())),
        (Locale::Bangla, "P1054") => (format!("browser navigation descriptor অনুমোদিত policy মানে না: `{detail}`"), Some("প্রতিটি GET URL-কে exact allowlisted HTTPS origin ও simple absolute path হতে হবে।".into())),
        (Locale::English, "P1054") => (format!("Browser navigation descriptor violates the reviewed policy: `{detail}`"), Some("Each GET URL must use an exact allowlisted HTTPS origin and a simple absolute path.".into())),
        (Locale::Bangla, "P1055") => ("browser execution এই Padma version-এ নিষিদ্ধ".into(), Some("শুধু `padma browser inspect` অথবা `padma browser plan` ব্যবহার করুন; কোনো browser চালু হবে না।".into())),
        (Locale::English, "P1055") => ("Browser execution is prohibited in this Padma version".into(), Some("Use only `padma browser inspect` or `padma browser plan`; no browser will be launched.".into())),
        (Locale::Bangla, "P1056") => (format!("AI tool planning manifest নিরাপদ বা সঠিক নয়: `{detail}`"), Some("padma-ai-tools.toml-এ সীমাবদ্ধ tool, declared capability, এবং plan-only policy ব্যবহার করুন।".into())),
        (Locale::English, "P1056") => (format!("AI tool planning manifest is unsafe or invalid: `{detail}`"), Some("Use only supported tools, their declared capabilities, and the plan-only policy in padma-ai-tools.toml.".into())),
        (Locale::Bangla, "P1057") => ("AI tool বা agent execution এই Padma version-এ নিষিদ্ধ".into(), Some("শুধু `padma ai tools inspect` অথবা `padma ai tools plan` ব্যবহার করুন; কোনো tool বা agent চালু হবে না।".into())),
        (Locale::English, "P1057") => ("AI tool or agent execution is prohibited in this Padma version".into(), Some("Use only `padma ai tools inspect` or `padma ai tools plan`; no tool or agent will be started.".into())),
        (Locale::Bangla, "P1058") => (format!("AI training planning manifest নিরাপদ বা সঠিক নয়: `{detail}`"), Some("padma-ai-training.toml-এ local path, resource limit, এবং plan-only policy ব্যবহার করুন।".into())),
        (Locale::English, "P1058") => (format!("AI training planning manifest is unsafe or invalid: `{detail}`"), Some("Use project-relative paths, bounded resource limits, and the plan-only policy in padma-ai-training.toml.".into())),
        (Locale::Bangla, "P1059") => ("AI training execution এই Padma version-এ নিষিদ্ধ".into(), Some("শুধু `padma ai training inspect` অথবা `padma ai training plan` ব্যবহার করুন; dataset পড়া বা training চালু হবে না।".into())),
        (Locale::English, "P1059") => ("AI training execution is prohibited in this Padma version".into(), Some("Use only `padma ai training inspect` or `padma ai training plan`; no dataset will be read and no training will be started.".into())),
        (Locale::Bangla, "P1060") => (format!("browser confirmation-session manifest নিরাপদ বা সঠিক নয়: `{detail}`"), Some("reviewed browser plan digest, navigation index, এবং short-lived local confirmation policy ব্যবহার করুন।".into())),
        (Locale::English, "P1060") => (format!("Browser confirmation-session manifest is unsafe or invalid: `{detail}`"), Some("Use a reviewed browser-plan digest, navigation index, and short-lived local confirmation policy.".into())),
        (Locale::Bangla, "P1061") => ("browser confirmation বা navigation action এই Padma version-এ নিষিদ্ধ".into(), Some("শুধু `padma browser confirm inspect` অথবা `padma browser confirm plan` ব্যবহার করুন; browser, DNS, network, cookie, বা credential ব্যবহার হবে না।".into())),
        (Locale::English, "P1061") => ("Browser confirmation or navigation action is prohibited in this Padma version".into(), Some("Use only `padma browser confirm inspect` or `padma browser confirm plan`; no browser, DNS, network, cookie, or credential will be used.".into())),
        (Locale::Bangla, "P1062") => ("Android browser handoff অনুমোদিত হয়নি বা বাতিল করা হয়েছে".into(), Some("reviewed URL দেখে foreground terminal-এ `OPEN` লিখুন; অন্য কোনো input browser খুলবে না।".into())),
        (Locale::English, "P1062") => ("Android Browser Handoff was not authorized or was cancelled".into(), Some("Review the URL and type `OPEN` in the foreground terminal; no other input will open a browser.".into())),
        (Locale::Bangla, "P1063") => ("Android browser handoff চালু করা যায়নি".into(), Some("Termux-এর `termux-open-url` command আছে কি না পরীক্ষা করুন; Padma retry বা fallback browser service ব্যবহার করবে না।".into())),
        (Locale::English, "P1063") => ("Could not start Android Browser Handoff".into(), Some("Check that Termux provides `termux-open-url`; Padma will not retry or use a fallback browser service.".into())),
        (Locale::Bangla, "P1064") => ("Android browser handoff audit নিরাপদে লেখা যায়নি".into(), Some("project-এর ভেতরে `audit/`-এর অধীনে একটি regular `.jsonl` path ও bounded audit policy ব্যবহার করুন; Padma raw URL বা browser data লিখবে না।".into())),
        (Locale::English, "P1064") => ("Could not safely write the Android Browser Handoff audit".into(), Some("Use a regular bounded `.jsonl` path below `audit/` inside the project; Padma will not write raw URLs or browser data.".into())),
        (Locale::Bangla, "P1065") => (format!("browser interaction draft manifest নিরাপদ বা সঠিক নয়: `{detail}`"), Some("শুধু reviewed plan digest, bounded draft text, এবং project-relative attachment metadata ব্যবহার করুন; Padma attachment পড়বে বা upload করবে না।".into())),
        (Locale::English, "P1065") => (format!("Browser interaction draft manifest is unsafe or invalid: `{detail}`"), Some("Use only a reviewed plan digest, bounded draft text, and project-relative attachment metadata; Padma will not read or upload an attachment.".into())),
        (Locale::Bangla, "P1066") => ("browser interaction draft execute করা নিষিদ্ধ".into(), Some("`padma browser draft inspect` অথবা `padma browser draft plan` ব্যবহার করুন; login, CAPTCHA, form, post, upload, account, payment, এবং browser control আপনার visible browser-এ আপনার হাতে থাকবে।".into())),
        (Locale::English, "P1066") => ("Browser interaction draft execution is prohibited".into(), Some("Use `padma browser draft inspect` or `padma browser draft plan`; login, CAPTCHA, forms, posts, uploads, accounts, payments, and browser control remain in your visible browser and under your control.".into())),
        (Locale::Bangla, "P1067") => (format!("browser user-takeover manifest নিরাপদ বা সঠিক নয়: `{detail}`"), Some("শুধু reviewed plan digest, navigation index, fixed sensitive-action label, এবং short local review policy ব্যবহার করুন; Padma browser state বা user decision পড়বে না।".into())),
        (Locale::English, "P1067") => (format!("Browser user-takeover manifest is unsafe or invalid: `{detail}`"), Some("Use only a reviewed plan digest, navigation index, fixed sensitive-action label, and short local review policy; Padma will not read browser state or a user decision.".into())),
        (Locale::Bangla, "P1068") => ("browser user-takeover execute করা নিষিদ্ধ".into(), Some("শুধু `padma browser takeover inspect` অথবা `padma browser takeover plan` ব্যবহার করুন; visible browser-এ login, CAPTCHA, form, post, upload, account, purchase, এবং payment আপনার হাতে থাকবে।".into())),
        (Locale::English, "P1068") => ("Browser user-takeover execution is prohibited".into(), Some("Use only `padma browser takeover inspect` or `padma browser takeover plan`; login, CAPTCHA, forms, posts, uploads, accounts, purchases, and payments remain under your control in the visible browser.".into())),
        (Locale::Bangla, "P1069") => (format!("structured table data নিরাপদ বা সঠিক নয়: `{detail}`"), Some("CSV/TSV/JSON table-এ bounded header, row, column, ও cell policy ব্যবহার করুন; path project root-এর ভেতরে রাখুন।".into())),
        (Locale::English, "P1069") => (format!("Structured table data is unsafe or invalid: `{detail}`"), Some("Use bounded CSV/TSV/JSON table headers, rows, columns, and cells; keep the path inside the project root.".into())),
        (Locale::Bangla, "P1013") => ("input পড়া যায়নি".into(), Some("আবার চেষ্টা করুন।".into())),
        (Locale::English, "P1013") => ("Could not read input".into(), Some("Try again.".into())),
        _ => (format!("Internal Padma error: {detail}"), None),
    };
    PadmaError::new(locale, code, message, hint, position)
}

#[derive(Clone, Debug, PartialEq)]
enum TokenKind {
    Let,
    Print,
    If,
    Else,
    While,
    For,
    In,
    Function,
    Return,
    Import,
    Export,
    True,
    False,
    Null,
    Identifier(String),
    Number(f64),
    String(String),
    Equal,
    EqualEqual,
    BangEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Plus,
    Minus,
    Star,
    Slash,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Colon,
    LeftBracket,
    RightBracket,
    Newline,
    Eof,
}

#[derive(Clone, Debug)]
struct Token {
    kind: TokenKind,
    position: Position,
}

struct Lexer {
    chars: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
    locale: Locale,
}

impl Lexer {
    fn new(source: &str, locale: Locale) -> Self {
        Self {
            chars: source.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
            locale,
        }
    }

    fn tokenize(mut self) -> Result<Vec<Token>, PadmaError> {
        let mut tokens = Vec::new();
        while !self.is_at_end() {
            let position = self.position();
            let character = self.advance();
            match character {
                ' ' | '\t' | '\r' => {}
                '\n' => tokens.push(self.token(TokenKind::Newline, position)),
                '#' => self.consume_comment(),
                '"' => tokens.push(self.string(position)?),
                '=' => {
                    if self.matches('=') {
                        tokens.push(self.token(TokenKind::EqualEqual, position));
                    } else {
                        tokens.push(self.token(TokenKind::Equal, position));
                    }
                }
                '!' => {
                    if self.matches('=') {
                        tokens.push(self.token(TokenKind::BangEqual, position));
                    } else {
                        return Err(error_for(self.locale, "P1001", position, "!"));
                    }
                }
                '>' => {
                    if self.matches('=') {
                        tokens.push(self.token(TokenKind::GreaterEqual, position));
                    } else {
                        tokens.push(self.token(TokenKind::Greater, position));
                    }
                }
                '<' => {
                    if self.matches('=') {
                        tokens.push(self.token(TokenKind::LessEqual, position));
                    } else {
                        tokens.push(self.token(TokenKind::Less, position));
                    }
                }
                '+' => tokens.push(self.token(TokenKind::Plus, position)),
                '-' => tokens.push(self.token(TokenKind::Minus, position)),
                '*' => tokens.push(self.token(TokenKind::Star, position)),
                '/' => tokens.push(self.token(TokenKind::Slash, position)),
                '(' => tokens.push(self.token(TokenKind::LeftParen, position)),
                ')' => tokens.push(self.token(TokenKind::RightParen, position)),
                '{' => tokens.push(self.token(TokenKind::LeftBrace, position)),
                '}' => tokens.push(self.token(TokenKind::RightBrace, position)),
                ',' => tokens.push(self.token(TokenKind::Comma, position)),
                '.' => tokens.push(self.token(TokenKind::Dot, position)),
                ':' => tokens.push(self.token(TokenKind::Colon, position)),
                '[' => tokens.push(self.token(TokenKind::LeftBracket, position)),
                ']' => tokens.push(self.token(TokenKind::RightBracket, position)),
                ch if is_digit(ch) => tokens.push(self.number(position)?),
                ch if is_identifier_start(ch) => tokens.push(self.identifier(position)),
                ch => return Err(error_for(self.locale, "P1001", position, &ch.to_string())),
            }
        }
        tokens.push(self.token(TokenKind::Eof, self.position()));
        Ok(tokens)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.chars.len()
    }

    fn position(&self) -> Position {
        Position::new(self.line, self.column)
    }

    fn token(&self, kind: TokenKind, position: Position) -> Token {
        Token { kind, position }
    }

    fn advance(&mut self) -> char {
        let value = self.chars[self.current];
        self.current += 1;
        if value == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        value
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.current).copied()
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume_comment(&mut self) {
        while let Some(character) = self.peek() {
            if character == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn string(&mut self, position: Position) -> Result<Token, PadmaError> {
        let mut value = String::new();
        while let Some(character) = self.peek() {
            if character == '"' {
                self.advance();
                return Ok(self.token(TokenKind::String(value), position));
            }
            if character == '\\' {
                self.advance();
                let escaped = self
                    .peek()
                    .ok_or_else(|| error_for(self.locale, "P1002", position, "string"))?;
                self.advance();
                value.push(match escaped {
                    'n' => '\n',
                    't' => '\t',
                    '"' => '"',
                    '\\' => '\\',
                    other => other,
                });
            } else {
                value.push(self.advance());
            }
        }
        Err(error_for(self.locale, "P1002", position, "string"))
    }

    fn number(&mut self, position: Position) -> Result<Token, PadmaError> {
        let mut raw = String::new();
        raw.push(normalize_digit(self.chars[self.current - 1]));
        while let Some(character) = self.peek() {
            if is_digit(character) {
                raw.push(normalize_digit(self.advance()));
            } else if character == '.' {
                raw.push(self.advance());
            } else {
                break;
            }
        }
        match raw.parse::<f64>() {
            Ok(number) => Ok(self.token(TokenKind::Number(number), position)),
            Err(_) => Err(error_for(self.locale, "P1001", position, &raw)),
        }
    }

    fn identifier(&mut self, position: Position) -> Token {
        let mut word = String::new();
        word.push(self.chars[self.current - 1]);
        while let Some(character) = self.peek() {
            if is_identifier_continue(character) {
                word.push(self.advance());
            } else {
                break;
            }
        }
        let kind = match word.as_str() {
            "ধরি" | "let" => TokenKind::Let,
            "দেখাও" | "print" => TokenKind::Print,
            "যদি" | "if" => TokenKind::If,
            "নইলে" | "else" => TokenKind::Else,
            "যতক্ষণ" | "while" => TokenKind::While,
            "প্রতি" | "for" => TokenKind::For,
            "মধ্যে" | "in" => TokenKind::In,
            "ফাংশন" | "function" | "fn" => TokenKind::Function,
            "ফেরত" | "return" => TokenKind::Return,
            "ইমপোর্ট" | "import" => TokenKind::Import,
            "রপ্তানি" | "export" => TokenKind::Export,
            "সত্য" | "true" => TokenKind::True,
            "মিথ্যা" | "false" => TokenKind::False,
            "কিছুইনা" | "none" => TokenKind::Null,
            _ => TokenKind::Identifier(word),
        };
        self.token(kind, position)
    }
}

fn is_digit(character: char) -> bool {
    character.is_ascii_digit() || ('০'..='৯').contains(&character)
}

fn normalize_digit(character: char) -> char {
    match character {
        '০' => '0',
        '১' => '1',
        '২' => '2',
        '৩' => '3',
        '৪' => '4',
        '৫' => '5',
        '৬' => '6',
        '৭' => '7',
        '৮' => '8',
        '৯' => '9',
        other => other,
    }
}

fn is_bangla(character: char) -> bool {
    ('\u{0980}'..='\u{09ff}').contains(&character)
}

fn is_identifier_start(character: char) -> bool {
    character == '_' || character.is_alphabetic() || is_bangla(character)
}

fn is_identifier_continue(character: char) -> bool {
    is_identifier_start(character) || character.is_ascii_digit() || is_digit(character)
}

fn resolve_output_path(path: &str) -> Result<PathBuf, ()> {
    if path.is_empty() || path.split('/').any(|part| part == "..") {
        return Err(());
    }
    if path == "@downloads" || path.starts_with("@downloads/") {
        let home = env::var_os("HOME").ok_or(())?;
        let relative = path
            .strip_prefix("@downloads")
            .unwrap_or("")
            .trim_start_matches('/');
        return Ok(PathBuf::from(home).join("storage/downloads").join(relative));
    }
    let candidate = std::path::Path::new(path);
    if candidate.is_absolute() {
        return Err(());
    }
    Ok(PathBuf::from(path))
}

fn safe_relative_path(path: &str) -> Result<PathBuf, ()> {
    let candidate = Path::new(path);
    if path.is_empty()
        || path == "@downloads"
        || path.starts_with("@downloads/")
        || candidate.is_absolute()
        || candidate.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        })
    {
        return Err(());
    }
    let normalized = candidate
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(part) => Some(part),
            _ => None,
        })
        .collect::<PathBuf>();
    if normalized.as_os_str().is_empty() {
        return Err(());
    }
    Ok(normalized)
}

fn sqlite_hex_parameter(name: &str, value: &[u8]) -> String {
    let encoded = value
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!(".parameter set {name} x'{encoded}'")
}

fn sqlite_number_parameter(name: &str, value: usize) -> String {
    format!(".parameter set {name} {value}")
}

fn sqlite_script(parameters: &[String], statement: &str, json_output: bool) -> String {
    let mut lines = vec![
        ".bail on".to_string(),
        ".timeout 5000".to_string(),
        ".parameter init".to_string(),
    ];
    lines.extend(parameters.iter().cloned());
    lines.push(
        "CREATE TABLE IF NOT EXISTS padma_records (namespace TEXT NOT NULL, record_key TEXT NOT NULL, value_json TEXT NOT NULL, updated_at INTEGER NOT NULL, PRIMARY KEY(namespace, record_key)) WITHOUT ROWID;".to_string(),
    );
    lines.push(
        "CREATE TABLE IF NOT EXISTS padma_meta (id INTEGER PRIMARY KEY CHECK(id = 1), schema_version INTEGER NOT NULL CHECK(schema_version = 1));".to_string(),
    );
    lines.push("INSERT OR IGNORE INTO padma_meta(id, schema_version) VALUES(1, 1);".to_string());
    if json_output {
        lines.push(".mode json".to_string());
    }
    lines.push(statement.to_string());
    lines.join("\n")
}

fn safe_http_url(url: &str) -> bool {
    if !(url.starts_with("https://") || url.starts_with("http://"))
        || url.chars().any(char::is_whitespace)
        || url.contains('@')
    {
        return false;
    }
    let host = url
        .split_once("://")
        .map(|(_, rest)| {
            rest.split('/')
                .next()
                .unwrap_or(rest)
                .split(':')
                .next()
                .unwrap_or(rest)
        })
        .unwrap_or("")
        .to_ascii_lowercase();
    !host.is_empty()
        && host != "localhost"
        && host != "0.0.0.0"
        && host != "127.0.0.1"
        && !host.starts_with("127.")
        && !host.starts_with("10.")
        && !host.starts_with("192.168.")
        && !host.starts_with("169.254.")
}

fn next_non_cryptographic_random() -> u64 {
    let time_seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0);
    let mut state = time_seed
        ^ (process::id() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ RANDOM_COUNTER
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(1);
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state
}

fn format_from_map(
    template: &str,
    values: &BTreeMap<String, Value>,
    locale: Locale,
    position: Position,
) -> Result<String, PadmaError> {
    let mut output = String::new();
    let mut characters = template.chars().peekable();
    while let Some(character) = characters.next() {
        if character != '{' {
            output.push(character);
            continue;
        }
        if characters.peek() == Some(&'{') {
            characters.next();
            output.push('{');
            continue;
        }
        let mut key = String::new();
        let mut closed = false;
        for candidate in characters.by_ref() {
            if candidate == '}' {
                closed = true;
                break;
            }
            key.push(candidate);
        }
        let key = key.trim();
        let valid_key = key.chars().next().map(is_identifier_start).unwrap_or(false)
            && key.chars().all(is_identifier_continue);
        if !closed || !valid_key {
            return Err(error_for(locale, "P1031", position, key));
        }
        let value = values
            .get(key)
            .ok_or_else(|| error_for(locale, "P1031", position, key))?;
        output.push_str(&value.to_string());
    }
    Ok(output)
}

fn resolve_import_path(importer: &Path, requested: &str) -> Result<PathBuf, ()> {
    let candidate = Path::new(requested);
    if requested.is_empty()
        || !requested.ends_with(".pd")
        || candidate.is_absolute()
        || candidate
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(());
    }
    let base = importer.parent().unwrap_or_else(|| Path::new("."));
    Ok(base.join(candidate))
}

fn expect_string<'a>(
    value: &'a Value,
    locale: Locale,
    position: Position,
    label: &str,
) -> Result<&'a str, PadmaError> {
    match value {
        Value::String(value) => Ok(value),
        _ => Err(error_for(locale, "P1010", position, label)),
    }
}

fn expect_number(
    value: &Value,
    locale: Locale,
    position: Position,
    label: &str,
) -> Result<f64, PadmaError> {
    match value {
        Value::Number(value) if value.is_finite() => Ok(*value),
        _ => Err(error_for(locale, "P1010", position, label)),
    }
}

fn expect_map_key<'a>(
    value: &'a Value,
    locale: Locale,
    position: Position,
) -> Result<&'a str, PadmaError> {
    match value {
        Value::String(value) => Ok(value),
        other => Err(error_for(locale, "P1020", position, &other.to_string())),
    }
}

fn expect_collection_index(
    value: &Value,
    locale: Locale,
    position: Position,
) -> Result<usize, PadmaError> {
    match value {
        Value::Number(value) if value.is_finite() && *value >= 0.0 && value.fract() == 0.0 => {
            usize::try_from(*value as u128)
                .map_err(|_| error_for(locale, "P1026", position, &value.to_string()))
        }
        other => Err(error_for(locale, "P1026", position, &other.to_string())),
    }
}
#[derive(Clone, Debug)]
enum Stmt {
    Let {
        name: String,
        name_position: Position,
        value: Expr,
    },
    Print {
        value: Expr,
    },
    Expression {
        value: Expr,
    },
    Assign {
        name: String,
        value: Expr,
        position: Position,
    },
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Vec<Stmt>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
        position: Position,
    },
    For {
        name: String,
        name_position: Position,
        collection: Expr,
        body: Vec<Stmt>,
        position: Position,
    },
    Function {
        name: String,
        name_position: Position,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Return {
        value: Option<Expr>,
    },
    Import {
        path: String,
        alias: Option<String>,
        position: Position,
    },
    Export(Box<Stmt>),
}

fn exported_symbol_names(program: &[Stmt]) -> (HashSet<String>, HashSet<String>) {
    let mut values = HashSet::new();
    let mut functions = HashSet::new();
    for statement in program {
        match statement {
            Stmt::Export(inner) => match inner.as_ref() {
                Stmt::Let { name, .. } => {
                    values.insert(name.clone());
                }
                Stmt::Function { name, .. } => {
                    functions.insert(name.clone());
                }
                _ => {}
            },
            _ => {}
        }
    }
    (values, functions)
}

#[derive(Clone, Debug)]
enum Expr {
    Literal(Value, Position),
    Variable(String, Position),
    Unary {
        operator: TokenKind,
        right: Box<Expr>,
        position: Position,
    },
    Binary {
        left: Box<Expr>,
        operator: TokenKind,
        right: Box<Expr>,
        position: Position,
    },
    Call {
        name: String,
        arguments: Vec<Expr>,
        position: Position,
    },
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
        position: Position,
    },
    Slice {
        target: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        position: Position,
    },
    List(Vec<Expr>),
    Map(Vec<(Expr, Expr)>),
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
    locale: Locale,
}

impl Parser {
    fn new(tokens: Vec<Token>, locale: Locale) -> Self {
        Self {
            tokens,
            current: 0,
            locale,
        }
    }

    fn parse(mut self) -> Result<Vec<Stmt>, PadmaError> {
        let mut statements = Vec::new();
        self.skip_newlines();
        while !self.is_at_end() {
            statements.push(self.statement()?);
            self.skip_newlines();
        }
        Ok(statements)
    }

    fn parse_recovering(mut self) -> (Vec<Stmt>, Vec<PadmaError>) {
        let mut statements = Vec::new();
        let mut errors = Vec::new();
        self.skip_newlines();
        while !self.is_at_end() {
            match self.statement() {
                Ok(statement) => statements.push(statement),
                Err(error) => {
                    errors.push(error);
                    self.synchronize();
                }
            }
            self.skip_newlines();
        }
        (statements, errors)
    }

    fn statement(&mut self) -> Result<Stmt, PadmaError> {
        if self.matches(|kind| matches!(kind, TokenKind::Let)) {
            return self.let_statement(self.previous().position);
        }
        if self.matches(|kind| matches!(kind, TokenKind::Print)) {
            return self.print_statement(self.previous().position);
        }
        if self.matches(|kind| matches!(kind, TokenKind::If)) {
            return self.if_statement(self.previous().position);
        }
        if self.matches(|kind| matches!(kind, TokenKind::While)) {
            return self.while_statement(self.previous().position);
        }
        if self.matches(|kind| matches!(kind, TokenKind::For)) {
            return self.for_statement(self.previous().position);
        }
        if self.matches(|kind| matches!(kind, TokenKind::Function)) {
            return self.function_statement();
        }
        if self.matches(|kind| matches!(kind, TokenKind::Return)) {
            return self.return_statement(self.previous().position);
        }
        if self.matches(|kind| matches!(kind, TokenKind::Import)) {
            return self.import_statement(self.previous().position);
        }
        if self.matches(|kind| matches!(kind, TokenKind::Export)) {
            return self.export_statement(self.previous().position);
        }
        if self.check(|kind| matches!(kind, TokenKind::Identifier(_)))
            && self
                .tokens
                .get(self.current + 1)
                .is_some_and(|token| matches!(token.kind, TokenKind::Equal))
        {
            return self.assign_statement();
        }
        let value = self.expression()?;
        self.consume_statement_end()?;
        Ok(Stmt::Expression { value })
    }

    fn let_statement(&mut self, position: Position) -> Result<Stmt, PadmaError> {
        let name_token = self.advance().clone();
        let name = match name_token.clone() {
            Token {
                kind: TokenKind::Identifier(name),
                ..
            } => name,
            token => {
                return Err(error_for(
                    self.locale,
                    "P1003",
                    token.position,
                    "variable name",
                ))
            }
        };
        self.consume(|kind| matches!(kind, TokenKind::Equal), "=")?;
        let value = self.expression()?;
        self.consume_statement_end()?;
        let _ = position;
        Ok(Stmt::Let {
            name,
            name_position: name_token.position,
            value,
        })
    }

    fn print_statement(&mut self, position: Position) -> Result<Stmt, PadmaError> {
        let value = self.expression()?;
        self.consume_statement_end()?;
        let _ = position;
        Ok(Stmt::Print { value })
    }

    fn import_statement(&mut self, position: Position) -> Result<Stmt, PadmaError> {
        let path = match self.advance().clone() {
            Token {
                kind: TokenKind::String(path),
                ..
            } => path,
            token => {
                return Err(error_for(
                    self.locale,
                    "P1003",
                    token.position,
                    "module path text",
                ))
            }
        };
        let alias = if self.check(
            |kind| matches!(kind, TokenKind::Identifier(name) if name == "as" || name == "হিসেবে"),
        ) {
            self.advance();
            match self.advance().clone().kind {
                TokenKind::Identifier(alias) => Some(alias),
                _ => {
                    return Err(error_for(
                        self.locale,
                        "P1003",
                        self.previous().position,
                        "module alias",
                    ))
                }
            }
        } else {
            None
        };
        self.consume_statement_end()?;
        Ok(Stmt::Import {
            path,
            alias,
            position,
        })
    }

    fn export_statement(&mut self, position: Position) -> Result<Stmt, PadmaError> {
        let declaration = if self.matches(|kind| matches!(kind, TokenKind::Let)) {
            self.let_statement(self.previous().position)?
        } else if self.matches(|kind| matches!(kind, TokenKind::Function)) {
            self.function_statement()?
        } else {
            return Err(error_for(
                self.locale,
                "P1003",
                position,
                "exported let or function declaration",
            ));
        };
        Ok(Stmt::Export(Box::new(declaration)))
    }

    fn assign_statement(&mut self) -> Result<Stmt, PadmaError> {
        let token = self.advance().clone();
        let name = match token.kind {
            TokenKind::Identifier(name) => name,
            _ => unreachable!("assignment lookahead guarantees an identifier"),
        };
        self.consume(|kind| matches!(kind, TokenKind::Equal), "=")?;
        let value = self.expression()?;
        self.consume_statement_end()?;
        Ok(Stmt::Assign {
            name,
            value,
            position: token.position,
        })
    }

    fn if_statement(&mut self, position: Position) -> Result<Stmt, PadmaError> {
        let condition = self.expression()?;
        self.consume(|kind| matches!(kind, TokenKind::LeftBrace), "{")?;
        let then_branch = self.block()?;
        self.skip_newlines();
        let else_branch = if self.matches(|kind| matches!(kind, TokenKind::Else)) {
            self.consume(|kind| matches!(kind, TokenKind::LeftBrace), "{")?;
            self.block()?
        } else {
            Vec::new()
        };
        let _ = position;
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self, position: Position) -> Result<Stmt, PadmaError> {
        let condition = self.expression()?;
        self.consume(|kind| matches!(kind, TokenKind::LeftBrace), "{")?;
        let body = self.block()?;
        Ok(Stmt::While {
            condition,
            body,
            position,
        })
    }

    fn for_statement(&mut self, position: Position) -> Result<Stmt, PadmaError> {
        let name_token = self.advance().clone();
        let name = match name_token.clone() {
            Token {
                kind: TokenKind::Identifier(name),
                ..
            } => name,
            token => {
                return Err(error_for(
                    self.locale,
                    "P1003",
                    token.position,
                    "loop variable name",
                ))
            }
        };
        self.consume(|kind| matches!(kind, TokenKind::In), "in")?;
        let collection = self.expression()?;
        self.consume(|kind| matches!(kind, TokenKind::LeftBrace), "{")?;
        let body = self.block()?;
        Ok(Stmt::For {
            name,
            name_position: name_token.position,
            collection,
            body,
            position,
        })
    }

    fn function_statement(&mut self) -> Result<Stmt, PadmaError> {
        let name_token = self.advance().clone();
        let name = match name_token.clone().kind {
            TokenKind::Identifier(name) => name,
            _token => {
                return Err(error_for(
                    self.locale,
                    "P1003",
                    self.previous().position,
                    "function name",
                ))
            }
        };
        self.consume(|kind| matches!(kind, TokenKind::LeftParen), "(")?;
        let mut params = Vec::new();
        if !self.check(|kind| matches!(kind, TokenKind::RightParen)) {
            loop {
                match self.advance().clone().kind {
                    TokenKind::Identifier(name) => params.push(name),
                    _ => {
                        return Err(error_for(
                            self.locale,
                            "P1003",
                            self.previous().position,
                            "parameter name",
                        ))
                    }
                }
                if !self.matches(|kind| matches!(kind, TokenKind::Comma)) {
                    break;
                }
            }
        }
        self.consume(|kind| matches!(kind, TokenKind::RightParen), ")")?;
        self.consume(|kind| matches!(kind, TokenKind::LeftBrace), "{")?;
        let body = self.block()?;
        Ok(Stmt::Function {
            name,
            name_position: name_token.position,
            params,
            body,
        })
    }

    fn return_statement(&mut self, position: Position) -> Result<Stmt, PadmaError> {
        let value = if self.check(|kind| {
            matches!(
                kind,
                TokenKind::Newline | TokenKind::RightBrace | TokenKind::Eof
            )
        }) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume_statement_end()?;
        let _ = position;
        Ok(Stmt::Return { value })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, PadmaError> {
        let mut statements = Vec::new();
        self.skip_newlines();
        while !self.check(|kind| matches!(kind, TokenKind::RightBrace)) && !self.is_at_end() {
            statements.push(self.statement()?);
            self.skip_newlines();
        }
        self.consume(|kind| matches!(kind, TokenKind::RightBrace), "}")?;
        Ok(statements)
    }

    fn expression(&mut self) -> Result<Expr, PadmaError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, PadmaError> {
        let mut expression = self.comparison()?;
        while self.matches(|kind| matches!(kind, TokenKind::EqualEqual | TokenKind::BangEqual)) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expression = Expr::Binary {
                left: Box::new(expression),
                operator: operator.kind,
                right: Box::new(right),
                position: operator.position,
            };
        }
        Ok(expression)
    }

    fn comparison(&mut self) -> Result<Expr, PadmaError> {
        let mut expression = self.term()?;
        while self.matches(|kind| {
            matches!(
                kind,
                TokenKind::Greater
                    | TokenKind::GreaterEqual
                    | TokenKind::Less
                    | TokenKind::LessEqual
            )
        }) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expression = Expr::Binary {
                left: Box::new(expression),
                operator: operator.kind,
                right: Box::new(right),
                position: operator.position,
            };
        }
        Ok(expression)
    }

    fn term(&mut self) -> Result<Expr, PadmaError> {
        let mut expression = self.factor()?;
        while self.matches(|kind| matches!(kind, TokenKind::Plus | TokenKind::Minus)) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expression = Expr::Binary {
                left: Box::new(expression),
                operator: operator.kind,
                right: Box::new(right),
                position: operator.position,
            };
        }
        Ok(expression)
    }

    fn factor(&mut self) -> Result<Expr, PadmaError> {
        let mut expression = self.unary()?;
        while self.matches(|kind| matches!(kind, TokenKind::Star | TokenKind::Slash)) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expression = Expr::Binary {
                left: Box::new(expression),
                operator: operator.kind,
                right: Box::new(right),
                position: operator.position,
            };
        }
        Ok(expression)
    }

    fn unary(&mut self) -> Result<Expr, PadmaError> {
        if self.matches(|kind| matches!(kind, TokenKind::Minus)) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator: operator.kind,
                right: Box::new(right),
                position: operator.position,
            });
        }
        self.postfix()
    }

    fn postfix(&mut self) -> Result<Expr, PadmaError> {
        let mut expression = self.primary()?;
        while self.matches(|kind| matches!(kind, TokenKind::LeftBracket)) {
            let position = self.previous().position;
            let start = if self.check(|kind| matches!(kind, TokenKind::Colon)) {
                None
            } else {
                Some(self.expression()?)
            };
            if self.matches(|kind| matches!(kind, TokenKind::Colon)) {
                let end = if self.check(|kind| matches!(kind, TokenKind::RightBracket)) {
                    None
                } else {
                    Some(self.expression()?)
                };
                self.consume(|kind| matches!(kind, TokenKind::RightBracket), "]")?;
                expression = Expr::Slice {
                    target: Box::new(expression),
                    start: start.map(Box::new),
                    end: end.map(Box::new),
                    position,
                };
                continue;
            }
            let index = start.ok_or_else(|| error_for(self.locale, "P1003", position, "index"))?;
            self.consume(|kind| matches!(kind, TokenKind::RightBracket), "]")?;
            expression = Expr::Index {
                target: Box::new(expression),
                index: Box::new(index),
                position,
            };
        }
        Ok(expression)
    }

    fn primary(&mut self) -> Result<Expr, PadmaError> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Number(value) => Ok(Expr::Literal(Value::Number(value), token.position)),
            TokenKind::String(value) => Ok(Expr::Literal(Value::String(value), token.position)),
            TokenKind::True => Ok(Expr::Literal(Value::Boolean(true), token.position)),
            TokenKind::False => Ok(Expr::Literal(Value::Boolean(false), token.position)),
            TokenKind::Null => Ok(Expr::Literal(Value::Null, token.position)),
            TokenKind::Identifier(name) => {
                let name = if self.matches(|kind| matches!(kind, TokenKind::Dot)) {
                    match self.advance().clone().kind {
                        TokenKind::Identifier(member) => format!("{name}.{member}"),
                        _ => {
                            return Err(error_for(
                                self.locale,
                                "P1003",
                                token.position,
                                "member name",
                            ))
                        }
                    }
                } else {
                    name
                };
                if self.matches(|kind| matches!(kind, TokenKind::LeftParen)) {
                    let mut arguments = Vec::new();
                    if !self.check(|kind| matches!(kind, TokenKind::RightParen)) {
                        loop {
                            arguments.push(self.expression()?);
                            if !self.matches(|kind| matches!(kind, TokenKind::Comma)) {
                                break;
                            }
                        }
                    }
                    self.consume(|kind| matches!(kind, TokenKind::RightParen), ")")?;
                    Ok(Expr::Call {
                        name,
                        arguments,
                        position: token.position,
                    })
                } else {
                    Ok(Expr::Variable(name, token.position))
                }
            }
            TokenKind::LeftBracket => {
                let mut values = Vec::new();
                if !self.check(|kind| matches!(kind, TokenKind::RightBracket)) {
                    loop {
                        values.push(self.expression()?);
                        if !self.matches(|kind| matches!(kind, TokenKind::Comma)) {
                            break;
                        }
                    }
                }
                self.consume(|kind| matches!(kind, TokenKind::RightBracket), "]")?;
                Ok(Expr::List(values))
            }
            TokenKind::LeftBrace => {
                let mut entries = Vec::new();
                if !self.check(|kind| matches!(kind, TokenKind::RightBrace)) {
                    loop {
                        let key = self.expression()?;
                        self.consume(|kind| matches!(kind, TokenKind::Colon), ":")?;
                        let value = self.expression()?;
                        entries.push((key, value));
                        if !self.matches(|kind| matches!(kind, TokenKind::Comma)) {
                            break;
                        }
                    }
                }
                self.consume(|kind| matches!(kind, TokenKind::RightBrace), "}")?;
                Ok(Expr::Map(entries))
            }
            TokenKind::LeftParen => {
                let expression = self.expression()?;
                self.consume(|kind| matches!(kind, TokenKind::RightParen), ")")?;
                Ok(expression)
            }
            _ => Err(error_for(
                self.locale,
                "P1003",
                token.position,
                "expression",
            )),
        }
    }

    fn consume_statement_end(&mut self) -> Result<(), PadmaError> {
        if self.check(|kind| {
            matches!(
                kind,
                TokenKind::Newline | TokenKind::RightBrace | TokenKind::Eof
            )
        }) {
            return Ok(());
        }
        let token = self.peek().clone();
        Err(error_for(self.locale, "P1003", token.position, "new line"))
    }

    fn consume<F>(&mut self, predicate: F, expected: &str) -> Result<Token, PadmaError>
    where
        F: Fn(&TokenKind) -> bool,
    {
        if self.check(predicate) {
            Ok(self.advance().clone())
        } else {
            let token = self.peek().clone();
            Err(error_for(self.locale, "P1003", token.position, expected))
        }
    }

    fn skip_newlines(&mut self) {
        while self.matches(|kind| matches!(kind, TokenKind::Newline)) {}
    }

    fn synchronize(&mut self) {
        while !self.is_at_end() {
            if self.matches(|kind| matches!(kind, TokenKind::Newline | TokenKind::RightBrace)) {
                return;
            }
            self.advance();
        }
    }

    fn matches<F>(&mut self, predicate: F) -> bool
    where
        F: Fn(&TokenKind) -> bool,
    {
        if self.check(predicate) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check<F>(&self, predicate: F) -> bool
    where
        F: Fn(&TokenKind) -> bool,
    {
        !self.is_at_end() && predicate(&self.peek().kind)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    List(Vec<Value>),
    Map(BTreeMap<String, Value>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TableData {
    format: String,
    headers: Vec<String>,
    rows: Vec<BTreeMap<String, String>>,
}

const TABLE_MAX_BYTES: usize = 1_048_576;
const TABLE_MAX_ROWS: usize = 4_096;
const TABLE_MAX_COLUMNS: usize = 64;
const TABLE_MAX_HEADER_BYTES: usize = 128;
const TABLE_MAX_CELL_BYTES: usize = 4_096;

fn value_from_json(value: JsonValue) -> Result<Value, String> {
    match value {
        JsonValue::Null => Ok(Value::Null),
        JsonValue::Bool(value) => Ok(Value::Boolean(value)),
        JsonValue::Number(value) => value
            .as_f64()
            .filter(|number| number.is_finite())
            .map(Value::Number)
            .ok_or_else(|| "number is outside Padma's finite number range".to_string()),
        JsonValue::String(value) => Ok(Value::String(value)),
        JsonValue::Array(values) => values
            .into_iter()
            .map(value_from_json)
            .collect::<Result<Vec<_>, _>>()
            .map(Value::List),
        JsonValue::Object(values) => values
            .into_iter()
            .map(|(key, value)| value_from_json(value).map(|value| (key, value)))
            .collect::<Result<BTreeMap<_, _>, _>>()
            .map(Value::Map),
    }
}

fn value_to_json(value: &Value) -> Result<JsonValue, String> {
    match value {
        Value::Null => Ok(JsonValue::Null),
        Value::Boolean(value) => Ok(JsonValue::Bool(*value)),
        Value::Number(value) => serde_json::Number::from_f64(*value)
            .map(JsonValue::Number)
            .ok_or_else(|| "number is outside JSON's finite number range".to_string()),
        Value::String(value) => Ok(JsonValue::String(value.clone())),
        Value::List(values) => values
            .iter()
            .map(value_to_json)
            .collect::<Result<Vec<_>, _>>()
            .map(JsonValue::Array),
        Value::Map(values) => values
            .iter()
            .map(|(key, value)| value_to_json(value).map(|value| (key.clone(), value)))
            .collect::<Result<serde_json::Map<_, _>, _>>()
            .map(JsonValue::Object),
    }
}

fn table_error(locale: Locale, position: Position, detail: &str) -> PadmaError {
    error_for(locale, "P1069", position, detail)
}

fn table_validate_headers(
    headers: Vec<String>,
    locale: Locale,
    position: Position,
) -> Result<Vec<String>, PadmaError> {
    if headers.is_empty() || headers.len() > TABLE_MAX_COLUMNS {
        return Err(table_error(
            locale,
            position,
            "header count is outside the table limit",
        ));
    }
    let mut seen = BTreeSet::new();
    let mut normalized = Vec::with_capacity(headers.len());
    for header in headers {
        let header = header.trim().to_string();
        if header.is_empty()
            || header.len() > TABLE_MAX_HEADER_BYTES
            || header.chars().any(char::is_control)
            || !seen.insert(header.clone())
        {
            return Err(table_error(
                locale,
                position,
                "headers must be unique bounded text values",
            ));
        }
        normalized.push(header);
    }
    Ok(normalized)
}

fn table_validate_cell(
    value: String,
    locale: Locale,
    position: Position,
) -> Result<String, PadmaError> {
    if value.len() > TABLE_MAX_CELL_BYTES
        || value
            .chars()
            .any(|character| character == '\n' || character == '\r')
    {
        return Err(table_error(
            locale,
            position,
            "cell exceeds the bounded single-line table policy",
        ));
    }
    Ok(value)
}

fn parse_delimited_table_row(
    line: &str,
    delimiter: char,
    locale: Locale,
    position: Position,
) -> Result<Vec<String>, PadmaError> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut characters = line.chars().peekable();
    let mut quoted = false;
    let mut after_quote = false;
    while let Some(character) = characters.next() {
        if quoted {
            if character == '"' {
                if characters.peek() == Some(&'"') {
                    characters.next();
                    field.push('"');
                } else {
                    quoted = false;
                    after_quote = true;
                }
            } else {
                field.push(character);
            }
            continue;
        }
        if after_quote {
            if character == delimiter {
                fields.push(table_validate_cell(field, locale, position)?);
                field = String::new();
                after_quote = false;
                continue;
            }
            return Err(table_error(
                locale,
                position,
                "quoted field must end at a delimiter or line boundary",
            ));
        }
        if character == delimiter {
            fields.push(table_validate_cell(field, locale, position)?);
            field = String::new();
        } else if character == '"' && field.is_empty() {
            quoted = true;
        } else {
            field.push(character);
        }
    }
    if quoted {
        return Err(table_error(locale, position, "quoted field is not closed"));
    }
    fields.push(table_validate_cell(field, locale, position)?);
    Ok(fields)
}

fn table_data_from_delimited_text(
    source: &str,
    format: &str,
    locale: Locale,
    position: Position,
) -> Result<TableData, PadmaError> {
    let delimiter = match format {
        "csv" => ',',
        "tsv" => '\t',
        _ => {
            return Err(table_error(
                locale,
                position,
                "table format must be csv, tsv, or json",
            ))
        }
    };
    let mut lines = source
        .lines()
        .map(|line| line.trim_end_matches('\r'))
        .filter(|line| !line.is_empty());
    let header_line = lines
        .next()
        .ok_or_else(|| table_error(locale, position, "table requires a header row"))?;
    let headers = table_validate_headers(
        parse_delimited_table_row(header_line, delimiter, locale, position)?,
        locale,
        position,
    )?;
    let mut rows = Vec::new();
    for line in lines {
        if rows.len() >= TABLE_MAX_ROWS {
            return Err(table_error(
                locale,
                position,
                "row count exceeds the table limit",
            ));
        }
        let cells = parse_delimited_table_row(line, delimiter, locale, position)?;
        if cells.len() != headers.len() {
            return Err(table_error(
                locale,
                position,
                "each row must contain exactly the declared header count",
            ));
        }
        rows.push(headers.iter().cloned().zip(cells).collect());
    }
    Ok(TableData {
        format: format.to_string(),
        headers,
        rows,
    })
}

fn table_data_from_json(
    source: &str,
    locale: Locale,
    position: Position,
) -> Result<TableData, PadmaError> {
    let value: JsonValue = serde_json::from_str(source)
        .map_err(|_| table_error(locale, position, "json table must be valid JSON"))?;
    let rows = value
        .as_array()
        .ok_or_else(|| table_error(locale, position, "json table must be an array of objects"))?;
    if rows.is_empty() || rows.len() > TABLE_MAX_ROWS {
        return Err(table_error(
            locale,
            position,
            "row count is outside the table limit",
        ));
    }
    let mut header_set = BTreeSet::new();
    for row in rows {
        let object = row
            .as_object()
            .ok_or_else(|| table_error(locale, position, "json table rows must be objects"))?;
        header_set.extend(object.keys().cloned());
    }
    let headers = table_validate_headers(header_set.into_iter().collect(), locale, position)?;
    let mut parsed_rows = Vec::with_capacity(rows.len());
    for row in rows {
        let object = row.as_object().expect("validated JSON table object");
        let mut parsed = BTreeMap::new();
        for header in &headers {
            let value = object.get(header).cloned().unwrap_or(JsonValue::Null);
            let text = match value {
                JsonValue::Null => String::new(),
                JsonValue::Bool(value) => value.to_string(),
                JsonValue::Number(value) => value.to_string(),
                JsonValue::String(value) => value,
                JsonValue::Array(_) | JsonValue::Object(_) => {
                    return Err(table_error(
                        locale,
                        position,
                        "json table cells must be scalar values",
                    ))
                }
            };
            parsed.insert(header.clone(), table_validate_cell(text, locale, position)?);
        }
        parsed_rows.push(parsed);
    }
    Ok(TableData {
        format: "json".to_string(),
        headers,
        rows: parsed_rows,
    })
}

fn table_data_to_value(table: &TableData) -> Value {
    let mut result = BTreeMap::new();
    result.insert("format".into(), Value::String(table.format.clone()));
    result.insert(
        "headers".into(),
        Value::List(table.headers.iter().cloned().map(Value::String).collect()),
    );
    result.insert(
        "rows".into(),
        Value::List(
            table
                .rows
                .iter()
                .map(|row| {
                    Value::Map(
                        row.iter()
                            .map(|(key, value)| (key.clone(), Value::String(value.clone())))
                            .collect(),
                    )
                })
                .collect(),
        ),
    );
    Value::Map(result)
}

fn table_data_from_value(
    value: &Value,
    locale: Locale,
    position: Position,
) -> Result<TableData, PadmaError> {
    let Value::Map(table) = value else {
        return Err(table_error(locale, position, "table must be a table value"));
    };
    if !table
        .keys()
        .all(|key| matches!(key.as_str(), "format" | "headers" | "rows"))
    {
        return Err(table_error(
            locale,
            position,
            "table contains unsupported fields",
        ));
    }
    let Some(Value::String(format)) = table.get("format") else {
        return Err(table_error(locale, position, "table format is required"));
    };
    if !matches!(format.as_str(), "csv" | "tsv" | "json") {
        return Err(table_error(
            locale,
            position,
            "table format must be csv, tsv, or json",
        ));
    }
    let Some(Value::List(header_values)) = table.get("headers") else {
        return Err(table_error(locale, position, "table headers are required"));
    };
    let headers = table_validate_headers(
        header_values
            .iter()
            .map(|value| match value {
                Value::String(value) => Ok(value.clone()),
                _ => Err(table_error(locale, position, "table headers must be text")),
            })
            .collect::<Result<Vec<_>, _>>()?,
        locale,
        position,
    )?;
    let Some(Value::List(row_values)) = table.get("rows") else {
        return Err(table_error(locale, position, "table rows are required"));
    };
    if row_values.len() > TABLE_MAX_ROWS {
        return Err(table_error(
            locale,
            position,
            "row count exceeds the table limit",
        ));
    }
    let mut rows = Vec::with_capacity(row_values.len());
    for row in row_values {
        let Value::Map(row) = row else {
            return Err(table_error(locale, position, "table rows must be maps"));
        };
        if row.len() != headers.len() || row.keys().any(|key| !headers.contains(key)) {
            return Err(table_error(
                locale,
                position,
                "row fields must exactly match table headers",
            ));
        }
        let mut parsed = BTreeMap::new();
        for header in &headers {
            let Some(Value::String(cell)) = row.get(header) else {
                return Err(table_error(locale, position, "table cells must be text"));
            };
            parsed.insert(
                header.clone(),
                table_validate_cell(cell.clone(), locale, position)?,
            );
        }
        rows.push(parsed);
    }
    Ok(TableData {
        format: format.clone(),
        headers,
        rows,
    })
}

fn table_csv_escape(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn table_data_to_csv(table: &TableData) -> String {
    let mut lines = Vec::with_capacity(table.rows.len().saturating_add(1));
    lines.push(
        table
            .headers
            .iter()
            .map(|header| table_csv_escape(header))
            .collect::<Vec<_>>()
            .join(","),
    );
    for row in &table.rows {
        lines.push(
            table
                .headers
                .iter()
                .map(|header| table_csv_escape(row.get(header).map(String::as_str).unwrap_or("")))
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    format!("{}\n", lines.join("\n"))
}

fn read_bridge_stream(mut stream: impl Read) -> Result<Vec<u8>, ()> {
    let mut result = Vec::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = stream.read(&mut buffer).map_err(|_| ())?;
        if read == 0 {
            return Ok(result);
        }
        if result.len().saturating_add(read) > BRIDGE_MAX_BYTES {
            return Err(());
        }
        result.extend_from_slice(&buffer[..read]);
    }
}

fn read_bounded_stream(mut stream: impl Read, limit: usize) -> Result<Vec<u8>, ()> {
    let mut result = Vec::new();
    let mut buffer = [0_u8; 8192];
    let mut exceeded = false;
    loop {
        let read = stream.read(&mut buffer).map_err(|_| ())?;
        if read == 0 {
            return if exceeded { Err(()) } else { Ok(result) };
        }
        if result.len().saturating_add(read) > limit {
            exceeded = true;
            continue;
        }
        result.extend_from_slice(&buffer[..read]);
    }
}

#[derive(Debug)]
struct ParsedUrl {
    normalized: String,
    scheme: String,
    host: String,
    path: String,
    query: Option<String>,
    fragment: Option<String>,
    port: Option<u16>,
}

fn parse_http_url(text: &str) -> Result<ParsedUrl, String> {
    if text.is_empty()
        || text
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
    {
        return Err("URL must not be empty or contain whitespace".into());
    }
    let (scheme, remainder) = text
        .split_once("://")
        .ok_or_else(|| "URL needs an http:// or https:// scheme".to_string())?;
    if !matches!(scheme, "http" | "https") {
        return Err("only http and https URLs are supported".into());
    }
    let (without_fragment, fragment) = match remainder.split_once('#') {
        Some((value, fragment)) => (value, Some(fragment.to_string())),
        None => (remainder, None),
    };
    let (authority_and_path, query) = match without_fragment.split_once('?') {
        Some((value, query)) => (value, Some(query.to_string())),
        None => (without_fragment, None),
    };
    let (authority, path) = match authority_and_path.split_once('/') {
        Some((authority, path)) => (authority, format!("/{path}")),
        None => (authority_and_path, "/".to_string()),
    };
    if authority.is_empty() || authority.contains('@') {
        return Err("URL host is missing or contains unsupported credentials".into());
    }
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, candidate))
            if !host.is_empty()
                && candidate
                    .chars()
                    .all(|character| character.is_ascii_digit()) =>
        {
            let port = candidate
                .parse::<u16>()
                .map_err(|_| "URL port is outside 0-65535".to_string())?;
            (host, Some(port))
        }
        _ => (authority, None),
    };
    if host.is_empty()
        || host
            .chars()
            .any(|character| matches!(character, '/' | ':' | '?' | '#'))
    {
        return Err("URL host is invalid".into());
    }
    Ok(ParsedUrl {
        normalized: text.to_string(),
        scheme: scheme.to_string(),
        host: host.to_string(),
        path,
        query,
        fragment,
        port,
    })
}

impl Value {
    fn truthy(&self) -> bool {
        match self {
            Self::Boolean(value) => *value,
            Self::Null => false,
            Self::Number(value) => *value != 0.0,
            Self::String(value) => !value.is_empty(),
            Self::List(value) => !value.is_empty(),
            Self::Map(value) => !value.is_empty(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(value) if value.fract() == 0.0 => write!(formatter, "{value:.0}"),
            Self::Number(value) => write!(formatter, "{value}"),
            Self::String(value) => write!(formatter, "{value}"),
            Self::Boolean(true) => write!(formatter, "true"),
            Self::Boolean(false) => write!(formatter, "false"),
            Self::Null => write!(formatter, "none"),
            Self::List(values) => {
                write!(formatter, "[")?;
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        write!(formatter, ", ")?;
                    }
                    write!(formatter, "{value}")?;
                }
                write!(formatter, "]")
            }
            Self::Map(values) => {
                write!(formatter, "{{")?;
                for (index, (key, value)) in values.iter().enumerate() {
                    if index > 0 {
                        write!(formatter, ", ")?;
                    }
                    write!(formatter, "\"{key}\": {value}")?;
                }
                write!(formatter, "}}")
            }
        }
    }
}

#[derive(Clone)]
struct ModuleNamespace {
    environment: HashMap<String, Value>,
    functions: HashMap<String, (Vec<String>, Vec<Stmt>)>,
    modules: HashMap<String, ModuleNamespace>,
}

struct Interpreter {
    environment: HashMap<String, Value>,
    local_scopes: Vec<HashMap<String, Value>>,
    functions: HashMap<String, (Vec<String>, Vec<Stmt>)>,
    return_value: Option<Value>,
    output: Vec<String>,
    locale: Locale,
    current_source: PathBuf,
    loaded_modules: HashSet<PathBuf>,
    active_modules: HashSet<PathBuf>,
    modules: HashMap<String, ModuleNamespace>,
    module_cache: HashMap<PathBuf, ModuleNamespace>,
    project_capabilities: Option<BTreeSet<String>>,
    project_root: Option<PathBuf>,
}

impl Interpreter {
    fn new(locale: Locale) -> Self {
        Self::with_source_path(locale, PathBuf::from("repl.pd"))
    }

    fn with_source_path(locale: Locale, current_source: PathBuf) -> Self {
        Self {
            environment: HashMap::new(),
            local_scopes: Vec::new(),
            functions: HashMap::new(),
            return_value: None,
            output: Vec::new(),
            locale,
            current_source,
            loaded_modules: HashSet::new(),
            active_modules: HashSet::new(),
            modules: HashMap::new(),
            module_cache: HashMap::new(),
            project_capabilities: None,
            project_root: None,
        }
    }

    fn with_project_capabilities(
        locale: Locale,
        current_source: PathBuf,
        project_root: PathBuf,
        project_capabilities: BTreeSet<String>,
    ) -> Self {
        let mut interpreter = Self::with_source_path(locale, current_source);
        interpreter.project_capabilities = Some(project_capabilities);
        interpreter.project_root = fs::canonicalize(project_root).ok();
        interpreter
    }

    fn require_capability(
        &self,
        capability: &str,
        operation: &str,
        position: Position,
    ) -> Result<(), PadmaError> {
        if self
            .project_capabilities
            .as_ref()
            .is_some_and(|grants| !grants.contains(capability))
        {
            return Err(error_for(
                self.locale,
                "P1034",
                position,
                &format!("{capability} for {operation}"),
            ));
        }
        Ok(())
    }

    fn require_project_capability(
        &self,
        capability: &str,
        operation: &str,
        position: Position,
    ) -> Result<(), PadmaError> {
        if !self
            .project_capabilities
            .as_ref()
            .is_some_and(|grants| grants.contains(capability))
        {
            return Err(error_for(
                self.locale,
                "P1034",
                position,
                &format!("{capability} for {operation}"),
            ));
        }
        Ok(())
    }

    fn resolve_file_path(&self, path: &str) -> Result<PathBuf, ()> {
        let Some(root) = self.project_root.as_ref() else {
            return resolve_output_path(path);
        };
        let relative = safe_relative_path(path)?;
        let resolved = root.join(relative);
        let parent = resolved.parent().ok_or(())?;
        let canonical_parent = fs::canonicalize(parent).map_err(|_| ())?;
        if !canonical_parent.starts_with(root) {
            return Err(());
        }
        if resolved.exists()
            && !fs::canonicalize(&resolved)
                .map_err(|_| ())?
                .starts_with(root)
        {
            return Err(());
        }
        Ok(resolved)
    }

    fn sqlite_database_path(&self, path: &str, position: Position) -> Result<PathBuf, PadmaError> {
        self.require_project_capability("database:sqlite", "db", position)?;
        if !path.ends_with(".sqlite") {
            return Err(error_for(self.locale, "P1014", position, path));
        }
        self.resolve_file_path(path)
            .map_err(|_| error_for(self.locale, "P1014", position, path))
    }

    fn sqlite_execute(
        &self,
        path: &str,
        parameters: &[String],
        statement: &str,
        json_output: bool,
        position: Position,
    ) -> Result<Vec<u8>, PadmaError> {
        let database = self.sqlite_database_path(path, position)?;
        let script = sqlite_script(parameters, statement, json_output);
        if script.len() > SQLITE_MAX_BYTES {
            return Err(error_for(self.locale, "P1043", position, "request"));
        }
        let path = env::var_os("PATH")
            .ok_or_else(|| error_for(self.locale, "P1041", position, "sqlite3"))?;
        let working_directory = self
            .project_root
            .clone()
            .ok_or_else(|| error_for(self.locale, "P1034", position, "database:sqlite for db"))?;
        let mut child = process::Command::new("sqlite3")
            .arg(&database)
            .args(["-batch", "-bail"])
            .current_dir(working_directory)
            .env_clear()
            .env("PATH", path)
            .env("LANG", "C.UTF-8")
            .env("LC_ALL", "C.UTF-8")
            .stdin(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .spawn()
            .map_err(|_| error_for(self.locale, "P1041", position, "sqlite3"))?;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| error_for(self.locale, "P1041", position, "sqlite3"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| error_for(self.locale, "P1041", position, "sqlite3"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| error_for(self.locale, "P1041", position, "sqlite3"))?;
        let writer = thread::spawn(move || {
            stdin
                .write_all(script.as_bytes())
                .and_then(|_| stdin.flush())
        });
        let stdout_reader = thread::spawn(move || read_bridge_stream(stdout));
        let stderr_reader = thread::spawn(move || read_bridge_stream(stderr));
        let started = Instant::now();
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if started.elapsed() > SQLITE_TIMEOUT => {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = writer.join();
                    let _ = stdout_reader.join();
                    let _ = stderr_reader.join();
                    return Err(error_for(self.locale, "P1043", position, "time"));
                }
                Ok(None) => thread::sleep(Duration::from_millis(10)),
                Err(_) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = writer.join();
                    let _ = stdout_reader.join();
                    let _ = stderr_reader.join();
                    return Err(error_for(self.locale, "P1041", position, "sqlite3"));
                }
            }
        };
        let write_result = writer
            .join()
            .map_err(|_| error_for(self.locale, "P1041", position, "sqlite3"))?;
        let output = stdout_reader
            .join()
            .map_err(|_| error_for(self.locale, "P1043", position, "output"))?
            .map_err(|_| error_for(self.locale, "P1043", position, "output"))?;
        let _ = stderr_reader.join();
        if write_result.is_err() || !status.success() {
            return Err(error_for(self.locale, "P1041", position, "sqlite3"));
        }
        Ok(output)
    }

    fn ai_workflow(&self, input: &Value, position: Position) -> Result<Value, PadmaError> {
        self.require_project_capability("network:ai", "ai.workflow", position)?;
        let root = self.project_root.as_ref().ok_or_else(|| {
            error_for(self.locale, "P1034", position, "network:ai for ai.workflow")
        })?;
        let (_, manifest) = load_ai_workflow_manifest(root)
            .map_err(|_| error_for(self.locale, "P1050", position, "padma-ai.toml"))?;
        let request = ai_workflow_request_payload(input, &manifest, self.locale, position)?;
        let secret = env::var(&manifest.secret_env)
            .map_err(|_| error_for(self.locale, "P1051", position, "credential is unavailable"))?;
        if secret.is_empty() || secret.chars().any(char::is_control) {
            return Err(error_for(
                self.locale,
                "P1051",
                position,
                "credential is unavailable",
            ));
        }
        let response = self.ai_workflow_transport(&manifest, &secret, &request, position)?;
        ai_workflow_response_value(&response, &manifest, self.locale, position)
    }

    fn ai_workflow_transport(
        &self,
        manifest: &AiWorkflowManifest,
        secret: &str,
        request: &[u8],
        position: Position,
    ) -> Result<Vec<u8>, PadmaError> {
        let path = env::var_os("PATH")
            .ok_or_else(|| error_for(self.locale, "P1051", position, "curl is unavailable"))?;
        let working_directory = self.project_root.clone().ok_or_else(|| {
            error_for(self.locale, "P1034", position, "network:ai for ai.workflow")
        })?;
        let config = ai_workflow_curl_config(manifest, secret, request).ok_or_else(|| {
            error_for(
                self.locale,
                "P1050",
                position,
                "unsafe workflow request descriptor",
            )
        })?;
        let mut child = process::Command::new(Self::ai_workflow_curl_program())
            .args(["--config", "-"])
            .current_dir(working_directory)
            .env_clear()
            .env("PATH", path)
            .env("LANG", "C.UTF-8")
            .env("LC_ALL", "C.UTF-8")
            .stdin(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .spawn()
            .map_err(|_| error_for(self.locale, "P1051", position, "curl is unavailable"))?;
        let mut stdin = child.stdin.take().ok_or_else(|| {
            error_for(self.locale, "P1051", position, "curl input is unavailable")
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            error_for(self.locale, "P1051", position, "curl output is unavailable")
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            error_for(
                self.locale,
                "P1051",
                position,
                "curl error stream is unavailable",
            )
        })?;
        let writer = thread::spawn(move || stdin.write_all(&config).and_then(|_| stdin.flush()));
        let max_response_bytes = manifest.max_response_bytes;
        let stdout_reader = thread::spawn(move || read_bounded_stream(stdout, max_response_bytes));
        let stderr_reader =
            thread::spawn(move || read_bounded_stream(stderr, AI_WORKFLOW_MAX_STDERR_BYTES));
        let started = Instant::now();
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None)
                    if started.elapsed()
                        > Duration::from_secs(u64::from(manifest.timeout_seconds)) =>
                {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = writer.join();
                    let _ = stdout_reader.join();
                    let _ = stderr_reader.join();
                    return Err(error_for(
                        self.locale,
                        "P1051",
                        position,
                        "request timed out",
                    ));
                }
                Ok(None) => thread::sleep(Duration::from_millis(10)),
                Err(_) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = writer.join();
                    let _ = stdout_reader.join();
                    let _ = stderr_reader.join();
                    return Err(error_for(self.locale, "P1051", position, "curl failed"));
                }
            }
        };
        let write_result = writer
            .join()
            .map_err(|_| error_for(self.locale, "P1051", position, "curl input failed"))?;
        let output = stdout_reader
            .join()
            .map_err(|_| error_for(self.locale, "P1052", position, "response is unavailable"))?
            .map_err(|_| {
                error_for(
                    self.locale,
                    "P1052",
                    position,
                    "response exceeds declared limit",
                )
            })?;
        let _ = stderr_reader.join();
        if write_result.is_err() || !status.success() {
            return Err(error_for(
                self.locale,
                "P1051",
                position,
                "curl rejected the request",
            ));
        }
        Ok(output)
    }

    fn ai_workflow_curl_program() -> PathBuf {
        #[cfg(test)]
        if let Some(program) = AI_WORKFLOW_CURL_TEST_PROGRAM
            .get_or_init(|| Mutex::new(None))
            .lock()
            .ok()
            .and_then(|program| program.clone())
        {
            return program;
        }
        PathBuf::from("curl")
    }

    fn bridge_call(
        &self,
        runtime: &str,
        script_path: &str,
        value: &Value,
        position: Position,
    ) -> Result<Value, PadmaError> {
        let (program, required_extension) = match runtime {
            "python" => ("python", "py"),
            "javascript" => ("node", "js"),
            _ => return Err(error_for(self.locale, "P1035", position, runtime)),
        };
        self.require_capability(&format!("process:{program}"), "bridge.call", position)?;

        let relative = safe_relative_path(script_path)
            .map_err(|_| error_for(self.locale, "P1036", position, script_path))?;
        let resolved = if self.project_root.is_some() {
            self.resolve_file_path(script_path)
                .map_err(|_| error_for(self.locale, "P1036", position, script_path))?
        } else {
            relative
        };
        let script = fs::canonicalize(&resolved)
            .map_err(|_| error_for(self.locale, "P1036", position, script_path))?;
        if !script.is_file()
            || script.extension().and_then(|extension| extension.to_str())
                != Some(required_extension)
        {
            return Err(error_for(self.locale, "P1036", position, script_path));
        }
        if let Some(root) = self.project_root.as_ref() {
            if !script.starts_with(root) {
                return Err(error_for(self.locale, "P1036", position, script_path));
            }
        }

        let input = serde_json::to_vec(
            &value_to_json(value)
                .map_err(|detail| error_for(self.locale, "P1029", position, &detail))?,
        )
        .map_err(|error| error_for(self.locale, "P1029", position, &error.to_string()))?;
        if input.len() > BRIDGE_MAX_BYTES {
            return Err(error_for(self.locale, "P1037", position, "input"));
        }
        let path = env::var_os("PATH")
            .ok_or_else(|| error_for(self.locale, "P1038", position, program))?;
        let working_directory = self.project_root.clone().unwrap_or(
            env::current_dir().map_err(|_| error_for(self.locale, "P1038", position, program))?,
        );
        let mut child = process::Command::new(program)
            .arg(&script)
            .current_dir(working_directory)
            .env_clear()
            .env("PATH", path)
            .env("LANG", "C.UTF-8")
            .env("LC_ALL", "C.UTF-8")
            .stdin(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .spawn()
            .map_err(|_| error_for(self.locale, "P1038", position, program))?;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| error_for(self.locale, "P1038", position, program))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| error_for(self.locale, "P1038", position, program))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| error_for(self.locale, "P1038", position, program))?;
        let writer = thread::spawn(move || stdin.write_all(&input).and_then(|_| stdin.flush()));
        let stdout_reader = thread::spawn(move || read_bridge_stream(stdout));
        let stderr_reader = thread::spawn(move || read_bridge_stream(stderr));
        let started = Instant::now();
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if started.elapsed() > BRIDGE_TIMEOUT => {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = writer.join();
                    let _ = stdout_reader.join();
                    let _ = stderr_reader.join();
                    return Err(error_for(self.locale, "P1039", position, runtime));
                }
                Ok(None) => thread::sleep(Duration::from_millis(10)),
                Err(_) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = writer.join();
                    let _ = stdout_reader.join();
                    let _ = stderr_reader.join();
                    return Err(error_for(self.locale, "P1038", position, program));
                }
            }
        };
        let write_result = writer
            .join()
            .map_err(|_| error_for(self.locale, "P1038", position, program))?;
        let output = stdout_reader
            .join()
            .map_err(|_| error_for(self.locale, "P1038", position, program))?
            .map_err(|_| error_for(self.locale, "P1037", position, "output"))?;
        let _ = stderr_reader.join();
        if write_result.is_err() || !status.success() {
            return Err(error_for(self.locale, "P1038", position, program));
        }
        let output: JsonValue = serde_json::from_slice(&output)
            .map_err(|_| error_for(self.locale, "P1040", position, runtime))?;
        value_from_json(output).map_err(|_| error_for(self.locale, "P1040", position, runtime))
    }

    fn run(&mut self, program: &[Stmt]) -> Result<(), PadmaError> {
        for statement in program {
            self.execute(statement)?;
            if self.return_value.is_some() {
                break;
            }
        }
        Ok(())
    }

    fn run_in_scope(&mut self, program: &[Stmt]) -> Result<(), PadmaError> {
        self.local_scopes.push(HashMap::new());
        let result = self.run(program);
        self.local_scopes.pop();
        result
    }

    fn define_variable(&mut self, name: String, value: Value) {
        if let Some(scope) = self.local_scopes.last_mut() {
            scope.insert(name, value);
        } else {
            self.environment.insert(name, value);
        }
    }

    fn variable(&self, name: &str) -> Option<&Value> {
        self.local_scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name))
            .or_else(|| self.environment.get(name))
    }

    fn variable_mut(&mut self, name: &str) -> Option<&mut Value> {
        let scope_index = self
            .local_scopes
            .iter()
            .rposition(|scope| scope.contains_key(name));
        if let Some(scope_index) = scope_index {
            return self.local_scopes[scope_index].get_mut(name);
        }
        self.environment.get_mut(name)
    }

    fn assign_variable(&mut self, name: &str, value: Value) -> bool {
        if let Some(target) = self.variable_mut(name) {
            *target = value;
            true
        } else {
            false
        }
    }

    fn execute(&mut self, statement: &Stmt) -> Result<(), PadmaError> {
        match statement {
            Stmt::Let { name, value, .. } => {
                let value = self.evaluate(value)?;
                self.define_variable(name.clone(), value);
            }
            Stmt::Print { value, .. } => {
                let value = self.evaluate(value)?;
                self.output.push(value.to_string());
            }
            Stmt::Expression { value } => {
                self.evaluate(value)?;
            }
            Stmt::Assign {
                name,
                value,
                position,
            } => {
                let value = self.evaluate(value)?;
                if !self.assign_variable(name, value) {
                    return Err(error_for(self.locale, "P1007", *position, name));
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let branch = if self.evaluate(condition)?.truthy() {
                    then_branch
                } else {
                    else_branch
                };
                self.run_in_scope(branch)?;
            }
            Stmt::While {
                condition,
                body,
                position,
            } => {
                let mut iterations = 0usize;
                while self.evaluate(condition)?.truthy() {
                    iterations += 1;
                    if iterations > 1_000_000 {
                        return Err(error_for(
                            self.locale,
                            "P1012",
                            *position,
                            "loop iteration limit",
                        ));
                    }
                    self.run_in_scope(body)?;
                    if self.return_value.is_some() {
                        break;
                    }
                }
            }
            Stmt::For {
                name,
                collection,
                body,
                position,
                ..
            } => {
                let collection = self.evaluate(collection)?;
                let values = match collection {
                    Value::List(values) => values,
                    Value::Map(values) => values.into_keys().map(Value::String).collect::<Vec<_>>(),
                    Value::String(value) => value
                        .chars()
                        .map(|character| Value::String(character.to_string()))
                        .collect::<Vec<_>>(),
                    _ => return Err(error_for(self.locale, "P1010", *position, "for")),
                };
                if values.len() > 1_000_000 {
                    return Err(error_for(
                        self.locale,
                        "P1012",
                        *position,
                        "loop iteration limit",
                    ));
                }
                self.local_scopes.push(HashMap::new());
                let result = (|| {
                    for value in values {
                        self.define_variable(name.clone(), value);
                        self.run_in_scope(body)?;
                        if self.return_value.is_some() {
                            break;
                        }
                    }
                    Ok(())
                })();
                self.local_scopes.pop();
                result?;
            }
            Stmt::Function {
                name, params, body, ..
            } => {
                self.functions
                    .insert(name.clone(), (params.clone(), body.clone()));
            }
            Stmt::Return { value, .. } => {
                self.return_value = Some(match value {
                    Some(expression) => self.evaluate(expression)?,
                    None => Value::Null,
                });
            }
            Stmt::Import {
                path,
                alias,
                position,
            } => match alias {
                Some(alias) => self.import_module_as(path, alias, *position)?,
                None => self.import_module(path, *position)?,
            },
            Stmt::Export(statement) => self.execute(statement)?,
        }
        Ok(())
    }

    fn import_module(&mut self, requested: &str, position: Position) -> Result<(), PadmaError> {
        let relative_path = resolve_import_path(&self.current_source, requested)
            .map_err(|_| error_for(self.locale, "P1022", position, requested))?;
        let path = fs::canonicalize(&relative_path)
            .map_err(|_| error_for(self.locale, "P1023", position, requested))?;
        if self.loaded_modules.contains(&path) {
            return Ok(());
        }
        if !self.active_modules.insert(path.clone()) {
            return Err(error_for(self.locale, "P1024", position, requested));
        }

        let result = (|| {
            let source = fs::read_to_string(&path)
                .map_err(|_| error_for(self.locale, "P1023", position, requested))?;
            let module_locale = Locale::from_source(&source);
            let (program, module_locale) = compile(&source).map_err(|error| {
                error.with_source_context(path.clone(), source.clone(), module_locale)
            })?;
            let previous_source = std::mem::replace(&mut self.current_source, path.clone());
            let previous_locale = self.locale;
            self.locale = module_locale;
            let run_result = self.run(&program).map_err(|error| {
                error.with_source_context(path.clone(), source.clone(), module_locale)
            });
            self.current_source = previous_source;
            self.locale = previous_locale;
            run_result
        })();

        self.active_modules.remove(&path);
        if result.is_ok() {
            self.loaded_modules.insert(path);
        }
        result
    }

    fn import_module_as(
        &mut self,
        requested: &str,
        alias: &str,
        position: Position,
    ) -> Result<(), PadmaError> {
        let relative_path = resolve_import_path(&self.current_source, requested)
            .map_err(|_| error_for(self.locale, "P1022", position, requested))?;
        let path = fs::canonicalize(&relative_path)
            .map_err(|_| error_for(self.locale, "P1023", position, requested))?;
        if let Some(namespace) = self.module_cache.get(&path).cloned() {
            self.modules.insert(alias.to_string(), namespace);
            return Ok(());
        }
        if !self.active_modules.insert(path.clone()) {
            return Err(error_for(self.locale, "P1024", position, requested));
        }
        let result = (|| {
            let source = fs::read_to_string(&path)
                .map_err(|_| error_for(self.locale, "P1023", position, requested))?;
            let module_locale = Locale::from_source(&source);
            let (program, module_locale) = compile(&source).map_err(|error| {
                error.with_source_context(path.clone(), source.clone(), module_locale)
            })?;
            let (public_values, public_functions) = exported_symbol_names(&program);
            let has_explicit_exports = !public_values.is_empty() || !public_functions.is_empty();
            let mut child = Interpreter::with_source_path(module_locale, path.clone());
            child.active_modules = self.active_modules.clone();
            child.project_capabilities = self.project_capabilities.clone();
            child.project_root = self.project_root.clone();
            child.run(&program).map_err(|error| {
                error.with_source_context(path.clone(), source.clone(), module_locale)
            })?;
            self.output.extend(child.output);
            if has_explicit_exports {
                child
                    .environment
                    .retain(|name, _| public_values.contains(name));
                child
                    .functions
                    .retain(|name, _| public_functions.contains(name));
            }
            let namespace = ModuleNamespace {
                environment: child.environment,
                functions: child.functions,
                modules: child.modules,
            };
            self.module_cache.insert(path.clone(), namespace.clone());
            self.modules.insert(alias.to_string(), namespace);
            Ok(())
        })();
        self.active_modules.remove(&path);
        result
    }

    fn evaluate(&mut self, expression: &Expr) -> Result<Value, PadmaError> {
        match expression {
            Expr::Literal(Value::String(value), position) => {
                Ok(Value::String(self.interpolate(value, *position)?))
            }
            Expr::Literal(value, _) => Ok(value.clone()),
            Expr::Variable(name, position) => {
                if let Some((module, member)) = name.split_once('.') {
                    if let Some(namespace) = self.modules.get(module) {
                        return namespace
                            .environment
                            .get(member)
                            .cloned()
                            .ok_or_else(|| error_for(self.locale, "P1007", *position, name));
                    }
                }
                self.variable(name)
                    .cloned()
                    .ok_or_else(|| error_for(self.locale, "P1007", *position, name))
            }
            Expr::Index {
                target,
                index,
                position,
            } => {
                let target = self.evaluate(target)?;
                let index = self.evaluate(index)?;
                match target {
                    Value::List(values) => {
                        let index = expect_collection_index(&index, self.locale, *position)?;
                        values.get(index).cloned().ok_or_else(|| {
                            error_for(self.locale, "P1027", *position, &index.to_string())
                        })
                    }
                    Value::Map(values) => {
                        let key = expect_map_key(&index, self.locale, *position)?;
                        values
                            .get(key)
                            .cloned()
                            .ok_or_else(|| error_for(self.locale, "P1021", *position, key))
                    }
                    _ => Err(error_for(self.locale, "P1010", *position, "index")),
                }
            }
            Expr::Slice {
                target,
                start,
                end,
                position,
            } => {
                let target = self.evaluate(target)?;
                let Value::List(values) = target else {
                    return Err(error_for(self.locale, "P1010", *position, "slice"));
                };
                let start = match start {
                    Some(expression) => {
                        let value = self.evaluate(expression)?;
                        expect_collection_index(&value, self.locale, *position)?
                    }
                    None => 0,
                };
                let end = match end {
                    Some(expression) => {
                        let value = self.evaluate(expression)?;
                        expect_collection_index(&value, self.locale, *position)?
                    }
                    None => values.len(),
                };
                if start > end || end > values.len() {
                    return Err(error_for(
                        self.locale,
                        "P1027",
                        *position,
                        &format!("{start}:{end}"),
                    ));
                }
                Ok(Value::List(values[start..end].to_vec()))
            }
            Expr::Unary {
                operator,
                right,
                position,
            } => match (operator, self.evaluate(right)?) {
                (TokenKind::Minus, Value::Number(value)) => Ok(Value::Number(-value)),
                (TokenKind::Minus, _) => Err(error_for(self.locale, "P1010", *position, "-")),
                _ => unreachable!("parser only creates supported unary expressions"),
            },
            Expr::Binary {
                left,
                operator,
                right,
                position,
            } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;
                self.binary(left, operator, right, *position)
            }
            Expr::Call {
                name,
                arguments,
                position,
            } => {
                if let Some((module, member)) = name.split_once('.') {
                    if let Some(namespace) = self.modules.get(module).cloned() {
                        let (parameters, body) = namespace
                            .functions
                            .get(member)
                            .cloned()
                            .ok_or_else(|| error_for(self.locale, "P1008", *position, name))?;
                        if parameters.len() != arguments.len() {
                            return Err(error_for(self.locale, "P1009", *position, name));
                        }
                        let values = arguments
                            .iter()
                            .map(|argument| self.evaluate(argument))
                            .collect::<Result<Vec<_>, _>>()?;
                        let previous_environment =
                            std::mem::replace(&mut self.environment, namespace.environment);
                        let previous_local_scopes = std::mem::take(&mut self.local_scopes);
                        let previous_functions =
                            std::mem::replace(&mut self.functions, namespace.functions);
                        let previous_modules =
                            std::mem::replace(&mut self.modules, namespace.modules);
                        let previous_return = self.return_value.take();
                        for (parameter, value) in parameters.iter().zip(values) {
                            self.environment.insert(parameter.clone(), value);
                        }
                        let run_result = self.run(&body);
                        let result = self.return_value.take().unwrap_or(Value::Null);
                        let updated_namespace = ModuleNamespace {
                            environment: std::mem::replace(
                                &mut self.environment,
                                previous_environment,
                            ),
                            functions: std::mem::replace(&mut self.functions, previous_functions),
                            modules: std::mem::replace(&mut self.modules, previous_modules),
                        };
                        self.modules.insert(module.to_string(), updated_namespace);
                        self.local_scopes = previous_local_scopes;
                        self.return_value = previous_return;
                        run_result?;
                        return Ok(result);
                    }
                }
                if name == "range" || name == "পরিসর" {
                    if arguments.len() != 1 && arguments.len() != 2 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let values = arguments
                        .iter()
                        .map(|argument| self.evaluate(argument))
                        .collect::<Result<Vec<_>, _>>()?;
                    let (start, end) = if values.len() == 1 {
                        (
                            0,
                            expect_collection_index(&values[0], self.locale, *position)?,
                        )
                    } else {
                        (
                            expect_collection_index(&values[0], self.locale, *position)?,
                            expect_collection_index(&values[1], self.locale, *position)?,
                        )
                    };
                    if end < start {
                        return Err(error_for(
                            self.locale,
                            "P1027",
                            *position,
                            &format!("{start}:{end}"),
                        ));
                    }
                    let length = end - start;
                    if length > 1_000_000 {
                        return Err(error_for(
                            self.locale,
                            "P1012",
                            *position,
                            "range iteration limit",
                        ));
                    }
                    return Ok(Value::List(
                        (start..end)
                            .map(|number| Value::Number(number as f64))
                            .collect(),
                    ));
                }
                if matches!(
                    name.as_str(),
                    "text.len" | "text.trim" | "text.upper" | "text.lower"
                ) {
                    if arguments.len() != 1 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let value = self.evaluate(&arguments[0])?;
                    let text = expect_string(&value, self.locale, *position, "text")?;
                    return Ok(match name.as_str() {
                        "text.len" => Value::Number(text.chars().count() as f64),
                        "text.trim" => Value::String(text.trim().to_string()),
                        "text.upper" => Value::String(text.to_uppercase()),
                        "text.lower" => Value::String(text.to_lowercase()),
                        _ => unreachable!("checked text builtin"),
                    });
                }
                if matches!(
                    name.as_str(),
                    "text.contains" | "text.split" | "text.replace"
                ) {
                    let expected = if name == "text.replace" { 3 } else { 2 };
                    if arguments.len() != expected {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let values = arguments
                        .iter()
                        .map(|argument| self.evaluate(argument))
                        .collect::<Result<Vec<_>, _>>()?;
                    let text = expect_string(&values[0], self.locale, *position, "text")?;
                    let search = expect_string(&values[1], self.locale, *position, "text")?;
                    return Ok(match name.as_str() {
                        "text.contains" => Value::Boolean(text.contains(search)),
                        "text.split" => Value::List(
                            text.split(search)
                                .map(|part| Value::String(part.to_string()))
                                .collect(),
                        ),
                        "text.replace" => Value::String(text.replace(
                            search,
                            expect_string(&values[2], self.locale, *position, "text")?,
                        )),
                        _ => unreachable!("checked text builtin"),
                    });
                }
                if name == "text.join" {
                    if arguments.len() != 2 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let values = arguments
                        .iter()
                        .map(|argument| self.evaluate(argument))
                        .collect::<Result<Vec<_>, _>>()?;
                    let Value::List(items) = &values[0] else {
                        return Err(error_for(self.locale, "P1010", *position, "text.join"));
                    };
                    let separator = expect_string(&values[1], self.locale, *position, "separator")?;
                    let joined = items
                        .iter()
                        .map(|item| {
                            expect_string(item, self.locale, *position, "text").map(str::to_string)
                        })
                        .collect::<Result<Vec<_>, _>>()?
                        .join(separator);
                    return Ok(Value::String(joined));
                }
                if name == "text.format" {
                    if arguments.len() != 2 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let template = match &arguments[0] {
                        Expr::Literal(Value::String(text), _) => Value::String(text.clone()),
                        argument => self.evaluate(argument)?,
                    };
                    let values = self.evaluate(&arguments[1])?;
                    let template = expect_string(&template, self.locale, *position, "template")?;
                    let Value::Map(values) = values else {
                        return Err(error_for(self.locale, "P1010", *position, "text.format"));
                    };
                    return format_from_map(template, &values, self.locale, *position)
                        .map(Value::String);
                }
                if name == "path.basename" || name == "path.extension" {
                    if arguments.len() != 1 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let value = self.evaluate(&arguments[0])?;
                    let path = expect_string(&value, self.locale, *position, "path")?;
                    let path = safe_relative_path(path)
                        .map_err(|_| error_for(self.locale, "P1014", *position, path))?;
                    let value = if name == "path.basename" {
                        path.file_name()
                            .and_then(|part| part.to_str())
                            .map(str::to_string)
                            .ok_or_else(|| error_for(self.locale, "P1014", *position, "path"))?
                    } else {
                        path.extension()
                            .and_then(|part| part.to_str())
                            .map(str::to_string)
                            .unwrap_or_default()
                    };
                    return Ok(Value::String(value));
                }
                if name == "path.join" {
                    if arguments.is_empty() {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let mut result = PathBuf::new();
                    for argument in arguments {
                        let value = self.evaluate(argument)?;
                        let part = expect_string(&value, self.locale, *position, "path")?;
                        let part = safe_relative_path(part)
                            .map_err(|_| error_for(self.locale, "P1014", *position, part))?;
                        result.push(part);
                    }
                    let normalized = safe_relative_path(&result.to_string_lossy())
                        .map_err(|_| error_for(self.locale, "P1014", *position, "path"))?;
                    return Ok(Value::String(
                        normalized.to_string_lossy().replace('\\', "/"),
                    ));
                }
                if name == "random.int" {
                    if arguments.len() != 2 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let start = self.evaluate(&arguments[0])?;
                    let end = self.evaluate(&arguments[1])?;
                    let start = expect_number(&start, self.locale, *position, "random.int")?;
                    let end = expect_number(&end, self.locale, *position, "random.int")?;
                    if start.fract() != 0.0 || end.fract() != 0.0 || start >= end {
                        return Err(error_for(self.locale, "P1010", *position, "random.int"));
                    }
                    let start = start as i64;
                    let end = end as i64;
                    let span = end
                        .checked_sub(start)
                        .ok_or_else(|| error_for(self.locale, "P1010", *position, "random.int"))?;
                    if span > 1_000_000_000 {
                        return Err(error_for(
                            self.locale,
                            "P1012",
                            *position,
                            "random range limit",
                        ));
                    }
                    return Ok(Value::Number(
                        start as f64 + (next_non_cryptographic_random() % span as u64) as f64,
                    ));
                }
                if name == "random.pick" {
                    if arguments.len() != 1 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let value = self.evaluate(&arguments[0])?;
                    let Value::List(items) = value else {
                        return Err(error_for(self.locale, "P1010", *position, "random.pick"));
                    };
                    if items.is_empty() || items.len() > 1_000_000 {
                        return Err(error_for(
                            self.locale,
                            "P1012",
                            *position,
                            "random pick limit",
                        ));
                    }
                    let index = (next_non_cryptographic_random() % items.len() as u64) as usize;
                    return Ok(items[index].clone());
                }
                if matches!(
                    name.as_str(),
                    "math.abs" | "math.round" | "math.floor" | "math.ceil"
                ) {
                    if arguments.len() != 1 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let value = self.evaluate(&arguments[0])?;
                    let number = expect_number(&value, self.locale, *position, name)?;
                    return Ok(Value::Number(match name.as_str() {
                        "math.abs" => number.abs(),
                        "math.round" => number.round(),
                        "math.floor" => number.floor(),
                        "math.ceil" => number.ceil(),
                        _ => unreachable!("checked math builtin"),
                    }));
                }
                if name == "math.min" || name == "math.max" {
                    if arguments.is_empty() {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let values = arguments
                        .iter()
                        .map(|argument| self.evaluate(argument))
                        .collect::<Result<Vec<_>, _>>()?;
                    let mut numbers = values
                        .iter()
                        .map(|value| expect_number(value, self.locale, *position, name))
                        .collect::<Result<Vec<_>, _>>()?
                        .into_iter();
                    let first = numbers.next().expect("non-empty arguments checked above");
                    let result = if name == "math.min" {
                        numbers.fold(first, f64::min)
                    } else {
                        numbers.fold(first, f64::max)
                    };
                    return Ok(Value::Number(result));
                }
                if name == "time.now" {
                    if !arguments.is_empty() {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let seconds = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map_err(|_| error_for(self.locale, "P1010", *position, "time.now"))?
                        .as_secs_f64();
                    return Ok(Value::Number(seconds));
                }
                if name == "json.parse" {
                    if arguments.len() != 1 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let value = match &arguments[0] {
                        Expr::Literal(Value::String(text), _) => Value::String(text.clone()),
                        argument => self.evaluate(argument)?,
                    };
                    let text = expect_string(&value, self.locale, *position, "JSON text")?;
                    let json = serde_json::from_str(text).map_err(|error| {
                        error_for(self.locale, "P1029", *position, &error.to_string())
                    })?;
                    return value_from_json(json)
                        .map_err(|detail| error_for(self.locale, "P1029", *position, &detail));
                }
                if name == "json.stringify" {
                    if arguments.len() != 1 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let value = self.evaluate(&arguments[0])?;
                    let json = value_to_json(&value)
                        .map_err(|detail| error_for(self.locale, "P1029", *position, &detail))?;
                    return serde_json::to_string(&json)
                        .map(Value::String)
                        .map_err(|error| {
                            error_for(self.locale, "P1029", *position, &error.to_string())
                        });
                }
                if name == "url.is_valid" || name == "url.parse" {
                    if arguments.len() != 1 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let value = self.evaluate(&arguments[0])?;
                    let text = expect_string(&value, self.locale, *position, "URL text")?;
                    let parsed = parse_http_url(text);
                    if name == "url.is_valid" {
                        return Ok(Value::Boolean(parsed.is_ok()));
                    }
                    let parsed =
                        parsed.map_err(|_| error_for(self.locale, "P1030", *position, text))?;
                    let mut result = BTreeMap::new();
                    result.insert("url".into(), Value::String(parsed.normalized));
                    result.insert("scheme".into(), Value::String(parsed.scheme));
                    result.insert("host".into(), Value::String(parsed.host));
                    result.insert("path".into(), Value::String(parsed.path));
                    result.insert(
                        "query".into(),
                        parsed.query.map(Value::String).unwrap_or(Value::Null),
                    );
                    result.insert(
                        "fragment".into(),
                        parsed.fragment.map(Value::String).unwrap_or(Value::Null),
                    );
                    result.insert(
                        "port".into(),
                        parsed
                            .port
                            .map(|port| Value::Number(port as f64))
                            .unwrap_or(Value::Null),
                    );
                    return Ok(Value::Map(result));
                }
                if name == "time.sleep" {
                    if arguments.len() != 1 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let value = self.evaluate(&arguments[0])?;
                    let seconds = expect_number(&value, self.locale, *position, "time.sleep")?;
                    if !(0.0..=60.0).contains(&seconds) {
                        return Err(error_for(self.locale, "P1012", *position, "sleep limit"));
                    }
                    std::thread::sleep(Duration::from_secs_f64(seconds));
                    return Ok(Value::Null);
                }
                if name == "file.read" || name == "file.exists" {
                    if arguments.len() != 1 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let value = self.evaluate(&arguments[0])?;
                    let path = expect_string(&value, self.locale, *position, "path")?;
                    self.require_capability("filesystem:read", name, *position)?;
                    let resolved_path = self
                        .resolve_file_path(path)
                        .map_err(|_| error_for(self.locale, "P1014", *position, path))?;
                    if name == "file.exists" {
                        return Ok(Value::Boolean(resolved_path.is_file()));
                    }
                    let contents = fs::read_to_string(&resolved_path)
                        .map_err(|_| error_for(self.locale, "P1028", *position, path))?;
                    return Ok(Value::String(contents));
                }
                if name == "file.write" {
                    let values = arguments
                        .iter()
                        .map(|argument| self.evaluate(argument))
                        .collect::<Result<Vec<_>, _>>()?;
                    if values.len() != 2 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let path = match &values[0] {
                        Value::String(value) => value,
                        _ => return Err(error_for(self.locale, "P1010", *position, "path")),
                    };
                    let content = match &values[1] {
                        Value::String(value) => value,
                        _ => return Err(error_for(self.locale, "P1010", *position, "content")),
                    };
                    self.require_capability("filesystem:write", name, *position)?;
                    let resolved_path = self
                        .resolve_file_path(path)
                        .map_err(|_| error_for(self.locale, "P1014", *position, path))?;
                    fs::write(&resolved_path, content)
                        .map_err(|_| error_for(self.locale, "P1015", *position, path))?;
                    return Ok(Value::Boolean(true));
                }
                if name == "table.read" {
                    if arguments.len() != 2 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let path = self.evaluate(&arguments[0])?;
                    let path = expect_string(&path, self.locale, *position, "table path")?;
                    let format = self.evaluate(&arguments[1])?;
                    let format = expect_string(&format, self.locale, *position, "table format")?;
                    if !matches!(format, "csv" | "tsv" | "json") {
                        return Err(table_error(
                            self.locale,
                            *position,
                            "table format must be csv, tsv, or json",
                        ));
                    }
                    self.require_capability("filesystem:read", name, *position)?;
                    let resolved_path = self
                        .resolve_file_path(path)
                        .map_err(|_| error_for(self.locale, "P1014", *position, "table path"))?;
                    let metadata = fs::metadata(&resolved_path)
                        .map_err(|_| error_for(self.locale, "P1028", *position, "table path"))?;
                    if !metadata.is_file() || metadata.len() > TABLE_MAX_BYTES as u64 {
                        return Err(table_error(
                            self.locale,
                            *position,
                            "table file is missing or exceeds the byte limit",
                        ));
                    }
                    let source = fs::read_to_string(&resolved_path).map_err(|_| {
                        table_error(self.locale, *position, "table file must be UTF-8 text")
                    })?;
                    if source.len() > TABLE_MAX_BYTES {
                        return Err(table_error(
                            self.locale,
                            *position,
                            "table file exceeds the byte limit",
                        ));
                    }
                    let table = match format {
                        "csv" | "tsv" => {
                            table_data_from_delimited_text(&source, format, self.locale, *position)?
                        }
                        "json" => table_data_from_json(&source, self.locale, *position)?,
                        _ => unreachable!(),
                    };
                    return Ok(table_data_to_value(&table));
                }
                if name == "table.headers" || name == "table.rows" {
                    if arguments.len() != 1 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let value = self.evaluate(&arguments[0])?;
                    let table = table_data_from_value(&value, self.locale, *position)?;
                    if name == "table.headers" {
                        return Ok(Value::List(
                            table.headers.into_iter().map(Value::String).collect(),
                        ));
                    }
                    return Ok(Value::List(
                        table
                            .rows
                            .into_iter()
                            .map(|row| {
                                Value::Map(
                                    row.into_iter()
                                        .map(|(key, value)| (key, Value::String(value)))
                                        .collect(),
                                )
                            })
                            .collect(),
                    ));
                }
                if name == "table.filter_equal" {
                    if arguments.len() != 3 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let value = self.evaluate(&arguments[0])?;
                    let mut table = table_data_from_value(&value, self.locale, *position)?;
                    let column = self.evaluate(&arguments[1])?;
                    let column = expect_string(&column, self.locale, *position, "table column")?;
                    let expected = self.evaluate(&arguments[2])?;
                    let expected = expect_string(&expected, self.locale, *position, "table cell")?;
                    if !table.headers.iter().any(|header| header == column) {
                        return Err(table_error(
                            self.locale,
                            *position,
                            "table column is not declared",
                        ));
                    }
                    table
                        .rows
                        .retain(|row| row.get(column) == Some(&expected.to_string()));
                    return Ok(table_data_to_value(&table));
                }
                if name == "table.select" {
                    if arguments.len() != 2 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let value = self.evaluate(&arguments[0])?;
                    let table = table_data_from_value(&value, self.locale, *position)?;
                    let requested = self.evaluate(&arguments[1])?;
                    let Value::List(requested) = requested else {
                        return Err(table_error(
                            self.locale,
                            *position,
                            "selected table columns must be a text list",
                        ));
                    };
                    if requested.is_empty() || requested.len() > TABLE_MAX_COLUMNS {
                        return Err(table_error(
                            self.locale,
                            *position,
                            "selected column count is outside the table limit",
                        ));
                    }
                    let mut selected = Vec::with_capacity(requested.len());
                    let mut selected_set = BTreeSet::new();
                    for entry in requested {
                        let Value::String(column) = entry else {
                            return Err(table_error(
                                self.locale,
                                *position,
                                "selected table columns must be text",
                            ));
                        };
                        if !table.headers.contains(&column) || !selected_set.insert(column.clone())
                        {
                            return Err(table_error(
                                self.locale,
                                *position,
                                "selected columns must be declared and unique",
                            ));
                        }
                        selected.push(column);
                    }
                    let rows = table
                        .rows
                        .iter()
                        .map(|row| {
                            selected
                                .iter()
                                .map(|column| {
                                    (column.clone(), row.get(column).cloned().unwrap_or_default())
                                })
                                .collect()
                        })
                        .collect();
                    return Ok(table_data_to_value(&TableData {
                        format: table.format,
                        headers: selected,
                        rows,
                    }));
                }
                if name == "table.count_by" {
                    if arguments.len() != 2 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let value = self.evaluate(&arguments[0])?;
                    let table = table_data_from_value(&value, self.locale, *position)?;
                    let column = self.evaluate(&arguments[1])?;
                    let column = expect_string(&column, self.locale, *position, "table column")?;
                    if !table.headers.iter().any(|header| header == column) {
                        return Err(table_error(
                            self.locale,
                            *position,
                            "table column is not declared",
                        ));
                    }
                    let mut counts: BTreeMap<String, f64> = BTreeMap::new();
                    for row in table.rows {
                        let key = row.get(column).cloned().unwrap_or_default();
                        let next = counts.get(&key).copied().unwrap_or(0.0) + 1.0;
                        counts.insert(key, next);
                    }
                    return Ok(Value::Map(
                        counts
                            .into_iter()
                            .map(|(key, value)| (key, Value::Number(value)))
                            .collect(),
                    ));
                }
                if name == "table.write_csv" {
                    if arguments.len() != 2 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let path = self.evaluate(&arguments[0])?;
                    let path = expect_string(&path, self.locale, *position, "table output path")?;
                    if !path.ends_with(".csv") {
                        return Err(table_error(
                            self.locale,
                            *position,
                            "CSV output path must end with .csv",
                        ));
                    }
                    let value = self.evaluate(&arguments[1])?;
                    let table = table_data_from_value(&value, self.locale, *position)?;
                    let output = table_data_to_csv(&table);
                    if output.len() > TABLE_MAX_BYTES {
                        return Err(table_error(
                            self.locale,
                            *position,
                            "CSV output exceeds the byte limit",
                        ));
                    }
                    self.require_capability("filesystem:write", name, *position)?;
                    let resolved_path = self.resolve_file_path(path).map_err(|_| {
                        error_for(self.locale, "P1014", *position, "table output path")
                    })?;
                    fs::write(&resolved_path, output).map_err(|_| {
                        error_for(self.locale, "P1015", *position, "table output path")
                    })?;
                    return Ok(Value::Boolean(true));
                }
                if name == "http.get" {
                    if arguments.len() != 1 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let url = self.evaluate(&arguments[0])?;
                    let url = expect_string(&url, self.locale, *position, "url")?;
                    self.require_capability("network:http", name, *position)?;
                    if !safe_http_url(url) {
                        return Err(error_for(self.locale, "P1016", *position, url));
                    }
                    let result = process::Command::new("curl")
                        .args([
                            "--fail",
                            "--silent",
                            "--show-error",
                            "--location",
                            "--max-time",
                            "30",
                            "--",
                            url,
                        ])
                        .output()
                        .map_err(|_| error_for(self.locale, "P1018", *position, "curl"))?;
                    if !result.status.success() {
                        return Err(error_for(self.locale, "P1019", *position, "curl"));
                    }
                    return Ok(Value::String(
                        String::from_utf8_lossy(&result.stdout).to_string(),
                    ));
                }
                if name == "http.post" || name == "http.json" {
                    let values = arguments
                        .iter()
                        .map(|argument| self.evaluate(argument))
                        .collect::<Result<Vec<_>, _>>()?;
                    if values.len() != 2 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let url = expect_string(&values[0], self.locale, *position, "url")?;
                    if !safe_http_url(url) {
                        return Err(error_for(self.locale, "P1016", *position, url));
                    }
                    self.require_capability("network:http", name, *position)?;
                    let json =
                        serde_json::to_string(&value_to_json(&values[1]).map_err(|detail| {
                            error_for(self.locale, "P1029", *position, &detail)
                        })?)
                        .map_err(|_| error_for(self.locale, "P1029", *position, "request data"))?;
                    if json.len() > 262_144 {
                        return Err(error_for(self.locale, "P1037", *position, "http request"));
                    }
                    let result = process::Command::new("curl")
                        .args([
                            "--fail",
                            "--silent",
                            "--show-error",
                            "--location",
                            "--max-time",
                            "30",
                            "--max-filesize",
                            "262144",
                            "--request",
                            "POST",
                            "--header",
                            "Content-Type: application/json",
                            "--data-binary",
                            &json,
                            "--",
                            url,
                        ])
                        .output()
                        .map_err(|_| error_for(self.locale, "P1018", *position, "curl"))?;
                    if !result.status.success() || result.stdout.len() > 262_144 {
                        return Err(error_for(self.locale, "P1019", *position, "curl"));
                    }
                    if name == "http.json" {
                        let json: JsonValue =
                            serde_json::from_slice(&result.stdout).map_err(|_| {
                                error_for(self.locale, "P1029", *position, "http.json response")
                            })?;
                        return value_from_json(json).map_err(|_| {
                            error_for(self.locale, "P1029", *position, "http.json response")
                        });
                    }
                    return Ok(Value::String(
                        String::from_utf8_lossy(&result.stdout).to_string(),
                    ));
                }
                if name == "auth.password_hash" {
                    if arguments.len() != 1 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    self.require_project_capability("identity:local", name, *position)?;
                    if matches!(&arguments[0], Expr::Literal(Value::String(_), _)) {
                        return Err(error_for(
                            self.locale,
                            "P1045",
                            *position,
                            "password literal",
                        ));
                    }
                    let password = self.evaluate(&arguments[0])?;
                    let password = expect_string(&password, self.locale, *position, "password")?;
                    return Ok(Value::String(password_record_from_secret(
                        password,
                        self.locale,
                        *position,
                    )?));
                }
                if name == "auth.password_verify" {
                    if arguments.len() != 2 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    self.require_project_capability("identity:local", name, *position)?;
                    if matches!(&arguments[1], Expr::Literal(Value::String(_), _)) {
                        return Err(error_for(
                            self.locale,
                            "P1045",
                            *position,
                            "password literal",
                        ));
                    }
                    let record = self.evaluate(&arguments[0])?;
                    let record = expect_string(&record, self.locale, *position, "password record")?;
                    let password = self.evaluate(&arguments[1])?;
                    let password = expect_string(&password, self.locale, *position, "password")?;
                    return Ok(Value::Boolean(verify_password_record(record, password)));
                }
                if name == "auth.session_issue" {
                    if arguments.len() != 3 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    self.require_project_capability("identity:local", name, *position)?;
                    let subject = self.evaluate(&arguments[0])?;
                    let subject =
                        expect_string(&subject, self.locale, *position, "session subject")?;
                    let environment_name = self.evaluate(&arguments[1])?;
                    let environment_name = expect_string(
                        &environment_name,
                        self.locale,
                        *position,
                        "session secret environment name",
                    )?;
                    let ttl = self.evaluate(&arguments[2])?;
                    let Value::Number(ttl) = ttl else {
                        return Err(error_for(
                            self.locale,
                            "P1010",
                            *position,
                            "session lifetime",
                        ));
                    };
                    if ttl.fract() != 0.0 || ttl < 0.0 {
                        return Err(error_for(
                            self.locale,
                            "P1045",
                            *position,
                            "session lifetime",
                        ));
                    }
                    let secret =
                        session_secret_from_environment(environment_name, self.locale, *position)?;
                    return Ok(Value::String(issue_signed_session(
                        subject,
                        &secret,
                        ttl as u64,
                        self.locale,
                        *position,
                    )?));
                }
                if name == "auth.session_verify" {
                    if arguments.len() != 2 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    self.require_project_capability("identity:local", name, *position)?;
                    let environment_name = self.evaluate(&arguments[0])?;
                    let environment_name = expect_string(
                        &environment_name,
                        self.locale,
                        *position,
                        "session secret environment name",
                    )?;
                    let token = self.evaluate(&arguments[1])?;
                    let token = expect_string(&token, self.locale, *position, "session token")?;
                    let secret =
                        session_secret_from_environment(environment_name, self.locale, *position)?;
                    let (subject, expires_at) =
                        verify_signed_session(token, &secret).ok_or_else(|| {
                            error_for(self.locale, "P1046", *position, "session token")
                        })?;
                    let mut session = BTreeMap::new();
                    session.insert("subject".to_string(), Value::String(subject));
                    session.insert("expires_at".to_string(), Value::Number(expires_at as f64));
                    session.insert("version".to_string(), Value::Number(1.0));
                    return Ok(Value::Map(session));
                }
                if name == "auth.csrf_token" {
                    if !arguments.is_empty() {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    self.require_project_capability("identity:local", name, *position)?;
                    let token = secure_random_bytes(32)
                        .map_err(|_| error_for(self.locale, "P1047", *position, "CSRF token"))?;
                    return Ok(Value::String(hex_encode(&token)));
                }
                if name == "auth.csrf_verify" {
                    if arguments.len() != 2 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    self.require_project_capability("identity:local", name, *position)?;
                    let expected = self.evaluate(&arguments[0])?;
                    let expected = expect_string(&expected, self.locale, *position, "CSRF token")?;
                    let provided = self.evaluate(&arguments[1])?;
                    let provided = expect_string(&provided, self.locale, *position, "CSRF token")?;
                    if expected.len() > 128 || provided.len() > 128 {
                        return Err(error_for(
                            self.locale,
                            "P1045",
                            *position,
                            "CSRF token size",
                        ));
                    }
                    return Ok(Value::Boolean(constant_time_eq(
                        expected.as_bytes(),
                        provided.as_bytes(),
                    )));
                }
                if name == "auth.cookie" {
                    if arguments.len() != 3 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    self.require_project_capability("identity:local", name, *position)?;
                    let cookie_name = self.evaluate(&arguments[0])?;
                    let cookie_name =
                        expect_string(&cookie_name, self.locale, *position, "cookie name")?;
                    let token = self.evaluate(&arguments[1])?;
                    let token = expect_string(&token, self.locale, *position, "session token")?;
                    let environment_name = self.evaluate(&arguments[2])?;
                    let environment_name = expect_string(
                        &environment_name,
                        self.locale,
                        *position,
                        "session secret environment name",
                    )?;
                    if !is_safe_cookie_name(cookie_name) || token.contains(['\r', '\n', ';']) {
                        return Err(error_for(self.locale, "P1045", *position, "cookie"));
                    }
                    let secret =
                        session_secret_from_environment(environment_name, self.locale, *position)?;
                    let (_, expires_at) =
                        verify_signed_session(token, &secret).ok_or_else(|| {
                            error_for(self.locale, "P1046", *position, "session token")
                        })?;
                    let now = unix_seconds()
                        .map_err(|_| error_for(self.locale, "P1047", *position, "system clock"))?;
                    return Ok(Value::String(format!(
                        "{cookie_name}={token}; Path=/; Max-Age={}; HttpOnly; Secure; SameSite=Strict",
                        expires_at.saturating_sub(now)
                    )));
                }
                if name == "db.version" {
                    if arguments.len() != 1 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let database = self.evaluate(&arguments[0])?;
                    let database =
                        expect_string(&database, self.locale, *position, "database path")?;
                    let output = self.sqlite_execute(
                        database,
                        &[],
                        "SELECT schema_version FROM padma_meta WHERE id = 1;",
                        true,
                        *position,
                    )?;
                    let version = serde_json::from_slice::<JsonValue>(&output)
                        .ok()
                        .and_then(|rows| rows.as_array().cloned())
                        .and_then(|rows| rows.into_iter().next())
                        .and_then(|row| row.get("schema_version").and_then(JsonValue::as_i64))
                        .filter(|version| *version == 1)
                        .ok_or_else(|| {
                            error_for(self.locale, "P1042", *position, "schema version")
                        })?;
                    return Ok(Value::Number(version as f64));
                }
                if name == "db.apply" {
                    if arguments.len() != 2 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let database = self.evaluate(&arguments[0])?;
                    let database =
                        expect_string(&database, self.locale, *position, "database path")?;
                    let operations = self.evaluate(&arguments[1])?;
                    let Value::List(operations) = operations else {
                        return Err(error_for(
                            self.locale,
                            "P1010",
                            *position,
                            "db.apply operations",
                        ));
                    };
                    if operations.is_empty() || operations.len() > 32 {
                        return Err(error_for(self.locale, "P1043", *position, "batch size"));
                    }
                    let mut parameters = Vec::new();
                    let mut statements = Vec::with_capacity(operations.len());
                    for (index, operation) in operations.iter().enumerate() {
                        let Value::Map(operation) = operation else {
                            return Err(error_for(
                                self.locale,
                                "P1010",
                                *position,
                                "db.apply operation",
                            ));
                        };
                        let Some(Value::String(kind)) = operation.get("op") else {
                            return Err(error_for(self.locale, "P1010", *position, "db.apply op"));
                        };
                        if !matches!(kind.as_str(), "put" | "delete")
                            || !operation.keys().all(|key| {
                                matches!(key.as_str(), "op" | "namespace" | "key" | "value")
                            })
                        {
                            return Err(error_for(
                                self.locale,
                                "P1010",
                                *position,
                                "db.apply operation",
                            ));
                        }
                        let Some(Value::String(namespace)) = operation.get("namespace") else {
                            return Err(error_for(self.locale, "P1010", *position, "namespace"));
                        };
                        let Some(Value::String(key)) = operation.get("key") else {
                            return Err(error_for(self.locale, "P1010", *position, "record key"));
                        };
                        if namespace.len() > 256 || key.len() > 256 {
                            return Err(error_for(self.locale, "P1043", *position, "batch record"));
                        }
                        let namespace_parameter = format!(":namespace{index}");
                        let key_parameter = format!(":key{index}");
                        parameters.push(sqlite_hex_parameter(
                            &namespace_parameter,
                            namespace.as_bytes(),
                        ));
                        parameters.push(sqlite_hex_parameter(&key_parameter, key.as_bytes()));
                        if kind == "put" {
                            let Some(value) = operation.get("value") else {
                                return Err(error_for(
                                    self.locale,
                                    "P1010",
                                    *position,
                                    "record value",
                                ));
                            };
                            let value =
                                serde_json::to_vec(&value_to_json(value).map_err(|detail| {
                                    error_for(self.locale, "P1029", *position, &detail)
                                })?)
                                .map_err(|_| {
                                    error_for(self.locale, "P1029", *position, "database value")
                                })?;
                            if value.len() > SQLITE_MAX_BYTES {
                                return Err(error_for(
                                    self.locale,
                                    "P1043",
                                    *position,
                                    "batch value",
                                ));
                            }
                            let value_parameter = format!(":value{index}");
                            parameters.push(sqlite_hex_parameter(&value_parameter, &value));
                            statements.push(format!(
                                "INSERT INTO padma_records(namespace, record_key, value_json, updated_at) VALUES(CAST({namespace_parameter} AS TEXT), CAST({key_parameter} AS TEXT), CAST({value_parameter} AS TEXT), CAST(strftime('%s', 'now') AS INTEGER)) ON CONFLICT(namespace, record_key) DO UPDATE SET value_json = excluded.value_json, updated_at = excluded.updated_at;"
                            ));
                        } else if operation.contains_key("value") {
                            return Err(error_for(self.locale, "P1010", *position, "delete value"));
                        } else {
                            statements.push(format!(
                                "DELETE FROM padma_records WHERE namespace = CAST({namespace_parameter} AS TEXT) AND record_key = CAST({key_parameter} AS TEXT);"
                            ));
                        }
                    }
                    let statement = format!("BEGIN IMMEDIATE;\n{}\nCOMMIT;", statements.join("\n"));
                    self.sqlite_execute(database, &parameters, &statement, false, *position)?;
                    return Ok(Value::Boolean(true));
                }
                if matches!(name.as_str(), "db.put" | "db.get" | "db.delete" | "db.list") {
                    let values = arguments
                        .iter()
                        .map(|argument| self.evaluate(argument))
                        .collect::<Result<Vec<_>, _>>()?;
                    let expected = match name.as_str() {
                        "db.put" => 4,
                        "db.get" | "db.delete" => 3,
                        "db.list" => 3,
                        _ => unreachable!(),
                    };
                    if values.len() != expected {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let database =
                        expect_string(&values[0], self.locale, *position, "database path")?;
                    let namespace = expect_string(&values[1], self.locale, *position, "namespace")?;
                    if namespace.len() > 256 {
                        return Err(error_for(self.locale, "P1043", *position, "namespace"));
                    }
                    let namespace_parameter =
                        sqlite_hex_parameter(":namespace", namespace.as_bytes());
                    if name == "db.put" {
                        let key = expect_string(&values[2], self.locale, *position, "record key")?;
                        let value =
                            serde_json::to_vec(&value_to_json(&values[3]).map_err(|detail| {
                                error_for(self.locale, "P1029", *position, &detail)
                            })?)
                            .map_err(|_| {
                                error_for(self.locale, "P1029", *position, "database value")
                            })?;
                        if key.len() > 256 || value.len() > SQLITE_MAX_BYTES {
                            return Err(error_for(self.locale, "P1043", *position, "record"));
                        }
                        self.sqlite_execute(
                            database,
                            &[
                                namespace_parameter,
                                sqlite_hex_parameter(":key", key.as_bytes()),
                                sqlite_hex_parameter(":value", &value),
                            ],
                            "INSERT INTO padma_records(namespace, record_key, value_json, updated_at) VALUES(CAST(:namespace AS TEXT), CAST(:key AS TEXT), CAST(:value AS TEXT), CAST(strftime('%s', 'now') AS INTEGER)) ON CONFLICT(namespace, record_key) DO UPDATE SET value_json = excluded.value_json, updated_at = excluded.updated_at;",
                            false,
                            *position,
                        )?;
                        return Ok(Value::Boolean(true));
                    }
                    if name == "db.list" {
                        let limit = expect_number(&values[2], self.locale, *position, "limit")?;
                        if !(1.0..=100.0).contains(&limit) || limit.fract() != 0.0 {
                            return Err(error_for(self.locale, "P1010", *position, "limit"));
                        }
                        let output = self.sqlite_execute(
                            database,
                            &[
                                namespace_parameter,
                                sqlite_number_parameter(":limit", limit as usize),
                            ],
                            "SELECT record_key AS key, value_json FROM padma_records WHERE namespace = CAST(:namespace AS TEXT) ORDER BY record_key LIMIT :limit;",
                            true,
                            *position,
                        )?;
                        if output.iter().all(u8::is_ascii_whitespace) {
                            return Ok(Value::List(Vec::new()));
                        }
                        let rows: JsonValue = serde_json::from_slice(&output)
                            .map_err(|_| error_for(self.locale, "P1042", *position, "list"))?;
                        let rows = rows
                            .as_array()
                            .ok_or_else(|| error_for(self.locale, "P1042", *position, "list"))?;
                        let mut result = Vec::with_capacity(rows.len());
                        for row in rows {
                            let key =
                                row.get("key").and_then(JsonValue::as_str).ok_or_else(|| {
                                    error_for(self.locale, "P1042", *position, "record key")
                                })?;
                            let value_json = row
                                .get("value_json")
                                .and_then(JsonValue::as_str)
                                .ok_or_else(|| {
                                    error_for(self.locale, "P1042", *position, "record value")
                                })?;
                            let mut item = BTreeMap::new();
                            item.insert("key".into(), Value::String(key.to_string()));
                            item.insert(
                                "value".into(),
                                value_from_json(serde_json::from_str(value_json).map_err(
                                    |_| error_for(self.locale, "P1042", *position, "record value"),
                                )?)
                                .map_err(|_| {
                                    error_for(self.locale, "P1042", *position, "record value")
                                })?,
                            );
                            result.push(Value::Map(item));
                        }
                        return Ok(Value::List(result));
                    }
                    let key = expect_string(&values[2], self.locale, *position, "record key")?;
                    if key.len() > 256 {
                        return Err(error_for(self.locale, "P1043", *position, "record key"));
                    }
                    let output = self.sqlite_execute(
                        database,
                        &[
                            namespace_parameter,
                            sqlite_hex_parameter(":key", key.as_bytes()),
                        ],
                        if name == "db.get" {
                            "SELECT value_json FROM padma_records WHERE namespace = CAST(:namespace AS TEXT) AND record_key = CAST(:key AS TEXT) LIMIT 1;"
                        } else {
                            "DELETE FROM padma_records WHERE namespace = CAST(:namespace AS TEXT) AND record_key = CAST(:key AS TEXT);"
                        },
                        name == "db.get",
                        *position,
                    )?;
                    if name == "db.delete" {
                        return Ok(Value::Boolean(true));
                    }
                    if output.iter().all(u8::is_ascii_whitespace) {
                        return Ok(Value::Null);
                    }
                    let rows: JsonValue = serde_json::from_slice(&output)
                        .map_err(|_| error_for(self.locale, "P1042", *position, "get"))?;
                    let Some(value_json) = rows
                        .as_array()
                        .and_then(|rows| rows.first())
                        .and_then(|row| row.get("value_json"))
                        .and_then(JsonValue::as_str)
                    else {
                        return Ok(Value::Null);
                    };
                    return value_from_json(serde_json::from_str(value_json).map_err(|_| {
                        error_for(self.locale, "P1042", *position, "record value")
                    })?)
                    .map_err(|_| error_for(self.locale, "P1042", *position, "record value"));
                }
                if name == "backend.response" {
                    if arguments.len() != 3 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let status = self.evaluate(&arguments[0])?;
                    let headers = self.evaluate(&arguments[1])?;
                    let body = self.evaluate(&arguments[2])?;
                    let Value::Number(status) = status else {
                        return Err(error_for(self.locale, "P1010", *position, "status"));
                    };
                    if !(100.0..=599.0).contains(&status) || status.fract() != 0.0 {
                        return Err(error_for(self.locale, "P1010", *position, "status"));
                    }
                    let Value::Map(headers) = headers else {
                        return Err(error_for(self.locale, "P1010", *position, "headers"));
                    };
                    let mut result = BTreeMap::new();
                    result.insert("status".into(), Value::Number(status));
                    result.insert("headers".into(), Value::Map(headers));
                    result.insert("body".into(), body);
                    return Ok(Value::Map(result));
                }
                if name == "automation.write_json" {
                    if arguments.len() != 2 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let path = self.evaluate(&arguments[0])?;
                    let path = expect_string(&path, self.locale, *position, "path")?;
                    self.require_capability("filesystem:write", name, *position)?;
                    let value = self.evaluate(&arguments[1])?;
                    let json =
                        serde_json::to_string_pretty(&value_to_json(&value).map_err(|detail| {
                            error_for(self.locale, "P1029", *position, &detail)
                        })?)
                        .map_err(|_| {
                            error_for(self.locale, "P1029", *position, "automation data")
                        })?;
                    let resolved = self
                        .resolve_file_path(path)
                        .map_err(|_| error_for(self.locale, "P1014", *position, path))?;
                    fs::write(resolved, json)
                        .map_err(|_| error_for(self.locale, "P1015", *position, path))?;
                    return Ok(Value::Boolean(true));
                }
                if name == "ai.workflow" {
                    if arguments.len() != 1 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let input = self.evaluate(&arguments[0])?;
                    return self.ai_workflow(&input, *position);
                }
                if name == "ai.request" {
                    if arguments.len() != 3 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let endpoint = self.evaluate(&arguments[0])?;
                    let endpoint = expect_string(&endpoint, self.locale, *position, "endpoint")?;
                    let secret_name = self.evaluate(&arguments[1])?;
                    let secret_name = expect_string(
                        &secret_name,
                        self.locale,
                        *position,
                        "secret environment name",
                    )?;
                    if !secret_name
                        .chars()
                        .next()
                        .is_some_and(|ch| ch.is_ascii_uppercase() || ch == '_')
                        || !secret_name
                            .chars()
                            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
                    {
                        return Err(error_for(
                            self.locale,
                            "P1010",
                            *position,
                            "secret environment name",
                        ));
                    }
                    if !safe_http_url(endpoint) {
                        return Err(error_for(self.locale, "P1016", *position, endpoint));
                    }
                    self.require_capability("network:ai", name, *position)?;
                    let secret = env::var(secret_name).map_err(|_| {
                        error_for(
                            self.locale,
                            "P1013",
                            *position,
                            "AI secret environment variable",
                        )
                    })?;
                    if secret.is_empty() || secret.chars().any(char::is_control) {
                        return Err(error_for(
                            self.locale,
                            "P1013",
                            *position,
                            "AI secret environment variable",
                        ));
                    }
                    let payload = self.evaluate(&arguments[2])?;
                    let payload =
                        serde_json::to_string(&value_to_json(&payload).map_err(|detail| {
                            error_for(self.locale, "P1029", *position, &detail)
                        })?)
                        .map_err(|_| error_for(self.locale, "P1029", *position, "AI payload"))?;
                    if payload.len() > 262_144 {
                        return Err(error_for(self.locale, "P1037", *position, "AI payload"));
                    }
                    let path = env::var_os("PATH")
                        .ok_or_else(|| error_for(self.locale, "P1018", *position, "curl"))?;
                    let result = process::Command::new("curl")
                        .env_clear()
                        .env("PATH", path)
                        .args([
                            "--fail",
                            "--silent",
                            "--show-error",
                            "--location",
                            "--max-time",
                            "30",
                            "--max-filesize",
                            "262144",
                            "--request",
                            "POST",
                            "--header",
                            "Content-Type: application/json",
                            "--header",
                            &format!("Authorization: Bearer {secret}"),
                            "--data-binary",
                            &payload,
                            "--",
                            endpoint,
                        ])
                        .output()
                        .map_err(|_| error_for(self.locale, "P1018", *position, "curl"))?;
                    if !result.status.success() || result.stdout.len() > 262_144 {
                        return Err(error_for(self.locale, "P1019", *position, "AI request"));
                    }
                    let json: JsonValue = serde_json::from_slice(&result.stdout)
                        .map_err(|_| error_for(self.locale, "P1029", *position, "AI response"))?;
                    return value_from_json(json)
                        .map_err(|_| error_for(self.locale, "P1029", *position, "AI response"));
                }
                if name == "bridge.call" {
                    if arguments.len() != 3 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let runtime = self.evaluate(&arguments[0])?;
                    let runtime = expect_string(&runtime, self.locale, *position, "runtime")?;
                    let script_path = self.evaluate(&arguments[1])?;
                    let script_path =
                        expect_string(&script_path, self.locale, *position, "script path")?;
                    let data = self.evaluate(&arguments[2])?;
                    return self.bridge_call(runtime, script_path, &data, *position);
                }
                if name == "process.run" || name == "media.download" {
                    let values = arguments
                        .iter()
                        .map(|argument| self.evaluate(argument))
                        .collect::<Result<Vec<_>, _>>()?;
                    let (program, args) = if name == "media.download" {
                        if values.len() != 1 && values.len() != 2 {
                            return Err(error_for(self.locale, "P1009", *position, name));
                        }
                        let url = expect_string(&values[0], self.locale, *position, "url")?;
                        let output = if values.len() == 2 {
                            expect_string(&values[1], self.locale, *position, "output")?.to_string()
                        } else {
                            "@downloads/%(title)s.%(ext)s".to_string()
                        };
                        if !(url.starts_with("https://") || url.starts_with("http://")) {
                            return Err(error_for(self.locale, "P1016", *position, url));
                        }
                        self.require_capability("media:download", name, *position)?;
                        self.require_capability("filesystem:write", name, *position)?;
                        let resolved_output = self
                            .resolve_file_path(&output)
                            .map_err(|_| error_for(self.locale, "P1014", *position, &output))?;
                        (
                            "yt-dlp".to_string(),
                            vec![
                                "-o".to_string(),
                                resolved_output.to_string_lossy().to_string(),
                                url.to_string(),
                            ],
                        )
                    } else {
                        if values.is_empty() {
                            return Err(error_for(self.locale, "P1009", *position, name));
                        }
                        self.require_capability(
                            &format!(
                                "process:{}",
                                expect_string(&values[0], self.locale, *position, "program")?
                            ),
                            name,
                            *position,
                        )?;
                        let program = expect_string(&values[0], self.locale, *position, "program")?;
                        let args = values[1..]
                            .iter()
                            .map(|value| {
                                expect_string(value, self.locale, *position, "argument")
                                    .map(str::to_string)
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        (program.to_string(), args)
                    };
                    let allowed = ["yt-dlp", "curl", "ffmpeg", "python", "python3"];
                    if self.project_capabilities.is_none() && !allowed.contains(&program.as_str()) {
                        return Err(error_for(self.locale, "P1017", *position, &program));
                    }
                    let result = process::Command::new(&program)
                        .args(&args)
                        .output()
                        .map_err(|_| error_for(self.locale, "P1018", *position, &program))?;
                    if !result.status.success() {
                        return Err(error_for(self.locale, "P1019", *position, &program));
                    }
                    return Ok(Value::String(
                        String::from_utf8_lossy(&result.stdout).trim().to_string(),
                    ));
                }
                if name == "input" {
                    if arguments.len() != 1 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let prompt = self.evaluate(&arguments[0])?;
                    print!("{prompt}");
                    io::stdout()
                        .flush()
                        .map_err(|_| error_for(self.locale, "P1013", *position, "input"))?;
                    let mut line = String::new();
                    io::stdin()
                        .read_line(&mut line)
                        .map_err(|_| error_for(self.locale, "P1013", *position, "input"))?;
                    return Ok(Value::String(
                        line.trim_end_matches(['\r', '\n']).to_string(),
                    ));
                }
                if let Some(list_name) = name.strip_suffix(".get") {
                    if self
                        .variable(list_name)
                        .is_some_and(|value| matches!(value, Value::List(_)))
                    {
                        if arguments.len() != 1 {
                            return Err(error_for(self.locale, "P1009", *position, name));
                        }
                        let index = self.evaluate(&arguments[0])?;
                        let index = expect_collection_index(&index, self.locale, *position)?;
                        let Value::List(values) = self.variable(list_name).unwrap() else {
                            unreachable!("list check above guarantees a list")
                        };
                        return values.get(index).cloned().ok_or_else(|| {
                            error_for(self.locale, "P1027", *position, &index.to_string())
                        });
                    }
                }
                if let Some(list_name) = name.strip_suffix(".set") {
                    if self
                        .variable(list_name)
                        .is_some_and(|value| matches!(value, Value::List(_)))
                    {
                        if arguments.len() != 2 {
                            return Err(error_for(self.locale, "P1009", *position, name));
                        }
                        let index = self.evaluate(&arguments[0])?;
                        let index = expect_collection_index(&index, self.locale, *position)?;
                        let value = self.evaluate(&arguments[1])?;
                        let locale = self.locale;
                        let Value::List(values) = self.variable_mut(list_name).unwrap() else {
                            unreachable!("list check above guarantees a list")
                        };
                        let slot = values.get_mut(index).ok_or_else(|| {
                            error_for(locale, "P1027", *position, &index.to_string())
                        })?;
                        *slot = value;
                        return Ok(Value::Boolean(true));
                    }
                }
                if let Some(list_name) = name.strip_suffix(".push") {
                    if arguments.len() != 1 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let value = self.evaluate(&arguments[0])?;
                    let locale = self.locale;
                    let list = self
                        .variable_mut(list_name)
                        .ok_or_else(|| error_for(locale, "P1007", *position, list_name))?;
                    let Value::List(values) = list else {
                        return Err(error_for(self.locale, "P1010", *position, "list.push"));
                    };
                    values.push(value);
                    return Ok(Value::Boolean(true));
                }
                if let Some(list_name) = name.strip_suffix(".remove") {
                    if arguments.len() != 1 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let index = self.evaluate(&arguments[0])?;
                    let index = expect_collection_index(&index, self.locale, *position)?;
                    let locale = self.locale;
                    let list = self
                        .variable_mut(list_name)
                        .ok_or_else(|| error_for(locale, "P1007", *position, list_name))?;
                    let Value::List(values) = list else {
                        return Err(error_for(self.locale, "P1010", *position, "list.remove"));
                    };
                    if index >= values.len() {
                        return Err(error_for(
                            self.locale,
                            "P1027",
                            *position,
                            &index.to_string(),
                        ));
                    }
                    return Ok(values.remove(index));
                }
                if let Some(collection_name) = name.strip_suffix(".len") {
                    if !arguments.is_empty() {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let collection = self.variable(collection_name).ok_or_else(|| {
                        error_for(self.locale, "P1007", *position, collection_name)
                    })?;
                    return match collection {
                        Value::List(values) => Ok(Value::Number(values.len() as f64)),
                        Value::Map(values) => Ok(Value::Number(values.len() as f64)),
                        _ => Err(error_for(self.locale, "P1010", *position, "collection.len")),
                    };
                }
                if let Some(collection_name) = name.strip_suffix(".contains") {
                    if arguments.len() != 1 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let needle = self.evaluate(&arguments[0])?;
                    let collection = self.variable(collection_name).ok_or_else(|| {
                        error_for(self.locale, "P1007", *position, collection_name)
                    })?;
                    return match collection {
                        Value::List(values) => Ok(Value::Boolean(values.contains(&needle))),
                        Value::Map(values) => {
                            let key = expect_map_key(&needle, self.locale, *position)?;
                            Ok(Value::Boolean(values.contains_key(key)))
                        }
                        _ => Err(error_for(
                            self.locale,
                            "P1010",
                            *position,
                            "collection.contains",
                        )),
                    };
                }
                if let Some(map_name) = name.strip_suffix(".get") {
                    if arguments.len() != 1 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let key = self.evaluate(&arguments[0])?;
                    let key = expect_map_key(&key, self.locale, *position)?;
                    let map = self
                        .variable(map_name)
                        .ok_or_else(|| error_for(self.locale, "P1007", *position, map_name))?;
                    let Value::Map(values) = map else {
                        return Err(error_for(self.locale, "P1010", *position, "map.get"));
                    };
                    return values
                        .get(key)
                        .cloned()
                        .ok_or_else(|| error_for(self.locale, "P1021", *position, key));
                }
                if let Some(map_name) = name.strip_suffix(".set") {
                    if arguments.len() != 2 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let key = self.evaluate(&arguments[0])?;
                    let key = expect_map_key(&key, self.locale, *position)?.to_owned();
                    let value = self.evaluate(&arguments[1])?;
                    let locale = self.locale;
                    let map = self
                        .variable_mut(map_name)
                        .ok_or_else(|| error_for(locale, "P1007", *position, map_name))?;
                    let Value::Map(values) = map else {
                        return Err(error_for(self.locale, "P1010", *position, "map.set"));
                    };
                    values.insert(key, value);
                    return Ok(Value::Boolean(true));
                }
                let (parameters, body) = self
                    .functions
                    .get(name)
                    .cloned()
                    .ok_or_else(|| error_for(self.locale, "P1008", *position, name))?;
                if parameters.len() != arguments.len() {
                    return Err(error_for(self.locale, "P1009", *position, name));
                }
                let values = arguments
                    .iter()
                    .map(|argument| self.evaluate(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                let previous_environment = std::mem::replace(&mut self.environment, HashMap::new());
                let previous_local_scopes = std::mem::take(&mut self.local_scopes);
                let previous_return = self.return_value.take();
                for (parameter, value) in parameters.iter().zip(values) {
                    self.environment.insert(parameter.clone(), value);
                }
                let run_result = self.run(&body);
                let result = self.return_value.take().unwrap_or(Value::Null);
                self.environment = previous_environment;
                self.local_scopes = previous_local_scopes;
                self.return_value = previous_return;
                run_result?;
                Ok(result)
            }
            Expr::List(expressions) => expressions
                .iter()
                .map(|expression| self.evaluate(expression))
                .collect::<Result<Vec<_>, _>>()
                .map(Value::List),
            Expr::Map(entries) => {
                let mut values = BTreeMap::new();
                for (key, value) in entries {
                    let key = self.evaluate(key)?;
                    let key = expect_string(&key, self.locale, Position::new(1, 1), "map key")?;
                    values.insert(key.to_owned(), self.evaluate(value)?);
                }
                Ok(Value::Map(values))
            }
        }
    }

    fn binary(
        &self,
        left: Value,
        operator: &TokenKind,
        right: Value,
        position: Position,
    ) -> Result<Value, PadmaError> {
        use TokenKind::*;
        match operator {
            Plus => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left + right)),
                (Value::String(left), Value::String(right)) => Ok(Value::String(left + &right)),
                _ => Err(error_for(self.locale, "P1010", position, "+")),
            },
            Minus => numeric(self.locale, position, "-", left, right, |a, b| a - b),
            Star => numeric(self.locale, position, "*", left, right, |a, b| a * b),
            Slash => match (left, right) {
                (Value::Number(left), Value::Number(right)) if right == 0.0 => {
                    let _ = left;
                    Err(error_for(self.locale, "P1011", position, "/"))
                }
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left / right)),
                _ => Err(error_for(self.locale, "P1010", position, "/")),
            },
            EqualEqual => Ok(Value::Boolean(left == right)),
            BangEqual => Ok(Value::Boolean(left != right)),
            Greater => compare(self.locale, position, ">", left, right, |a, b| a > b),
            GreaterEqual => compare(self.locale, position, ">=", left, right, |a, b| a >= b),
            Less => compare(self.locale, position, "<", left, right, |a, b| a < b),
            LessEqual => compare(self.locale, position, "<=", left, right, |a, b| a <= b),
            _ => unreachable!("parser only creates supported binary expressions"),
        }
    }

    fn interpolate(&self, text: &str, position: Position) -> Result<String, PadmaError> {
        let mut result = String::new();
        let mut characters = text.chars().peekable();
        while let Some(character) = characters.next() {
            if character != '{' {
                result.push(character);
                continue;
            }
            let mut variable = String::new();
            let mut closed = false;
            for candidate in characters.by_ref() {
                if candidate == '}' {
                    closed = true;
                    break;
                }
                variable.push(candidate);
            }
            let name = variable.trim();
            let valid_identifier = name
                .chars()
                .next()
                .map(is_identifier_start)
                .unwrap_or(false)
                && name.chars().all(is_identifier_continue);
            if !closed || !valid_identifier {
                result.push('{');
                result.push_str(&variable);
                if closed {
                    result.push('}');
                }
                continue;
            }
            let value = self
                .variable(name)
                .ok_or_else(|| error_for(self.locale, "P1007", position, name))?;
            result.push_str(&value.to_string());
        }
        Ok(result)
    }
}

fn numeric<F>(
    locale: Locale,
    position: Position,
    operator: &str,
    left: Value,
    right: Value,
    operation: F,
) -> Result<Value, PadmaError>
where
    F: FnOnce(f64, f64) -> f64,
{
    match (left, right) {
        (Value::Number(left), Value::Number(right)) => Ok(Value::Number(operation(left, right))),
        _ => Err(error_for(locale, "P1010", position, operator)),
    }
}

fn compare<F>(
    locale: Locale,
    position: Position,
    operator: &str,
    left: Value,
    right: Value,
    comparison: F,
) -> Result<Value, PadmaError>
where
    F: FnOnce(f64, f64) -> bool,
{
    match (left, right) {
        (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(comparison(left, right))),
        _ => Err(error_for(locale, "P1010", position, operator)),
    }
}

fn compile(source: &str) -> Result<(Vec<Stmt>, Locale), PadmaError> {
    let locale = Locale::from_source(source);
    let tokens = Lexer::new(source, locale).tokenize()?;
    let program = Parser::new(tokens, locale).parse()?;
    Ok((program, locale))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectManifest {
    name: String,
    version: String,
    entry: String,
    locale: String,
    capabilities: BTreeSet<String>,
    lint_disabled: BTreeSet<String>,
    dependencies: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PackageManifest {
    name: String,
    version: String,
    entry: String,
    exports: BTreeSet<String>,
    capabilities: BTreeSet<String>,
    digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeploymentManifest {
    entry: String,
    target: String,
    base_url: String,
    environment_names: BTreeSet<String>,
    rollback: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GuiManifest {
    version: u32,
    backend: String,
    entry: String,
    assets: String,
    title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RenderReleaseManifest {
    service: String,
    repository: String,
    branch: String,
    commit: String,
    rollback_deploy: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RenderApiManifest {
    service: String,
    token_env: String,
    commit: String,
    clear_cache: String,
    rollback_deploy: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AndroidBuildManifest {
    application_id: String,
    min_sdk: u32,
    target_sdk: u32,
    artifact: String,
    signing_key_env: String,
    signing_cert_sha256: String,
    permissions: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AiWorkflowManifest {
    endpoint: String,
    secret_env: String,
    model: String,
    timeout_seconds: u32,
    max_input_bytes: usize,
    max_response_bytes: usize,
    retry_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AiToolPlanManifest {
    max_steps: u32,
    max_wall_seconds: u32,
    tools: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AiTrainingPlanManifest {
    dataset_path: String,
    artifact_path: String,
    max_epochs: u32,
    max_wall_seconds: u32,
    max_dataset_bytes: u64,
    max_memory_mb: u32,
    max_cpu_threads: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BrowserPlanManifest {
    intent: String,
    redirect_policy: String,
    max_steps: usize,
    origins: BTreeSet<String>,
    navigation_urls: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BrowserConfirmationPlanManifest {
    browser_plan_digest: String,
    navigation_index: usize,
    max_session_seconds: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BrowserHandoffAuditManifest {
    path: PathBuf,
    max_records: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BrowserDraftManifest {
    browser_plan_digest: String,
    navigation_index: usize,
    action: String,
    title: String,
    body: String,
    attachment_path: Option<PathBuf>,
    max_review_seconds: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BrowserTakeoverManifest {
    browser_plan_digest: String,
    navigation_index: usize,
    sensitive_action: String,
    max_review_seconds: u32,
}

#[derive(Debug, Clone)]
struct BrowserHandoffContext {
    locale: Locale,
    root: PathBuf,
    destination: String,
    browser_plan_digest: String,
    navigation_index: usize,
    audit: Option<BrowserHandoffAuditManifest>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BrowserHandoffDecision {
    Open,
    Cancelled,
}

const PROCESS_CAPABILITIES: [&str; 7] = [
    "git", "yt-dlp", "curl", "ffmpeg", "python", "python3", "node",
];
const LINT_RULES: [&str; 3] = ["L1001", "L1002", "L1003"];
const PACKAGE_MAX_FILES: usize = 256;
const PACKAGE_MAX_BYTES: usize = 5 * 1024 * 1024;
const AUTH_PBKDF2_ITERATIONS: u32 = 600_000;
const AUTH_PASSWORD_MAX_BYTES: usize = 1_024;
const AUTH_SESSION_MAX_SECONDS: u64 = 86_400;
const DEPLOYMENT_MAX_SOURCE_FILES: usize = 256;
const DEPLOYMENT_MAX_SOURCE_BYTES: usize = 5 * 1024 * 1024;
const GUI_MAX_SOURCE_FILES: usize = 256;
const GUI_MAX_SOURCE_BYTES: usize = 5 * 1024 * 1024;

fn parse_manifest_string_list(value: &str, line_number: usize) -> Result<Vec<String>, String> {
    let value = value.trim();
    if !(value.starts_with('[') && value.ends_with(']')) {
        return Err(format!(
            "P1032: capability values must use a string list on line {line_number}"
        ));
    }
    let inner = value[1..value.len() - 1].trim();
    if inner.is_empty() {
        return Ok(Vec::new());
    }
    inner
        .split(',')
        .map(|item| {
            let item = item.trim();
            if item.len() < 2 || !item.starts_with('"') || !item.ends_with('"') {
                return Err(format!(
                    "P1032: capability values must be quoted strings on line {line_number}"
                ));
            }
            Ok(item[1..item.len() - 1].to_string())
        })
        .collect()
}

fn capability_grants_for_field(
    key: &str,
    values: Vec<String>,
    line_number: usize,
) -> Result<BTreeSet<String>, String> {
    let permitted: &[&str] = match key {
        "database" => &["sqlite"],
        "identity" => &["local"],
        "gui" => &["local"],
        "android" => &["plan"],
        "browser" => &[
            "plan",
            "confirm-plan",
            "handoff",
            "audit",
            "draft",
            "takeover",
        ],
        "ai" => &["tools", "training-plan"],
        "deployment" => &["render"],
        "filesystem" => &["read", "write"],
        "network" => &["http", "ai"],
        "server" => &["local"],
        "process" => &PROCESS_CAPABILITIES,
        "media" => &["download"],
        _ => {
            return Err(format!(
                "P1032: unsupported capability `{key}` on line {line_number}"
            ))
        }
    };
    let mut grants = BTreeSet::new();
    for value in values {
        if !permitted.contains(&value.as_str()) {
            return Err(format!(
                "P1032: unsupported `{key}` grant `{value}` on line {line_number}"
            ));
        }
        if !grants.insert(format!("{key}:{value}")) {
            return Err(format!(
                "P1032: duplicate `{key}` grant `{value}` on line {line_number}"
            ));
        }
    }
    Ok(grants)
}

fn parse_project_manifest(source: &str) -> Result<ProjectManifest, String> {
    let mut section = String::new();
    let mut fields = BTreeMap::new();
    let mut capabilities = BTreeSet::new();
    let mut capability_fields = BTreeSet::new();
    let mut lint_disabled = BTreeSet::new();
    let mut lint_fields = BTreeSet::new();
    let mut dependencies = BTreeMap::new();
    for (line_number, raw_line) in source.lines().enumerate() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            if section != "padma"
                && section != "dependencies"
                && section != "capabilities"
                && section != "lint"
            {
                return Err(format!(
                    "P1032: unsupported manifest section `{section}` on line {}",
                    line_number + 1
                ));
            }
            continue;
        }
        let (key, raw_value) = line
            .split_once('=')
            .ok_or_else(|| format!("P1032: expected `key = value` on line {}", line_number + 1))?;
        let key = key.trim();
        if section == "dependencies" {
            if !is_safe_package_name(key) {
                return Err(format!(
                    "P1032: unsafe package dependency name `{key}` on line {}",
                    line_number + 1
                ));
            }
            let raw_value = raw_value.trim();
            if raw_value.len() < 2 || !raw_value.starts_with('"') || !raw_value.ends_with('"') {
                return Err(format!(
                    "P1032: package path must be a quoted string on line {}",
                    line_number + 1
                ));
            }
            let path = raw_value[1..raw_value.len() - 1].to_string();
            safe_package_relative_path(&path)?;
            if dependencies.insert(key.to_string(), path).is_some() {
                return Err(format!(
                    "P1032: duplicate dependency `{key}` on line {}",
                    line_number + 1
                ));
            }
            continue;
        }
        if section == "capabilities" {
            if !capability_fields.insert(key.to_string()) {
                return Err(format!(
                    "P1032: duplicate capability field `{key}` on line {}",
                    line_number + 1
                ));
            }
            let grants = capability_grants_for_field(
                key,
                parse_manifest_string_list(raw_value, line_number + 1)?,
                line_number + 1,
            )?;
            capabilities.extend(grants);
            continue;
        }
        if section == "lint" {
            if key != "disable" {
                return Err(format!(
                    "P1032: unsupported lint field `{key}` on line {}",
                    line_number + 1
                ));
            }
            if !lint_fields.insert(key.to_string()) {
                return Err(format!(
                    "P1032: duplicate lint field `{key}` on line {}",
                    line_number + 1
                ));
            }
            for rule in parse_manifest_string_list(raw_value, line_number + 1)? {
                if !LINT_RULES.contains(&rule.as_str()) {
                    return Err(format!(
                        "P1032: unsupported lint rule `{rule}` on line {}",
                        line_number + 1
                    ));
                }
                if !lint_disabled.insert(rule.clone()) {
                    return Err(format!(
                        "P1032: duplicate disabled lint rule `{rule}` on line {}",
                        line_number + 1
                    ));
                }
            }
            continue;
        }
        let value = raw_value.trim().trim_matches('"').to_string();
        if section != "padma" || !matches!(key, "name" | "version" | "entry" | "locale") {
            return Err(format!(
                "P1032: unsupported manifest field `{key}` on line {}",
                line_number + 1
            ));
        }
        if fields.insert(key.to_string(), value).is_some() {
            return Err(format!(
                "P1032: duplicate manifest field `{key}` on line {}",
                line_number + 1
            ));
        }
    }
    let required = |key: &str| {
        fields
            .get(key)
            .cloned()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("P1032: missing `[padma]` field `{key}`"))
    };
    let manifest = ProjectManifest {
        name: required("name")?,
        version: required("version")?,
        entry: required("entry")?,
        locale: fields
            .get("locale")
            .cloned()
            .unwrap_or_else(|| "auto".to_string()),
        capabilities,
        lint_disabled,
        dependencies,
    };
    if !matches!(manifest.locale.as_str(), "auto" | "bn" | "en") {
        return Err("P1032: `locale` must be `auto`, `bn`, or `en`".to_string());
    }
    Ok(manifest)
}

fn is_safe_environment_name(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_uppercase() || first == '_')
        && value.len() <= 128
        && chars.all(|character| {
            character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_'
        })
}

fn is_safe_ai_endpoint(value: &str) -> bool {
    if value.is_empty()
        || value.len() > 2_048
        || !value.is_ascii()
        || value
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
        || !value.starts_with("https://")
        || value.contains(['@', '?', '#', '\\', '%'])
    {
        return false;
    }
    let authority_and_path = &value["https://".len()..];
    let (host, path) = authority_and_path
        .split_once('/')
        .map_or((authority_and_path, ""), |(host, path)| (host, path));
    if host.is_empty()
        || host.len() > 253
        || host.contains([':', '[', ']'])
        || !host.contains('.')
        || host
            .chars()
            .all(|character| character.is_ascii_digit() || character == '.')
    {
        return false;
    }
    let normalized_host = host.to_ascii_lowercase();
    if normalized_host == "localhost"
        || normalized_host.ends_with(".localhost")
        || normalized_host.ends_with(".local")
        || normalized_host.ends_with(".internal")
    {
        return false;
    }
    if !host.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    }) {
        return false;
    }
    !path.contains("//") && !path.split('/').any(|segment| matches!(segment, "." | ".."))
}

fn is_safe_ai_model_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && !value.starts_with('.')
        && !value.contains("..")
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b':' | b'/')
        })
}

fn is_safe_ai_task_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn is_safe_ai_instruction(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 8_192
        && value
            .chars()
            .all(|character| !character.is_control() || matches!(character, '\n' | '\t'))
}

fn json_depth(value: &JsonValue) -> usize {
    match value {
        JsonValue::Array(values) => 1 + values.iter().map(json_depth).max().unwrap_or(0),
        JsonValue::Object(values) => 1 + values.values().map(json_depth).max().unwrap_or(0),
        _ => 1,
    }
}

fn ai_workflow_request_payload(
    input: &Value,
    manifest: &AiWorkflowManifest,
    locale: Locale,
    position: Position,
) -> Result<Vec<u8>, PadmaError> {
    let Value::Map(input) = input else {
        return Err(error_for(locale, "P1050", position, "input must be a map"));
    };
    if input.len() != 3
        || !input.contains_key("task")
        || !input.contains_key("instruction")
        || !input.contains_key("data")
    {
        return Err(error_for(
            locale,
            "P1050",
            position,
            "input must contain exactly task, instruction, and data",
        ));
    }
    let task = input
        .get("task")
        .and_then(|value| match value {
            Value::String(value) => Some(value.as_str()),
            _ => None,
        })
        .filter(|value| is_safe_ai_task_identifier(value))
        .ok_or_else(|| error_for(locale, "P1050", position, "task must be a safe identifier"))?;
    let instruction = input
        .get("instruction")
        .and_then(|value| match value {
            Value::String(value) => Some(value.as_str()),
            _ => None,
        })
        .filter(|value| is_safe_ai_instruction(value))
        .ok_or_else(|| {
            error_for(
                locale,
                "P1050",
                position,
                "instruction must be bounded text",
            )
        })?;
    let data = value_to_json(input.get("data").expect("validated input key"))
        .map_err(|_| error_for(locale, "P1050", position, "data must be JSON-compatible"))?;
    let payload = serde_json::json!({
        "protocol": "padma-ai-workflow-v1",
        "model": manifest.model,
        "task": task,
        "instruction": instruction,
        "data": data,
    });
    if json_depth(&payload) > AI_WORKFLOW_MAX_JSON_DEPTH {
        return Err(error_for(
            locale,
            "P1050",
            position,
            "input exceeds maximum JSON depth",
        ));
    }
    let payload = serde_json::to_vec(&payload)
        .map_err(|_| error_for(locale, "P1050", position, "input cannot be encoded"))?;
    if payload.len() > manifest.max_input_bytes {
        return Err(error_for(
            locale,
            "P1050",
            position,
            "input exceeds declared byte limit",
        ));
    }
    Ok(payload)
}

fn curl_config_quote(value: &str) -> Option<String> {
    if value.chars().any(char::is_control) {
        return None;
    }
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for character in value.chars() {
        match character {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            _ => quoted.push(character),
        }
    }
    quoted.push('"');
    Some(quoted)
}

fn ai_workflow_curl_config(
    manifest: &AiWorkflowManifest,
    secret: &str,
    request: &[u8],
) -> Option<Vec<u8>> {
    let request = std::str::from_utf8(request).ok()?;
    let endpoint = curl_config_quote(&manifest.endpoint)?;
    let authorization = curl_config_quote(&format!("Authorization: Bearer {secret}"))?;
    let payload = curl_config_quote(request)?;
    Some(
        format!(
            "silent\nshow-error\nfail\nrequest = \"POST\"\nheader = \"Content-Type: application/json\"\nheader = {authorization}\ndata-binary = {payload}\nmax-time = \"{}\"\nmax-filesize = \"{}\"\nurl = {endpoint}\n",
            manifest.timeout_seconds, manifest.max_response_bytes
        )
        .into_bytes(),
    )
}

fn ai_workflow_response_value(
    response: &[u8],
    manifest: &AiWorkflowManifest,
    locale: Locale,
    position: Position,
) -> Result<Value, PadmaError> {
    if response.is_empty() || response.len() > manifest.max_response_bytes {
        return Err(error_for(
            locale,
            "P1052",
            position,
            "response exceeds declared byte limit",
        ));
    }
    let response: JsonValue = serde_json::from_slice(response)
        .map_err(|_| error_for(locale, "P1052", position, "response is not valid JSON"))?;
    if json_depth(&response) > AI_WORKFLOW_MAX_JSON_DEPTH {
        return Err(error_for(
            locale,
            "P1052",
            position,
            "response exceeds maximum JSON depth",
        ));
    }
    let response = response
        .as_object()
        .ok_or_else(|| error_for(locale, "P1052", position, "response must be a JSON object"))?;
    if response.len() != 2
        || !response.contains_key("protocol")
        || !response.contains_key("output")
        || response.get("protocol").and_then(JsonValue::as_str) != Some("padma-ai-workflow-v1")
    {
        return Err(error_for(
            locale,
            "P1052",
            position,
            "response must contain only the workflow protocol and output",
        ));
    }
    let output = response
        .get("output")
        .filter(|value| value.is_object())
        .ok_or_else(|| error_for(locale, "P1052", position, "output must be a JSON object"))?;
    let output = value_from_json(output.clone())
        .map_err(|_| error_for(locale, "P1052", position, "output is not Padma-compatible"))?;
    let mut meta = BTreeMap::new();
    meta.insert(
        "adapter".to_string(),
        Value::String("json-http-v1".to_string()),
    );
    meta.insert("model".to_string(), Value::String(manifest.model.clone()));
    meta.insert("attempts".to_string(), Value::Number(1.0));
    let mut result = BTreeMap::new();
    result.insert("output".to_string(), output);
    result.insert("meta".to_string(), Value::Map(meta));
    Ok(Value::Map(result))
}

fn ai_workflow_error(locale: Locale, code: &'static str, detail: &str) -> String {
    let diagnostic = error_for(locale, code, Position::new(1, 1), detail);
    let label = if locale == Locale::Bangla {
        "পরামর্শ"
    } else {
        "help"
    };
    match diagnostic.hint {
        Some(hint) => format!(
            "{}: {}\n  = {label}: {hint}",
            diagnostic.code, diagnostic.message
        ),
        None => format!("{}: {}", diagnostic.code, diagnostic.message),
    }
}

fn parse_ai_workflow_manifest(source: &str, locale: Locale) -> Result<AiWorkflowManifest, String> {
    let mut section = String::new();
    let mut fields = BTreeMap::new();
    for (line_number, raw_line) in source.lines().enumerate() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            if section != "workflow" {
                return Err(ai_workflow_error(
                    locale,
                    "P1050",
                    &format!(
                        "unsupported section `{section}` on line {}",
                        line_number + 1
                    ),
                ));
            }
            continue;
        }
        let (key, raw_value) = line.split_once('=').ok_or_else(|| {
            ai_workflow_error(
                locale,
                "P1050",
                &format!("expected `key = value` on line {}", line_number + 1),
            )
        })?;
        let key = key.trim();
        if section != "workflow"
            || !matches!(
                key,
                "version"
                    | "adapter"
                    | "endpoint"
                    | "secret_env"
                    | "model"
                    | "timeout_seconds"
                    | "max_input_bytes"
                    | "max_response_bytes"
                    | "retry_policy"
            )
        {
            return Err(ai_workflow_error(
                locale,
                "P1050",
                &format!("unsupported field `{key}` on line {}", line_number + 1),
            ));
        }
        let raw_value = raw_value.trim();
        let value = if matches!(
            key,
            "timeout_seconds" | "max_input_bytes" | "max_response_bytes"
        ) {
            if raw_value.is_empty() || !raw_value.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(ai_workflow_error(
                    locale,
                    "P1050",
                    &format!(
                        "`{key}` must be an unsigned integer on line {}",
                        line_number + 1
                    ),
                ));
            }
            raw_value.to_string()
        } else {
            if raw_value.len() < 2
                || !raw_value.starts_with('"')
                || !raw_value.ends_with('"')
                || raw_value[1..raw_value.len() - 1].contains('"')
            {
                return Err(ai_workflow_error(
                    locale,
                    "P1050",
                    &format!(
                        "`{key}` must be a quoted string on line {}",
                        line_number + 1
                    ),
                ));
            }
            raw_value[1..raw_value.len() - 1].to_string()
        };
        if fields.insert(key.to_string(), value).is_some() {
            return Err(ai_workflow_error(
                locale,
                "P1050",
                &format!("duplicate field `{key}` on line {}", line_number + 1),
            ));
        }
    }
    let required = |key: &str| {
        fields
            .get(key)
            .cloned()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                ai_workflow_error(
                    locale,
                    "P1050",
                    &format!("missing `[workflow]` field `{key}`"),
                )
            })
    };
    if required("version")? != "1" {
        return Err(ai_workflow_error(
            locale,
            "P1050",
            "workflow version must be `1`",
        ));
    }
    if required("adapter")? != "json-http-v1" {
        return Err(ai_workflow_error(
            locale,
            "P1050",
            "workflow adapter must be `json-http-v1`",
        ));
    }
    let endpoint = required("endpoint")?;
    if !is_safe_ai_endpoint(&endpoint) {
        return Err(ai_workflow_error(
            locale,
            "P1050",
            "endpoint must be a public HTTPS DNS URL without credentials, query, fragment, port, or traversal",
        ));
    }
    let secret_env = required("secret_env")?;
    if !is_safe_environment_name(&secret_env) {
        return Err(ai_workflow_error(
            locale,
            "P1050",
            "secret_env must be a safe uppercase environment variable name",
        ));
    }
    let model = required("model")?;
    if !is_safe_ai_model_identifier(&model) {
        return Err(ai_workflow_error(
            locale,
            "P1050",
            "model must be a bounded printable identifier",
        ));
    }
    let timeout_seconds = required("timeout_seconds")?.parse::<u32>().map_err(|_| {
        ai_workflow_error(
            locale,
            "P1050",
            "timeout_seconds must be an unsigned integer",
        )
    })?;
    if !(1..=30).contains(&timeout_seconds) {
        return Err(ai_workflow_error(
            locale,
            "P1050",
            "timeout_seconds must be between 1 and 30",
        ));
    }
    let max_input_bytes = required("max_input_bytes")?.parse::<usize>().map_err(|_| {
        ai_workflow_error(
            locale,
            "P1050",
            "max_input_bytes must be an unsigned integer",
        )
    })?;
    if !(1..=32_768).contains(&max_input_bytes) {
        return Err(ai_workflow_error(
            locale,
            "P1050",
            "max_input_bytes must be between 1 and 32768",
        ));
    }
    let max_response_bytes = required("max_response_bytes")?
        .parse::<usize>()
        .map_err(|_| {
            ai_workflow_error(
                locale,
                "P1050",
                "max_response_bytes must be an unsigned integer",
            )
        })?;
    if !(1..=65_536).contains(&max_response_bytes) {
        return Err(ai_workflow_error(
            locale,
            "P1050",
            "max_response_bytes must be between 1 and 65536",
        ));
    }
    let retry_policy = required("retry_policy")?;
    if retry_policy != "never" {
        return Err(ai_workflow_error(
            locale,
            "P1050",
            "retry_policy must be `never` in workflow version 1",
        ));
    }
    Ok(AiWorkflowManifest {
        endpoint,
        secret_env,
        model,
        timeout_seconds,
        max_input_bytes,
        max_response_bytes,
        retry_policy,
    })
}

fn ai_tool_plan_error(locale: Locale, code: &'static str, detail: &str) -> String {
    let diagnostic = error_for(locale, code, Position::new(1, 1), detail);
    let label = if locale == Locale::Bangla {
        "পরামর্শ"
    } else {
        "help"
    };
    match diagnostic.hint {
        Some(hint) => format!(
            "{}: {}\n  = {label}: {hint}",
            diagnostic.code, diagnostic.message
        ),
        None => format!("{}: {}", diagnostic.code, diagnostic.message),
    }
}

fn parse_ai_tool_string_list(
    lines: &[&str],
    index: &mut usize,
    locale: Locale,
) -> Result<BTreeSet<String>, String> {
    let mut tools = BTreeSet::new();
    loop {
        *index += 1;
        let Some(raw_line) = lines.get(*index) else {
            return Err(ai_tool_plan_error(
                locale,
                "P1056",
                "unterminated tools list",
            ));
        };
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line == "]" {
            break;
        }
        if line.is_empty() {
            continue;
        }
        let value = line.strip_suffix(',').unwrap_or(line).trim();
        if value.len() < 2
            || !value.starts_with('"')
            || !value.ends_with('"')
            || value[1..value.len() - 1].contains('"')
        {
            return Err(ai_tool_plan_error(
                locale,
                "P1056",
                &format!("tools list requires a quoted string on line {}", *index + 1),
            ));
        }
        let tool = value[1..value.len() - 1].to_string();
        if !matches!(tool.as_str(), "ai-workflow" | "file-read" | "http-request") {
            return Err(ai_tool_plan_error(
                locale,
                "P1056",
                &format!("unsupported tool at list index {}", tools.len() + 1),
            ));
        }
        if !tools.insert(tool) {
            return Err(ai_tool_plan_error(
                locale,
                "P1056",
                &format!(
                    "tools list contains a duplicate at index {}",
                    tools.len() + 1
                ),
            ));
        }
        if tools.len() > 3 {
            return Err(ai_tool_plan_error(
                locale,
                "P1056",
                "tools list exceeds the version 1 limit",
            ));
        }
    }
    if tools.is_empty() {
        return Err(ai_tool_plan_error(
            locale,
            "P1056",
            "tools list cannot be empty",
        ));
    }
    Ok(tools)
}

fn parse_ai_tool_plan_manifest(source: &str, locale: Locale) -> Result<AiToolPlanManifest, String> {
    let lines = source.lines().collect::<Vec<_>>();
    let mut section = String::new();
    let mut agent_fields = BTreeMap::new();
    let mut tools: Option<BTreeSet<String>> = None;
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index].split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            index += 1;
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            if !matches!(section.as_str(), "agent" | "toolset") {
                return Err(ai_tool_plan_error(
                    locale,
                    "P1056",
                    &format!("unsupported section on line {}", index + 1),
                ));
            }
            index += 1;
            continue;
        }
        let (key, raw_value) = line.split_once('=').ok_or_else(|| {
            ai_tool_plan_error(
                locale,
                "P1056",
                &format!("expected `key = value` on line {}", index + 1),
            )
        })?;
        let key = key.trim();
        let raw_value = raw_value.trim();
        if section == "toolset" && key == "tools" {
            if !raw_value.is_empty() && raw_value != "[" {
                return Err(ai_tool_plan_error(
                    locale,
                    "P1056",
                    &format!(
                        "tools list must begin on a new list line at line {}",
                        index + 1
                    ),
                ));
            }
            if tools.is_some() {
                return Err(ai_tool_plan_error(
                    locale,
                    "P1056",
                    &format!("duplicate tools field on line {}", index + 1),
                ));
            }
            tools = Some(parse_ai_tool_string_list(&lines, &mut index, locale)?);
            index += 1;
            continue;
        }
        if section != "agent"
            || !matches!(
                key,
                "version" | "mode" | "max_steps" | "max_wall_seconds" | "retry_policy"
            )
        {
            return Err(ai_tool_plan_error(
                locale,
                "P1056",
                &format!("unsupported field on line {}", index + 1),
            ));
        }
        let value = if matches!(key, "max_steps" | "max_wall_seconds") {
            if raw_value.is_empty() || !raw_value.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(ai_tool_plan_error(
                    locale,
                    "P1056",
                    &format!("`{key}` must be an unsigned integer on line {}", index + 1),
                ));
            }
            raw_value.to_string()
        } else {
            if raw_value.len() < 2
                || !raw_value.starts_with('"')
                || !raw_value.ends_with('"')
                || raw_value[1..raw_value.len() - 1].contains('"')
            {
                return Err(ai_tool_plan_error(
                    locale,
                    "P1056",
                    &format!("`{key}` must be a quoted string on line {}", index + 1),
                ));
            }
            raw_value[1..raw_value.len() - 1].to_string()
        };
        if agent_fields.insert(key.to_string(), value).is_some() {
            return Err(ai_tool_plan_error(
                locale,
                "P1056",
                &format!("duplicate field `{key}` on line {}", index + 1),
            ));
        }
        index += 1;
    }
    let required = |key: &str| {
        agent_fields
            .get(key)
            .cloned()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                ai_tool_plan_error(locale, "P1056", &format!("missing `[agent]` field `{key}`"))
            })
    };
    if required("version")? != "1" {
        return Err(ai_tool_plan_error(
            locale,
            "P1056",
            "agent version must be `1`",
        ));
    }
    if required("mode")? != "plan-only" {
        return Err(ai_tool_plan_error(
            locale,
            "P1056",
            "agent mode must be `plan-only`",
        ));
    }
    if required("retry_policy")? != "never" {
        return Err(ai_tool_plan_error(
            locale,
            "P1056",
            "retry_policy must be `never`",
        ));
    }
    let max_steps = required("max_steps")?.parse::<u32>().map_err(|_| {
        ai_tool_plan_error(locale, "P1056", "max_steps must be an unsigned integer")
    })?;
    if !(1..=8).contains(&max_steps) {
        return Err(ai_tool_plan_error(
            locale,
            "P1056",
            "max_steps must be between 1 and 8",
        ));
    }
    let max_wall_seconds = required("max_wall_seconds")?.parse::<u32>().map_err(|_| {
        ai_tool_plan_error(
            locale,
            "P1056",
            "max_wall_seconds must be an unsigned integer",
        )
    })?;
    if !(1..=600).contains(&max_wall_seconds) {
        return Err(ai_tool_plan_error(
            locale,
            "P1056",
            "max_wall_seconds must be between 1 and 600",
        ));
    }
    let tools =
        tools.ok_or_else(|| ai_tool_plan_error(locale, "P1056", "missing `[toolset] tools`"))?;
    Ok(AiToolPlanManifest {
        max_steps,
        max_wall_seconds,
        tools,
    })
}

fn ai_tool_required_capability(tool: &str) -> &'static str {
    match tool {
        "ai-workflow" => "network:ai",
        "file-read" => "filesystem:read",
        "http-request" => "network:http",
        _ => unreachable!("AI tool manifest parser validates tool names"),
    }
}

fn ai_training_plan_error(locale: Locale, code: &'static str, detail: &str) -> String {
    let diagnostic = error_for(locale, code, Position::new(1, 1), detail);
    let label = if locale == Locale::Bangla {
        "পরামর্শ"
    } else {
        "help"
    };
    match diagnostic.hint {
        Some(hint) => format!(
            "{}: {}\n  = {label}: {hint}",
            diagnostic.code, diagnostic.message
        ),
        None => format!("{}: {}", diagnostic.code, diagnostic.message),
    }
}

fn parse_ai_training_plan_manifest(
    source: &str,
    locale: Locale,
) -> Result<AiTrainingPlanManifest, String> {
    let mut section = String::new();
    let mut fields = BTreeMap::new();
    for (line_number, raw_line) in source.lines().enumerate() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            if section != "training" {
                return Err(ai_training_plan_error(
                    locale,
                    "P1058",
                    &format!("unsupported section on line {}", line_number + 1),
                ));
            }
            continue;
        }
        let (key, raw_value) = line.split_once('=').ok_or_else(|| {
            ai_training_plan_error(
                locale,
                "P1058",
                &format!("expected `key = value` on line {}", line_number + 1),
            )
        })?;
        let key = key.trim();
        if section != "training"
            || !matches!(
                key,
                "version"
                    | "mode"
                    | "backend"
                    | "dataset_path"
                    | "artifact_path"
                    | "max_epochs"
                    | "max_wall_seconds"
                    | "max_dataset_bytes"
                    | "max_memory_mb"
                    | "max_cpu_threads"
            )
        {
            return Err(ai_training_plan_error(
                locale,
                "P1058",
                &format!("unsupported field on line {}", line_number + 1),
            ));
        }
        let raw_value = raw_value.trim();
        let value = if matches!(
            key,
            "max_epochs"
                | "max_wall_seconds"
                | "max_dataset_bytes"
                | "max_memory_mb"
                | "max_cpu_threads"
        ) {
            if raw_value.is_empty() || !raw_value.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(ai_training_plan_error(
                    locale,
                    "P1058",
                    &format!(
                        "`{key}` must be an unsigned integer on line {}",
                        line_number + 1
                    ),
                ));
            }
            raw_value.to_string()
        } else {
            if raw_value.len() < 2
                || !raw_value.starts_with('"')
                || !raw_value.ends_with('"')
                || raw_value[1..raw_value.len() - 1].contains('"')
            {
                return Err(ai_training_plan_error(
                    locale,
                    "P1058",
                    &format!(
                        "`{key}` must be a quoted string on line {}",
                        line_number + 1
                    ),
                ));
            }
            raw_value[1..raw_value.len() - 1].to_string()
        };
        if fields.insert(key.to_string(), value).is_some() {
            return Err(ai_training_plan_error(
                locale,
                "P1058",
                &format!("duplicate field `{key}` on line {}", line_number + 1),
            ));
        }
    }
    let required = |key: &str| {
        fields
            .get(key)
            .cloned()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                ai_training_plan_error(
                    locale,
                    "P1058",
                    &format!("missing `[training]` field `{key}`"),
                )
            })
    };
    if required("version")? != "1" {
        return Err(ai_training_plan_error(
            locale,
            "P1058",
            "training version must be `1`",
        ));
    }
    if required("mode")? != "plan-only" {
        return Err(ai_training_plan_error(
            locale,
            "P1058",
            "training mode must be `plan-only`",
        ));
    }
    if required("backend")? != "local-adapter-v1" {
        return Err(ai_training_plan_error(
            locale,
            "P1058",
            "training backend must be `local-adapter-v1`",
        ));
    }
    let dataset_path = safe_relative_path(&required("dataset_path")?)
        .map_err(|_| {
            ai_training_plan_error(locale, "P1058", "dataset_path must be project-relative")
        })?
        .to_string_lossy()
        .replace('\\', "/");
    if !(dataset_path.ends_with(".jsonl") || dataset_path.ends_with(".csv")) {
        return Err(ai_training_plan_error(
            locale,
            "P1058",
            "dataset_path must end in `.jsonl` or `.csv`",
        ));
    }
    let artifact_path = safe_relative_path(&required("artifact_path")?)
        .map_err(|_| {
            ai_training_plan_error(locale, "P1058", "artifact_path must be project-relative")
        })?
        .to_string_lossy()
        .replace('\\', "/");
    if !artifact_path.starts_with("artifacts/") || !artifact_path.ends_with(".padma-model") {
        return Err(ai_training_plan_error(
            locale,
            "P1058",
            "artifact_path must be under `artifacts/` and end in `.padma-model`",
        ));
    }
    let parse_u32 = |key: &str| {
        required(key)?.parse::<u32>().map_err(|_| {
            ai_training_plan_error(
                locale,
                "P1058",
                &format!("{key} must be an unsigned integer"),
            )
        })
    };
    let max_epochs = parse_u32("max_epochs")?;
    if !(1..=64).contains(&max_epochs) {
        return Err(ai_training_plan_error(
            locale,
            "P1058",
            "max_epochs must be between 1 and 64",
        ));
    }
    let max_wall_seconds = parse_u32("max_wall_seconds")?;
    if !(1..=3_600).contains(&max_wall_seconds) {
        return Err(ai_training_plan_error(
            locale,
            "P1058",
            "max_wall_seconds must be between 1 and 3600",
        ));
    }
    let max_dataset_bytes = required("max_dataset_bytes")?.parse::<u64>().map_err(|_| {
        ai_training_plan_error(
            locale,
            "P1058",
            "max_dataset_bytes must be an unsigned integer",
        )
    })?;
    if !(1_024..=1_073_741_824).contains(&max_dataset_bytes) {
        return Err(ai_training_plan_error(
            locale,
            "P1058",
            "max_dataset_bytes must be between 1024 and 1073741824",
        ));
    }
    let max_memory_mb = parse_u32("max_memory_mb")?;
    if !(64..=4_096).contains(&max_memory_mb) {
        return Err(ai_training_plan_error(
            locale,
            "P1058",
            "max_memory_mb must be between 64 and 4096",
        ));
    }
    let max_cpu_threads = parse_u32("max_cpu_threads")?;
    if !(1..=8).contains(&max_cpu_threads) {
        return Err(ai_training_plan_error(
            locale,
            "P1058",
            "max_cpu_threads must be between 1 and 8",
        ));
    }
    Ok(AiTrainingPlanManifest {
        dataset_path,
        artifact_path,
        max_epochs,
        max_wall_seconds,
        max_dataset_bytes,
        max_memory_mb,
        max_cpu_threads,
    })
}

fn browser_plan_error(locale: Locale, code: &'static str, detail: &str) -> String {
    let diagnostic = error_for(locale, code, Position::new(1, 1), detail);
    let label = if locale == Locale::Bangla {
        "পরামর্শ"
    } else {
        "help"
    };
    match diagnostic.hint {
        Some(hint) => format!(
            "{}: {}\n  = {label}: {hint}",
            diagnostic.code, diagnostic.message
        ),
        None => format!("{}: {}", diagnostic.code, diagnostic.message),
    }
}

fn browser_confirmation_error(locale: Locale, code: &'static str, detail: &str) -> String {
    let diagnostic = error_for(locale, code, Position::new(1, 1), detail);
    let label = if locale == Locale::Bangla {
        "পরামর্শ"
    } else {
        "help"
    };
    match diagnostic.hint {
        Some(hint) => format!(
            "{}: {}\n  = {label}: {hint}",
            diagnostic.code, diagnostic.message
        ),
        None => format!("{}: {}", diagnostic.code, diagnostic.message),
    }
}

fn browser_handoff_error(locale: Locale, code: &'static str, detail: &str) -> String {
    browser_confirmation_error(locale, code, detail)
}

fn parse_browser_quoted_value(
    raw_value: &str,
    locale: Locale,
    line_number: usize,
) -> Result<String, String> {
    let raw_value = raw_value.trim();
    if raw_value.len() < 2
        || !raw_value.starts_with('"')
        || !raw_value.ends_with('"')
        || raw_value[1..raw_value.len() - 1].contains('"')
    {
        return Err(browser_plan_error(
            locale,
            "P1053",
            &format!("expected a quoted string on line {line_number}"),
        ));
    }
    Ok(raw_value[1..raw_value.len() - 1].to_string())
}

fn parse_browser_string_list(
    lines: &[&str],
    index: &mut usize,
    locale: Locale,
    field: &str,
) -> Result<Vec<String>, String> {
    let mut values = Vec::new();
    loop {
        *index += 1;
        let Some(raw_line) = lines.get(*index) else {
            return Err(browser_plan_error(locale, "P1053", "unterminated list"));
        };
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line == "]" {
            break;
        }
        if line.is_empty() {
            continue;
        }
        let line = line.strip_suffix(',').unwrap_or(line).trim();
        let value = parse_browser_quoted_value(line, locale, *index + 1)?;
        if value.is_empty() || values.iter().any(|existing| existing == &value) {
            return Err(browser_plan_error(
                locale,
                "P1053",
                &format!(
                    "{field} list has an empty or duplicate value at index {}",
                    values.len() + 1
                ),
            ));
        }
        values.push(value);
    }
    Ok(values)
}

fn is_canonical_browser_origin(value: &str) -> bool {
    let Some(host) = value.strip_prefix("https://") else {
        return false;
    };
    if host.is_empty()
        || !host.contains('.')
        || host.len() > 253
        || host.ends_with('.')
        || host.contains(['/', '?', '#', '@', ':'])
        || !host.is_ascii()
        || host.chars().any(char::is_whitespace)
        || host.parse::<std::net::Ipv4Addr>().is_ok()
    {
        return false;
    }
    host.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && !label.starts_with('-')
            && !label.ends_with('-')
            && !label.starts_with("xn--")
            && label
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    })
}

fn browser_navigation_matches_origin(url: &str, origins: &BTreeSet<String>) -> bool {
    if !url.is_ascii()
        || url.chars().any(char::is_control)
        || url.contains(['?', '#', '@', '%'])
        || url.contains(char::is_whitespace)
    {
        return false;
    }
    let Some(rest) = url.strip_prefix("https://") else {
        return false;
    };
    let Some((host, path)) = rest.split_once('/') else {
        return false;
    };
    let origin = format!("https://{host}");
    if !origins.contains(&origin) || !is_canonical_browser_origin(&origin) || path.contains("//") {
        return false;
    }
    path.split('/')
        .all(|segment| segment != "." && segment != "..")
}

fn parse_browser_plan_manifest(
    source: &str,
    locale: Locale,
) -> Result<BrowserPlanManifest, String> {
    let lines: Vec<&str> = source.lines().collect();
    let mut section = String::new();
    let mut scalar_fields = BTreeMap::new();
    let mut origins: Option<Vec<String>> = None;
    let mut navigation_urls: Option<Vec<String>> = None;
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index].split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            index += 1;
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            if !matches!(section.as_str(), "browser" | "allowlist" | "navigation") {
                return Err(browser_plan_error(
                    locale,
                    "P1053",
                    &format!("unsupported section on line {}", index + 1),
                ));
            }
            index += 1;
            continue;
        }
        let (key, raw_value) = line.split_once('=').ok_or_else(|| {
            browser_plan_error(
                locale,
                "P1053",
                &format!("expected `key = value` on line {}", index + 1),
            )
        })?;
        let key = key.trim();
        if (section == "allowlist" && key == "origins")
            || (section == "navigation" && key == "urls")
        {
            if raw_value.trim() != "[" {
                return Err(browser_plan_error(
                    locale,
                    "P1053",
                    "list must begin with `[`",
                ));
            }
            let values = parse_browser_string_list(&lines, &mut index, locale, key)?;
            let destination = if section == "allowlist" {
                &mut origins
            } else {
                &mut navigation_urls
            };
            if destination.replace(values).is_some() {
                return Err(browser_plan_error(locale, "P1053", "duplicate list field"));
            }
            index += 1;
            continue;
        }
        if !matches!(
            (section.as_str(), key),
            ("browser", "version")
                | ("browser", "intent")
                | ("browser", "redirect_policy")
                | ("browser", "max_steps")
        ) {
            return Err(browser_plan_error(
                locale,
                "P1053",
                &format!("unsupported field on line {}", index + 1),
            ));
        }
        let value = if key == "max_steps" {
            let value = raw_value.trim();
            if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(browser_plan_error(
                    locale,
                    "P1053",
                    "max_steps must be an unsigned integer",
                ));
            }
            value.to_string()
        } else {
            parse_browser_quoted_value(raw_value, locale, index + 1)?
        };
        if scalar_fields.insert(key.to_string(), value).is_some() {
            return Err(browser_plan_error(
                locale,
                "P1053",
                "duplicate browser field",
            ));
        }
        index += 1;
    }
    let required = |key: &str| {
        scalar_fields
            .get(key)
            .cloned()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                browser_plan_error(locale, "P1053", &format!("missing [browser] field `{key}`"))
            })
    };
    if required("version")? != "1"
        || required("intent")? != "navigation-review"
        || required("redirect_policy")? != "deny"
    {
        return Err(browser_plan_error(
            locale,
            "P1053",
            "browser policy fields do not match version 1",
        ));
    }
    let max_steps = required("max_steps")?.parse::<usize>().map_err(|_| {
        browser_plan_error(locale, "P1053", "max_steps must be an unsigned integer")
    })?;
    let origins: BTreeSet<String> = origins
        .ok_or_else(|| browser_plan_error(locale, "P1053", "missing [allowlist].origins"))?
        .into_iter()
        .collect();
    let navigation_urls = navigation_urls
        .ok_or_else(|| browser_plan_error(locale, "P1053", "missing [navigation].urls"))?;
    if !(1..=16).contains(&max_steps)
        || origins.is_empty()
        || origins.len() > 16
        || navigation_urls.is_empty()
        || navigation_urls.len() > max_steps
        || origins
            .iter()
            .any(|origin| !is_canonical_browser_origin(origin))
    {
        return Err(browser_plan_error(
            locale,
            "P1053",
            "browser list or max_steps policy is invalid",
        ));
    }
    if navigation_urls
        .iter()
        .any(|url| !browser_navigation_matches_origin(url, &origins))
    {
        return Err(browser_plan_error(
            locale,
            "P1054",
            "navigation URL violates the reviewed origin/path policy",
        ));
    }
    Ok(BrowserPlanManifest {
        intent: "navigation-review".to_string(),
        redirect_policy: "deny".to_string(),
        max_steps,
        origins,
        navigation_urls,
    })
}

fn browser_plan_digest(manifest: &BrowserPlanManifest) -> String {
    let mut canonical = Vec::new();
    canonical.extend_from_slice(b"padma-browser-plan-v1\0");
    canonical.extend_from_slice(manifest.intent.as_bytes());
    canonical.push(0);
    canonical.extend_from_slice(manifest.redirect_policy.as_bytes());
    canonical.push(0);
    canonical.extend_from_slice(manifest.max_steps.to_string().as_bytes());
    canonical.push(0);
    for origin in &manifest.origins {
        canonical.extend_from_slice(origin.as_bytes());
        canonical.push(0);
    }
    canonical.push(0);
    for url in &manifest.navigation_urls {
        canonical.extend_from_slice(url.as_bytes());
        canonical.push(0);
    }
    format!("sha256:{}", sha256_hex(&canonical))
}

fn parse_browser_confirmation_quoted_value(
    raw_value: &str,
    locale: Locale,
    line_number: usize,
) -> Result<String, String> {
    let raw_value = raw_value.trim();
    if raw_value.len() < 2
        || !raw_value.starts_with('"')
        || !raw_value.ends_with('"')
        || raw_value[1..raw_value.len() - 1].contains('"')
    {
        return Err(browser_confirmation_error(
            locale,
            "P1060",
            &format!("expected a quoted string on line {line_number}"),
        ));
    }
    Ok(raw_value[1..raw_value.len() - 1].to_string())
}

fn parse_browser_confirmation_plan_manifest(
    source: &str,
    locale: Locale,
) -> Result<BrowserConfirmationPlanManifest, String> {
    let mut section = String::new();
    let mut fields = BTreeMap::new();
    for (line_number, raw_line) in source.lines().enumerate() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            if section != "confirmation" {
                return Err(browser_confirmation_error(
                    locale,
                    "P1060",
                    &format!("unsupported section on line {}", line_number + 1),
                ));
            }
            continue;
        }
        let (key, raw_value) = line.split_once('=').ok_or_else(|| {
            browser_confirmation_error(
                locale,
                "P1060",
                &format!("expected `key = value` on line {}", line_number + 1),
            )
        })?;
        let key = key.trim();
        if section != "confirmation"
            || !matches!(
                key,
                "version"
                    | "mode"
                    | "browser_plan_digest"
                    | "navigation_index"
                    | "max_session_seconds"
            )
        {
            return Err(browser_confirmation_error(
                locale,
                "P1060",
                &format!("unsupported field on line {}", line_number + 1),
            ));
        }
        let value = if matches!(key, "navigation_index" | "max_session_seconds") {
            let value = raw_value.trim();
            if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(browser_confirmation_error(
                    locale,
                    "P1060",
                    &format!("{key} must be an unsigned integer"),
                ));
            }
            value.to_string()
        } else {
            parse_browser_confirmation_quoted_value(raw_value, locale, line_number + 1)?
        };
        if fields.insert(key.to_string(), value).is_some() {
            return Err(browser_confirmation_error(
                locale,
                "P1060",
                "duplicate confirmation field",
            ));
        }
    }
    let required = |key: &str| {
        fields
            .get(key)
            .cloned()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                browser_confirmation_error(
                    locale,
                    "P1060",
                    &format!("missing [confirmation] field `{key}`"),
                )
            })
    };
    if required("version")? != "1" || required("mode")? != "local-session-plan" {
        return Err(browser_confirmation_error(
            locale,
            "P1060",
            "confirmation policy fields do not match version 1",
        ));
    }
    let browser_plan_digest = required("browser_plan_digest")?;
    if !is_sha256_digest(&browser_plan_digest) {
        return Err(browser_confirmation_error(
            locale,
            "P1060",
            "browser_plan_digest must use a lowercase sha256 digest",
        ));
    }
    let navigation_index = required("navigation_index")?
        .parse::<usize>()
        .map_err(|_| {
            browser_confirmation_error(
                locale,
                "P1060",
                "navigation_index must be an unsigned integer",
            )
        })?;
    let max_session_seconds = required("max_session_seconds")?
        .parse::<u32>()
        .map_err(|_| {
            browser_confirmation_error(
                locale,
                "P1060",
                "max_session_seconds must be an unsigned integer",
            )
        })?;
    if navigation_index == 0 || !(15..=300).contains(&max_session_seconds) {
        return Err(browser_confirmation_error(
            locale,
            "P1060",
            "navigation_index or max_session_seconds is outside the local session policy",
        ));
    }
    Ok(BrowserConfirmationPlanManifest {
        browser_plan_digest,
        navigation_index,
        max_session_seconds,
    })
}

fn parse_browser_handoff_audit_manifest(
    source: &str,
    locale: Locale,
) -> Result<BrowserHandoffAuditManifest, String> {
    let mut section = String::new();
    let mut fields = BTreeMap::new();
    for (line_number, raw_line) in source.lines().enumerate() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            if section != "audit" {
                return Err(browser_handoff_error(
                    locale,
                    "P1064",
                    &format!("unsupported audit section on line {}", line_number + 1),
                ));
            }
            continue;
        }
        let (key, raw_value) = line.split_once('=').ok_or_else(|| {
            browser_handoff_error(
                locale,
                "P1064",
                &format!("expected `key = value` on line {}", line_number + 1),
            )
        })?;
        let key = key.trim();
        if section != "audit" || !matches!(key, "version" | "mode" | "path" | "max_records") {
            return Err(browser_handoff_error(
                locale,
                "P1064",
                &format!("unsupported audit field on line {}", line_number + 1),
            ));
        }
        let value = if key == "max_records" {
            let value = raw_value.trim();
            if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(browser_handoff_error(
                    locale,
                    "P1064",
                    "max_records must be an unsigned integer",
                ));
            }
            value.to_string()
        } else {
            parse_browser_confirmation_quoted_value(raw_value, locale, line_number + 1).map_err(
                |_| {
                    browser_handoff_error(
                        locale,
                        "P1064",
                        &format!("expected a quoted audit value on line {}", line_number + 1),
                    )
                },
            )?
        };
        if fields.insert(key.to_string(), value).is_some() {
            return Err(browser_handoff_error(
                locale,
                "P1064",
                "duplicate audit field",
            ));
        }
    }
    let required = |key: &str| {
        fields
            .get(key)
            .cloned()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                browser_handoff_error(locale, "P1064", &format!("missing [audit] field `{key}`"))
            })
    };
    if required("version")? != "1" || required("mode")? != "redacted-local-v1" {
        return Err(browser_handoff_error(
            locale,
            "P1064",
            "audit policy fields do not match version 1",
        ));
    }
    let path_value = required("path")?;
    let path = safe_relative_path(&path_value).map_err(|_| {
        browser_handoff_error(
            locale,
            "P1064",
            "audit path must be project-relative and may not traverse directories",
        )
    })?;
    if path.components().count() < 2
        || path
            .components()
            .next()
            .and_then(|part| part.as_os_str().to_str())
            != Some("audit")
        || path.extension().and_then(|value| value.to_str()) != Some("jsonl")
    {
        return Err(browser_handoff_error(
            locale,
            "P1064",
            "audit path must be a `.jsonl` file below audit/",
        ));
    }
    let max_records = required("max_records")?.parse::<usize>().map_err(|_| {
        browser_handoff_error(locale, "P1064", "max_records must be an unsigned integer")
    })?;
    if !(1..=128).contains(&max_records) {
        return Err(browser_handoff_error(
            locale,
            "P1064",
            "max_records must be between 1 and 128",
        ));
    }
    Ok(BrowserHandoffAuditManifest { path, max_records })
}

fn parse_browser_draft_manifest(
    source: &str,
    locale: Locale,
) -> Result<BrowserDraftManifest, String> {
    let mut section = String::new();
    let mut fields = BTreeMap::new();
    for (line_number, raw_line) in source.lines().enumerate() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            if section != "draft" {
                return Err(browser_confirmation_error(
                    locale,
                    "P1065",
                    &format!("unsupported draft section on line {}", line_number + 1),
                ));
            }
            continue;
        }
        let (key, raw_value) = line.split_once('=').ok_or_else(|| {
            browser_confirmation_error(
                locale,
                "P1065",
                &format!("expected `key = value` on line {}", line_number + 1),
            )
        })?;
        let key = key.trim();
        if section != "draft"
            || !matches!(
                key,
                "version"
                    | "mode"
                    | "browser_plan_digest"
                    | "navigation_index"
                    | "action"
                    | "title"
                    | "body"
                    | "attachment_path"
                    | "max_review_seconds"
            )
        {
            return Err(browser_confirmation_error(
                locale,
                "P1065",
                &format!("unsupported draft field on line {}", line_number + 1),
            ));
        }
        let value = if matches!(key, "navigation_index" | "max_review_seconds") {
            let value = raw_value.trim();
            if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(browser_confirmation_error(
                    locale,
                    "P1065",
                    &format!("{key} must be an unsigned integer"),
                ));
            }
            value.to_string()
        } else {
            let value = raw_value.trim();
            if value.len() < 2
                || !value.starts_with('"')
                || !value.ends_with('"')
                || value[1..value.len() - 1].contains('"')
            {
                return Err(browser_confirmation_error(
                    locale,
                    "P1065",
                    &format!("expected a quoted draft value on line {}", line_number + 1),
                ));
            }
            value[1..value.len() - 1].to_string()
        };
        if fields.insert(key.to_string(), value).is_some() {
            return Err(browser_confirmation_error(
                locale,
                "P1065",
                "duplicate draft field",
            ));
        }
    }
    let required = |key: &str| {
        fields
            .get(key)
            .cloned()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                browser_confirmation_error(
                    locale,
                    "P1065",
                    &format!("missing [draft] field `{key}`"),
                )
            })
    };
    if required("version")? != "1" || required("mode")? != "user-review-only" {
        return Err(browser_confirmation_error(
            locale,
            "P1065",
            "draft policy fields do not match version 1",
        ));
    }
    let browser_plan_digest = required("browser_plan_digest")?;
    if !is_sha256_digest(&browser_plan_digest) {
        return Err(browser_confirmation_error(
            locale,
            "P1065",
            "browser_plan_digest must use a lowercase sha256 digest",
        ));
    }
    let navigation_index = required("navigation_index")?
        .parse::<usize>()
        .map_err(|_| {
            browser_confirmation_error(
                locale,
                "P1065",
                "navigation_index must be an unsigned integer",
            )
        })?;
    let action = required("action")?;
    if !matches!(
        action.as_str(),
        "form-draft"
            | "message-draft"
            | "upload-draft"
            | "download-request"
            | "account-request"
            | "payment-request"
    ) {
        return Err(browser_confirmation_error(
            locale,
            "P1065",
            "draft action is outside the fixed user-takeover vocabulary",
        ));
    }
    let title = required("title")?;
    let body = required("body")?;
    if title.len() > 160
        || body.len() > 4096
        || title.chars().any(char::is_control)
        || body.chars().any(char::is_control)
    {
        return Err(browser_confirmation_error(
            locale,
            "P1065",
            "draft title or body exceeds the local review policy",
        ));
    }
    let attachment_path = fields
        .get("attachment_path")
        .filter(|value| !value.is_empty())
        .map(|value| {
            let path = safe_relative_path(value).map_err(|_| {
                browser_confirmation_error(
                    locale,
                    "P1065",
                    "attachment_path must be project-relative and may not traverse directories",
                )
            })?;
            if path.components().count() < 2
                || path
                    .components()
                    .next()
                    .and_then(|part| part.as_os_str().to_str())
                    != Some("attachments")
            {
                return Err(browser_confirmation_error(
                    locale,
                    "P1065",
                    "attachment_path must be metadata below attachments/",
                ));
            }
            Ok(path)
        })
        .transpose()?;
    let max_review_seconds = required("max_review_seconds")?
        .parse::<u32>()
        .map_err(|_| {
            browser_confirmation_error(
                locale,
                "P1065",
                "max_review_seconds must be an unsigned integer",
            )
        })?;
    if navigation_index == 0 || !(15..=300).contains(&max_review_seconds) {
        return Err(browser_confirmation_error(
            locale,
            "P1065",
            "navigation_index or max_review_seconds is outside the local draft policy",
        ));
    }
    Ok(BrowserDraftManifest {
        browser_plan_digest,
        navigation_index,
        action,
        title,
        body,
        attachment_path,
        max_review_seconds,
    })
}

fn parse_browser_takeover_manifest(
    source: &str,
    locale: Locale,
) -> Result<BrowserTakeoverManifest, String> {
    let mut section = String::new();
    let mut fields = BTreeMap::new();
    for (line_number, raw_line) in source.lines().enumerate() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            if section != "takeover" {
                return Err(browser_confirmation_error(
                    locale,
                    "P1067",
                    &format!("unsupported takeover section on line {}", line_number + 1),
                ));
            }
            continue;
        }
        let (key, raw_value) = line.split_once('=').ok_or_else(|| {
            browser_confirmation_error(
                locale,
                "P1067",
                &format!("expected `key = value` on line {}", line_number + 1),
            )
        })?;
        let key = key.trim();
        if section != "takeover"
            || !matches!(
                key,
                "version"
                    | "mode"
                    | "browser_plan_digest"
                    | "navigation_index"
                    | "sensitive_action"
                    | "max_review_seconds"
            )
        {
            return Err(browser_confirmation_error(
                locale,
                "P1067",
                &format!("unsupported takeover field on line {}", line_number + 1),
            ));
        }
        let value = if matches!(key, "navigation_index" | "max_review_seconds") {
            let value = raw_value.trim();
            if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(browser_confirmation_error(
                    locale,
                    "P1067",
                    &format!("{key} must be an unsigned integer"),
                ));
            }
            value.to_string()
        } else {
            let value = raw_value.trim();
            if value.len() < 2
                || !value.starts_with('"')
                || !value.ends_with('"')
                || value[1..value.len() - 1].contains('"')
            {
                return Err(browser_confirmation_error(
                    locale,
                    "P1067",
                    &format!(
                        "expected a quoted takeover value on line {}",
                        line_number + 1
                    ),
                ));
            }
            value[1..value.len() - 1].to_string()
        };
        if fields.insert(key.to_string(), value).is_some() {
            return Err(browser_confirmation_error(
                locale,
                "P1067",
                "duplicate takeover field",
            ));
        }
    }
    let required = |key: &str| {
        fields
            .get(key)
            .cloned()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                browser_confirmation_error(
                    locale,
                    "P1067",
                    &format!("missing [takeover] field `{key}`"),
                )
            })
    };
    if required("version")? != "1" || required("mode")? != "visible-user-takeover-only" {
        return Err(browser_confirmation_error(
            locale,
            "P1067",
            "takeover policy fields do not match version 1",
        ));
    }
    let browser_plan_digest = required("browser_plan_digest")?;
    if !is_sha256_digest(&browser_plan_digest) {
        return Err(browser_confirmation_error(
            locale,
            "P1067",
            "browser_plan_digest must use a lowercase sha256 digest",
        ));
    }
    let navigation_index = required("navigation_index")?
        .parse::<usize>()
        .map_err(|_| {
            browser_confirmation_error(
                locale,
                "P1067",
                "navigation_index must be an unsigned integer",
            )
        })?;
    let sensitive_action = required("sensitive_action")?;
    if !matches!(
        sensitive_action.as_str(),
        "login"
            | "captcha"
            | "form-completion"
            | "message-post"
            | "upload"
            | "download"
            | "account-change"
            | "purchase"
            | "payment"
    ) {
        return Err(browser_confirmation_error(
            locale,
            "P1067",
            "sensitive_action is outside the fixed user-takeover vocabulary",
        ));
    }
    let max_review_seconds = required("max_review_seconds")?
        .parse::<u32>()
        .map_err(|_| {
            browser_confirmation_error(
                locale,
                "P1067",
                "max_review_seconds must be an unsigned integer",
            )
        })?;
    if navigation_index == 0 || !(15..=300).contains(&max_review_seconds) {
        return Err(browser_confirmation_error(
            locale,
            "P1067",
            "navigation_index or max_review_seconds is outside the local takeover policy",
        ));
    }
    Ok(BrowserTakeoverManifest {
        browser_plan_digest,
        navigation_index,
        sensitive_action,
        max_review_seconds,
    })
}

fn safe_public_https_url(value: &str) -> bool {
    let Some(host) = value.strip_prefix("https://") else {
        return false;
    };
    if host.is_empty()
        || host.contains(['/', '?', '#', '@'])
        || host.chars().any(char::is_whitespace)
    {
        return false;
    }
    !matches!(
        host.to_ascii_lowercase().as_str(),
        "localhost" | "127.0.0.1" | "[::1]"
    )
}

fn safe_deployment_rollback_path(value: &str) -> Result<PathBuf, String> {
    let path = safe_relative_path(value)
        .map_err(|_| "P1046: rollback must be a project-relative `.json` path".to_string())?;
    if !value.ends_with(".json") {
        return Err("P1046: rollback must be a project-relative `.json` path".to_string());
    }
    Ok(path)
}

fn parse_deployment_manifest(source: &str) -> Result<DeploymentManifest, String> {
    let mut section = String::new();
    let mut fields = BTreeMap::new();
    let mut environment_names = BTreeSet::new();
    let mut environment_seen = false;
    for (line_number, raw_line) in source.lines().enumerate() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            if !matches!(section.as_str(), "deployment" | "environment") {
                return Err(format!(
                    "P1046: unsupported deployment section `{section}` on line {}",
                    line_number + 1
                ));
            }
            continue;
        }
        let (key, raw_value) = line
            .split_once('=')
            .ok_or_else(|| format!("P1046: expected `key = value` on line {}", line_number + 1))?;
        let key = key.trim();
        if section == "environment" {
            if key != "names" || environment_seen {
                return Err(format!(
                    "P1046: only one `environment.names` field is allowed on line {}",
                    line_number + 1
                ));
            }
            environment_seen = true;
            for name in parse_manifest_string_list(raw_value, line_number + 1)? {
                if !is_safe_environment_name(&name) {
                    return Err(format!(
                        "P1046: unsafe environment variable name `{name}` on line {}",
                        line_number + 1
                    ));
                }
                if !environment_names.insert(name.clone()) {
                    return Err(format!(
                        "P1046: duplicate environment variable name `{name}` on line {}",
                        line_number + 1
                    ));
                }
            }
            continue;
        }
        if section != "deployment"
            || !matches!(
                key,
                "version" | "entry" | "target" | "base_url" | "rollback"
            )
        {
            return Err(format!(
                "P1046: unsupported deployment field `{key}` on line {}",
                line_number + 1
            ));
        }
        let value = raw_value.trim();
        if value.len() < 2 || !value.starts_with('"') || !value.ends_with('"') {
            return Err(format!(
                "P1046: deployment values must be quoted strings on line {}",
                line_number + 1
            ));
        }
        if fields
            .insert(key.to_string(), value[1..value.len() - 1].to_string())
            .is_some()
        {
            return Err(format!(
                "P1046: duplicate deployment field `{key}` on line {}",
                line_number + 1
            ));
        }
    }
    let required = |key: &str| {
        fields
            .get(key)
            .cloned()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("P1046: missing `[deployment]` field `{key}`"))
    };
    if required("version")? != "1" {
        return Err("P1046: deployment manifest version must be `1`".to_string());
    }
    let entry = required("entry")?;
    safe_project_relative_path(&entry)
        .map_err(|_| "P1046: deployment entry must be a project-relative `.pd` path".to_string())?;
    let target = required("target")?;
    if !matches!(target.as_str(), "static" | "container" | "loopback") {
        return Err("P1046: target must be `static`, `container`, or `loopback`".to_string());
    }
    let base_url = required("base_url")?;
    if !safe_public_https_url(&base_url) {
        return Err("P1046: base_url must be a public HTTPS origin without path, query, fragment, or credentials".to_string());
    }
    let rollback = required("rollback")?;
    safe_deployment_rollback_path(&rollback)?;
    Ok(DeploymentManifest {
        entry,
        target,
        base_url,
        environment_names,
        rollback,
    })
}

fn is_safe_render_identifier(value: &str, prefix: &str) -> bool {
    value.starts_with(prefix)
        && value.len() > prefix.len()
        && value.len() <= 128
        && value[prefix.len()..]
            .chars()
            .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
}

fn is_safe_render_repository(value: &str) -> bool {
    let mut segments = value.split('/');
    let Some(owner) = segments.next() else {
        return false;
    };
    let Some(repository) = segments.next() else {
        return false;
    };
    segments.next().is_none()
        && !owner.is_empty()
        && !repository.is_empty()
        && owner.len() <= 64
        && repository.len() <= 100
        && [owner, repository].iter().all(|segment| {
            segment.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
            }) && !segment.starts_with('.')
                && !segment.ends_with('.')
        })
}

fn is_safe_render_branch(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && !value.starts_with('/')
        && !value.starts_with('.')
        && !value.ends_with('/')
        && !value.ends_with('.')
        && !value.contains("//")
        && !value.contains("..")
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '/' | '-' | '_' | '.')
        })
}

fn is_immutable_git_commit(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn parse_render_release_manifest(source: &str) -> Result<RenderReleaseManifest, String> {
    let mut section = String::new();
    let mut fields = BTreeMap::new();
    for (line_number, raw_line) in source.lines().enumerate() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            if section != "render" {
                return Err(format!(
                    "P1048: unsupported Render section `{section}` on line {}",
                    line_number + 1
                ));
            }
            continue;
        }
        let (key, raw_value) = line
            .split_once('=')
            .ok_or_else(|| format!("P1048: expected `key = value` on line {}", line_number + 1))?;
        let key = key.trim();
        if section != "render"
            || !matches!(
                key,
                "version"
                    | "mode"
                    | "service"
                    | "repository"
                    | "branch"
                    | "commit"
                    | "rollback_deploy"
            )
        {
            return Err(format!(
                "P1048: unsupported Render field `{key}` on line {}",
                line_number + 1
            ));
        }
        let value = raw_value.trim();
        if value.len() < 2 || !value.starts_with('"') || !value.ends_with('"') {
            return Err(format!(
                "P1048: Render values must be quoted strings on line {}",
                line_number + 1
            ));
        }
        if fields
            .insert(key.to_string(), value[1..value.len() - 1].to_string())
            .is_some()
        {
            return Err(format!(
                "P1048: duplicate Render field `{key}` on line {}",
                line_number + 1
            ));
        }
    }
    let required = |key: &str| {
        fields
            .get(key)
            .cloned()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("P1048: missing `[render]` field `{key}`"))
    };
    if required("version")? != "1" {
        return Err("P1048: Render manifest version must be `1`".to_string());
    }
    if required("mode")? != "git-linked" {
        return Err("P1048: Render mode must be `git-linked`".to_string());
    }
    let service = required("service")?;
    if !is_safe_render_identifier(&service, "srv-") {
        return Err("P1048: Render service must be a `srv-` identifier".to_string());
    }
    let repository = required("repository")?;
    if !is_safe_render_repository(&repository) {
        return Err("P1048: Render repository must be an `owner/repository` identity".to_string());
    }
    let branch = required("branch")?;
    if !is_safe_render_branch(&branch) {
        return Err("P1048: Render branch contains unsafe characters".to_string());
    }
    let commit = required("commit")?.to_ascii_lowercase();
    if !is_immutable_git_commit(&commit) {
        return Err(
            "P1048: Render commit must be a full 40 or 64 character commit SHA".to_string(),
        );
    }
    let rollback_deploy = fields.get("rollback_deploy").cloned();
    if let Some(rollback_deploy) = &rollback_deploy {
        if !is_safe_render_identifier(rollback_deploy, "dep-") {
            return Err("P1048: Render rollback_deploy must be a `dep-` identifier".to_string());
        }
    }
    Ok(RenderReleaseManifest {
        service,
        repository,
        branch,
        commit,
        rollback_deploy,
    })
}

fn parse_render_api_manifest(source: &str) -> Result<RenderApiManifest, String> {
    let mut section = String::new();
    let mut fields = BTreeMap::new();
    for (line_number, raw_line) in source.lines().enumerate() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            if section != "render_api" {
                return Err(format!(
                    "P1048: unsupported Render API section `{section}` on line {}",
                    line_number + 1
                ));
            }
            continue;
        }
        let (key, raw_value) = line
            .split_once('=')
            .ok_or_else(|| format!("P1048: expected `key = value` on line {}", line_number + 1))?;
        let key = key.trim();
        if section != "render_api"
            || !matches!(
                key,
                "version" | "service" | "token_env" | "commit" | "clear_cache" | "rollback_deploy"
            )
        {
            return Err(format!(
                "P1048: unsupported Render API field `{key}` on line {}",
                line_number + 1
            ));
        }
        let value = raw_value.trim();
        if value.len() < 2 || !value.starts_with('"') || !value.ends_with('"') {
            return Err(format!(
                "P1048: Render API values must be quoted strings on line {}",
                line_number + 1
            ));
        }
        if fields
            .insert(key.to_string(), value[1..value.len() - 1].to_string())
            .is_some()
        {
            return Err(format!(
                "P1048: duplicate Render API field `{key}` on line {}",
                line_number + 1
            ));
        }
    }
    let required = |key: &str| {
        fields
            .get(key)
            .cloned()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("P1048: missing `[render_api]` field `{key}`"))
    };
    if required("version")? != "1" {
        return Err("P1048: Render API manifest version must be `1`".to_string());
    }
    let service = required("service")?;
    if !is_safe_render_identifier(&service, "srv-") {
        return Err("P1048: Render API service must be a `srv-` identifier".to_string());
    }
    let token_env = required("token_env")?;
    if !is_safe_environment_name(&token_env) {
        return Err(
            "P1048: Render API token_env must be a safe environment variable name".to_string(),
        );
    }
    let commit = required("commit")?.to_ascii_lowercase();
    if !is_immutable_git_commit(&commit) {
        return Err(
            "P1048: Render API commit must be a full 40 or 64 character commit SHA".to_string(),
        );
    }
    let clear_cache = required("clear_cache")?;
    if !matches!(clear_cache.as_str(), "clear" | "do_not_clear") {
        return Err("P1048: Render API clear_cache must be `clear` or `do_not_clear`".to_string());
    }
    let rollback_deploy = required("rollback_deploy")?;
    if !is_safe_render_identifier(&rollback_deploy, "dep-") {
        return Err("P1048: Render API rollback_deploy must be a `dep-` identifier".to_string());
    }
    Ok(RenderApiManifest {
        service,
        token_env,
        commit,
        clear_cache,
        rollback_deploy,
    })
}

fn is_safe_android_application_id(value: &str) -> bool {
    let segments = value.split('.').collect::<Vec<_>>();
    segments.len() >= 2
        && value.len() <= 180
        && segments.iter().all(|segment| {
            let mut characters = segment.chars();
            matches!(characters.next(), Some(first) if first.is_ascii_alphabetic())
                && segment.len() <= 63
                && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
        })
}

fn parse_android_permission_list(
    value: &str,
    line_number: usize,
) -> Result<BTreeSet<String>, String> {
    let value = value.trim();
    if !(value.starts_with('[') && value.ends_with(']')) {
        return Err(format!(
            "P1049: Android permissions must use a quoted string list on line {line_number}"
        ));
    }
    let permitted = [
        "android.permission.INTERNET",
        "android.permission.ACCESS_NETWORK_STATE",
        "android.permission.POST_NOTIFICATIONS",
        "android.permission.CAMERA",
        "android.permission.RECORD_AUDIO",
    ];
    let mut permissions = BTreeSet::new();
    let inner = value[1..value.len() - 1].trim();
    if inner.is_empty() {
        return Ok(permissions);
    }
    for raw_permission in inner.split(',') {
        let permission = raw_permission.trim();
        if permission.len() < 2 || !permission.starts_with('"') || !permission.ends_with('"') {
            return Err(format!(
                "P1049: Android permissions must be quoted strings on line {line_number}"
            ));
        }
        let permission = permission[1..permission.len() - 1].to_string();
        if !permitted.contains(&permission.as_str()) {
            return Err(format!(
                "P1049: Android permission `{permission}` is not in Padma's reviewed allowlist"
            ));
        }
        if !permissions.insert(permission.clone()) {
            return Err(format!(
                "P1049: duplicate Android permission `{permission}` on line {line_number}"
            ));
        }
    }
    Ok(permissions)
}

fn parse_android_build_manifest(source: &str) -> Result<AndroidBuildManifest, String> {
    let mut section = String::new();
    let mut fields = BTreeMap::new();
    let mut permissions = BTreeSet::new();
    let mut permissions_seen = false;
    for (line_number, raw_line) in source.lines().enumerate() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            if !matches!(section.as_str(), "android" | "permissions") {
                return Err(format!(
                    "P1049: unsupported Android manifest section `{section}` on line {}",
                    line_number + 1
                ));
            }
            continue;
        }
        let (key, raw_value) = line
            .split_once('=')
            .ok_or_else(|| format!("P1049: expected `key = value` on line {}", line_number + 1))?;
        let key = key.trim();
        if section == "permissions" {
            if key != "names" || permissions_seen {
                return Err(format!(
                    "P1049: only one `permissions.names` field is allowed on line {}",
                    line_number + 1
                ));
            }
            permissions_seen = true;
            permissions = parse_android_permission_list(raw_value, line_number + 1)?;
            continue;
        }
        if section != "android"
            || !matches!(
                key,
                "version"
                    | "application_id"
                    | "min_sdk"
                    | "target_sdk"
                    | "artifact"
                    | "signing_key_env"
                    | "signing_cert_sha256"
            )
        {
            return Err(format!(
                "P1049: unsupported Android manifest field `{key}` on line {}",
                line_number + 1
            ));
        }
        let value = raw_value.trim();
        let value = if matches!(key, "min_sdk" | "target_sdk") {
            if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(format!(
                    "P1049: Android `{key}` must be an unsigned integer on line {}",
                    line_number + 1
                ));
            }
            value.to_string()
        } else {
            if value.len() < 2 || !value.starts_with('"') || !value.ends_with('"') {
                return Err(format!(
                    "P1049: Android `{key}` must be a quoted string on line {}",
                    line_number + 1
                ));
            }
            value[1..value.len() - 1].to_string()
        };
        if fields.insert(key.to_string(), value).is_some() {
            return Err(format!(
                "P1049: duplicate Android manifest field `{key}` on line {}",
                line_number + 1
            ));
        }
    }
    let required = |key: &str| {
        fields
            .get(key)
            .cloned()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("P1049: missing `[android]` field `{key}`"))
    };
    if required("version")? != "1" {
        return Err("P1049: Android manifest version must be `1`".to_string());
    }
    let application_id = required("application_id")?;
    if !is_safe_android_application_id(&application_id) {
        return Err("P1049: Android application_id must be a dotted identifier".to_string());
    }
    let min_sdk = required("min_sdk")?
        .parse::<u32>()
        .map_err(|_| "P1049: Android min_sdk must be an unsigned integer".to_string())?;
    let target_sdk = required("target_sdk")?
        .parse::<u32>()
        .map_err(|_| "P1049: Android target_sdk must be an unsigned integer".to_string())?;
    if !(23..=35).contains(&min_sdk) || !(min_sdk..=35).contains(&target_sdk) {
        return Err(
            "P1049: Android SDK levels must be 23 through 35 with target_sdk >= min_sdk"
                .to_string(),
        );
    }
    let artifact = required("artifact")?;
    safe_gui_relative_path(&artifact, Some(".apk")).map_err(|_| {
        "P1049: Android artifact must be a project-relative `.apk` path".to_string()
    })?;
    let signing_key_env = required("signing_key_env")?;
    if !is_safe_environment_name(&signing_key_env) {
        return Err(
            "P1049: Android signing_key_env must be a safe environment variable name".to_string(),
        );
    }
    let signing_cert_sha256 = required("signing_cert_sha256")?;
    if !is_sha256_digest(&signing_cert_sha256) {
        return Err(
            "P1049: Android signing_cert_sha256 must use a lowercase sha256 digest".to_string(),
        );
    }
    Ok(AndroidBuildManifest {
        application_id,
        min_sdk,
        target_sdk,
        artifact,
        signing_key_env,
        signing_cert_sha256,
        permissions,
    })
}

fn safe_gui_relative_path(value: &str, extension: Option<&str>) -> Result<PathBuf, String> {
    let path = Path::new(value);
    let forbidden_location = value.to_ascii_lowercase().contains("@downloads")
        || value.contains(':')
        || value.contains('\\')
        || value.contains("://");
    if value.is_empty()
        || value.len() > 512
        || forbidden_location
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::CurDir
                    | std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        })
        || extension.is_some_and(|extension| !value.ends_with(extension))
    {
        return Err(
            "P1047: GUI path must be a project-relative local path without `..`, URLs, `@downloads`, or symlink indirection"
                .to_string(),
        );
    }
    Ok(path.to_path_buf())
}

fn parse_gui_manifest(source: &str) -> Result<GuiManifest, String> {
    let mut section = String::new();
    let mut fields = BTreeMap::new();
    for (line_number, raw_line) in source.lines().enumerate() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            if section != "gui" {
                return Err(format!(
                    "P1047: unsupported GUI manifest section `{section}` on line {}",
                    line_number + 1
                ));
            }
            continue;
        }
        let (key, raw_value) = line
            .split_once('=')
            .ok_or_else(|| format!("P1047: expected `key = value` on line {}", line_number + 1))?;
        let key = key.trim();
        if section != "gui" || !matches!(key, "version" | "backend" | "entry" | "assets" | "title")
        {
            return Err(format!(
                "P1047: unsupported GUI manifest field `{key}` on line {}",
                line_number + 1
            ));
        }
        let value = raw_value.trim();
        let value = if key == "version" {
            if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(format!(
                    "P1047: GUI version must be an unsigned integer on line {}",
                    line_number + 1
                ));
            }
            value.to_string()
        } else {
            if value.len() < 2 || !value.starts_with('"') || !value.ends_with('"') {
                return Err(format!(
                    "P1047: GUI values must be quoted strings on line {}",
                    line_number + 1
                ));
            }
            value[1..value.len() - 1].to_string()
        };
        if fields.insert(key.to_string(), value).is_some() {
            return Err(format!(
                "P1047: duplicate GUI manifest field `{key}` on line {}",
                line_number + 1
            ));
        }
    }
    let required = |key: &str| {
        fields
            .get(key)
            .cloned()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("P1047: missing `[gui]` field `{key}`"))
    };
    let version = required("version")?
        .parse::<u32>()
        .map_err(|_| "P1047: GUI version must be an unsigned integer".to_string())?;
    if version != 1 {
        return Err("P1047: GUI manifest version must be `1`".to_string());
    }
    let backend = required("backend")?;
    if backend != "html-static" {
        return Err("P1047: GUI backend must be `html-static`".to_string());
    }
    let entry = required("entry")?;
    safe_gui_relative_path(&entry, Some(".html"))?;
    let assets = required("assets")?;
    safe_gui_relative_path(&assets, None)?;
    let title = required("title")?;
    if title.chars().count() > 128 || title.chars().any(char::is_control) {
        return Err("P1047: GUI title must contain at most 128 printable characters".to_string());
    }
    Ok(GuiManifest {
        version,
        backend,
        entry,
        assets,
        title,
    })
}

fn is_safe_package_name(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_alphabetic())
        && value.len() <= 64
        && chars
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}

fn is_semver(value: &str) -> bool {
    let pieces = value.split('.').collect::<Vec<_>>();
    pieces.len() == 3
        && pieces
            .iter()
            .all(|piece| !piece.is_empty() && piece.bytes().all(|byte| byte.is_ascii_digit()))
}

fn is_sha256_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

fn safe_package_relative_path(value: &str) -> Result<PathBuf, String> {
    let path = safe_relative_path(value)
        .map_err(|_| "P1032: package path must be a relative path without `..`".to_string())?;
    if path
        .components()
        .next()
        .and_then(|component| component.as_os_str().to_str())
        != Some("packages")
    {
        return Err(
            "P1032: package path must be inside the project `packages/` directory".to_string(),
        );
    }
    Ok(path)
}

fn parse_package_manifest(source: &str) -> Result<PackageManifest, String> {
    let mut section = String::new();
    let mut fields = BTreeMap::new();
    let mut capabilities = BTreeSet::new();
    let mut capability_fields = BTreeSet::new();
    for (line_number, raw_line) in source.lines().enumerate() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            if !matches!(section.as_str(), "package" | "capabilities") {
                return Err(format!(
                    "P1032: unsupported package manifest section `{section}` on line {}",
                    line_number + 1
                ));
            }
            continue;
        }
        let (key, raw_value) = line
            .split_once('=')
            .ok_or_else(|| format!("P1032: expected `key = value` on line {}", line_number + 1))?;
        let key = key.trim();
        if section == "capabilities" {
            if !capability_fields.insert(key.to_string()) {
                return Err(format!(
                    "P1032: duplicate package capability `{key}` on line {}",
                    line_number + 1
                ));
            }
            capabilities.extend(capability_grants_for_field(
                key,
                parse_manifest_string_list(raw_value, line_number + 1)?,
                line_number + 1,
            )?);
            continue;
        }
        if section != "package"
            || !matches!(key, "name" | "version" | "entry" | "exports" | "digest")
        {
            return Err(format!(
                "P1032: unsupported package manifest field `{key}` on line {}",
                line_number + 1
            ));
        }
        let value = if key == "exports" {
            parse_manifest_string_list(raw_value, line_number + 1)?.join("\u{1f}")
        } else {
            let raw_value = raw_value.trim();
            if raw_value.len() < 2 || !raw_value.starts_with('"') || !raw_value.ends_with('"') {
                return Err(format!(
                    "P1032: package `{key}` must be a quoted string on line {}",
                    line_number + 1
                ));
            }
            raw_value[1..raw_value.len() - 1].to_string()
        };
        if fields.insert(key.to_string(), value).is_some() {
            return Err(format!(
                "P1032: duplicate package field `{key}` on line {}",
                line_number + 1
            ));
        }
    }
    let required = |key: &str| {
        fields
            .get(key)
            .cloned()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("P1032: missing `[package]` field `{key}`"))
    };
    let name = required("name")?;
    let version = required("version")?;
    let entry = required("entry")?;
    let digest = required("digest")?;
    if !is_safe_package_name(&name) || !is_semver(&version) || !is_sha256_digest(&digest) {
        return Err("P1032: package name, version, or sha256 digest is invalid".to_string());
    }
    safe_project_relative_path(&entry)?;
    let exports = fields
        .get("exports")
        .map(|value| {
            value
                .split('\u{1f}')
                .filter(|item| !item.is_empty())
                .map(str::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    if exports.is_empty()
        || exports
            .iter()
            .any(|item| item.len() > 64 || item.contains(['/', '\\', ' ']))
    {
        return Err("P1032: package exports must be a non-empty safe string list".to_string());
    }
    Ok(PackageManifest {
        name,
        version,
        entry,
        exports,
        capabilities,
        digest,
    })
}

fn sha256_bytes(input: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut bytes = input.to_vec();
    let bit_length = (bytes.len() as u64).saturating_mul(8);
    bytes.push(0x80);
    while bytes.len() % 64 != 56 {
        bytes.push(0);
    }
    bytes.extend_from_slice(&bit_length.to_be_bytes());
    let mut hash = [
        0x6a09e667u32,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];
    for chunk in bytes.chunks_exact(64) {
        let mut words = [0u32; 64];
        for (index, word) in words.iter_mut().take(16).enumerate() {
            *word = u32::from_be_bytes(chunk[index * 4..index * 4 + 4].try_into().unwrap());
        }
        for index in 16..64 {
            let s0 = words[index - 15].rotate_right(7)
                ^ words[index - 15].rotate_right(18)
                ^ (words[index - 15] >> 3);
            let s1 = words[index - 2].rotate_right(17)
                ^ words[index - 2].rotate_right(19)
                ^ (words[index - 2] >> 10);
            words[index] = words[index - 16]
                .wrapping_add(s0)
                .wrapping_add(words[index - 7])
                .wrapping_add(s1);
        }
        let mut state = hash;
        for index in 0..64 {
            let s1 =
                state[4].rotate_right(6) ^ state[4].rotate_right(11) ^ state[4].rotate_right(25);
            let choice = (state[4] & state[5]) ^ ((!state[4]) & state[6]);
            let one = state[7]
                .wrapping_add(s1)
                .wrapping_add(choice)
                .wrapping_add(K[index])
                .wrapping_add(words[index]);
            let s0 =
                state[0].rotate_right(2) ^ state[0].rotate_right(13) ^ state[0].rotate_right(22);
            let majority = (state[0] & state[1]) ^ (state[0] & state[2]) ^ (state[1] & state[2]);
            let two = s0.wrapping_add(majority);
            state = [
                one.wrapping_add(two),
                state[0],
                state[1],
                state[2],
                state[3].wrapping_add(one),
                state[4],
                state[5],
                state[6],
            ];
        }
        for (target, value) in hash.iter_mut().zip(state) {
            *target = target.wrapping_add(value);
        }
    }
    let mut output = [0u8; 32];
    for (index, word) in hash.iter().enumerate() {
        output[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    output
}

fn hex_encode(input: &[u8]) -> String {
    input.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn hex_decode(input: &str) -> Option<Vec<u8>> {
    if input.len() % 2 != 0 || !input.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    (0..input.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&input[index..index + 2], 16).ok())
        .collect()
}

fn sha256_hex(input: &[u8]) -> String {
    hex_encode(&sha256_bytes(input))
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut normalized = [0u8; 64];
    if key.len() > normalized.len() {
        normalized[..32].copy_from_slice(&sha256_bytes(key));
    } else {
        normalized[..key.len()].copy_from_slice(key);
    }
    let mut inner = [0u8; 64];
    let mut outer = [0u8; 64];
    for index in 0..64 {
        inner[index] = normalized[index] ^ 0x36;
        outer[index] = normalized[index] ^ 0x5c;
    }
    let mut inner_input = Vec::with_capacity(64 + message.len());
    inner_input.extend_from_slice(&inner);
    inner_input.extend_from_slice(message);
    let inner_hash = sha256_bytes(&inner_input);
    let mut outer_input = Vec::with_capacity(96);
    outer_input.extend_from_slice(&outer);
    outer_input.extend_from_slice(&inner_hash);
    sha256_bytes(&outer_input)
}

fn pbkdf2_sha256(password: &[u8], salt: &[u8], iterations: u32) -> [u8; 32] {
    let mut block = Vec::with_capacity(salt.len() + 4);
    block.extend_from_slice(salt);
    block.extend_from_slice(&1u32.to_be_bytes());
    let mut value = hmac_sha256(password, &block);
    let mut derived = value;
    for _ in 1..iterations {
        value = hmac_sha256(password, &value);
        for index in 0..derived.len() {
            derived[index] ^= value[index];
        }
    }
    derived
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    let mut difference = left.len() ^ right.len();
    for index in 0..left.len().max(right.len()) {
        let left_byte = left.get(index).copied().unwrap_or(0);
        let right_byte = right.get(index).copied().unwrap_or(0);
        difference |= usize::from(left_byte ^ right_byte);
    }
    difference == 0
}

fn secure_random_bytes(length: usize) -> Result<Vec<u8>, ()> {
    let mut random = vec![0u8; length];
    fs::File::open("/dev/urandom")
        .and_then(|mut source| source.read_exact(&mut random))
        .map_err(|_| ())?;
    Ok(random)
}

fn is_safe_secret_environment_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.bytes().enumerate().all(|(index, byte)| match index {
            0 => byte.is_ascii_uppercase(),
            _ => byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_',
        })
}

fn session_secret_from_environment(
    environment_name: &str,
    locale: Locale,
    position: Position,
) -> Result<Vec<u8>, PadmaError> {
    if !is_safe_secret_environment_name(environment_name) {
        return Err(error_for(
            locale,
            "P1045",
            position,
            "session secret environment name",
        ));
    }
    let secret = env::var(environment_name)
        .map_err(|_| error_for(locale, "P1045", position, environment_name))?;
    if secret.len() < 32 || secret.len() > 4_096 {
        return Err(error_for(locale, "P1045", position, environment_name));
    }
    Ok(secret.into_bytes())
}

fn password_record_from_secret(
    password: &str,
    locale: Locale,
    position: Position,
) -> Result<String, PadmaError> {
    if password.is_empty() || password.len() > AUTH_PASSWORD_MAX_BYTES || password.contains('\0') {
        return Err(error_for(locale, "P1045", position, "password"));
    }
    let salt = secure_random_bytes(16)
        .map_err(|_| error_for(locale, "P1047", position, "password salt"))?;
    let digest = pbkdf2_sha256(password.as_bytes(), &salt, AUTH_PBKDF2_ITERATIONS);
    Ok(format!(
        "$padma-pbkdf2-sha256${}${}${}",
        AUTH_PBKDF2_ITERATIONS,
        hex_encode(&salt),
        hex_encode(&digest)
    ))
}

fn verify_password_record(record: &str, password: &str) -> bool {
    let pieces = record.split('$').collect::<Vec<_>>();
    if pieces.len() != 5
        || pieces[0] != ""
        || pieces[1] != "padma-pbkdf2-sha256"
        || pieces[2] != AUTH_PBKDF2_ITERATIONS.to_string()
        || password.is_empty()
        || password.len() > AUTH_PASSWORD_MAX_BYTES
    {
        return false;
    }
    let Some(salt) = hex_decode(pieces[3]) else {
        return false;
    };
    let Some(expected) = hex_decode(pieces[4]) else {
        return false;
    };
    salt.len() == 16
        && expected.len() == 32
        && constant_time_eq(
            &pbkdf2_sha256(password.as_bytes(), &salt, AUTH_PBKDF2_ITERATIONS),
            &expected,
        )
}

fn unix_seconds() -> Result<u64, ()> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| ())
}

fn is_safe_session_subject(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && !value.chars().any(|character| character.is_control())
}

fn issue_signed_session(
    subject: &str,
    secret: &[u8],
    ttl_seconds: u64,
    locale: Locale,
    position: Position,
) -> Result<String, PadmaError> {
    if !is_safe_session_subject(subject) || !(60..=AUTH_SESSION_MAX_SECONDS).contains(&ttl_seconds)
    {
        return Err(error_for(
            locale,
            "P1045",
            position,
            "session subject or lifetime",
        ));
    }
    let expires_at = unix_seconds()
        .map_err(|_| error_for(locale, "P1047", position, "system clock"))?
        .checked_add(ttl_seconds)
        .ok_or_else(|| error_for(locale, "P1045", position, "session lifetime"))?;
    let nonce = secure_random_bytes(16)
        .map_err(|_| error_for(locale, "P1047", position, "session nonce"))?;
    let unsigned = format!(
        "v1.{}.{}.{}",
        hex_encode(subject.as_bytes()),
        expires_at,
        hex_encode(&nonce)
    );
    Ok(format!(
        "{unsigned}.{}",
        hex_encode(&hmac_sha256(secret, unsigned.as_bytes()))
    ))
}

fn verify_signed_session(token: &str, secret: &[u8]) -> Option<(String, u64)> {
    let pieces = token.split('.').collect::<Vec<_>>();
    if pieces.len() != 5 || pieces[0] != "v1" || pieces[2].is_empty() || pieces[3].len() != 32 {
        return None;
    }
    let expires_at = pieces[2].parse::<u64>().ok()?;
    if unix_seconds().ok()? >= expires_at {
        return None;
    }
    let subject = String::from_utf8(hex_decode(pieces[1])?).ok()?;
    if !is_safe_session_subject(&subject) || hex_decode(pieces[3])?.len() != 16 {
        return None;
    }
    let signature = hex_decode(pieces[4])?;
    if signature.len() != 32 {
        return None;
    }
    let unsigned = pieces[..4].join(".");
    if !constant_time_eq(&signature, &hmac_sha256(secret, unsigned.as_bytes())) {
        return None;
    }
    Some((subject, expires_at))
}

fn is_safe_cookie_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn safe_project_relative_path(value: &str) -> Result<PathBuf, String> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || !value.ends_with(".pd")
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        })
    {
        return Err("P1032: project entry must be a relative `.pd` path without `..`".to_string());
    }
    Ok(path.to_path_buf())
}

fn load_project_manifest(directory: &Path) -> Result<(ProjectManifest, PathBuf), String> {
    let manifest_path = directory.join("padma.toml");
    let content = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("P1032: cannot read `{}`: {error}", manifest_path.display()))?;
    let manifest = parse_project_manifest(&content)?;
    let entry = directory.join(safe_project_relative_path(&manifest.entry)?);
    Ok((manifest, entry))
}

fn lint_disabled_rules_for_path(path: &str) -> Result<BTreeSet<String>, String> {
    let source_path = Path::new(path);
    let mut directory = source_path.parent().unwrap_or_else(|| Path::new("."));
    loop {
        let manifest_path = directory.join("padma.toml");
        if manifest_path.is_file() {
            let source = fs::read_to_string(&manifest_path).map_err(|error| {
                format!("P1032: cannot read `{}`: {error}", manifest_path.display())
            })?;
            return parse_project_manifest(&source).map(|manifest| manifest.lint_disabled);
        }
        let Some(parent) = directory.parent() else {
            return Ok(BTreeSet::new());
        };
        if parent == directory {
            return Ok(BTreeSet::new());
        }
        directory = parent;
    }
}

fn initialize_project(directory: &Path) -> Result<ProjectManifest, String> {
    if directory.exists() {
        let mut entries = fs::read_dir(directory)
            .map_err(|error| format!("P1032: cannot inspect `{}`: {error}", directory.display()))?;
        if entries.next().is_some() {
            return Err(format!(
                "P1032: project directory `{}` is not empty",
                directory.display()
            ));
        }
    } else {
        fs::create_dir_all(directory)
            .map_err(|error| format!("P1032: cannot create `{}`: {error}", directory.display()))?;
    }
    let name = directory
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty() && *name != ".")
        .unwrap_or("padma-project")
        .to_string();
    let manifest = ProjectManifest {
        name,
        version: "0.1.0".to_string(),
        entry: "src/main.pd".to_string(),
        locale: "bn".to_string(),
        capabilities: BTreeSet::new(),
        lint_disabled: BTreeSet::new(),
        dependencies: BTreeMap::new(),
    };
    fs::create_dir_all(directory.join("src"))
        .map_err(|error| format!("P1032: cannot create project source directory: {error}"))?;
    fs::write(
        directory.join("padma.toml"),
        format!(
            "[padma]\nname = \"{}\"\nversion = \"{}\"\nentry = \"{}\"\nlocale = \"{}\"\n\n# Project mode denies sensitive actions until they are granted below.\n[capabilities]\ndatabase = []\nidentity = []\ngui = []\nfilesystem = []\nnetwork = []\nprocess = []\nmedia = []\nserver = []\n\n# Optional reviewed source-style warnings to suppress.\n[lint]\ndisable = []\n",
            manifest.name, manifest.version, manifest.entry, manifest.locale
        ),
    )
    .map_err(|error| format!("P1032: cannot write manifest: {error}"))?;
    write_package_lock(directory)?;
    fs::write(
        directory.join("src/main.pd"),
        "# padma:locale=bn\nদেখাও \"পদ্ম project শুরু হয়েছে\"\n",
    )
    .map_err(|error| format!("P1032: cannot write starter source: {error}"))?;
    Ok(manifest)
}

fn package_digest(directory: &Path) -> Result<String, String> {
    fn collect(
        root: &Path,
        current: &Path,
        entries: &mut BTreeMap<String, Vec<u8>>,
        total: &mut usize,
    ) -> Result<(), String> {
        for entry in fs::read_dir(current)
            .map_err(|error| format!("P1044: cannot inspect package directory: {error}"))?
        {
            let entry =
                entry.map_err(|error| format!("P1044: cannot read package entry: {error}"))?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)
                .map_err(|error| format!("P1044: cannot inspect package entry: {error}"))?;
            if metadata.file_type().is_symlink() {
                return Err("P1044: symbolic links are not allowed in local packages".to_string());
            }
            if metadata.is_dir() {
                collect(root, &path, entries, total)?;
                continue;
            }
            if !metadata.is_file() {
                return Err("P1044: local packages may contain only regular files".to_string());
            }
            let relative = path
                .strip_prefix(root)
                .map_err(|_| "P1044: package path escaped its source root".to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            if relative == "padma-package.toml" {
                continue;
            }
            let content = fs::read(&path).map_err(|error| {
                format!("P1044: cannot read package file `{relative}`: {error}")
            })?;
            *total = total.saturating_add(content.len());
            if entries.len() >= PACKAGE_MAX_FILES || *total > PACKAGE_MAX_BYTES {
                return Err("P1044: package exceeds local cache file or byte limit".to_string());
            }
            entries.insert(relative, content);
        }
        Ok(())
    }

    let mut entries = BTreeMap::new();
    let mut total = 0usize;
    collect(directory, directory, &mut entries, &mut total)?;
    let mut canonical = Vec::new();
    for (path, content) in entries {
        canonical.extend_from_slice(path.as_bytes());
        canonical.push(0);
        canonical.extend_from_slice(&(content.len() as u64).to_be_bytes());
        canonical.extend_from_slice(&content);
    }
    Ok(format!("sha256:{}", sha256_hex(&canonical)))
}

fn resolve_local_package(
    project_root: &Path,
    dependency_name: &str,
    relative_path: &str,
) -> Result<(PackageManifest, PathBuf, String), String> {
    let root = fs::canonicalize(project_root)
        .map_err(|error| format!("P1044: cannot resolve project root: {error}"))?;
    let relative = safe_package_relative_path(relative_path)?;
    let source = root.join(relative);
    let source_metadata = fs::symlink_metadata(&source)
        .map_err(|error| format!("P1044: package `{dependency_name}` is unavailable: {error}"))?;
    if source_metadata.file_type().is_symlink() || !source_metadata.is_dir() {
        return Err(format!(
            "P1044: package `{dependency_name}` source must be a directory"
        ));
    }
    let source = fs::canonicalize(source)
        .map_err(|error| format!("P1044: cannot resolve package `{dependency_name}`: {error}"))?;
    if !source.starts_with(&root) {
        return Err(format!(
            "P1044: package `{dependency_name}` escaped the project root"
        ));
    }
    let manifest_path = source.join("padma-package.toml");
    let manifest_source = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("P1044: cannot read package manifest: {error}"))?;
    let manifest = parse_package_manifest(&manifest_source)?;
    if manifest.name != dependency_name {
        return Err(format!(
            "P1044: dependency `{dependency_name}` does not match package name `{}`",
            manifest.name
        ));
    }
    let entry = source.join(safe_project_relative_path(&manifest.entry)?);
    let entry_metadata = fs::symlink_metadata(&entry)
        .map_err(|error| format!("P1044: cannot inspect package entry: {error}"))?;
    if entry_metadata.file_type().is_symlink() || !entry_metadata.is_file() {
        return Err("P1044: package entry must be a regular local `.pd` file".to_string());
    }
    let digest = package_digest(&source)?;
    if digest != manifest.digest {
        return Err(format!(
            "P1044: package `{dependency_name}` digest does not match its manifest"
        ));
    }
    Ok((manifest, source, digest))
}

fn package_lock_contents(directory: &Path) -> Result<String, String> {
    let (project, _) = load_project_manifest(directory)?;
    let mut packages = Vec::new();
    for (name, path) in &project.dependencies {
        let (package, _source, digest) = resolve_local_package(directory, name, path)?;
        if !package.capabilities.is_subset(&project.capabilities) {
            return Err(format!(
                "P1034: package `{name}` requests capabilities not granted by this project"
            ));
        }
        packages.push(serde_json::json!({
            "name": package.name,
            "version": package.version,
            "path": path,
            "entry": package.entry,
            "exports": package.exports.into_iter().collect::<Vec<_>>(),
            "capabilities": package.capabilities.into_iter().collect::<Vec<_>>(),
            "digest": digest,
        }));
    }
    serde_json::to_string_pretty(&serde_json::json!({
        "lockfileVersion": 1,
        "project": {"name": project.name, "version": project.version},
        "packages": packages,
    }))
    .map(|value| format!("{value}\n"))
    .map_err(|error| format!("P1044: cannot encode lockfile: {error}"))
}

fn write_package_lock(directory: &Path) -> Result<(), String> {
    let contents = package_lock_contents(directory)?;
    let temporary = directory.join(".padma.lock.tmp");
    fs::write(&temporary, contents)
        .map_err(|error| format!("P1044: cannot write temporary lockfile: {error}"))?;
    fs::rename(&temporary, directory.join("padma.lock"))
        .map_err(|error| format!("P1044: cannot replace lockfile: {error}"))?;
    Ok(())
}

fn verify_package_lock(directory: &Path) -> Result<(), String> {
    let expected = package_lock_contents(directory)?;
    let actual = fs::read_to_string(directory.join("padma.lock"))
        .map_err(|error| format!("P1044: cannot read padma.lock: {error}"))?;
    if actual != expected {
        return Err(
            "P1044: padma.lock does not match the verified local package sources".to_string(),
        );
    }
    Ok(())
}

fn inspect_local_package(directory: &Path, name: &str) -> Result<(), String> {
    let (project, _) = load_project_manifest(directory)?;
    let path = project
        .dependencies
        .get(name)
        .ok_or_else(|| format!("P1044: `{name}` is not a direct local dependency"))?;
    let (package, source, digest) = resolve_local_package(directory, name, path)?;
    println!("Padma local package `{}`", package.name);
    println!("  version: {}", package.version);
    println!("  path: {}", source.display());
    println!("  entry: {}", package.entry);
    println!("  digest: {digest}");
    println!(
        "  exports: {}",
        package.exports.into_iter().collect::<Vec<_>>().join(", ")
    );
    println!(
        "  requested capabilities: {}",
        package
            .capabilities
            .into_iter()
            .collect::<Vec<_>>()
            .join(", ")
    );
    Ok(())
}

fn deployment_source_digest(directory: &Path, entry: &Path) -> Result<String, String> {
    let root = fs::canonicalize(directory)
        .map_err(|error| format!("P1046: cannot resolve project root: {error}"))?;
    let entry_metadata = fs::symlink_metadata(entry)
        .map_err(|error| format!("P1046: cannot inspect deployment entry: {error}"))?;
    if entry_metadata.file_type().is_symlink() || !entry_metadata.is_file() {
        return Err("P1046: deployment entry must be a regular project file".to_string());
    }
    let entry = fs::canonicalize(entry)
        .map_err(|error| format!("P1046: cannot resolve deployment entry: {error}"))?;
    if !entry.starts_with(&root) {
        return Err("P1046: deployment entry escaped the project root".to_string());
    }
    let source_root = root.join("src");
    let source_metadata = fs::symlink_metadata(&source_root)
        .map_err(|error| format!("P1046: cannot inspect project source directory: {error}"))?;
    if source_metadata.file_type().is_symlink() || !source_metadata.is_dir() {
        return Err("P1046: project source directory must be a regular directory".to_string());
    }
    fn collect(
        root: &Path,
        current: &Path,
        entries: &mut BTreeMap<String, Vec<u8>>,
        total: &mut usize,
    ) -> Result<(), String> {
        for item in fs::read_dir(current)
            .map_err(|error| format!("P1046: cannot inspect project source: {error}"))?
        {
            let item =
                item.map_err(|error| format!("P1046: cannot read project source: {error}"))?;
            let path = item.path();
            let metadata = fs::symlink_metadata(&path)
                .map_err(|error| format!("P1046: cannot inspect project source: {error}"))?;
            if metadata.file_type().is_symlink() {
                return Err(
                    "P1046: symbolic links are not allowed in deployment source".to_string()
                );
            }
            if metadata.is_dir() {
                collect(root, &path, entries, total)?;
                continue;
            }
            if !metadata.is_file() {
                return Err("P1046: deployment source may contain only regular files".to_string());
            }
            let relative = path
                .strip_prefix(root)
                .map_err(|_| "P1046: deployment source escaped project root".to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            let contents = fs::read(&path)
                .map_err(|error| format!("P1046: cannot read source `{relative}`: {error}"))?;
            *total = total.saturating_add(contents.len());
            if entries.len() >= DEPLOYMENT_MAX_SOURCE_FILES || *total > DEPLOYMENT_MAX_SOURCE_BYTES
            {
                return Err("P1046: deployment source exceeds file or byte limit".to_string());
            }
            entries.insert(relative, contents);
        }
        Ok(())
    }
    let mut entries = BTreeMap::new();
    let mut total = 0usize;
    collect(&root, &source_root, &mut entries, &mut total)?;
    for fixed_file in ["padma.toml", "padma.lock"] {
        let path = root.join(fixed_file);
        if !path.is_file() {
            continue;
        }
        let contents = fs::read(&path)
            .map_err(|error| format!("P1046: cannot read `{fixed_file}`: {error}"))?;
        total = total.saturating_add(contents.len());
        if entries.len() >= DEPLOYMENT_MAX_SOURCE_FILES || total > DEPLOYMENT_MAX_SOURCE_BYTES {
            return Err("P1046: deployment source exceeds file or byte limit".to_string());
        }
        entries.insert(fixed_file.to_string(), contents);
    }
    let mut canonical = Vec::new();
    for (path, contents) in entries {
        canonical.extend_from_slice(path.as_bytes());
        canonical.push(0);
        canonical.extend_from_slice(&(contents.len() as u64).to_be_bytes());
        canonical.extend_from_slice(&contents);
    }
    Ok(format!("sha256:{}", sha256_hex(&canonical)))
}

fn deployment_plan_contents(directory: &Path) -> Result<String, String> {
    let (project, project_entry) = load_project_manifest(directory)?;
    let manifest_path = directory.join("padma-deploy.toml");
    let source = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("P1046: cannot read `{}`: {error}", manifest_path.display()))?;
    let deployment = parse_deployment_manifest(&source)?;
    if deployment.entry != project.entry {
        return Err("P1046: deployment entry must match `[padma] entry`".to_string());
    }
    let source_digest = deployment_source_digest(directory, &project_entry)?;
    serde_json::to_string_pretty(&serde_json::json!({
        "deploymentPlanVersion": 1,
        "mode": "dry-run-only",
        "project": {"name": project.name, "version": project.version},
        "entry": deployment.entry,
        "target": deployment.target,
        "baseUrl": deployment.base_url,
        "environmentNames": deployment.environment_names.into_iter().collect::<Vec<_>>(),
        "sourceDigest": source_digest,
        "rollback": {"descriptor": deployment.rollback, "remoteAction": "not-configured"},
        "network": "disabled",
        "artifactUpload": "disabled",
        "remoteMutation": "disabled"
    }))
    .map(|value| format!("{value}\n"))
    .map_err(|error| format!("P1046: cannot encode deployment plan: {error}"))
}

fn inspect_deployment_plan(directory: &Path) -> Result<(), String> {
    let plan = deployment_plan_contents(directory)?;
    println!("Padma deployment plan (inspection only)");
    println!("{plan}");
    Ok(())
}

fn render_capability_error(project: &ProjectManifest) -> Result<(), String> {
    if project.capabilities.contains("deployment:render") {
        return Ok(());
    }
    let locale = if project.locale == "bn" {
        Locale::Bangla
    } else {
        Locale::English
    };
    let diagnostic = error_for(locale, "P1034", Position::new(1, 1), "deployment:render");
    Err(format!(
        "{}: {}\n  = {}: {}",
        diagnostic.code,
        diagnostic.message,
        if locale == Locale::Bangla {
            "পরামর্শ"
        } else {
            "help"
        },
        diagnostic.hint.unwrap_or_default()
    ))
}

fn load_render_release_manifest(
    directory: &Path,
) -> Result<
    (
        ProjectManifest,
        DeploymentManifest,
        RenderReleaseManifest,
        String,
    ),
    String,
> {
    let root = fs::canonicalize(directory)
        .map_err(|error| format!("P1048: cannot resolve project root: {error}"))?;
    let (project, project_entry) = load_project_manifest(&root)?;
    render_capability_error(&project)?;
    let deployment_path = root.join("padma-deploy.toml");
    let deployment_metadata = fs::symlink_metadata(&deployment_path)
        .map_err(|error| format!("P1048: cannot inspect deployment manifest: {error}"))?;
    if deployment_metadata.file_type().is_symlink() || !deployment_metadata.is_file() {
        return Err("P1048: deployment manifest must be a regular project file".to_string());
    }
    let deployment = parse_deployment_manifest(
        &fs::read_to_string(&deployment_path)
            .map_err(|error| format!("P1048: cannot read deployment manifest: {error}"))?,
    )?;
    if deployment.entry != project.entry {
        return Err("P1048: deployment entry must match `[padma] entry`".to_string());
    }
    if !matches!(deployment.target.as_str(), "static" | "container") {
        return Err(
            "P1048: Render releases require `static` or `container` deployment target".to_string(),
        );
    }
    let manifest_path = root.join("padma-render.toml");
    let manifest_metadata = fs::symlink_metadata(&manifest_path)
        .map_err(|error| format!("P1048: cannot inspect Render manifest: {error}"))?;
    if manifest_metadata.file_type().is_symlink() || !manifest_metadata.is_file() {
        return Err("P1048: Render manifest must be a regular project file".to_string());
    }
    let render = parse_render_release_manifest(
        &fs::read_to_string(&manifest_path)
            .map_err(|error| format!("P1048: cannot read Render manifest: {error}"))?,
    )?;
    let source_digest = deployment_source_digest(&root, &project_entry)?;
    Ok((project, deployment, render, source_digest))
}

fn render_release_plan_contents(directory: &Path) -> Result<String, String> {
    let (project, deployment, render, source_digest) = load_render_release_manifest(directory)?;
    serde_json::to_string_pretty(&serde_json::json!({
        "renderPlanVersion": 1,
        "mode": "git-linked-release-plan",
        "project": {"name": project.name, "version": project.version},
        "provider": "render",
        "service": render.service,
        "git": {"repository": render.repository, "branch": render.branch, "commit": render.commit},
        "deployment": {"target": deployment.target, "baseUrl": deployment.base_url, "environmentNames": deployment.environment_names.into_iter().collect::<Vec<_>>()},
        "artifact": {"sourceDigest": source_digest, "localBuild": "disabled", "artifactUpload": "disabled", "providerBuild": "requires-render-dashboard-confirmation"},
        "rollback": {"descriptor": deployment.rollback, "targetDeploy": render.rollback_deploy, "execution": "disabled"},
        "confirmation": {"required": true, "method": "render-dashboard", "status": "not-confirmed"},
        "network": "disabled",
        "providerApi": "disabled",
        "remoteMutation": "disabled"
    }))
    .map(|value| format!("{value}\n"))
    .map_err(|error| format!("P1048: cannot encode Render release plan: {error}"))
}

fn inspect_render_release_plan(directory: &Path) -> Result<(), String> {
    let plan = render_release_plan_contents(directory)?;
    println!("Padma Render release plan (inspection only)");
    println!("{plan}");
    Ok(())
}

fn load_render_api_manifest(
    directory: &Path,
) -> Result<
    (
        ProjectManifest,
        DeploymentManifest,
        RenderReleaseManifest,
        RenderApiManifest,
        String,
    ),
    String,
> {
    let root = fs::canonicalize(directory)
        .map_err(|error| format!("P1048: cannot resolve project root: {error}"))?;
    let (project, deployment, release, source_digest) = load_render_release_manifest(&root)?;
    let manifest_path = root.join("padma-render-api.toml");
    let manifest_metadata = fs::symlink_metadata(&manifest_path)
        .map_err(|error| format!("P1048: cannot inspect Render API manifest: {error}"))?;
    if manifest_metadata.file_type().is_symlink() || !manifest_metadata.is_file() {
        return Err("P1048: Render API manifest must be a regular project file".to_string());
    }
    let api = parse_render_api_manifest(
        &fs::read_to_string(&manifest_path)
            .map_err(|error| format!("P1048: cannot read Render API manifest: {error}"))?,
    )?;
    if api.service != release.service || api.commit != release.commit {
        return Err(
            "P1048: Render API service and commit must match the reviewed git-linked manifest"
                .to_string(),
        );
    }
    if release.rollback_deploy.as_deref() != Some(api.rollback_deploy.as_str()) {
        return Err(
            "P1048: Render API rollback_deploy must match the reviewed git-linked manifest"
                .to_string(),
        );
    }
    Ok((project, deployment, release, api, source_digest))
}

fn render_confirmation_token(
    action: &str,
    service: &str,
    commit: &str,
    rollback_deploy: &str,
    source_digest: &str,
) -> String {
    let material = format!(
        "padma-render-confirmation-v1\0{action}\0{service}\0{commit}\0{rollback_deploy}\0{source_digest}"
    );
    format!("render-{}", &sha256_hex(material.as_bytes())[..24])
}

fn render_api_plan_contents(directory: &Path) -> Result<String, String> {
    let (project, deployment, release, api, source_digest) = load_render_api_manifest(directory)?;
    let deploy_confirmation = render_confirmation_token(
        "deploy",
        &api.service,
        &api.commit,
        &api.rollback_deploy,
        &source_digest,
    );
    let rollback_confirmation = render_confirmation_token(
        "rollback",
        &api.service,
        &api.commit,
        &api.rollback_deploy,
        &source_digest,
    );
    serde_json::to_string_pretty(&serde_json::json!({
        "renderApiPlanVersion": 1,
        "project": {"name": project.name, "version": project.version},
        "provider": "render",
        "service": api.service,
        "git": {"repository": release.repository, "branch": release.branch, "commit": api.commit},
        "deployment": {"target": deployment.target, "baseUrl": deployment.base_url, "sourceDigest": source_digest},
        "credential": {"environmentName": api.token_env, "value": "not-read-in-planning-mode", "manifestStorage": "prohibited"},
        "deploy": {"request": "POST /v1/services/{serviceId}/deploys", "body": {"commitId": api.commit, "clearCache": api.clear_cache}, "confirmationToken": deploy_confirmation},
        "rollback": {"request": "POST /v1/services/{serviceId}/rollback", "body": {"deployId": api.rollback_deploy}, "confirmationToken": rollback_confirmation},
        "artifactUpload": "disabled",
        "localBuild": "disabled",
        "network": "disabled-in-planning-mode",
        "remoteMutation": "disabled-in-planning-mode"
    }))
    .map(|value| format!("{value}\n"))
    .map_err(|error| format!("P1048: cannot encode Render API plan: {error}"))
}

fn render_api_curl_config(url: &str, token: &str, body: &str) -> Result<String, String> {
    if token.len() < 20
        || token.len() > 512
        || token
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err("P1048: Render API token is missing or has an unsafe format".to_string());
    }
    let escape = |value: &str| value.replace('\\', "\\\\").replace('"', "\\\"");
    Ok(format!(
        "url = \"{}\"\nrequest = \"POST\"\nheader = \"accept: application/json\"\nheader = \"content-type: application/json\"\nheader = \"authorization: Bearer {}\"\ndata = \"{}\"\n",
        escape(url),
        escape(token),
        escape(body),
    ))
}

fn run_render_api_request(
    action: &str,
    directory: &Path,
    confirmation: &str,
) -> Result<(), String> {
    let (_, _, _, api, source_digest) = load_render_api_manifest(directory)?;
    let expected_confirmation = render_confirmation_token(
        action,
        &api.service,
        &api.commit,
        &api.rollback_deploy,
        &source_digest,
    );
    if confirmation != expected_confirmation {
        return Err(format!(
            "P1048: confirmation token does not match the current reviewed `{action}` plan; run `padma render api-plan` again"
        ));
    }
    let token = env::var(&api.token_env).map_err(|_| {
        format!(
            "P1048: required Render API credential `{}` is not set",
            api.token_env
        )
    })?;
    let (path, body) = match action {
        "deploy" => (
            format!("/v1/services/{}/deploys", api.service),
            serde_json::json!({"commitId": api.commit, "clearCache": api.clear_cache}).to_string(),
        ),
        "rollback" => (
            format!("/v1/services/{}/rollback", api.service),
            serde_json::json!({"deployId": api.rollback_deploy}).to_string(),
        ),
        _ => return Err("P1048: unsupported Render API action".to_string()),
    };
    let url = format!("https://api.render.com{path}");
    let config = render_api_curl_config(&url, &token, &body)?;
    let mut child = process::Command::new("curl")
        .args([
            "--fail-with-body",
            "--silent",
            "--show-error",
            "--max-time",
            "60",
            "--config",
            "-",
        ])
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .spawn()
        .map_err(|error| format!("P1048: cannot start curl for Render API request: {error}"))?;
    child
        .stdin
        .as_mut()
        .ok_or_else(|| "P1048: cannot open secure curl configuration input".to_string())?
        .write_all(config.as_bytes())
        .map_err(|error| {
            format!("P1048: cannot provide Render API request configuration: {error}")
        })?;
    let output = child
        .wait_with_output()
        .map_err(|error| format!("P1048: Render API request did not finish: {error}"))?;
    let response = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
    .replace(&token, "[REDACTED]");
    if !output.status.success() {
        return Err(format!(
            "P1048: Render API {action} request failed (curl status {}): {}",
            output.status,
            response.trim().chars().take(8_192).collect::<String>()
        ));
    }
    println!(
        "Render {action} request accepted. Provider response:\n{}",
        response.trim().chars().take(8_192).collect::<String>()
    );
    Ok(())
}

fn gui_capability_error(project: &ProjectManifest) -> Result<(), String> {
    if project.capabilities.contains("gui:local") {
        return Ok(());
    }
    let locale = if project.locale == "bn" {
        Locale::Bangla
    } else {
        Locale::English
    };
    let diagnostic = error_for(locale, "P1034", Position::new(1, 1), "gui:local");
    Err(format!(
        "{}: {}\n  = {}: {}",
        diagnostic.code,
        diagnostic.message,
        if locale == Locale::Bangla {
            "পরামর্শ"
        } else {
            "help"
        },
        diagnostic.hint.unwrap_or_default()
    ))
}

fn load_gui_manifest(
    directory: &Path,
) -> Result<(ProjectManifest, GuiManifest, PathBuf, PathBuf), String> {
    let root = fs::canonicalize(directory)
        .map_err(|error| format!("P1047: cannot resolve project root: {error}"))?;
    let (project, _) = load_project_manifest(&root)?;
    gui_capability_error(&project)?;
    let manifest_path = root.join("padma-gui.toml");
    let manifest_metadata = fs::symlink_metadata(&manifest_path)
        .map_err(|error| format!("P1047: cannot inspect GUI manifest: {error}"))?;
    if manifest_metadata.file_type().is_symlink() || !manifest_metadata.is_file() {
        return Err("P1047: GUI manifest must be a regular project file".to_string());
    }
    let source = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("P1047: cannot read GUI manifest: {error}"))?;
    let manifest = parse_gui_manifest(&source)?;

    let entry = root.join(safe_gui_relative_path(&manifest.entry, Some(".html"))?);
    let entry_metadata = fs::symlink_metadata(&entry)
        .map_err(|error| format!("P1047: cannot inspect GUI entry: {error}"))?;
    if entry_metadata.file_type().is_symlink() || !entry_metadata.is_file() {
        return Err("P1047: GUI entry must be a regular project `.html` file".to_string());
    }
    let entry = fs::canonicalize(&entry)
        .map_err(|error| format!("P1047: cannot resolve GUI entry: {error}"))?;
    if !entry.starts_with(&root) {
        return Err("P1047: GUI entry escaped the project root".to_string());
    }

    let assets = root.join(safe_gui_relative_path(&manifest.assets, None)?);
    let assets_metadata = fs::symlink_metadata(&assets)
        .map_err(|error| format!("P1047: cannot inspect GUI assets: {error}"))?;
    if assets_metadata.file_type().is_symlink() || !assets_metadata.is_dir() {
        return Err("P1047: GUI assets must be a regular project directory".to_string());
    }
    let assets = fs::canonicalize(&assets)
        .map_err(|error| format!("P1047: cannot resolve GUI assets: {error}"))?;
    if !assets.starts_with(&root) {
        return Err("P1047: GUI assets escaped the project root".to_string());
    }
    Ok((project, manifest, entry, assets))
}

fn gui_source_digest(directory: &Path, entry: &Path, assets: &Path) -> Result<String, String> {
    fn insert_file(
        root: &Path,
        path: &Path,
        entries: &mut BTreeMap<String, Vec<u8>>,
        total: &mut usize,
    ) -> Result<(), String> {
        let relative = path
            .strip_prefix(root)
            .map_err(|_| "P1047: GUI source escaped the project root".to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        if entries.contains_key(&relative) {
            return Ok(());
        }
        let contents = fs::read(path)
            .map_err(|error| format!("P1047: cannot read GUI source `{relative}`: {error}"))?;
        *total = total.saturating_add(contents.len());
        if entries.len() >= GUI_MAX_SOURCE_FILES || *total > GUI_MAX_SOURCE_BYTES {
            return Err("P1047: GUI source exceeds file or byte limit".to_string());
        }
        entries.insert(relative, contents);
        Ok(())
    }

    fn collect(
        root: &Path,
        current: &Path,
        entries: &mut BTreeMap<String, Vec<u8>>,
        total: &mut usize,
    ) -> Result<(), String> {
        for item in fs::read_dir(current)
            .map_err(|error| format!("P1047: cannot inspect GUI assets: {error}"))?
        {
            let item = item.map_err(|error| format!("P1047: cannot read GUI asset: {error}"))?;
            let path = item.path();
            let metadata = fs::symlink_metadata(&path)
                .map_err(|error| format!("P1047: cannot inspect GUI asset: {error}"))?;
            if metadata.file_type().is_symlink() {
                return Err("P1047: symbolic links are not allowed in GUI assets".to_string());
            }
            if metadata.is_dir() {
                collect(root, &path, entries, total)?;
            } else if metadata.is_file() {
                insert_file(root, &path, entries, total)?;
            } else {
                return Err("P1047: GUI assets may contain only regular files".to_string());
            }
        }
        Ok(())
    }

    let root = fs::canonicalize(directory)
        .map_err(|error| format!("P1047: cannot resolve project root: {error}"))?;
    let mut entries = BTreeMap::new();
    let mut total = 0usize;
    collect(&root, assets, &mut entries, &mut total)?;
    insert_file(&root, entry, &mut entries, &mut total)?;
    let mut canonical = Vec::new();
    for (path, contents) in entries {
        canonical.extend_from_slice(path.as_bytes());
        canonical.push(0);
        canonical.extend_from_slice(&(contents.len() as u64).to_be_bytes());
        canonical.extend_from_slice(&contents);
    }
    Ok(format!("sha256:{}", sha256_hex(&canonical)))
}

fn gui_inspect_contents(directory: &Path) -> Result<String, String> {
    let (_project, manifest, _entry, _assets) = load_gui_manifest(directory)?;
    Ok(format!(
        "Padma GUI renderer manifest (read-only)\n  version: {}\n  backend: {}\n  entry: {}\n  assets: {}\n  title: {}\n",
        manifest.version, manifest.backend, manifest.entry, manifest.assets, manifest.title
    ))
}

fn gui_plan_contents(directory: &Path) -> Result<String, String> {
    let (project, manifest, entry, assets) = load_gui_manifest(directory)?;
    let source_digest = gui_source_digest(directory, &entry, &assets)?;
    serde_json::to_string_pretty(&serde_json::json!({
        "guiPlanVersion": 1,
        "mode": "read-only",
        "project": {"name": project.name, "version": project.version},
        "renderer": {
            "backend": manifest.backend,
            "entry": manifest.entry,
            "assets": manifest.assets,
            "title": manifest.title
        },
        "sourceDigest": source_digest,
        "network": "disabled",
        "rendererLaunch": "disabled",
        "javascriptExecution": "not-requested",
        "nativeBridge": "disabled",
        "androidPermissions": "not-requested"
    }))
    .map(|value| format!("{value}\n"))
    .map_err(|error| format!("P1047: cannot encode GUI plan: {error}"))
}

fn ai_workflow_locale(project: &ProjectManifest) -> Locale {
    if project.locale == "bn" {
        Locale::Bangla
    } else {
        Locale::English
    }
}

fn ai_workflow_capability_error(project: &ProjectManifest) -> Result<(), String> {
    if project.capabilities.contains("network:ai") {
        return Ok(());
    }
    let locale = ai_workflow_locale(project);
    let diagnostic = error_for(locale, "P1034", Position::new(1, 1), "network:ai");
    Err(format!(
        "{}: {}\n  = {}: {}",
        diagnostic.code,
        diagnostic.message,
        if locale == Locale::Bangla {
            "পরামর্শ"
        } else {
            "help"
        },
        diagnostic.hint.unwrap_or_default()
    ))
}

fn load_ai_workflow_manifest(
    directory: &Path,
) -> Result<(ProjectManifest, AiWorkflowManifest), String> {
    let root = fs::canonicalize(directory)
        .map_err(|error| format!("P1050: cannot resolve project root: {error}"))?;
    let (project, _) = load_project_manifest(&root)?;
    ai_workflow_capability_error(&project)?;
    let locale = ai_workflow_locale(&project);
    let manifest_path = root.join("padma-ai.toml");
    let metadata = fs::symlink_metadata(&manifest_path).map_err(|error| {
        ai_workflow_error(
            locale,
            "P1050",
            &format!("cannot inspect `padma-ai.toml`: {error}"),
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(ai_workflow_error(
            locale,
            "P1050",
            "padma-ai.toml must be a regular project file",
        ));
    }
    let source = fs::read_to_string(&manifest_path).map_err(|error| {
        ai_workflow_error(
            locale,
            "P1050",
            &format!("cannot read `padma-ai.toml`: {error}"),
        )
    })?;
    let manifest = parse_ai_workflow_manifest(&source, locale)?;
    Ok((project, manifest))
}

fn ai_workflow_plan_contents(directory: &Path) -> Result<String, String> {
    let (project, manifest) = load_ai_workflow_manifest(directory)?;
    serde_json::to_string_pretty(&serde_json::json!({
        "aiWorkflowPlanVersion": 1,
        "mode": "inspection-only",
        "project": {"name": project.name, "version": project.version},
        "adapter": "json-http-v1",
        "endpoint": manifest.endpoint,
        "secret": {"environmentName": manifest.secret_env, "value": "not-read"},
        "limits": {
            "timeoutSeconds": manifest.timeout_seconds,
            "maxInputBytes": manifest.max_input_bytes,
            "maxResponseBytes": manifest.max_response_bytes,
            "retryPolicy": manifest.retry_policy
        },
        "model": manifest.model,
        "network": "disabled",
        "environmentRead": "disabled",
        "dnsResolution": "disabled",
        "childProcess": "disabled",
        "modelExecution": "disabled",
        "generatedOutputExecution": "disabled"
    }))
    .map(|value| format!("{value}\n"))
    .map_err(|error| format!("P1050: cannot encode AI workflow plan: {error}"))
}

fn ai_workflow_inspect_contents(directory: &Path) -> Result<String, String> {
    let (project, manifest) = load_ai_workflow_manifest(directory)?;
    let heading = match ai_workflow_locale(&project) {
        Locale::Bangla => "Padma AI workflow manifest (শুধু inspection)\n",
        Locale::English => "Padma AI workflow manifest (inspection-only)\n",
    };
    let plan = serde_json::to_string_pretty(&serde_json::json!({
        "aiWorkflowPlanVersion": 1,
        "mode": "inspection-only",
        "project": {"name": project.name, "version": project.version},
        "adapter": "json-http-v1",
        "endpoint": manifest.endpoint,
        "secret": {"environmentName": manifest.secret_env, "value": "not-read"},
        "limits": {
            "timeoutSeconds": manifest.timeout_seconds,
            "maxInputBytes": manifest.max_input_bytes,
            "maxResponseBytes": manifest.max_response_bytes,
            "retryPolicy": manifest.retry_policy
        },
        "model": manifest.model,
        "network": "disabled",
        "environmentRead": "disabled",
        "dnsResolution": "disabled",
        "childProcess": "disabled",
        "modelExecution": "disabled",
        "generatedOutputExecution": "disabled"
    }))
    .map_err(|error| format!("P1050: cannot encode AI workflow inspection: {error}"))?;
    Ok(format!("{heading}{plan}\n"))
}

fn run_ai_workflow_inspect(directory: &Path) -> Result<(), String> {
    print!("{}", ai_workflow_inspect_contents(directory)?);
    Ok(())
}

fn run_ai_workflow_plan(directory: &Path) -> Result<(), String> {
    print!("{}", ai_workflow_plan_contents(directory)?);
    Ok(())
}

fn ai_tool_plan_locale(project: &ProjectManifest) -> Locale {
    ai_workflow_locale(project)
}

fn ai_tool_plan_capability_error(project: &ProjectManifest) -> Result<(), String> {
    if project.capabilities.contains("ai:tools") {
        return Ok(());
    }
    let locale = ai_tool_plan_locale(project);
    let diagnostic = error_for(locale, "P1034", Position::new(1, 1), "ai:tools");
    Err(format!(
        "{}: {}\n  = {}: {}",
        diagnostic.code,
        diagnostic.message,
        if locale == Locale::Bangla {
            "পরামর্শ"
        } else {
            "help"
        },
        diagnostic.hint.unwrap_or_default()
    ))
}

fn load_ai_tool_plan_manifest(
    directory: &Path,
) -> Result<(ProjectManifest, AiToolPlanManifest), String> {
    let root = fs::canonicalize(directory)
        .map_err(|error| format!("P1056: cannot resolve project root: {error}"))?;
    let (project, _) = load_project_manifest(&root)?;
    ai_tool_plan_capability_error(&project)?;
    let locale = ai_tool_plan_locale(&project);
    let manifest_path = root.join("padma-ai-tools.toml");
    let metadata = fs::symlink_metadata(&manifest_path).map_err(|error| {
        ai_tool_plan_error(
            locale,
            "P1056",
            &format!("cannot inspect `padma-ai-tools.toml`: {error}"),
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(ai_tool_plan_error(
            locale,
            "P1056",
            "padma-ai-tools.toml must be a regular project file",
        ));
    }
    let source = fs::read_to_string(&manifest_path).map_err(|error| {
        ai_tool_plan_error(
            locale,
            "P1056",
            &format!("cannot read `padma-ai-tools.toml`: {error}"),
        )
    })?;
    let manifest = parse_ai_tool_plan_manifest(&source, locale)?;
    for tool in &manifest.tools {
        let required = ai_tool_required_capability(tool);
        if !project.capabilities.contains(required) {
            let diagnostic = error_for(locale, "P1034", Position::new(1, 1), required);
            return Err(format!(
                "{}: {}\n  = {}: {}",
                diagnostic.code,
                diagnostic.message,
                if locale == Locale::Bangla {
                    "পরামর্শ"
                } else {
                    "help"
                },
                diagnostic.hint.unwrap_or_default()
            ));
        }
    }
    Ok((project, manifest))
}

fn ai_tool_plan_json(directory: &Path) -> Result<String, String> {
    let (project, manifest) = load_ai_tool_plan_manifest(directory)?;
    let tools = manifest
        .tools
        .iter()
        .map(|tool| {
            serde_json::json!({
                "name": tool,
                "requiredCapability": ai_tool_required_capability(tool),
                "execution": "disabled"
            })
        })
        .collect::<Vec<_>>();
    serde_json::to_string_pretty(&serde_json::json!({
        "aiToolPlanVersion": 1,
        "mode": "inspection-only",
        "project": {"name": project.name, "version": project.version},
        "agent": {
            "mode": "plan-only",
            "maxSteps": manifest.max_steps,
            "maxWallSeconds": manifest.max_wall_seconds,
            "retryPolicy": "never"
        },
        "tools": tools,
        "network": "disabled",
        "environmentRead": "disabled",
        "dnsResolution": "disabled",
        "childProcess": "disabled",
        "toolExecution": "disabled",
        "agentLoop": "disabled",
        "backgroundExecution": "disabled",
        "generatedOutputExecution": "disabled",
        "auditLog": "not-written"
    }))
    .map(|value| format!("{value}\n"))
    .map_err(|error| format!("P1056: cannot encode AI tool plan: {error}"))
}

fn ai_tool_inspect_contents(directory: &Path) -> Result<String, String> {
    let root = fs::canonicalize(directory)
        .map_err(|error| format!("P1056: cannot resolve project root: {error}"))?;
    let (project, _) = load_project_manifest(&root)?;
    let heading = match ai_tool_plan_locale(&project) {
        Locale::Bangla => "Padma AI tool manifest (শুধু inspection)\n",
        Locale::English => "Padma AI tool manifest (inspection-only)\n",
    };
    Ok(format!("{heading}{}", ai_tool_plan_json(&root)?))
}

fn run_ai_tool_inspect(directory: &Path) -> Result<(), String> {
    print!("{}", ai_tool_inspect_contents(directory)?);
    Ok(())
}

fn run_ai_tool_plan(directory: &Path) -> Result<(), String> {
    print!("{}", ai_tool_plan_json(directory)?);
    Ok(())
}

fn ai_training_plan_locale(project: &ProjectManifest) -> Locale {
    ai_workflow_locale(project)
}

fn ai_training_plan_capability_error(project: &ProjectManifest) -> Result<(), String> {
    if project.capabilities.contains("ai:training-plan") {
        return Ok(());
    }
    let locale = ai_training_plan_locale(project);
    let diagnostic = error_for(locale, "P1034", Position::new(1, 1), "ai:training-plan");
    Err(format!(
        "{}: {}\n  = {}: {}",
        diagnostic.code,
        diagnostic.message,
        if locale == Locale::Bangla {
            "পরামর্শ"
        } else {
            "help"
        },
        diagnostic.hint.unwrap_or_default()
    ))
}

fn load_ai_training_plan_manifest(
    directory: &Path,
) -> Result<(ProjectManifest, AiTrainingPlanManifest), String> {
    let root = fs::canonicalize(directory)
        .map_err(|error| format!("P1058: cannot resolve project root: {error}"))?;
    let (project, _) = load_project_manifest(&root)?;
    ai_training_plan_capability_error(&project)?;
    let locale = ai_training_plan_locale(&project);
    let manifest_path = root.join("padma-ai-training.toml");
    let metadata = fs::symlink_metadata(&manifest_path).map_err(|error| {
        ai_training_plan_error(
            locale,
            "P1058",
            &format!("cannot inspect `padma-ai-training.toml`: {error}"),
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(ai_training_plan_error(
            locale,
            "P1058",
            "padma-ai-training.toml must be a regular project file",
        ));
    }
    let source = fs::read_to_string(&manifest_path).map_err(|error| {
        ai_training_plan_error(
            locale,
            "P1058",
            &format!("cannot read `padma-ai-training.toml`: {error}"),
        )
    })?;
    Ok((project, parse_ai_training_plan_manifest(&source, locale)?))
}

fn ai_training_plan_json(directory: &Path) -> Result<String, String> {
    let (project, manifest) = load_ai_training_plan_manifest(directory)?;
    serde_json::to_string_pretty(&serde_json::json!({
        "aiTrainingPlanVersion": 1,
        "mode": "inspection-only",
        "project": {"name": project.name, "version": project.version},
        "backend": "local-adapter-v1",
        "dataset": {"path": manifest.dataset_path, "read": "disabled"},
        "artifact": {"path": manifest.artifact_path, "write": "disabled"},
        "limits": {
            "maxEpochs": manifest.max_epochs,
            "maxWallSeconds": manifest.max_wall_seconds,
            "maxDatasetBytes": manifest.max_dataset_bytes,
            "maxMemoryMb": manifest.max_memory_mb,
            "maxCpuThreads": manifest.max_cpu_threads
        },
        "training": "not-started",
        "localBackend": "not-started",
        "remoteCompute": "disabled",
        "datasetRead": "disabled",
        "artifactWrite": "disabled",
        "environmentRead": "disabled",
        "childProcess": "disabled",
        "network": "disabled",
        "generatedOutputExecution": "disabled"
    }))
    .map(|value| format!("{value}\n"))
    .map_err(|error| format!("P1058: cannot encode AI training plan: {error}"))
}

fn ai_training_inspect_contents(directory: &Path) -> Result<String, String> {
    let root = fs::canonicalize(directory)
        .map_err(|error| format!("P1058: cannot resolve project root: {error}"))?;
    let (project, _) = load_project_manifest(&root)?;
    let heading = match ai_training_plan_locale(&project) {
        Locale::Bangla => "Padma AI training manifest (শুধু inspection)\n",
        Locale::English => "Padma AI training manifest (inspection-only)\n",
    };
    Ok(format!("{heading}{}", ai_training_plan_json(&root)?))
}

fn run_ai_training_inspect(directory: &Path) -> Result<(), String> {
    print!("{}", ai_training_inspect_contents(directory)?);
    Ok(())
}

fn run_ai_training_plan(directory: &Path) -> Result<(), String> {
    print!("{}", ai_training_plan_json(directory)?);
    Ok(())
}

fn browser_plan_locale(project: &ProjectManifest) -> Locale {
    if project.locale == "bn" {
        Locale::Bangla
    } else {
        Locale::English
    }
}

fn browser_plan_capability_error(project: &ProjectManifest) -> Result<(), String> {
    if project.capabilities.contains("browser:plan") {
        return Ok(());
    }
    let locale = browser_plan_locale(project);
    let diagnostic = error_for(locale, "P1034", Position::new(1, 1), "browser:plan");
    Err(format!(
        "{}: {}\n  = {}: {}",
        diagnostic.code,
        diagnostic.message,
        if locale == Locale::Bangla {
            "পরামর্শ"
        } else {
            "help"
        },
        diagnostic.hint.unwrap_or_default()
    ))
}

fn load_browser_plan_manifest(
    directory: &Path,
) -> Result<(ProjectManifest, BrowserPlanManifest), String> {
    let root = fs::canonicalize(directory)
        .map_err(|error| format!("P1053: cannot resolve project root: {error}"))?;
    let (project, _) = load_project_manifest(&root)?;
    browser_plan_capability_error(&project)?;
    let locale = browser_plan_locale(&project);
    let manifest_path = root.join("padma-browser.toml");
    let metadata = fs::symlink_metadata(&manifest_path).map_err(|error| {
        browser_plan_error(
            locale,
            "P1053",
            &format!("cannot inspect `padma-browser.toml`: {error}"),
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(browser_plan_error(
            locale,
            "P1053",
            "padma-browser.toml must be a regular project file",
        ));
    }
    let source = fs::read_to_string(&manifest_path).map_err(|error| {
        browser_plan_error(
            locale,
            "P1053",
            &format!("cannot read `padma-browser.toml`: {error}"),
        )
    })?;
    let manifest = parse_browser_plan_manifest(&source, locale)?;
    Ok((project, manifest))
}

fn browser_plan_json(directory: &Path) -> Result<String, String> {
    let (project, manifest) = load_browser_plan_manifest(directory)?;
    let navigation = manifest
        .navigation_urls
        .iter()
        .map(|url| serde_json::json!({"method": "GET", "url": url}))
        .collect::<Vec<_>>();
    serde_json::to_string_pretty(&serde_json::json!({
        "browserPlanVersion": 1,
        "mode": "inspection-only",
        "project": {"name": project.name, "version": project.version},
        "planDigest": browser_plan_digest(&manifest),
        "intent": manifest.intent,
        "allowlistedOrigins": manifest.origins,
        "navigation": navigation,
        "limits": {"maxSteps": manifest.max_steps, "redirectPolicy": manifest.redirect_policy},
        "browser": "not-started",
        "network": "disabled",
        "dns": "disabled",
        "cookies": "not-read",
        "credentials": "not-read",
        "environmentRead": "disabled",
        "childProcess": "disabled",
        "browserProfile": "not-read",
        "redirectFollowing": "disabled",
        "unsafeActionExecution": "disabled"
    }))
    .map(|value| format!("{value}\n"))
    .map_err(|error| format!("P1053: cannot encode browser plan: {error}"))
}

fn browser_inspect_contents(directory: &Path) -> Result<String, String> {
    let root = fs::canonicalize(directory)
        .map_err(|error| format!("P1053: cannot resolve project root: {error}"))?;
    let (project, _) = load_project_manifest(&root)?;
    let heading = match browser_plan_locale(&project) {
        Locale::Bangla => "Padma browser plan manifest (শুধু inspection)\n",
        Locale::English => "Padma browser plan manifest (inspection-only)\n",
    };
    Ok(format!("{heading}{}", browser_plan_json(&root)?))
}

fn run_browser_inspect(directory: &Path) -> Result<(), String> {
    print!("{}", browser_inspect_contents(directory)?);
    Ok(())
}

fn run_browser_plan(directory: &Path) -> Result<(), String> {
    print!("{}", browser_plan_json(directory)?);
    Ok(())
}

fn browser_confirmation_capability_error(project: &ProjectManifest) -> Result<(), String> {
    if project.capabilities.contains("browser:confirm-plan") {
        return Ok(());
    }
    let locale = browser_plan_locale(project);
    let diagnostic = error_for(locale, "P1034", Position::new(1, 1), "browser:confirm-plan");
    Err(format!(
        "{}: {}\n  = {}: {}",
        diagnostic.code,
        diagnostic.message,
        if locale == Locale::Bangla {
            "পরামর্শ"
        } else {
            "help"
        },
        diagnostic.hint.unwrap_or_default()
    ))
}

fn browser_handoff_capability_error(project: &ProjectManifest) -> Result<(), String> {
    if project.capabilities.contains("browser:handoff") {
        return Ok(());
    }
    let locale = browser_plan_locale(project);
    let diagnostic = error_for(locale, "P1034", Position::new(1, 1), "browser:handoff");
    Err(format!(
        "{}: {}\n  = {}: {}",
        diagnostic.code,
        diagnostic.message,
        if locale == Locale::Bangla {
            "পরামর্শ"
        } else {
            "help"
        },
        diagnostic.hint.unwrap_or_default()
    ))
}

fn browser_draft_capability_error(project: &ProjectManifest) -> Result<(), String> {
    if project.capabilities.contains("browser:draft") {
        return Ok(());
    }
    let locale = browser_plan_locale(project);
    let diagnostic = error_for(locale, "P1034", Position::new(1, 1), "browser:draft");
    Err(format!(
        "{}: {}\n  = {}: {}",
        diagnostic.code,
        diagnostic.message,
        if locale == Locale::Bangla {
            "পরামর্শ"
        } else {
            "help"
        },
        diagnostic.hint.unwrap_or_default()
    ))
}

fn browser_takeover_capability_error(project: &ProjectManifest) -> Result<(), String> {
    if project.capabilities.contains("browser:takeover") {
        return Ok(());
    }
    let locale = browser_plan_locale(project);
    let diagnostic = error_for(locale, "P1034", Position::new(1, 1), "browser:takeover");
    Err(format!(
        "{}: {}\n  = {}: {}",
        diagnostic.code,
        diagnostic.message,
        if locale == Locale::Bangla {
            "পরামর্শ"
        } else {
            "help"
        },
        diagnostic.hint.unwrap_or_default()
    ))
}

fn load_browser_handoff_audit_manifest(
    root: &Path,
    project: &ProjectManifest,
) -> Result<Option<BrowserHandoffAuditManifest>, String> {
    if !project.capabilities.contains("browser:audit") {
        return Ok(None);
    }
    let locale = browser_plan_locale(project);
    let manifest_path = root.join("padma-browser-audit.toml");
    let metadata = fs::symlink_metadata(&manifest_path).map_err(|error| {
        browser_handoff_error(
            locale,
            "P1064",
            &format!("cannot inspect browser audit manifest: {error}"),
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(browser_handoff_error(
            locale,
            "P1064",
            "browser audit manifest must be a regular project file",
        ));
    }
    let source = fs::read_to_string(&manifest_path).map_err(|error| {
        browser_handoff_error(
            locale,
            "P1064",
            &format!("cannot read browser audit manifest: {error}"),
        )
    })?;
    parse_browser_handoff_audit_manifest(&source, locale).map(Some)
}

fn load_browser_confirmation_plan_manifest(
    directory: &Path,
) -> Result<
    (
        ProjectManifest,
        BrowserPlanManifest,
        BrowserConfirmationPlanManifest,
    ),
    String,
> {
    let root = fs::canonicalize(directory)
        .map_err(|error| format!("P1060: cannot resolve project root: {error}"))?;
    let (project, browser_plan) = load_browser_plan_manifest(&root)?;
    browser_confirmation_capability_error(&project)?;
    let locale = browser_plan_locale(&project);
    let manifest_path = root.join("padma-browser-confirm.toml");
    let metadata = fs::symlink_metadata(&manifest_path).map_err(|error| {
        browser_confirmation_error(
            locale,
            "P1060",
            &format!("cannot inspect `padma-browser-confirm.toml`: {error}"),
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(browser_confirmation_error(
            locale,
            "P1060",
            "padma-browser-confirm.toml must be a regular project file",
        ));
    }
    let source = fs::read_to_string(&manifest_path).map_err(|error| {
        browser_confirmation_error(
            locale,
            "P1060",
            &format!("cannot read `padma-browser-confirm.toml`: {error}"),
        )
    })?;
    let confirmation = parse_browser_confirmation_plan_manifest(&source, locale)?;
    if confirmation.browser_plan_digest != browser_plan_digest(&browser_plan) {
        return Err(browser_confirmation_error(
            locale,
            "P1060",
            "browser_plan_digest does not match the reviewed local browser plan",
        ));
    }
    if confirmation.navigation_index > browser_plan.navigation_urls.len() {
        return Err(browser_confirmation_error(
            locale,
            "P1060",
            "navigation_index does not identify a reviewed browser-plan URL",
        ));
    }
    Ok((project, browser_plan, confirmation))
}

fn load_browser_draft_manifest(
    directory: &Path,
) -> Result<(ProjectManifest, BrowserPlanManifest, BrowserDraftManifest), String> {
    let root = fs::canonicalize(directory)
        .map_err(|error| format!("P1065: cannot resolve project root: {error}"))?;
    let (project, browser_plan) = load_browser_plan_manifest(&root)?;
    browser_draft_capability_error(&project)?;
    let locale = browser_plan_locale(&project);
    let manifest_path = root.join("padma-browser-draft.toml");
    let metadata = fs::symlink_metadata(&manifest_path).map_err(|error| {
        browser_confirmation_error(
            locale,
            "P1065",
            &format!("cannot inspect `padma-browser-draft.toml`: {error}"),
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(browser_confirmation_error(
            locale,
            "P1065",
            "padma-browser-draft.toml must be a regular project file",
        ));
    }
    let source = fs::read_to_string(&manifest_path).map_err(|error| {
        browser_confirmation_error(
            locale,
            "P1065",
            &format!("cannot read `padma-browser-draft.toml`: {error}"),
        )
    })?;
    let draft = parse_browser_draft_manifest(&source, locale)?;
    if draft.browser_plan_digest != browser_plan_digest(&browser_plan) {
        return Err(browser_confirmation_error(
            locale,
            "P1065",
            "browser_plan_digest does not match the reviewed local browser plan",
        ));
    }
    if draft.navigation_index > browser_plan.navigation_urls.len() {
        return Err(browser_confirmation_error(
            locale,
            "P1065",
            "navigation_index does not identify a reviewed browser-plan URL",
        ));
    }
    Ok((project, browser_plan, draft))
}

fn load_browser_takeover_manifest(
    directory: &Path,
) -> Result<
    (
        ProjectManifest,
        BrowserPlanManifest,
        BrowserTakeoverManifest,
    ),
    String,
> {
    let root = fs::canonicalize(directory)
        .map_err(|error| format!("P1067: cannot resolve project root: {error}"))?;
    let (project, browser_plan) = load_browser_plan_manifest(&root)?;
    browser_takeover_capability_error(&project)?;
    let locale = browser_plan_locale(&project);
    let manifest_path = root.join("padma-browser-takeover.toml");
    let metadata = fs::symlink_metadata(&manifest_path).map_err(|error| {
        browser_confirmation_error(
            locale,
            "P1067",
            &format!("cannot inspect `padma-browser-takeover.toml`: {error}"),
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(browser_confirmation_error(
            locale,
            "P1067",
            "padma-browser-takeover.toml must be a regular project file",
        ));
    }
    let source = fs::read_to_string(&manifest_path).map_err(|error| {
        browser_confirmation_error(
            locale,
            "P1067",
            &format!("cannot read `padma-browser-takeover.toml`: {error}"),
        )
    })?;
    let takeover = parse_browser_takeover_manifest(&source, locale)?;
    if takeover.browser_plan_digest != browser_plan_digest(&browser_plan) {
        return Err(browser_confirmation_error(
            locale,
            "P1067",
            "browser_plan_digest does not match the reviewed local browser plan",
        ));
    }
    if takeover.navigation_index > browser_plan.navigation_urls.len() {
        return Err(browser_confirmation_error(
            locale,
            "P1067",
            "navigation_index does not identify a reviewed browser-plan URL",
        ));
    }
    Ok((project, browser_plan, takeover))
}

fn browser_confirmation_plan_json(directory: &Path) -> Result<String, String> {
    let (project, browser_plan, confirmation) = load_browser_confirmation_plan_manifest(directory)?;
    let destination = browser_plan.navigation_urls[confirmation.navigation_index - 1].clone();
    serde_json::to_string_pretty(&serde_json::json!({
        "browserConfirmationPlanVersion": 1,
        "mode": "local-confirmation-session-planning",
        "project": {"name": project.name, "version": project.version},
        "browserPlan": {
            "digest": browser_plan_digest(&browser_plan),
            "navigationIndex": confirmation.navigation_index,
            "method": "GET",
            "url": destination,
            "redirectPolicy": "deny"
        },
        "confirmation": {
            "required": true,
            "status": "not-issued",
            "challenge": "local-runner-required",
            "singleUse": true,
            "maxSessionSeconds": confirmation.max_session_seconds,
            "modelSupplied": "rejected"
        },
        "session": "awaiting-confirmation",
        "cancellation": "available-before-execution",
        "browser": "not-started",
        "network": "disabled",
        "dns": "disabled",
        "cookies": "not-read",
        "credentials": "not-read",
        "browserProfile": "not-read",
        "environmentRead": "disabled",
        "childProcess": "disabled",
        "javascriptExecution": "disabled",
        "formSubmission": "disabled",
        "posting": "disabled",
        "payment": "disabled",
        "upload": "disabled",
        "download": "disabled"
    }))
    .map(|value| format!("{value}\n"))
    .map_err(|error| format!("P1060: cannot encode browser confirmation plan: {error}"))
}

fn browser_confirmation_inspect_contents(directory: &Path) -> Result<String, String> {
    let root = fs::canonicalize(directory)
        .map_err(|error| format!("P1060: cannot resolve project root: {error}"))?;
    let (project, _, _) = load_browser_confirmation_plan_manifest(&root)?;
    let heading = match browser_plan_locale(&project) {
        Locale::Bangla => "Padma browser confirmation session (শুধু inspection)\n",
        Locale::English => "Padma browser confirmation session (inspection-only)\n",
    };
    Ok(format!(
        "{heading}{}",
        browser_confirmation_plan_json(&root)?
    ))
}

fn run_browser_confirmation_inspect(directory: &Path) -> Result<(), String> {
    print!("{}", browser_confirmation_inspect_contents(directory)?);
    Ok(())
}

fn run_browser_confirmation_plan(directory: &Path) -> Result<(), String> {
    print!("{}", browser_confirmation_plan_json(directory)?);
    Ok(())
}

fn browser_draft_plan_json(directory: &Path) -> Result<String, String> {
    let (project, browser_plan, draft) = load_browser_draft_manifest(directory)?;
    let destination = browser_plan.navigation_urls[draft.navigation_index - 1].clone();
    let attachment = draft.attachment_path.as_ref().map(|path| {
        serde_json::json!({
            "path": path.to_string_lossy(),
            "metadataOnly": true,
            "attachmentRead": "disabled",
            "upload": "disabled"
        })
    });
    serde_json::to_string_pretty(&serde_json::json!({
        "browserDraftPlanVersion": 1,
        "mode": "inspection-only",
        "project": {"name": project.name, "version": project.version},
        "browserPlan": {
            "digest": browser_plan_digest(&browser_plan),
            "navigationIndex": draft.navigation_index,
            "method": "GET",
            "url": destination,
            "redirectPolicy": "deny"
        },
        "draft": {
            "action": draft.action,
            "title": draft.title,
            "body": draft.body,
            "maxReviewSeconds": draft.max_review_seconds,
            "execution": "disabled",
            "copyOrManualEntryOnly": true
        },
        "attachment": attachment,
        "userTakeover": {
            "required": true,
            "decision": "not-collected",
            "visibleBrowser": "user-controlled",
            "login": "user-takeover-required",
            "captchaHandling": "user-takeover-required",
            "formCompletion": "user-takeover-required",
            "posting": "user-takeover-required",
            "upload": "user-takeover-required",
            "download": "user-takeover-required",
            "accountChange": "user-takeover-required",
            "payment": "user-takeover-required"
        },
        "browser": "not-started",
        "network": "disabled",
        "dns": "disabled",
        "attachmentRead": "disabled",
        "upload": "disabled",
        "formSubmission": "disabled",
        "posting": "disabled",
        "payment": "disabled",
        "credentialAccess": "disabled",
        "cookies": "not-read",
        "browserProfile": "not-read",
        "javascriptExecution": "disabled",
        "generatedOutputExecution": "disabled",
        "childProcess": "disabled"
    }))
    .map(|value| format!("{value}\n"))
    .map_err(|error| format!("P1065: cannot encode browser draft plan: {error}"))
}

fn browser_draft_inspect_contents(directory: &Path) -> Result<String, String> {
    let root = fs::canonicalize(directory)
        .map_err(|error| format!("P1065: cannot resolve project root: {error}"))?;
    let (project, _, _) = load_browser_draft_manifest(&root)?;
    let heading = match browser_plan_locale(&project) {
        Locale::Bangla => "Padma browser interaction draft (শুধু inspection)\n",
        Locale::English => "Padma browser interaction draft (inspection-only)\n",
    };
    Ok(format!("{heading}{}", browser_draft_plan_json(&root)?))
}

fn run_browser_draft_inspect(directory: &Path) -> Result<(), String> {
    print!("{}", browser_draft_inspect_contents(directory)?);
    Ok(())
}

fn run_browser_draft_plan(directory: &Path) -> Result<(), String> {
    print!("{}", browser_draft_plan_json(directory)?);
    Ok(())
}

fn browser_takeover_plan_json(directory: &Path) -> Result<String, String> {
    let (project, browser_plan, takeover) = load_browser_takeover_manifest(directory)?;
    let destination = browser_plan.navigation_urls[takeover.navigation_index - 1].clone();
    serde_json::to_string_pretty(&serde_json::json!({
        "browserTakeoverPlanVersion": 1,
        "mode": "inspection-only",
        "project": {"name": project.name, "version": project.version},
        "browserPlan": {
            "digest": browser_plan_digest(&browser_plan),
            "navigationIndex": takeover.navigation_index,
            "method": "GET",
            "url": destination,
            "redirectPolicy": "deny"
        },
        "takeover": {
            "sensitiveAction": takeover.sensitive_action,
            "status": "user-takeover-required",
            "maxReviewSeconds": takeover.max_review_seconds,
            "completion": "not-collected",
            "execution": "disabled",
            "checklist": [
                "Review the digest-bound destination in a visible browser.",
                "Use a separately confirmed Android Browser Handoff only if you choose to open the reviewed URL.",
                "Perform the labelled sensitive action yourself in the destination-controlled browser UI.",
                "Cancel or close the browser if the destination or requested action differs from your review."
            ]
        },
        "visibleHandoff": {
            "status": "not-started",
            "requiresSeparateConfirmationPlan": true,
            "requiresForegroundOpen": true,
            "browserControl": "user-controlled"
        },
        "browser": "not-started",
        "network": "disabled",
        "dns": "disabled",
        "credentialAccess": "disabled",
        "cookies": "not-read",
        "browserProfile": "not-read",
        "pageInspection": "disabled",
        "javascriptExecution": "disabled",
        "formFill": "disabled",
        "formSubmission": "disabled",
        "posting": "disabled",
        "upload": "disabled",
        "download": "disabled",
        "accountChange": "disabled",
        "purchase": "disabled",
        "payment": "disabled",
        "generatedOutputExecution": "disabled",
        "userDecision": "not-collected",
        "childProcess": "disabled"
    }))
    .map(|value| format!("{value}\n"))
    .map_err(|error| format!("P1067: cannot encode browser takeover plan: {error}"))
}

fn browser_takeover_inspect_contents(directory: &Path) -> Result<String, String> {
    let root = fs::canonicalize(directory)
        .map_err(|error| format!("P1067: cannot resolve project root: {error}"))?;
    let (project, _, _) = load_browser_takeover_manifest(&root)?;
    let heading = match browser_plan_locale(&project) {
        Locale::Bangla => "Padma visible browser takeover checklist (শুধু inspection)\n",
        Locale::English => "Padma visible browser takeover checklist (inspection-only)\n",
    };
    Ok(format!("{heading}{}", browser_takeover_plan_json(&root)?))
}

fn run_browser_takeover_inspect(directory: &Path) -> Result<(), String> {
    print!("{}", browser_takeover_inspect_contents(directory)?);
    Ok(())
}

fn run_browser_takeover_plan(directory: &Path) -> Result<(), String> {
    print!("{}", browser_takeover_plan_json(directory)?);
    Ok(())
}

fn load_browser_handoff_context(directory: &Path) -> Result<BrowserHandoffContext, String> {
    let root = fs::canonicalize(directory)
        .map_err(|error| format!("P1062: cannot resolve project root: {error}"))?;
    let (project, browser_plan, confirmation) = load_browser_confirmation_plan_manifest(&root)?;
    browser_handoff_capability_error(&project)?;
    let destination = browser_plan.navigation_urls[confirmation.navigation_index - 1].clone();
    let audit = load_browser_handoff_audit_manifest(&root, &project)?;
    Ok(BrowserHandoffContext {
        locale: browser_plan_locale(&project),
        root,
        destination,
        browser_plan_digest: browser_plan_digest(&browser_plan),
        navigation_index: confirmation.navigation_index,
        audit,
    })
}

fn browser_handoff_confirmation_decision(answer: &str) -> BrowserHandoffDecision {
    if answer.trim() == "OPEN" {
        BrowserHandoffDecision::Open
    } else {
        BrowserHandoffDecision::Cancelled
    }
}

fn browser_handoff_confirmation_prompt(
    locale: Locale,
    destination: &str,
) -> Result<BrowserHandoffDecision, String> {
    let message = match locale {
        Locale::Bangla => format!(
            "\nAndroid Browser Handoff\nReview করা HTTPS URL: {destination}\nPadma শুধু Android browser-এ এই URL handoff করবে। এটি cookie, credential, profile, page content, JavaScript, form, post, upload, download, বা payment access করবে না।\nBrowser খুলতে OPEN লিখে Enter চাপুন; বাতিল করতে অন্য কিছু লিখুন: "
        ),
        Locale::English => format!(
            "\nAndroid Browser Handoff\nReviewed HTTPS URL: {destination}\nPadma will only hand this URL to the Android browser. It will not access cookies, credentials, profiles, page content, JavaScript, forms, posts, uploads, downloads, or payments.\nType OPEN and press Enter to open the browser; type anything else to cancel: "
        ),
    };
    print!("{message}");
    io::stdout().flush().map_err(|_| {
        browser_handoff_error(locale, "P1062", "cannot display confirmation prompt")
    })?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer).map_err(|_| {
        browser_handoff_error(locale, "P1062", "cannot read foreground confirmation")
    })?;
    Ok(browser_handoff_confirmation_decision(&answer))
}

fn browser_handoff_audit_record(
    context: &BrowserHandoffContext,
    state: &str,
    outcome: &str,
) -> Result<(), String> {
    let Some(audit) = &context.audit else {
        return Ok(());
    };
    if !matches!(state, "cancelled" | "opener-requested" | "opener-failed")
        || !matches!(outcome, "P1062" | "P1063" | "requested")
    {
        return Err(browser_handoff_error(
            context.locale,
            "P1064",
            "audit event is outside the fixed handoff event vocabulary",
        ));
    }
    let audit_directory = context.root.join("audit");
    match fs::symlink_metadata(&audit_directory) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return Err(browser_handoff_error(
                context.locale,
                "P1064",
                "audit directory must be a real project directory",
            ));
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            fs::create_dir(&audit_directory).map_err(|_| {
                browser_handoff_error(context.locale, "P1064", "cannot create audit directory")
            })?;
        }
        Err(_) => {
            return Err(browser_handoff_error(
                context.locale,
                "P1064",
                "cannot inspect audit directory",
            ));
        }
    }
    let canonical_directory = fs::canonicalize(&audit_directory).map_err(|_| {
        browser_handoff_error(context.locale, "P1064", "cannot resolve audit directory")
    })?;
    if canonical_directory != audit_directory || !canonical_directory.starts_with(&context.root) {
        return Err(browser_handoff_error(
            context.locale,
            "P1064",
            "audit directory must remain inside the project root",
        ));
    }
    let file_name = audit.path.file_name().ok_or_else(|| {
        browser_handoff_error(context.locale, "P1064", "audit path has no file name")
    })?;
    let audit_path = canonical_directory.join(file_name);
    let mut records = Vec::new();
    match fs::symlink_metadata(&audit_path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(browser_handoff_error(
                    context.locale,
                    "P1064",
                    "audit path must be a regular file",
                ));
            }
            if metadata.len() > (audit.max_records as u64).saturating_mul(512) {
                return Err(browser_handoff_error(
                    context.locale,
                    "P1064",
                    "audit file exceeds the bounded local policy",
                ));
            }
            let existing = fs::read_to_string(&audit_path).map_err(|_| {
                browser_handoff_error(context.locale, "P1064", "cannot read audit file")
            })?;
            for line in existing.lines() {
                let value: JsonValue = serde_json::from_str(line).map_err(|_| {
                    browser_handoff_error(
                        context.locale,
                        "P1064",
                        "audit file has an invalid record",
                    )
                })?;
                let object = value.as_object().ok_or_else(|| {
                    browser_handoff_error(context.locale, "P1064", "audit record must be an object")
                })?;
                let permitted = [
                    "version",
                    "event",
                    "timestampEpochSeconds",
                    "browserPlanDigest",
                    "navigationIndex",
                    "state",
                    "outcome",
                ];
                if object.len() != permitted.len()
                    || object.keys().any(|key| !permitted.contains(&key.as_str()))
                    || value.get("version") != Some(&JsonValue::from(1))
                    || value.get("event") != Some(&JsonValue::from("android-browser-handoff"))
                    || !value
                        .get("browserPlanDigest")
                        .and_then(JsonValue::as_str)
                        .is_some_and(is_sha256_digest)
                    || value
                        .get("navigationIndex")
                        .and_then(JsonValue::as_u64)
                        .filter(|index| *index > 0 && *index <= 16)
                        .is_none()
                    || !value
                        .get("state")
                        .and_then(JsonValue::as_str)
                        .is_some_and(|entry| {
                            matches!(entry, "cancelled" | "opener-requested" | "opener-failed")
                        })
                    || !value
                        .get("outcome")
                        .and_then(JsonValue::as_str)
                        .is_some_and(|entry| matches!(entry, "P1062" | "P1063" | "requested"))
                {
                    return Err(browser_handoff_error(
                        context.locale,
                        "P1064",
                        "audit file contains a non-redacted or unsupported record",
                    ));
                }
                records.push(serde_json::to_string(&value).map_err(|_| {
                    browser_handoff_error(context.locale, "P1064", "cannot normalize audit record")
                })?);
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(_) => {
            return Err(browser_handoff_error(
                context.locale,
                "P1064",
                "cannot inspect audit file",
            ));
        }
    }
    let timestamp_epoch_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| browser_handoff_error(context.locale, "P1064", "system clock is invalid"))?
        .as_secs();
    let record = serde_json::to_string(&serde_json::json!({
        "version": 1,
        "event": "android-browser-handoff",
        "timestampEpochSeconds": timestamp_epoch_seconds,
        "browserPlanDigest": context.browser_plan_digest,
        "navigationIndex": context.navigation_index,
        "state": state,
        "outcome": outcome
    }))
    .map_err(|_| browser_handoff_error(context.locale, "P1064", "cannot encode audit record"))?;
    records.push(record);
    let keep_from = records.len().saturating_sub(audit.max_records);
    let contents = format!("{}\n", records[keep_from..].join("\n"));
    let temporary = canonical_directory.join(format!(
        ".padma-browser-audit-{}-{}.tmp",
        process::id(),
        RANDOM_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&temporary, contents).map_err(|_| {
        browser_handoff_error(context.locale, "P1064", "cannot write temporary audit file")
    })?;
    fs::rename(&temporary, &audit_path).map_err(|_| {
        let _ = fs::remove_file(&temporary);
        browser_handoff_error(context.locale, "P1064", "cannot finalize audit file")
    })
}

fn termux_browser_handoff_command(path: &std::ffi::OsStr, destination: &str) -> process::Command {
    let mut command = process::Command::new("termux-open-url");
    command
        .env_clear()
        .env("PATH", path)
        .stdin(process::Stdio::null())
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .arg(destination);
    command
}

fn termux_browser_handoff(destination: &str, locale: Locale) -> Result<(), String> {
    let path = env::var_os("PATH").ok_or_else(|| {
        browser_handoff_error(locale, "P1063", "Termux command path is unavailable")
    })?;
    let status = termux_browser_handoff_command(&path, destination)
        .status()
        .map_err(|_| browser_handoff_error(locale, "P1063", "termux-open-url is unavailable"))?;
    if !status.success() {
        return Err(browser_handoff_error(
            locale,
            "P1063",
            "termux-open-url returned a failure status",
        ));
    }
    Ok(())
}

fn run_browser_handoff(directory: &Path) -> Result<(), String> {
    let context = load_browser_handoff_context(directory)?;
    match browser_handoff_confirmation_prompt(context.locale, &context.destination)? {
        BrowserHandoffDecision::Cancelled => {
            browser_handoff_audit_record(&context, "cancelled", "P1062")?;
            return Err(browser_handoff_error(
                context.locale,
                "P1062",
                "confirmation was cancelled",
            ));
        }
        BrowserHandoffDecision::Open => {}
    }
    if let Err(error) = termux_browser_handoff(&context.destination, context.locale) {
        let _ = browser_handoff_audit_record(&context, "opener-failed", "P1063");
        return Err(error);
    }
    browser_handoff_audit_record(&context, "opener-requested", "requested")?;
    match context.locale {
        Locale::Bangla => println!(
            "Android browser handoff অনুরোধ করা হয়েছে। Padma browser content, cookie, credential, বা profile দেখতে বা নিয়ন্ত্রণ করতে পারে না।"
        ),
        Locale::English => println!(
            "Android Browser Handoff was requested. Padma cannot view or control browser content, cookies, credentials, or profiles."
        ),
    }
    Ok(())
}

fn run_gui_inspect(directory: &Path) -> Result<(), String> {
    print!("{}", gui_inspect_contents(directory)?);
    Ok(())
}

fn run_gui_plan(directory: &Path) -> Result<(), String> {
    print!("{}", gui_plan_contents(directory)?);
    Ok(())
}

fn android_capability_error(project: &ProjectManifest) -> Result<(), String> {
    if project.capabilities.contains("android:plan") {
        return Ok(());
    }
    let locale = if project.locale == "bn" {
        Locale::Bangla
    } else {
        Locale::English
    };
    let diagnostic = error_for(locale, "P1034", Position::new(1, 1), "android:plan");
    Err(format!(
        "{}: {}\n  = {}: {}",
        diagnostic.code,
        diagnostic.message,
        if locale == Locale::Bangla {
            "পরামর্শ"
        } else {
            "help"
        },
        diagnostic.hint.unwrap_or_default()
    ))
}

fn load_android_build_manifest(
    directory: &Path,
) -> Result<(ProjectManifest, GuiManifest, AndroidBuildManifest, String), String> {
    let root = fs::canonicalize(directory)
        .map_err(|error| format!("P1049: cannot resolve project root: {error}"))?;
    let (project, gui, entry, assets) = load_gui_manifest(&root)?;
    android_capability_error(&project)?;
    let manifest_path = root.join("padma-android.toml");
    let metadata = fs::symlink_metadata(&manifest_path)
        .map_err(|error| format!("P1049: cannot inspect Android manifest: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("P1049: Android manifest must be a regular project file".to_string());
    }
    let manifest = parse_android_build_manifest(
        &fs::read_to_string(&manifest_path)
            .map_err(|error| format!("P1049: cannot read Android manifest: {error}"))?,
    )?;
    let artifact = root.join(safe_gui_relative_path(&manifest.artifact, Some(".apk"))?);
    if artifact.exists() {
        let artifact_metadata = fs::symlink_metadata(&artifact)
            .map_err(|error| format!("P1049: cannot inspect Android artifact path: {error}"))?;
        if artifact_metadata.file_type().is_symlink() || !artifact_metadata.is_file() {
            return Err(
                "P1049: existing Android artifact path must be a regular project file".to_string(),
            );
        }
    }
    let source_digest = gui_source_digest(&root, &entry, &assets)?;
    Ok((project, gui, manifest, source_digest))
}

fn android_inspect_contents(directory: &Path) -> Result<String, String> {
    let (_project, gui, manifest, _source_digest) = load_android_build_manifest(directory)?;
    Ok(format!(
        "Padma Android build manifest (read-only)\n  application_id: {}\n  GUI entry: {}\n  min_sdk: {}\n  target_sdk: {}\n  artifact: {}\n  declared permissions: {}\n",
        manifest.application_id,
        gui.entry,
        manifest.min_sdk,
        manifest.target_sdk,
        manifest.artifact,
        if manifest.permissions.is_empty() {
            "(none)".to_string()
        } else {
            manifest.permissions.iter().cloned().collect::<Vec<_>>().join(", ")
        }
    ))
}

fn android_build_plan_contents(directory: &Path) -> Result<String, String> {
    let (project, gui, manifest, source_digest) = load_android_build_manifest(directory)?;
    let runtime_permissions = manifest
        .permissions
        .iter()
        .filter(|permission| {
            matches!(
                permission.as_str(),
                "android.permission.CAMERA"
                    | "android.permission.RECORD_AUDIO"
                    | "android.permission.POST_NOTIFICATIONS"
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    serde_json::to_string_pretty(&serde_json::json!({
        "androidBuildPlanVersion": 1,
        "mode": "read-only-build-plan",
        "project": {"name": project.name, "version": project.version},
        "application": {"id": manifest.application_id, "minSdk": manifest.min_sdk, "targetSdk": manifest.target_sdk},
        "gui": {"backend": gui.backend, "entry": gui.entry, "assets": gui.assets, "sourceDigest": source_digest},
        "artifact": {"expectedPath": manifest.artifact, "apkBuild": "disabled", "artifactWrite": "disabled"},
        "signing": {"keyEnvironmentName": manifest.signing_key_env, "certificateSha256": manifest.signing_cert_sha256, "keyRead": "disabled", "signing": "disabled"},
        "permissions": {"declared": manifest.permissions.into_iter().collect::<Vec<_>>(), "runtimeConsentRequired": runtime_permissions, "automaticGrant": "disabled"},
        "device": {"adb": "disabled", "install": "disabled", "control": "disabled"},
        "nativeCode": {"execution": "disabled", "jni": "disabled", "hooks": "disabled"},
        "network": "disabled",
        "rendererLaunch": "disabled"
    }))
    .map(|value| format!("{value}\n"))
    .map_err(|error| format!("P1049: cannot encode Android build plan: {error}"))
}

fn run_android_inspect(directory: &Path) -> Result<(), String> {
    print!("{}", android_inspect_contents(directory)?);
    Ok(())
}

fn run_android_build_plan(directory: &Path) -> Result<(), String> {
    print!("{}", android_build_plan_contents(directory)?);
    Ok(())
}

fn project_source_with_locale(source: String, locale: &str) -> String {
    match locale {
        "bn" => format!("# padma:locale=bn\n{source}"),
        "en" => format!("# padma:locale=en\n{source}"),
        _ => source,
    }
}

fn parse_source_recovering(source: &str) -> Result<(Vec<Stmt>, Locale), Vec<PadmaError>> {
    let locale = Locale::from_source(source);
    let tokens = Lexer::new(source, locale)
        .tokenize()
        .map_err(|error| vec![error])?;
    let (program, errors) = Parser::new(tokens, locale).parse_recovering();
    if errors.is_empty() {
        Ok((program, locale))
    } else {
        Err(errors)
    }
}

fn static_builtin_arity(name: &str) -> Option<(usize, usize)> {
    match name {
        "range" | "পরিসর" | "media.download" => Some((1, 2)),
        "process.run" => Some((1, usize::MAX)),
        "bridge.call" => Some((3, 3)),
        "auth.session_issue" | "auth.cookie" => Some((3, 3)),
        "auth.password_verify" | "auth.session_verify" | "auth.csrf_verify" => Some((2, 2)),
        "db.get" | "db.delete" | "db.list" | "table.filter_equal" => Some((3, 3)),
        "input" | "file.read" | "file.exists" | "http.get" | "text.len" | "text.trim"
        | "text.upper" | "text.lower" | "path.basename" | "path.extension" | "random.pick"
        | "json.parse" | "json.stringify" | "url.is_valid" | "url.parse" | "time.sleep"
        | "math.abs" | "math.round" | "math.floor" | "math.ceil" | "auth.password_hash"
        | "ai.workflow" | "table.headers" | "table.rows" => Some((1, 1)),
        "file.write" | "text.contains" | "text.split" | "text.join" | "text.format"
        | "random.int" | "table.read" | "table.select" | "table.count_by" | "table.write_csv" => {
            Some((2, 2))
        }
        "text.replace" => Some((3, 3)),
        "db.put" => Some((4, 4)),
        "db.version" => Some((1, 1)),
        "db.apply" => Some((2, 2)),
        "time.now" | "auth.csrf_token" => Some((0, 0)),
        _ => None,
    }
}

fn static_function_arities(statements: &[Stmt]) -> BTreeMap<String, usize> {
    let mut arities = BTreeMap::new();
    for statement in statements {
        let statement = match statement {
            Stmt::Export(inner) => inner.as_ref(),
            other => other,
        };
        if let Stmt::Function { name, params, .. } = statement {
            arities.insert(name.clone(), params.len());
        }
    }
    arities
}

fn static_check_expression(
    expression: &Expr,
    locale: Locale,
    function_arities: &BTreeMap<String, usize>,
    errors: &mut Vec<PadmaError>,
) {
    match expression {
        Expr::Unary { right, .. } => {
            static_check_expression(right, locale, function_arities, errors)
        }
        Expr::Binary {
            left,
            operator,
            right,
            position,
        } => {
            static_check_expression(left, locale, function_arities, errors);
            static_check_expression(right, locale, function_arities, errors);
            if matches!(operator, TokenKind::Slash)
                && matches!(right.as_ref(), Expr::Literal(Value::Number(value), _) if *value == 0.0)
            {
                errors.push(error_for(locale, "P1011", *position, "division"));
            }
        }
        Expr::Call {
            name,
            arguments,
            position,
        } => {
            for argument in arguments {
                static_check_expression(argument, locale, function_arities, errors);
            }
            let expected = static_builtin_arity(name)
                .or_else(|| function_arities.get(name).map(|value| (*value, *value)));
            if let Some((minimum, maximum)) = expected {
                if arguments.len() < minimum || arguments.len() > maximum {
                    errors.push(error_for(locale, "P1009", *position, name));
                }
            }
        }
        Expr::Index { target, index, .. } => {
            static_check_expression(target, locale, function_arities, errors);
            static_check_expression(index, locale, function_arities, errors);
        }
        Expr::Slice {
            target, start, end, ..
        } => {
            static_check_expression(target, locale, function_arities, errors);
            if let Some(start) = start {
                static_check_expression(start, locale, function_arities, errors);
            }
            if let Some(end) = end {
                static_check_expression(end, locale, function_arities, errors);
            }
        }
        Expr::List(values) => {
            for value in values {
                static_check_expression(value, locale, function_arities, errors);
            }
        }
        Expr::Map(entries) => {
            for (key, value) in entries {
                static_check_expression(key, locale, function_arities, errors);
                static_check_expression(value, locale, function_arities, errors);
            }
        }
        Expr::Literal(_, _) | Expr::Variable(_, _) => {}
    }
}

fn static_check_statements(
    statements: &[Stmt],
    locale: Locale,
    function_arities: &BTreeMap<String, usize>,
    errors: &mut Vec<PadmaError>,
) {
    for statement in statements {
        match statement {
            Stmt::Let { value, .. } | Stmt::Print { value } | Stmt::Expression { value } => {
                static_check_expression(value, locale, function_arities, errors)
            }
            Stmt::Assign { value, .. } => {
                static_check_expression(value, locale, function_arities, errors)
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                static_check_expression(condition, locale, function_arities, errors);
                static_check_statements(then_branch, locale, function_arities, errors);
                static_check_statements(else_branch, locale, function_arities, errors);
            }
            Stmt::While {
                condition, body, ..
            } => {
                static_check_expression(condition, locale, function_arities, errors);
                static_check_statements(body, locale, function_arities, errors);
            }
            Stmt::For {
                collection, body, ..
            } => {
                static_check_expression(collection, locale, function_arities, errors);
                static_check_statements(body, locale, function_arities, errors);
            }
            Stmt::Function { body, .. } => {
                static_check_statements(body, locale, function_arities, errors)
            }
            Stmt::Return { value } => {
                if let Some(value) = value {
                    static_check_expression(value, locale, function_arities, errors);
                }
            }
            Stmt::Export(inner) => static_check_statements(
                std::slice::from_ref(inner),
                locale,
                function_arities,
                errors,
            ),
            Stmt::Import { .. } => {}
        }
    }
}

fn syntax_check_source(source: &str) -> Result<Locale, Vec<PadmaError>> {
    parse_source_recovering(source).map(|(_, locale)| locale)
}

fn check_source(source: &str) -> Result<Locale, Vec<PadmaError>> {
    let (program, locale) = parse_source_recovering(source)?;
    let mut errors = Vec::new();
    let function_arities = static_function_arities(&program);
    static_check_statements(&program, locale, &function_arities, &mut errors);
    if errors.is_empty() {
        Ok(locale)
    } else {
        Err(errors)
    }
}

fn format_diagnostic(path: &str, source: &str, error: &PadmaError) -> String {
    let rendered_path = error
        .source_path
        .as_ref()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());
    let rendered_source = error.source_text.as_deref().unwrap_or(source);
    let rendered_locale = error.locale;
    let line = rendered_source
        .lines()
        .nth(error.position.line.saturating_sub(1))
        .unwrap_or("");
    let gutter_width = error.position.line.to_string().len();
    let marker = " ".repeat(error.position.column.saturating_sub(1));
    let (label, point, hint_label) = match rendered_locale {
        Locale::Bangla => ("ত্রুটি", "এই স্থানে", "পরামর্শ"),
        Locale::English => ("error", "here", "help"),
    };
    let mut rendered = format!(
        "{label}[{}]: {}\n  --> {}:{}:{}\n   |\n{:>width$} | {}\n   | {}^ {point}\n",
        error.code,
        error.message,
        rendered_path,
        error.position.line,
        error.position.column,
        error.position.line,
        line,
        marker,
        width = gutter_width
    );
    if let Some(hint) = &error.hint {
        rendered.push_str(&format!("   = {hint_label}: {hint}\n"));
    }
    rendered
}

fn diagnostic_json(path: &str, source: &str, error: &PadmaError) -> JsonValue {
    let rendered_path = error
        .source_path
        .as_ref()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());
    let rendered_source = error.source_text.as_deref().unwrap_or(source);
    let line_text = rendered_source
        .lines()
        .nth(error.position.line.saturating_sub(1))
        .unwrap_or("");
    let locale = match error.locale {
        Locale::Bangla => "bn",
        Locale::English => "en",
    };
    serde_json::json!({
        "code": error.code,
        "message": error.message,
        "hint": error.hint,
        "locale": locale,
        "path": rendered_path,
        "range": {
            "start": { "line": error.position.line, "column": error.position.column },
            "end": { "line": error.position.line, "column": error.position.column + 1 }
        },
        "source_line": line_text,
    })
}

fn check_json(path: &str, source: &str) -> String {
    match check_source(source) {
        Ok(locale) => {
            let locale = match locale {
                Locale::Bangla => "bn",
                Locale::English => "en",
            };
            serde_json::json!({
                "status": "ok",
                "path": path,
                "locale": locale,
                "diagnostics": []
            })
            .to_string()
        }
        Err(errors) => serde_json::json!({
            "status": "error",
            "path": path,
            "diagnostics": errors
                .iter()
                .map(|error| diagnostic_json(path, source, error))
                .collect::<Vec<_>>()
        })
        .to_string(),
    }
}

fn format_source(source: &str) -> String {
    let mut formatted_lines = Vec::new();
    let mut indentation = 0isize;
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            formatted_lines.push(String::new());
            continue;
        }
        let line_indentation = if line.starts_with('}') {
            (indentation - 1).max(0)
        } else {
            indentation.max(0)
        };
        formatted_lines.push(format!(
            "{}{}",
            "    ".repeat(line_indentation as usize),
            line
        ));
        indentation = (indentation + brace_delta(line)).max(0);
    }
    while formatted_lines.last().is_some_and(String::is_empty) {
        formatted_lines.pop();
    }
    if formatted_lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", formatted_lines.join("\n"))
    }
}

#[derive(Clone, Debug)]
struct LintWarning {
    code: &'static str,
    message: String,
    hint: String,
    position: Position,
    locale: Locale,
}

fn lint_warning(
    locale: Locale,
    code: &'static str,
    position: Position,
    message: &str,
    hint: &str,
) -> LintWarning {
    LintWarning {
        code,
        message: message.to_string(),
        hint: hint.to_string(),
        position,
        locale,
    }
}

fn source_has_keyword(source: &str, keyword: &str) -> bool {
    source.lines().any(|line| {
        let mut code = String::new();
        let mut in_string = false;
        let mut escaped = false;
        for character in line.chars() {
            if escaped {
                escaped = false;
                continue;
            }
            if in_string && character == '\\' {
                escaped = true;
                continue;
            }
            if character == '"' {
                in_string = !in_string;
                continue;
            }
            if !in_string && character == '#' {
                break;
            }
            if !in_string {
                code.push(character);
            }
        }
        if !keyword.is_ascii() {
            code.contains(keyword)
        } else {
            code.split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
                .any(|token| token == keyword)
        }
    })
}

fn lint_source(source: &str) -> (Locale, Vec<LintWarning>) {
    let locale = Locale::from_source(source);
    let mut warnings = Vec::new();
    for (index, raw_line) in source.lines().enumerate() {
        let line_number = index + 1;
        let trimmed_end = raw_line.trim_end_matches([' ', '\t']);
        if trimmed_end.len() != raw_line.len() {
            let column = trimmed_end.chars().count() + 1;
            let (message, hint) = match locale {
                Locale::Bangla => (
                    "লাইনের শেষে অপ্রয়োজনীয় ফাঁকা স্থান আছে",
                    "লাইনের শেষের space বা tab সরান।",
                ),
                Locale::English => (
                    "line has trailing whitespace",
                    "remove spaces or tabs from the end of the line.",
                ),
            };
            warnings.push(lint_warning(
                locale,
                "L1001",
                Position::new(line_number, column),
                message,
                hint,
            ));
        }
        if let Some(column) = raw_line
            .chars()
            .take_while(|character| character.is_whitespace())
            .position(|character| character == '\t')
        {
            let (message, hint) = match locale {
                Locale::Bangla => (
                    "indentation-এ tab ব্যবহার করা হয়েছে",
                    "সামঞ্জস্যপূর্ণ indentation-এর জন্য চারটি space ব্যবহার করুন।",
                ),
                Locale::English => (
                    "indentation uses a tab character",
                    "use four spaces for consistent indentation.",
                ),
            };
            warnings.push(lint_warning(
                locale,
                "L1002",
                Position::new(line_number, column + 1),
                message,
                hint,
            ));
        }
    }
    let bangla_keywords = [
        "ধরি",
        "দেখাও",
        "যদি",
        "নইলে",
        "যতক্ষণ",
        "প্রতি",
        "মধ্যে",
        "ফাংশন",
        "ফেরত",
        "ইমপোর্ট",
    ];
    let english_keywords = [
        "let", "print", "if", "else", "while", "for", "in", "function", "return", "import",
    ];
    if bangla_keywords
        .iter()
        .any(|keyword| source_has_keyword(source, keyword))
        && english_keywords
            .iter()
            .any(|keyword| source_has_keyword(source, keyword))
    {
        let (message, hint) = match locale {
            Locale::Bangla => (
                "Bangla এবং English keyword একসঙ্গে ব্যবহার করা হয়েছে",
                "একটি ফাইলে একটি keyword style বেছে নিন, অথবা এই warning অনুমোদন করুন।",
            ),
            Locale::English => (
                "Bangla and English keywords are mixed in this file",
                "choose one keyword style per file, or explicitly allow this warning.",
            ),
        };
        warnings.push(lint_warning(
            locale,
            "L1003",
            Position::new(1, 1),
            message,
            hint,
        ));
    }
    (locale, warnings)
}

fn lint_source_with_disabled(
    source: &str,
    disabled_rules: &BTreeSet<String>,
) -> (Locale, Vec<LintWarning>) {
    let (locale, warnings) = lint_source(source);
    (
        locale,
        warnings
            .into_iter()
            .filter(|warning| !disabled_rules.contains(warning.code))
            .collect(),
    )
}

fn format_lint_warning(path: &str, warning: &LintWarning) -> String {
    let label = match warning.locale {
        Locale::Bangla => "সতর্কতা",
        Locale::English => "warning",
    };
    let hint_label = match warning.locale {
        Locale::Bangla => "পরামর্শ",
        Locale::English => "help",
    };
    format!(
        "{label}[{}]: {}\n  --> {}:{}:{}\n   = {hint_label}: {}\n",
        warning.code,
        warning.message,
        path,
        warning.position.line,
        warning.position.column,
        warning.hint
    )
}

fn lint_json_with_disabled(path: &str, source: &str, disabled_rules: &BTreeSet<String>) -> String {
    let (locale, warnings) = lint_source_with_disabled(source, disabled_rules);
    let locale = match locale {
        Locale::Bangla => "bn",
        Locale::English => "en",
    };
    serde_json::json!({
        "status": if warnings.is_empty() { "ok" } else { "warning" },
        "path": path,
        "locale": locale,
        "warnings": warnings.iter().map(|warning| serde_json::json!({
            "code": warning.code,
            "message": warning.message,
            "hint": warning.hint,
            "range": {
                "start": { "line": warning.position.line, "column": warning.position.column },
                "end": { "line": warning.position.line, "column": warning.position.column + 1 }
            }
        })).collect::<Vec<_>>()
    })
    .to_string()
}

fn usage(locale: Locale) -> &'static str {
    match locale {
        Locale::Bangla => {
            "ব্যবহার: padma [file.pd|.] অথবা padma <run|check|fmt|lint|ast> <file.pd>\n\nকমান্ড:\n  padma                 interactive shell চালু করুন\n  padma <file.pd>       Padma script চালান\n  padma .               padma.toml project চালান\n  padma serve [project] local health server চালান\n  padma init [folder]   নতুন Padma project তৈরি করুন\n  padma capabilities <project>  project permission দেখুন\n  padma package lock [project]  verified local package lockfile লিখুন\n  padma package verify [project]  package digest ও lockfile যাচাই করুন\n  padma package inspect <name> [project]  local package metadata দেখুন\n  padma ai inspect [project]  AI workflow manifest নিরাপদভাবে inspect করুন\n  padma ai plan [project]  network ছাড়া AI workflow plan দেখুন\n  padma ai tools inspect [project]  AI tool manifest local inspect করুন\n  padma ai tools plan [project]  tool/agent ছাড়া AI tool plan দেখুন\n  padma ai training inspect [project]  AI training manifest local inspect করুন\n  padma ai training plan [project]  training ছাড়া resource-bounded plan দেখুন\n  padma browser inspect [project]  browser plan manifest inspect করুন\n  padma browser plan [project]  browser ছাড়া, network ছাড়া navigation plan দেখুন\n  padma browser draft inspect [project]  browser interaction draft local inspect করুন\n  padma browser draft plan [project]  browser ছাড়া inert user-takeover draft plan দেখুন\n  padma browser takeover inspect [project]  visible user-takeover checklist local inspect করুন\n  padma browser takeover plan [project]  browser ছাড়া sensitive-action takeover plan দেখুন\n  padma deploy plan [project]  dry-run deployment plan দেখুন\n  padma deploy inspect [project]  deployment manifest inspect করুন\n  padma render plan [project]  Git-linked Render release plan দেখুন\n  padma render inspect [project]  Render release manifest inspect করুন\n  padma render api-plan [project]  Render API deploy/rollback plan দেখুন\n  padma render deploy --confirm <token> [project]  confirmed Render deploy চালান\n  padma render rollback --confirm <token> [project]  confirmed Render rollback চালান\n  padma gui inspect [project]  local GUI manifest দেখুন\n  padma gui plan [project]  read-only GUI renderer plan দেখুন\n  padma android inspect [project]  Android build manifest দেখুন\n  padma android plan [project]  read-only Android APK build plan দেখুন\n  padma check --json <file.pd>  JSON diagnostic দিন\n  padma fmt <file.pd>   source format করুন\n  padma fmt --check <file.pd>  source পরিবর্তন দরকার কি না দেখুন\n  padma lint <file.pd>  style warning দেখুন\n  padma lint --json <file.pd>  JSON lint report দিন\n  padma --version       version দেখুন\n  padma --help          এই help দেখুন\n\nউদাহরণ:\n  padma init আমার-project\n  padma serve .\n  padma ai plan .\n  padma ai tools plan .\n  padma ai training plan .\n  padma browser plan .\n  padma browser draft plan .\n  padma browser takeover plan .\n  padma render api-plan .\n  padma gui plan .\n  padma android plan .\n  padma examples/hello-bn.pd\n"
        }
        Locale::English => {
            "Usage: padma [file.pd|.] or padma <run|check|fmt|lint|ast> <file.pd>\n\nCommands:\n  padma                 open the interactive shell\n  padma <file.pd>       run a Padma script\n  padma .               run a padma.toml project\n  padma serve [project] run a loopback local health server\n  padma init [folder]   create a new Padma project\n  padma capabilities <project>  inspect project permissions\n  padma package lock [project]  write a verified local package lockfile\n  padma package verify [project]  verify local package digests and lockfile\n  padma package inspect <name> [project]  inspect local package metadata\n  padma ai inspect [project]  inspect an AI workflow manifest safely\n  padma ai plan [project]  print an AI workflow plan without network access\n  padma ai tools inspect [project]  inspect an AI tool manifest locally\n  padma ai tools plan [project]  print an AI tool plan without tools or an agent\n  padma ai training inspect [project]  inspect an AI training manifest locally\n  padma ai training plan [project]  print a training plan without dataset reads or training\n  padma browser inspect [project]  inspect a browser plan manifest locally\n  padma browser plan [project]  print a navigation plan without browser or network access\n  padma browser draft inspect [project]  inspect a browser interaction draft locally\n  padma browser draft plan [project]  print an inert, user-takeover draft plan without a browser\n  padma browser takeover inspect [project]  inspect a visible user-takeover checklist locally\n  padma browser takeover plan [project]  print a sensitive-action takeover plan without a browser\n  padma deploy plan [project]  print a dry-run deployment plan\n  padma deploy inspect [project]  inspect a deployment manifest locally\n  padma render plan [project]  print a Git-linked Render release plan\n  padma render inspect [project]  inspect a Render release manifest locally\n  padma render api-plan [project]  print a Render API deploy/rollback plan\n  padma render deploy --confirm <token> [project]  run a confirmed Render deploy\n  padma render rollback --confirm <token> [project]  run a confirmed Render rollback\n  padma gui inspect [project]  inspect a local GUI manifest\n  padma gui plan [project]  print a read-only GUI renderer plan\n  padma android inspect [project]  inspect an Android build manifest\n  padma android plan [project]  print a read-only Android APK build plan\n  padma check --json <file.pd>  emit JSON diagnostics\n  padma fmt <file.pd>   format a source file in place\n  padma fmt --check <file.pd>  report whether formatting is needed\n  padma lint <file.pd>  report style warnings\n  padma lint --json <file.pd>  emit JSON lint warnings\n  padma --version       show the installed version\n  padma --help          show this help\n\nExamples:\n  padma init my-project\n  padma serve .\n  padma ai plan .\n  padma ai tools plan .\n  padma ai training plan .\n  padma browser plan .\n  padma browser draft plan .\n  padma browser takeover plan .\n  padma render api-plan .\n  padma gui plan .\n  padma android plan .\n  padma examples/hello-en.pd\n"
        }
    }
}
fn write_local_server_response(stream: &mut TcpStream, status: &str, body: &str) -> io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn serve_local_project(directory: &Path) -> Result<(), String> {
    let (manifest, _) = load_project_manifest(directory)?;
    if !manifest.capabilities.contains("server:local") {
        let locale = if manifest.locale == "bn" {
            Locale::Bangla
        } else {
            Locale::English
        };
        let diagnostic = error_for(locale, "P1034", Position::new(1, 1), "server:local");
        return Err(format!(
            "{}: {}\n  = {}: {}",
            diagnostic.code,
            diagnostic.message,
            if locale == Locale::Bangla {
                "পরামর্শ"
            } else {
                "help"
            },
            diagnostic.hint.unwrap_or_default()
        ));
    }

    let listener = TcpListener::bind("127.0.0.1:8080")
        .map_err(|error| format!("cannot bind loopback server: {error}"))?;
    println!(
        "Padma local server for `{}`: http://127.0.0.1:8080/health",
        manifest.name
    );
    for incoming in listener.incoming() {
        let mut stream = match incoming {
            Ok(stream) => stream,
            Err(_) => continue,
        };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let mut request = [0u8; 8192];
        let read = stream.read(&mut request).unwrap_or(0);
        let first_line = std::str::from_utf8(&request[..read])
            .ok()
            .and_then(|text| text.lines().next())
            .unwrap_or("");
        let (status, body) =
            if first_line.starts_with("GET /health ") || first_line.starts_with("HEAD /health ") {
                (
                    "200 OK",
                    serde_json::json!({"status":"ok", "project":manifest.name}).to_string(),
                )
            } else {
                ("404 Not Found", "{\"error\":\"not found\"}".to_string())
            };
        let _ = write_local_server_response(&mut stream, status, &body);
    }
    Ok(())
}

fn main() {
    let arguments: Vec<String> = env::args().collect();
    if arguments.len() == 1 {
        repl();
        return;
    }
    if arguments.len() == 2 && matches!(arguments[1].as_str(), "--version" | "-V") {
        println!("padma 0.1.0");
        return;
    }
    if arguments.len() == 2 && matches!(arguments[1].as_str(), "--help" | "-h") {
        println!("{}", usage(Locale::English));
        return;
    }
    if arguments.get(1).map(String::as_str) == Some("serve") {
        let directory = arguments
            .get(2)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        if arguments.len() > 3 {
            eprintln!("{}", usage(Locale::English));
            process::exit(64);
        }
        if let Err(error) = serve_local_project(&directory) {
            eprintln!("{error}");
            process::exit(1);
        }
        return;
    }
    if arguments.get(1).map(String::as_str) == Some("package") {
        let command = arguments.get(2).map(String::as_str);
        let result = match command {
            Some("lock") if arguments.len() <= 4 => {
                let directory = arguments
                    .get(3)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."));
                write_package_lock(&directory).map(|_| {
                    println!(
                        "Verified local package lockfile written to {}/padma.lock",
                        directory.display()
                    );
                })
            }
            Some("verify") if arguments.len() <= 4 => {
                let directory = arguments
                    .get(3)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."));
                verify_package_lock(&directory).map(|_| {
                    println!("Local package lockfile and digests are verified.");
                })
            }
            Some("inspect") if (4..=5).contains(&arguments.len()) => {
                let directory = arguments
                    .get(4)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."));
                inspect_local_package(&directory, &arguments[3])
            }
            _ => {
                eprintln!("{}", usage(Locale::English));
                process::exit(64);
            }
        };
        if let Err(error) = result {
            eprintln!("{error}");
            process::exit(1);
        }
        return;
    }
    if arguments.get(1).map(String::as_str) == Some("ai") {
        let command = arguments.get(2).map(String::as_str);
        let result = match command {
            Some("tools") => match arguments.get(3).map(String::as_str) {
                Some("inspect") if arguments.len() <= 5 => {
                    let directory = arguments
                        .get(4)
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from("."));
                    run_ai_tool_inspect(&directory)
                }
                Some("plan") if arguments.len() <= 5 => {
                    let directory = arguments
                        .get(4)
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from("."));
                    run_ai_tool_plan(&directory)
                }
                _ => {
                    eprintln!("{}", usage(Locale::English));
                    process::exit(64);
                }
            },
            Some("training") => match arguments.get(3).map(String::as_str) {
                Some("inspect") if arguments.len() <= 5 => {
                    let directory = arguments
                        .get(4)
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from("."));
                    run_ai_training_inspect(&directory)
                }
                Some("plan") if arguments.len() <= 5 => {
                    let directory = arguments
                        .get(4)
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from("."));
                    run_ai_training_plan(&directory)
                }
                _ => {
                    eprintln!("{}", usage(Locale::English));
                    process::exit(64);
                }
            },
            Some("inspect") if arguments.len() <= 4 => {
                let directory = arguments
                    .get(3)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."));
                run_ai_workflow_inspect(&directory)
            }
            Some("plan") if arguments.len() <= 4 => {
                let directory = arguments
                    .get(3)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."));
                run_ai_workflow_plan(&directory)
            }
            _ => {
                eprintln!("{}", usage(Locale::English));
                process::exit(64);
            }
        };
        if let Err(error) = result {
            eprintln!("{error}");
            process::exit(1);
        }
        return;
    }
    if arguments.get(1).map(String::as_str) == Some("browser") {
        let command = arguments.get(2).map(String::as_str);
        let result = match command {
            Some("inspect") if arguments.len() <= 4 => {
                let directory = arguments
                    .get(3)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."));
                run_browser_inspect(&directory)
            }
            Some("plan") if arguments.len() <= 4 => {
                let directory = arguments
                    .get(3)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."));
                run_browser_plan(&directory)
            }
            Some("confirm") => match arguments.get(3).map(String::as_str) {
                Some("inspect") if arguments.len() <= 5 => {
                    let directory = arguments
                        .get(4)
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from("."));
                    run_browser_confirmation_inspect(&directory)
                }
                Some("plan") if arguments.len() <= 5 => {
                    let directory = arguments
                        .get(4)
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from("."));
                    run_browser_confirmation_plan(&directory)
                }
                _ => {
                    eprintln!("{}", usage(Locale::English));
                    process::exit(64);
                }
            },
            Some("draft") => match arguments.get(3).map(String::as_str) {
                Some("inspect") if arguments.len() <= 5 => {
                    let directory = arguments
                        .get(4)
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from("."));
                    run_browser_draft_inspect(&directory)
                }
                Some("plan") if arguments.len() <= 5 => {
                    let directory = arguments
                        .get(4)
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from("."));
                    run_browser_draft_plan(&directory)
                }
                _ => {
                    eprintln!("{}", usage(Locale::English));
                    process::exit(64);
                }
            },
            Some("takeover") => match arguments.get(3).map(String::as_str) {
                Some("inspect") if arguments.len() <= 5 => {
                    let directory = arguments
                        .get(4)
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from("."));
                    run_browser_takeover_inspect(&directory)
                }
                Some("plan") if arguments.len() <= 5 => {
                    let directory = arguments
                        .get(4)
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from("."));
                    run_browser_takeover_plan(&directory)
                }
                _ => {
                    eprintln!("{}", usage(Locale::English));
                    process::exit(64);
                }
            },
            Some("handoff") if arguments.len() <= 4 => {
                let directory = arguments
                    .get(3)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."));
                run_browser_handoff(&directory)
            }
            _ => {
                eprintln!("{}", usage(Locale::English));
                process::exit(64);
            }
        };
        if let Err(error) = result {
            eprintln!("{error}");
            process::exit(1);
        }
        return;
    }
    if arguments.get(1).map(String::as_str) == Some("deploy") {
        let command = arguments.get(2).map(String::as_str);
        let result = match command {
            Some("plan") if arguments.len() <= 4 => {
                let directory = arguments
                    .get(3)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."));
                deployment_plan_contents(&directory).map(|plan| print!("{plan}"))
            }
            Some("inspect") if arguments.len() <= 4 => {
                let directory = arguments
                    .get(3)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."));
                inspect_deployment_plan(&directory)
            }
            _ => {
                eprintln!("{}", usage(Locale::English));
                process::exit(64);
            }
        };
        if let Err(error) = result {
            eprintln!("{error}");
            process::exit(1);
        }
        return;
    }
    if arguments.get(1).map(String::as_str) == Some("render") {
        let command = arguments.get(2).map(String::as_str);
        let result = match command {
            Some("plan") if arguments.len() <= 4 => {
                let directory = arguments
                    .get(3)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."));
                render_release_plan_contents(&directory).map(|plan| print!("{plan}"))
            }
            Some("inspect") if arguments.len() <= 4 => {
                let directory = arguments
                    .get(3)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."));
                inspect_render_release_plan(&directory)
            }
            Some("api-plan") if arguments.len() <= 4 => {
                let directory = arguments
                    .get(3)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."));
                render_api_plan_contents(&directory).map(|plan| print!("{plan}"))
            }
            Some("deploy") | Some("rollback") => {
                if !(5..=6).contains(&arguments.len())
                    || arguments.get(3).map(String::as_str) != Some("--confirm")
                {
                    eprintln!("{}", usage(Locale::English));
                    process::exit(64);
                }
                let directory = arguments
                    .get(5)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."));
                run_render_api_request(command.unwrap(), &directory, &arguments[4])
            }
            _ => {
                eprintln!("{}", usage(Locale::English));
                process::exit(64);
            }
        };
        if let Err(error) = result {
            eprintln!("{error}");
            process::exit(1);
        }
        return;
    }
    if arguments.get(1).map(String::as_str) == Some("gui") {
        let command = arguments.get(2).map(String::as_str);
        let result = match command {
            Some("inspect") if arguments.len() <= 4 => {
                let directory = arguments
                    .get(3)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."));
                run_gui_inspect(&directory)
            }
            Some("plan") if arguments.len() <= 4 => {
                let directory = arguments
                    .get(3)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."));
                run_gui_plan(&directory)
            }
            _ => {
                eprintln!("{}", usage(Locale::English));
                process::exit(64);
            }
        };
        if let Err(error) = result {
            eprintln!("{error}");
            process::exit(1);
        }
        return;
    }
    if arguments.get(1).map(String::as_str) == Some("android") {
        let command = arguments.get(2).map(String::as_str);
        let result = match command {
            Some("inspect") if arguments.len() <= 4 => {
                let directory = arguments
                    .get(3)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."));
                run_android_inspect(&directory)
            }
            Some("plan") if arguments.len() <= 4 => {
                let directory = arguments
                    .get(3)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."));
                run_android_build_plan(&directory)
            }
            _ => {
                eprintln!("{}", usage(Locale::English));
                process::exit(64);
            }
        };
        if let Err(error) = result {
            eprintln!("{error}");
            process::exit(1);
        }
        return;
    }
    if arguments.get(1).map(String::as_str) == Some("init") {
        let directory = arguments
            .get(2)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        if arguments.len() > 3 {
            eprintln!("{}", usage(Locale::English));
            process::exit(64);
        }
        match initialize_project(&directory) {
            Ok(manifest) => println!(
                "Created Padma project `{}`. Run: cd {} && padma .",
                manifest.name,
                directory.display()
            ),
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        }
        return;
    }
    if arguments.get(1).map(String::as_str) == Some("capabilities") {
        let directory = arguments
            .get(2)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        if arguments.len() > 3 {
            eprintln!("{}", usage(Locale::English));
            process::exit(64);
        }
        let (manifest, _) = match load_project_manifest(&directory) {
            Ok(project) => project,
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        };
        println!("Padma capabilities for `{}`:", manifest.name);
        if manifest.capabilities.is_empty() {
            println!("  (none; sensitive project actions are denied)");
        } else {
            for capability in manifest.capabilities {
                println!("  {capability}");
            }
        }
        return;
    }
    if arguments.len() == 2 && Path::new(&arguments[1]).is_dir() {
        let directory = Path::new(&arguments[1]);
        let (manifest, entry_path) = match load_project_manifest(directory) {
            Ok(project) => project,
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        };
        let source = match fs::read_to_string(&entry_path) {
            Ok(source) => project_source_with_locale(source, &manifest.locale),
            Err(error) => {
                eprintln!("P1032: cannot read `{}`: {error}", entry_path.display());
                process::exit(66);
            }
        };
        let (program, locale) = match compile(&source) {
            Ok(value) => value,
            Err(error) => {
                eprintln!(
                    "{}",
                    format_diagnostic(&entry_path.to_string_lossy(), &source, &error)
                );
                process::exit(1);
            }
        };
        let mut interpreter = Interpreter::with_project_capabilities(
            locale,
            entry_path.clone(),
            directory.to_path_buf(),
            manifest.capabilities,
        );
        if let Err(error) = interpreter.run(&program) {
            eprintln!(
                "{}",
                format_diagnostic(&entry_path.to_string_lossy(), &source, &error)
            );
            process::exit(1);
        }
        for line in interpreter.output {
            println!("{line}");
        }
        return;
    }
    let (command, path, json_diagnostics, format_check, json_lint) = match arguments.as_slice() {
        [_, path] if path.ends_with(".pd") => ("run", path.as_str(), false, false, false),
        [_, command, path] => (command.as_str(), path.as_str(), false, false, false),
        [_, command, flag, path] if command == "check" && flag == "--json" => {
            ("check", path.as_str(), true, false, false)
        }
        [_, command, path, flag] if command == "check" && flag == "--json" => {
            ("check", path.as_str(), true, false, false)
        }
        [_, command, flag, path] if command == "fmt" && flag == "--check" => {
            ("fmt", path.as_str(), false, true, false)
        }
        [_, command, path, flag] if command == "fmt" && flag == "--check" => {
            ("fmt", path.as_str(), false, true, false)
        }
        [_, command, flag, path] if command == "lint" && flag == "--json" => {
            ("lint", path.as_str(), false, false, true)
        }
        [_, command, path, flag] if command == "lint" && flag == "--json" => {
            ("lint", path.as_str(), false, false, true)
        }
        _ => {
            eprintln!("{}", usage(Locale::Bangla));
            process::exit(64);
        }
    };
    if path.is_empty() {
        eprintln!("{}", usage(Locale::Bangla));
        process::exit(64);
    }
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("Cannot read `{path}`: {error}");
            process::exit(66);
        }
    };
    if command == "check" {
        if json_diagnostics {
            let result = check_json(path, &source);
            let has_errors = result.contains("\"status\":\"error\"");
            println!("{result}");
            if has_errors {
                process::exit(1);
            }
            return;
        }
        match check_source(&source) {
            Ok(locale) => match locale {
                Locale::Bangla => println!("ঠিক আছে: `{path}`-এ কোনো syntax error পাওয়া যায়নি।"),
                Locale::English => println!("ok: no syntax errors found in `{path}`."),
            },
            Err(errors) => {
                for error in errors {
                    eprintln!("{}", format_diagnostic(path, &source, &error));
                }
                process::exit(1);
            }
        }
        return;
    }
    if command == "lint" {
        if let Err(errors) = syntax_check_source(&source) {
            for error in errors {
                eprintln!("{}", format_diagnostic(path, &source, &error));
            }
            process::exit(1);
        }
        let disabled_rules = match lint_disabled_rules_for_path(path) {
            Ok(rules) => rules,
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        };
        let (locale, warnings) = lint_source_with_disabled(&source, &disabled_rules);
        if json_lint {
            println!(
                "{}",
                lint_json_with_disabled(path, &source, &disabled_rules)
            );
        } else if warnings.is_empty() {
            match locale {
                Locale::Bangla => println!("ঠিক আছে: `{path}`-এ কোনো lint warning পাওয়া যায়নি।"),
                Locale::English => println!("ok: no lint warnings found in `{path}`."),
            }
        } else {
            for warning in &warnings {
                eprintln!("{}", format_lint_warning(path, warning));
            }
        }
        if !warnings.is_empty() {
            process::exit(1);
        }
        return;
    }
    if command == "fmt" {
        if let Err(errors) = syntax_check_source(&source) {
            for error in errors {
                eprintln!("{}", format_diagnostic(path, &source, &error));
            }
            process::exit(1);
        }
        let formatted = format_source(&source);
        if format_check {
            if formatted == source {
                println!("ok: `{path}` is already formatted.");
                return;
            }
            eprintln!("formatting required: `{path}`");
            process::exit(1);
        }
        if let Err(error) = fs::write(path, formatted) {
            eprintln!("Cannot write `{path}`: {error}");
            process::exit(73);
        }
        println!("formatted: `{path}`");
        return;
    }
    let compiled = compile(&source);
    let (program, locale) = match compiled {
        Ok(value) => value,
        Err(error) => {
            eprintln!("{}", format_diagnostic(path, &source, &error));
            process::exit(1);
        }
    };

    match command {
        "ast" => println!("{program:#?}"),
        "run" => {
            let mut interpreter = Interpreter::with_source_path(locale, PathBuf::from(path));
            if let Err(error) = interpreter.run(&program) {
                eprintln!("{}", format_diagnostic(path, &source, &error));
                process::exit(1);
            }
            for line in interpreter.output {
                println!("{line}");
            }
        }
        _ => {
            eprintln!("{}", usage(locale));
            process::exit(64);
        }
    }
}

fn brace_delta(line: &str) -> isize {
    let mut delta = 0;
    let mut in_string = false;
    let mut escaped = false;
    for character in line.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if in_string && character == '\\' {
            escaped = true;
            continue;
        }
        if character == '"' {
            in_string = !in_string;
            continue;
        }
        if !in_string {
            match character {
                '#' => break,
                '{' => delta += 1,
                '}' => delta -= 1,
                _ => {}
            }
        }
    }
    delta
}

fn run_repl_submission(
    interpreter: &mut Interpreter,
    source: &str,
) -> Result<Vec<String>, PadmaError> {
    let (program, locale) = compile(source)?;
    interpreter.locale = locale;
    interpreter.return_value = None;
    let output_start = interpreter.output.len();

    if let [Stmt::Expression { value }] = program.as_slice() {
        let value = interpreter.evaluate(value)?;
        return Ok(match value {
            Value::Null => Vec::new(),
            value => vec![value.to_string()],
        });
    }

    interpreter.run(&program)?;
    Ok(interpreter.output[output_start..].to_vec())
}

fn repl() {
    println!("Padma 0.1.0 (Bangla-English hybrid programming language)");
    println!("Interactive shell: help, copyright, credits, license; exit with exit() or বের হও.");
    println!("Use `{{` blocks across lines; Padma shows `...` until the block is complete.");
    let stdin = io::stdin();
    let mut interpreter = Interpreter::new(Locale::Bangla);
    let mut buffer = String::new();
    let mut open_braces = 0isize;
    loop {
        print!("{}", if buffer.is_empty() { "padma> " } else { "... " });
        let _ = io::stdout().flush();
        let mut line = String::new();
        if stdin.read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        let line = line.trim_end();
        if buffer.is_empty() && matches!(line, "exit" | "exit()" | "quit" | "quit()" | "বের হও")
        {
            break;
        }
        match (buffer.is_empty(), line) {
            (true, "help" | "help()" | "সাহায্য") => {
                println!("Padma interactive shell commands:");
                println!("  help / সাহায্য       show this help");
                println!("  copyright            show copyright information");
                println!("  credits              show project credits");
                println!("  license              show the MIT license notice");
                println!("  exit() / বের হও      leave the shell");
                println!("Examples: ১ + ১ | দেখাও ২ + ৩ | print \"hello\" | ধরি x = 10");
                continue;
            }
            (true, "copyright" | "copyright()") => {
                println!("Copyright (c) 2026 OfficialBiohub and Padma contributors.");
                continue;
            }
            (true, "credits" | "credits()") => {
                println!("Padma is an open-source Bangla-English language project by OfficialBiohub and its contributors.");
                continue;
            }
            (true, "license" | "license()") => {
                println!("Padma is released under the MIT License. See LICENSE in the repository.");
                continue;
            }
            _ => {}
        }
        if buffer.is_empty() && line.trim().is_empty() {
            continue;
        }
        open_braces += brace_delta(line);
        buffer.push_str(line);
        buffer.push('\n');
        if open_braces > 0 {
            continue;
        }
        let source = std::mem::take(&mut buffer);
        open_braces = 0;
        match run_repl_submission(&mut interpreter, &source) {
            Ok(output) => {
                for output in output {
                    println!("{output}");
                }
            }
            Err(error) => eprintln!("{}", error.message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn module_fixture_dir(label: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = env::temp_dir().join(format!("padma-{label}-{}-{nonce}", process::id()));
        fs::create_dir_all(&directory).unwrap();
        directory
    }

    fn run_bridge_project(
        root: &Path,
        capabilities: BTreeSet<String>,
        source: &str,
    ) -> Result<Vec<String>, PadmaError> {
        let (program, locale) = compile(source)?;
        let mut interpreter = Interpreter::with_project_capabilities(
            locale,
            root.join("main.pd"),
            root.to_path_buf(),
            capabilities,
        );
        interpreter.run(&program)?;
        Ok(interpreter.output)
    }

    #[test]
    fn exchanges_typed_json_with_python_and_javascript_bridges() {
        let root = module_fixture_dir("bridge-typed-json");
        fs::create_dir_all(root.join("bridges")).unwrap();
        fs::write(
            root.join("bridges/score.py"),
            "import json, sys\nrequest = json.load(sys.stdin)\njson.dump({'name': request['name'], 'total': sum(request['scores'])}, sys.stdout)\n",
        )
        .unwrap();
        fs::write(
            root.join("bridges/score.js"),
            "const request = JSON.parse(require('node:fs').readFileSync(0, 'utf8')); process.stdout.write(JSON.stringify({name: request.name, total: request.scores.reduce((a, b) => a + b, 0)}));\n",
        )
        .unwrap();
        let capabilities = BTreeSet::from(["process:python".into(), "process:node".into()]);
        let output = run_bridge_project(
            &root,
            capabilities,
            "let input = {\"name\": \"Rafi\", \"scores\": [2, 3, 4]}\nlet python_result = bridge.call(\"python\", \"bridges/score.py\", input)\nlet javascript_result = bridge.call(\"javascript\", \"bridges/score.js\", input)\nprint python_result[\"name\"]\nprint python_result[\"total\"]\nprint javascript_result[\"name\"]\nprint javascript_result[\"total\"]\n",
        )
        .unwrap();
        assert_eq!(output, vec!["Rafi", "9", "Rafi", "9"]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bridge_rejects_unsafe_requests_with_stable_localized_errors() {
        let root = module_fixture_dir("bridge-errors");
        fs::create_dir_all(root.join("bridges")).unwrap();
        fs::write(root.join("bridges/not_json.py"), "print('not json')\n").unwrap();
        fs::write(
            root.join("bridges/fail.py"),
            "raise RuntimeError('intentional test failure')\n",
        )
        .unwrap();

        let denied = run_bridge_project(
            &root,
            BTreeSet::new(),
            "# padma:locale=bn\nদেখাও bridge.call(\"python\", \"bridges/not_json.py\", {})\n",
        )
        .unwrap_err();
        assert_eq!(denied.code, "P1034");
        assert_eq!(denied.locale, Locale::Bangla);

        let capabilities = BTreeSet::from(["process:python".into()]);
        let unsafe_path = run_bridge_project(
            &root,
            capabilities.clone(),
            "print bridge.call(\"python\", \"../outside.py\", {})\n",
        )
        .unwrap_err();
        assert_eq!(unsafe_path.code, "P1036");

        let invalid_runtime = run_bridge_project(
            &root,
            capabilities.clone(),
            "print bridge.call(\"shell\", \"bridges/not_json.py\", {})\n",
        )
        .unwrap_err();
        assert_eq!(invalid_runtime.code, "P1035");

        let invalid_json = run_bridge_project(
            &root,
            capabilities.clone(),
            "print bridge.call(\"python\", \"bridges/not_json.py\", {})\n",
        )
        .unwrap_err();
        assert_eq!(invalid_json.code, "P1040");

        let failed_process = run_bridge_project(
            &root,
            capabilities,
            "print bridge.call(\"python\", \"bridges/fail.py\", {})\n",
        )
        .unwrap_err();
        assert_eq!(failed_process.code, "P1038");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn emits_machine_readable_json_check_diagnostics() {
        let source = "let = 3\nprint (\n";
        let output = check_json("broken.pd", source);
        let value: JsonValue = serde_json::from_str(&output).unwrap();
        assert_eq!(value["status"], "error");
        assert_eq!(value["path"], "broken.pd");
        let diagnostics = value["diagnostics"].as_array().unwrap();
        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0]["code"].as_str().unwrap().starts_with('P'));
        assert_eq!(diagnostics[0]["locale"], "en");
        assert!(diagnostics[0]["range"]["start"]["line"].is_number());
    }

    #[test]
    fn emits_json_check_success_without_diagnostics() {
        let output = check_json("valid.pd", "print \"ok\"\n");
        let value: JsonValue = serde_json::from_str(&output).unwrap();
        assert_eq!(value["status"], "ok");
        assert_eq!(value["diagnostics"], serde_json::json!([]));
    }

    #[test]
    fn check_reports_static_constant_division_by_zero_without_execution() {
        let errors = check_source("print 8 / 0\n").unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "P1011");

        let output = check_json("zero.pd", "print 8 / 0\n");
        let value: JsonValue = serde_json::from_str(&output).unwrap();
        assert_eq!(value["status"], "error");
        assert_eq!(value["diagnostics"][0]["code"], "P1011");
    }

    #[test]
    fn check_reports_provable_builtin_and_top_level_function_arity_errors() {
        let source = "function add(left, right) {\n    return left + right\n}\nprint add(1)\nprint text.replace(\"a\", \"b\")\n";
        let errors = check_source(source).unwrap_err();
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().all(|error| error.code == "P1009"));

        let source = "function add(left, right) {\n    return left + right\n}\nprint add(1, 2)\nprint text.replace(\"a\", \"a\", \"b\")\n";
        assert!(check_source(source).is_ok());
    }

    #[test]
    fn formatter_is_idempotent_and_ignores_braces_inside_strings() {
        let source =
            "function greeting() {\nprint \"{name}\"   \nif true {\nprint \"ok\"\n}\n}\n\n";
        let expected = "function greeting() {\n    print \"{name}\"\n    if true {\n        print \"ok\"\n    }\n}\n";
        let formatted = format_source(source);
        assert_eq!(formatted, expected);
        assert_eq!(format_source(&formatted), formatted);
    }

    #[test]
    fn lints_layout_and_mixed_keywords_without_reading_strings_or_comments() {
        let source = "\tlet name = \"Rafi\"   \nদেখাও name\n";
        let (_, warnings) = lint_source(source);
        let codes = warnings
            .iter()
            .map(|warning| warning.code)
            .collect::<Vec<_>>();
        assert!(codes.contains(&"L1001"));
        assert!(codes.contains(&"L1002"));
        assert!(codes.contains(&"L1003"));

        let (_, warnings) = lint_source("print \"ধরি let\"\n# দেখাও\n");
        assert!(!warnings.iter().any(|warning| warning.code == "L1003"));

        let json: JsonValue = serde_json::from_str(&lint_json_with_disabled(
            "mixed.pd",
            source,
            &BTreeSet::new(),
        ))
        .unwrap();
        assert_eq!(json["status"], "warning");
        assert_eq!(json["warnings"][0]["code"], "L1001");
    }

    fn run_file(path: &Path) -> Result<Vec<String>, PadmaError> {
        let source = fs::read_to_string(path).unwrap();
        let (program, locale) = compile(&source)?;
        let mut interpreter = Interpreter::with_source_path(locale, path.to_path_buf());
        interpreter.run(&program)?;
        Ok(interpreter.output)
    }

    fn run(source: &str) -> Result<Vec<String>, PadmaError> {
        let (program, locale) = compile(source)?;
        let mut interpreter = Interpreter::new(locale);
        interpreter.run(&program)?;
        Ok(interpreter.output)
    }

    #[test]
    fn initializes_and_runs_a_manifest_project() {
        let directory = module_fixture_dir("project-init");
        let project_directory = directory.join("bangla-project");
        let manifest = initialize_project(&project_directory).unwrap();
        assert_eq!(manifest.name, "bangla-project");
        assert!(project_directory.join("padma.toml").is_file());
        assert!(project_directory.join("padma.lock").is_file());

        let (loaded, entry) = load_project_manifest(&project_directory).unwrap();
        let source =
            project_source_with_locale(fs::read_to_string(&entry).unwrap(), &loaded.locale);
        let (program, locale) = compile(&source).unwrap();
        let mut interpreter = Interpreter::with_source_path(locale, entry);
        interpreter.run(&program).unwrap();
        fs::remove_dir_all(directory).unwrap();
        assert_eq!(interpreter.output, vec!["পদ্ম project শুরু হয়েছে"]);
    }

    #[test]
    fn rejects_unsafe_or_untrusted_manifest_configuration() {
        let dependencies = parse_project_manifest(
            "[padma]\nname = \"demo\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\n[dependencies]\nnetwork = \"1\"\n",
        )
        .unwrap_err();
        assert!(dependencies.starts_with("P1032"));

        let escaped_dependency = parse_project_manifest(
            "[padma]\nname = \"demo\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\n[dependencies]\nhelper = \"../outside\"\n",
        )
        .unwrap_err();
        assert!(escaped_dependency.starts_with("P1032"));

        let unsafe_entry = parse_project_manifest(
            "[padma]\nname = \"demo\"\nversion = \"0.1.0\"\nentry = \"../outside.pd\"\n",
        )
        .unwrap();
        assert!(safe_project_relative_path(&unsafe_entry.entry).is_err());

        let invalid_locale = parse_project_manifest(
            "[padma]\nname = \"demo\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\nlocale = \"mixed\"\n",
        )
        .unwrap_err();
        assert!(invalid_locale.starts_with("P1032"));
    }

    #[test]
    fn verifies_local_package_manifest_digest_and_canonical_lockfile() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let root = module_fixture_dir("package-lock");
        let package = root.join("packages/helper");
        fs::create_dir_all(&package).unwrap();
        fs::write(
            package.join("main.pd"),
            "export function greet() { return \"hello\" }\n",
        )
        .unwrap();
        let digest = package_digest(&package).unwrap();
        fs::write(
            package.join("padma-package.toml"),
            format!(
                "[package]\nname = \"helper\"\nversion = \"1.0.0\"\nentry = \"main.pd\"\nexports = [\"greet\"]\ndigest = \"{digest}\"\n\n[capabilities]\n"
            ),
        )
        .unwrap();
        fs::write(
            root.join("padma.toml"),
            "[padma]\nname = \"demo\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\n\n[dependencies]\nhelper = \"packages/helper\"\n\n[capabilities]\n",
        )
        .unwrap();
        fs::write(root.join("main.pd"), "print \"ok\"\n").unwrap();

        write_package_lock(&root).unwrap();
        verify_package_lock(&root).unwrap();
        let lock = fs::read_to_string(root.join("padma.lock")).unwrap();
        assert!(lock.contains("\"lockfileVersion\": 1"));
        assert!(lock.contains(&digest));

        fs::write(
            package.join("main.pd"),
            "export function greet() { return \"changed\" }\n",
        )
        .unwrap();
        let error = verify_package_lock(&root).unwrap_err();
        assert!(error.starts_with("P1044"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn parses_and_validates_manifest_capabilities() {
        let manifest = parse_project_manifest(
            "[padma]\nname = \"demo\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\n\n[capabilities]\ndatabase = [\"sqlite\"]\nfilesystem = [\"read\", \"write\"]\nnetwork = [\"http\"]\nprocess = [\"git\", \"yt-dlp\"]\nmedia = [\"download\"]\n",
        )
        .unwrap();
        assert!(manifest.capabilities.contains("filesystem:read"));
        assert!(manifest.capabilities.contains("process:git"));
        assert!(manifest.capabilities.contains("database:sqlite"));
        assert_eq!(manifest.capabilities.len(), 7);

        let lint_manifest = parse_project_manifest(
            "[padma]\nname = \"lint-demo\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\n\n[lint]\ndisable = [\"L1003\"]\n",
        )
        .unwrap();
        assert!(lint_manifest.lint_disabled.contains("L1003"));
        let (_, warnings) = lint_source_with_disabled(
            "let name = \"Rafi\"\nদেখাও name\n",
            &lint_manifest.lint_disabled,
        );
        assert!(!warnings.iter().any(|warning| warning.code == "L1003"));

        let unknown_lint_rule = parse_project_manifest(
            "[padma]\nname = \"lint-demo\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\n\n[lint]\ndisable = [\"L9999\"]\n",
        )
        .unwrap_err();
        assert!(unknown_lint_rule.contains("unsupported lint rule"));

        let duplicate = parse_project_manifest(
            "[padma]\nname = \"demo\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\n[capabilities]\nprocess = [\"git\", \"git\"]\n",
        )
        .unwrap_err();
        assert!(duplicate.contains("duplicate"));

        let unknown = parse_project_manifest(
            "[padma]\nname = \"demo\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\n[capabilities]\nnetwork = [\"all\"]\n",
        )
        .unwrap_err();
        assert!(unknown.contains("unsupported"));

        let approved = parse_project_manifest(
            "[padma]\nname = \"demo\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\n[capabilities]\nnetwork = [\"ai\"]\nserver = [\"local\"]\n",
        )
        .unwrap();
        assert!(approved.capabilities.contains("network:ai"));
        assert!(approved.capabilities.contains("server:local"));
    }

    #[test]
    fn local_server_rejects_projects_without_explicit_server_capability() {
        let root = module_fixture_dir("local-server-denied");
        fs::write(
            root.join("padma.toml"),
            "[padma]\nname = \"denied-server\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\nlocale = \"en\"\n\n[capabilities]\nfilesystem = []\nnetwork = []\nprocess = []\nmedia = []\n",
        )
        .unwrap();
        let error = serve_local_project(&root).unwrap_err();
        fs::remove_dir_all(root).unwrap();
        assert!(error.starts_with("P1034"));
        assert!(error.contains("server:local"));
        assert!(error.contains("[capabilities]"));
    }

    #[test]
    fn deployment_manifest_requires_bounded_target_origin_and_environment_names() {
        let manifest = parse_deployment_manifest(
            "[deployment]\nversion = \"1\"\nentry = \"main.pd\"\ntarget = \"loopback\"\nbase_url = \"https://padma.example\"\nrollback = \"deploy/rollback.json\"\n\n[environment]\nnames = [\"PADMA_TOKEN\", \"API_KEY\"]\n",
        )
        .unwrap();
        assert_eq!(manifest.target, "loopback");
        assert!(manifest.environment_names.contains("PADMA_TOKEN"));

        let unsafe_origin = parse_deployment_manifest(
            "[deployment]\nversion = \"1\"\nentry = \"main.pd\"\ntarget = \"static\"\nbase_url = \"http://localhost\"\nrollback = \"deploy/rollback.json\"\n",
        )
        .unwrap_err();
        assert!(unsafe_origin.starts_with("P1046"));
        assert!(unsafe_origin.contains("public HTTPS"));

        let unsafe_environment = parse_deployment_manifest(
            "[deployment]\nversion = \"1\"\nentry = \"main.pd\"\ntarget = \"static\"\nbase_url = \"https://padma.example\"\nrollback = \"deploy/rollback.json\"\n\n[environment]\nnames = [\"token=value\"]\n",
        )
        .unwrap_err();
        assert!(unsafe_environment.starts_with("P1046"));
        assert!(unsafe_environment.contains("unsafe environment variable name"));
    }

    #[test]
    fn deployment_plan_is_project_scoped_dry_run_and_never_contains_secret_values() {
        let root = module_fixture_dir("deployment-plan");
        fs::create_dir(root.join("deploy")).unwrap();
        fs::create_dir(root.join("src")).unwrap();
        fs::write(
            root.join("padma.toml"),
            "[padma]\nname = \"deploy-demo\"\nversion = \"0.1.0\"\nentry = \"src/main.pd\"\nlocale = \"en\"\n",
        )
        .unwrap();
        fs::write(root.join("src/main.pd"), "print \"safe\"\n").unwrap();
        fs::write(
            root.join("padma-deploy.toml"),
            "[deployment]\nversion = \"1\"\nentry = \"src/main.pd\"\ntarget = \"static\"\nbase_url = \"https://padma.example\"\nrollback = \"deploy/rollback.json\"\n\n[environment]\nnames = [\"PADMA_DEPLOY_TOKEN\"]\n",
        )
        .unwrap();
        let plan = deployment_plan_contents(&root).unwrap();
        fs::remove_dir_all(root).unwrap();
        assert!(plan.contains("\"mode\": \"dry-run-only\""));
        assert!(plan.contains("\"network\": \"disabled\""));
        assert!(plan.contains("\"remoteMutation\": \"disabled\""));
        assert!(plan.contains("PADMA_DEPLOY_TOKEN"));
        assert!(!plan.contains("token=value"));
        assert!(plan.contains("sha256:"));
    }

    #[test]
    fn render_release_manifest_requires_an_immutable_git_link_and_safe_identifiers() {
        let commit = "0123456789abcdef0123456789abcdef01234567";
        let manifest = parse_render_release_manifest(&format!(
            "[render]\nversion = \"1\"\nmode = \"git-linked\"\nservice = \"srv-abc123\"\nrepository = \"OfficialBiohub/padma-lang\"\nbranch = \"main\"\ncommit = \"{commit}\"\nrollback_deploy = \"dep-abc123\"\n"
        ))
        .unwrap();
        assert_eq!(manifest.repository, "OfficialBiohub/padma-lang");
        assert_eq!(manifest.commit, commit);
        assert_eq!(manifest.rollback_deploy.as_deref(), Some("dep-abc123"));

        let unsafe_branch = parse_render_release_manifest(
            "[render]\nversion = \"1\"\nmode = \"git-linked\"\nservice = \"srv-abc123\"\nrepository = \"OfficialBiohub/padma-lang\"\nbranch = \"../production\"\ncommit = \"0123456789abcdef0123456789abcdef01234567\"\n",
        )
        .unwrap_err();
        assert!(unsafe_branch.starts_with("P1048"));

        let mutable_commit = parse_render_release_manifest(
            "[render]\nversion = \"1\"\nmode = \"git-linked\"\nservice = \"srv-abc123\"\nrepository = \"OfficialBiohub/padma-lang\"\nbranch = \"main\"\ncommit = \"main\"\n",
        )
        .unwrap_err();
        assert!(mutable_commit.starts_with("P1048"));
    }

    #[test]
    fn render_release_plan_is_capability_gated_and_dashboard_confirmed_only() {
        let root = module_fixture_dir("render-release-plan");
        fs::create_dir_all(root.join("deploy")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("padma.toml"),
            "[padma]\nname = \"render-demo\"\nversion = \"0.1.0\"\nentry = \"src/main.pd\"\nlocale = \"en\"\n\n[capabilities]\ndeployment = [\"render\"]\n",
        )
        .unwrap();
        fs::write(root.join("src/main.pd"), "print \"safe\"\n").unwrap();
        fs::write(
            root.join("padma-deploy.toml"),
            "[deployment]\nversion = \"1\"\nentry = \"src/main.pd\"\ntarget = \"static\"\nbase_url = \"https://padma.example\"\nrollback = \"deploy/rollback.json\"\n\n[environment]\nnames = [\"RENDER_API_TOKEN\"]\n",
        )
        .unwrap();
        fs::write(
            root.join("padma-render.toml"),
            "[render]\nversion = \"1\"\nmode = \"git-linked\"\nservice = \"srv-abc123\"\nrepository = \"OfficialBiohub/padma-lang\"\nbranch = \"main\"\ncommit = \"0123456789abcdef0123456789abcdef01234567\"\nrollback_deploy = \"dep-abc123\"\n",
        )
        .unwrap();
        let plan = render_release_plan_contents(&root).unwrap();
        fs::remove_dir_all(root).unwrap();
        assert!(plan.contains("\"mode\": \"git-linked-release-plan\""));
        assert!(plan.contains("\"method\": \"render-dashboard\""));
        assert!(plan.contains("\"providerApi\": \"disabled\""));
        assert!(plan.contains("\"remoteMutation\": \"disabled\""));
        assert!(plan.contains("sha256:"));
    }

    #[test]
    fn render_api_plan_binds_immutable_commit_without_reading_or_exposing_token_value() {
        let root = module_fixture_dir("render-api-plan");
        fs::create_dir_all(root.join("deploy")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        let commit = "0123456789abcdef0123456789abcdef01234567";
        fs::write(
            root.join("padma.toml"),
            "[padma]\nname = \"render-api-demo\"\nversion = \"0.1.0\"\nentry = \"src/main.pd\"\nlocale = \"en\"\n\n[capabilities]\ndeployment = [\"render\"]\n",
        )
        .unwrap();
        fs::write(root.join("src/main.pd"), "print \"safe\"\n").unwrap();
        fs::write(
            root.join("padma-deploy.toml"),
            "[deployment]\nversion = \"1\"\nentry = \"src/main.pd\"\ntarget = \"static\"\nbase_url = \"https://padma.example\"\nrollback = \"deploy/rollback.json\"\n\n[environment]\nnames = [\"RENDER_API_TOKEN\"]\n",
        )
        .unwrap();
        fs::write(
            root.join("padma-render.toml"),
            format!("[render]\nversion = \"1\"\nmode = \"git-linked\"\nservice = \"srv-abc123\"\nrepository = \"OfficialBiohub/padma-lang\"\nbranch = \"main\"\ncommit = \"{commit}\"\nrollback_deploy = \"dep-abc123\"\n"),
        )
        .unwrap();
        fs::write(
            root.join("padma-render-api.toml"),
            format!("[render_api]\nversion = \"1\"\nservice = \"srv-abc123\"\ntoken_env = \"RENDER_API_TOKEN\"\ncommit = \"{commit}\"\nclear_cache = \"do_not_clear\"\nrollback_deploy = \"dep-abc123\"\n"),
        )
        .unwrap();

        let plan = render_api_plan_contents(&root).unwrap();
        let confirmation = serde_json::from_str::<JsonValue>(&plan).unwrap()["deploy"]
            ["confirmationToken"]
            .as_str()
            .unwrap()
            .to_string();
        let confirmation_error =
            run_render_api_request("deploy", &root, "wrong-confirmation").unwrap_err();
        fs::remove_dir_all(root).unwrap();
        assert!(plan.contains("\"commitId\": \"0123456789abcdef0123456789abcdef01234567\""));
        assert!(plan.contains("\"value\": \"not-read-in-planning-mode\""));
        assert!(!plan.contains("RENDER_API_TOKEN_VALUE"));
        assert!(confirmation.starts_with("render-"));
        assert!(confirmation_error.contains("confirmation token does not match"));
    }

    #[test]
    fn render_api_manifest_rejects_unsafe_credentials_mutable_commit_and_unknown_fields() {
        let unsafe_token = parse_render_api_manifest(
            "[render_api]\nversion = \"1\"\nservice = \"srv-abc123\"\ntoken_env = \"token;curl\"\ncommit = \"0123456789abcdef0123456789abcdef01234567\"\nclear_cache = \"do_not_clear\"\nrollback_deploy = \"dep-abc123\"\n",
        )
        .unwrap_err();
        assert!(unsafe_token.starts_with("P1048"));

        let mutable_commit = parse_render_api_manifest(
            "[render_api]\nversion = \"1\"\nservice = \"srv-abc123\"\ntoken_env = \"RENDER_API_TOKEN\"\ncommit = \"main\"\nclear_cache = \"do_not_clear\"\nrollback_deploy = \"dep-abc123\"\n",
        )
        .unwrap_err();
        assert!(mutable_commit.starts_with("P1048"));

        let unknown_field = parse_render_api_manifest(
            "[render_api]\nversion = \"1\"\nservice = \"srv-abc123\"\ntoken_env = \"RENDER_API_TOKEN\"\ncommit = \"0123456789abcdef0123456789abcdef01234567\"\nclear_cache = \"do_not_clear\"\nrollback_deploy = \"dep-abc123\"\ncommand = \"curl\"\n",
        )
        .unwrap_err();
        assert!(unknown_field.starts_with("P1048"));
    }

    #[test]
    fn render_release_plan_rejects_projects_without_render_capability() {
        let root = module_fixture_dir("render-capability-denied");
        fs::create_dir_all(root.join("deploy")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("padma.toml"),
            "[padma]\nname = \"render-denied\"\nversion = \"0.1.0\"\nentry = \"src/main.pd\"\nlocale = \"en\"\n",
        )
        .unwrap();
        fs::write(root.join("src/main.pd"), "print \"safe\"\n").unwrap();
        fs::write(
            root.join("padma-deploy.toml"),
            "[deployment]\nversion = \"1\"\nentry = \"src/main.pd\"\ntarget = \"static\"\nbase_url = \"https://padma.example\"\nrollback = \"deploy/rollback.json\"\n",
        )
        .unwrap();
        fs::write(
            root.join("padma-render.toml"),
            "[render]\nversion = \"1\"\nmode = \"git-linked\"\nservice = \"srv-abc123\"\nrepository = \"OfficialBiohub/padma-lang\"\nbranch = \"main\"\ncommit = \"0123456789abcdef0123456789abcdef01234567\"\n",
        )
        .unwrap();
        let error = render_release_plan_contents(&root).unwrap_err();
        fs::remove_dir_all(root).unwrap();
        assert!(error.starts_with("P1034"));
        assert!(error.contains("deployment:render"));
    }

    #[test]
    fn gui_manifest_accepts_a_scoped_static_renderer_and_emits_read_only_plan() {
        let root = module_fixture_dir("gui-valid");
        fs::create_dir_all(root.join("ui/assets")).unwrap();
        fs::write(
            root.join("padma.toml"),
            "[padma]\nname = \"gui-demo\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\nlocale = \"en\"\n\n[capabilities]\ngui = [\"local\"]\n",
        )
        .unwrap();
        fs::write(root.join("main.pd"), "print \"safe\"\n").unwrap();
        fs::write(
            root.join("padma-gui.toml"),
            "[gui]\nversion = 1\nbackend = \"html-static\"\nentry = \"ui/index.html\"\nassets = \"ui/assets\"\ntitle = \"Padma GUI\"\n",
        )
        .unwrap();
        fs::write(
            root.join("ui/index.html"),
            "<!doctype html><title>Padma</title>\n",
        )
        .unwrap();
        fs::write(
            root.join("ui/assets/logo.svg"),
            "<svg xmlns=\"http://www.w3.org/2000/svg\"/>\n",
        )
        .unwrap();

        let manifest =
            parse_gui_manifest(&fs::read_to_string(root.join("padma-gui.toml")).unwrap()).unwrap();
        assert_eq!(manifest.backend, "html-static");
        let inspect = gui_inspect_contents(&root).unwrap();
        let plan = gui_plan_contents(&root).unwrap();
        fs::remove_dir_all(root).unwrap();
        assert!(inspect.contains("read-only"));
        assert!(plan.contains("\"rendererLaunch\": \"disabled\""));
        assert!(plan.contains("\"network\": \"disabled\""));
        assert!(plan.contains("sha256:"));
    }

    #[test]
    fn android_build_plan_binds_gui_integrity_without_building_or_reading_signing_key() {
        let root = module_fixture_dir("android-build-plan");
        fs::create_dir_all(root.join("ui/assets")).unwrap();
        fs::write(
            root.join("padma.toml"),
            "[padma]\nname = \"android-demo\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\nlocale = \"en\"\n\n[capabilities]\ngui = [\"local\"]\nandroid = [\"plan\"]\n",
        )
        .unwrap();
        fs::write(root.join("main.pd"), "print \"safe\"\n").unwrap();
        fs::write(
            root.join("padma-gui.toml"),
            "[gui]\nversion = 1\nbackend = \"html-static\"\nentry = \"ui/index.html\"\nassets = \"ui/assets\"\ntitle = \"Padma Android\"\n",
        )
        .unwrap();
        fs::write(
            root.join("ui/index.html"),
            "<!doctype html><title>Padma</title>\n",
        )
        .unwrap();
        fs::write(root.join("ui/assets/logo.svg"), "<svg/>\n").unwrap();
        fs::write(
            root.join("padma-android.toml"),
            "[android]\nversion = \"1\"\napplication_id = \"org.officialbiohub.padma\"\nmin_sdk = 26\ntarget_sdk = 35\nartifact = \"build/padma-release.apk\"\nsigning_key_env = \"PADMA_ANDROID_SIGNING_KEY\"\nsigning_cert_sha256 = \"sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\"\n\n[permissions]\nnames = [\"android.permission.INTERNET\", \"android.permission.POST_NOTIFICATIONS\"]\n",
        )
        .unwrap();

        let inspect = android_inspect_contents(&root).unwrap();
        let plan = android_build_plan_contents(&root).unwrap();
        fs::remove_dir_all(root).unwrap();
        assert!(inspect.contains("org.officialbiohub.padma"));
        assert!(plan.contains("\"apkBuild\": \"disabled\""));
        assert!(plan.contains("\"keyRead\": \"disabled\""));
        assert!(plan.contains("\"automaticGrant\": \"disabled\""));
        assert!(plan.contains("\"install\": \"disabled\""));
        assert!(plan.contains("\"android.permission.POST_NOTIFICATIONS\""));
        assert!(plan.contains("sha256:"));
    }

    #[test]
    fn android_build_manifest_rejects_unsafe_permissions_commands_and_artifact_paths() {
        let unsafe_permission = parse_android_build_manifest(
            "[android]\nversion = \"1\"\napplication_id = \"org.padma.demo\"\nmin_sdk = 26\ntarget_sdk = 35\nartifact = \"build/demo.apk\"\nsigning_key_env = \"PADMA_SIGNING_KEY\"\nsigning_cert_sha256 = \"sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\"\n\n[permissions]\nnames = [\"android.permission.MANAGE_EXTERNAL_STORAGE\"]\n",
        )
        .unwrap_err();
        assert!(unsafe_permission.starts_with("P1049"));

        let unsafe_artifact = parse_android_build_manifest(
            "[android]\nversion = \"1\"\napplication_id = \"org.padma.demo\"\nmin_sdk = 26\ntarget_sdk = 35\nartifact = \"../demo.apk\"\nsigning_key_env = \"PADMA_SIGNING_KEY\"\nsigning_cert_sha256 = \"sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\"\n",
        )
        .unwrap_err();
        assert!(unsafe_artifact.starts_with("P1049"));

        let native_command = parse_android_build_manifest(
            "[android]\nversion = \"1\"\napplication_id = \"org.padma.demo\"\nmin_sdk = 26\ntarget_sdk = 35\nartifact = \"build/demo.apk\"\nsigning_key_env = \"PADMA_SIGNING_KEY\"\nsigning_cert_sha256 = \"sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\"\nadb_command = \"install\"\n",
        )
        .unwrap_err();
        assert!(native_command.starts_with("P1049"));
    }

    #[test]
    fn gui_manifest_rejects_unknown_backend_and_unsafe_paths() {
        let invalid_backend = parse_gui_manifest(
            "[gui]\nversion = 1\nbackend = \"webview\"\nentry = \"ui/index.html\"\nassets = \"ui/assets\"\ntitle = \"Padma\"\n",
        )
        .unwrap_err();
        assert!(invalid_backend.starts_with("P1047"));
        assert!(invalid_backend.contains("html-static"));

        let escaped_entry = parse_gui_manifest(
            "[gui]\nversion = 1\nbackend = \"html-static\"\nentry = \"../outside.html\"\nassets = \"ui/assets\"\ntitle = \"Padma\"\n",
        )
        .unwrap_err();
        assert!(escaped_entry.starts_with("P1047"));

        let external_assets = parse_gui_manifest(
            "[gui]\nversion = 1\nbackend = \"html-static\"\nentry = \"ui/index.html\"\nassets = \"https://example.test/assets\"\ntitle = \"Padma\"\n",
        )
        .unwrap_err();
        assert!(external_assets.starts_with("P1047"));
    }

    #[test]
    fn gui_manifest_rejects_missing_or_non_regular_entry_files() {
        let root = module_fixture_dir("gui-missing-entry");
        fs::create_dir_all(root.join("ui/assets")).unwrap();
        fs::write(
            root.join("padma.toml"),
            "[padma]\nname = \"gui-demo\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\nlocale = \"bn\"\n\n[capabilities]\ngui = [\"local\"]\n",
        )
        .unwrap();
        fs::write(root.join("main.pd"), "দেখাও \"নিরাপদ\"\n").unwrap();
        fs::write(
            root.join("padma-gui.toml"),
            "[gui]\nversion = 1\nbackend = \"html-static\"\nentry = \"ui/missing.html\"\nassets = \"ui/assets\"\ntitle = \"Padma\"\n",
        )
        .unwrap();

        let error = gui_inspect_contents(&root).unwrap_err();
        fs::remove_dir_all(root).unwrap();
        assert!(error.starts_with("P1047"));
        assert!(error.contains("GUI entry"));
    }

    #[test]
    fn database_rejects_missing_project_grant_and_unsafe_paths() {
        let source = "print db.get(\"data/app.sqlite\", \"settings\", \"theme\")\n";
        let (program, locale) = compile(source).unwrap();
        let mut standalone = Interpreter::new(locale);
        let denied = standalone.run(&program).unwrap_err();
        assert_eq!(denied.code, "P1034");
        assert!(denied.message.contains("database:sqlite"));

        let root = module_fixture_dir("sqlite-safe-path");
        fs::create_dir(root.join("data")).unwrap();
        let capabilities = BTreeSet::from(["database:sqlite".into()]);
        let (unsafe_program, unsafe_locale) =
            compile("print db.get(\"../outside.sqlite\", \"settings\", \"theme\")\n").unwrap();
        let mut interpreter = Interpreter::with_project_capabilities(
            unsafe_locale,
            root.join("main.pd"),
            root.clone(),
            capabilities,
        );
        let unsafe_error = interpreter.run(&unsafe_program).unwrap_err();
        fs::remove_dir_all(root).unwrap();
        assert_eq!(unsafe_error.code, "P1014");
    }

    #[test]
    fn sqlite_script_binds_values_without_interpolating_them_into_sql() {
        let malicious = "theme'); DROP TABLE padma_records; --";
        let binding = sqlite_hex_parameter(":key", malicious.as_bytes());
        let script = sqlite_script(
            &[binding],
            "SELECT value_json FROM padma_records WHERE record_key = CAST(:key AS TEXT);",
            true,
        );
        assert!(script.contains(".parameter set :key x'"));
        assert!(script.contains(".mode json"));
        assert!(!script.contains(malicious));
        assert!(!script.contains("DROP TABLE"));
        assert_eq!(static_builtin_arity("db.put"), Some((4, 4)));
        assert_eq!(static_builtin_arity("db.list"), Some((3, 3)));
    }

    #[test]
    fn sqlite_persistence_round_trip_when_cli_is_available() {
        if process::Command::new("sqlite3")
            .arg("--version")
            .output()
            .is_err()
        {
            return;
        }
        let root = module_fixture_dir("sqlite-round-trip");
        fs::create_dir(root.join("data")).unwrap();
        let source = "let saved = db.put(\"data/app.sqlite\", \"profile\", \"name\", {\"name\": \"Padma\", \"level\": 6})\nprint saved\nprint db.get(\"data/app.sqlite\", \"profile\", \"name\")\nprint db.list(\"data/app.sqlite\", \"profile\", 10)\nprint db.delete(\"data/app.sqlite\", \"profile\", \"name\")\nprint db.get(\"data/app.sqlite\", \"profile\", \"name\")\n";
        let (program, locale) = compile(source).unwrap();
        let mut interpreter = Interpreter::with_project_capabilities(
            locale,
            root.join("main.pd"),
            root.clone(),
            BTreeSet::from(["database:sqlite".into()]),
        );
        interpreter.run(&program).unwrap();
        fs::remove_dir_all(root).unwrap();
        assert_eq!(
            interpreter.output,
            vec![
                "true",
                "{\"level\": 6, \"name\": Padma}",
                "[{\"key\": name, \"value\": {\"level\": 6, \"name\": Padma}}]",
                "true",
                "none",
            ]
        );
    }

    #[test]
    fn sqlite_reports_fixed_schema_version_and_applies_bounded_batch() {
        if process::Command::new("sqlite3")
            .arg("--version")
            .output()
            .is_err()
        {
            return;
        }
        let root = module_fixture_dir("sqlite-batch");
        fs::create_dir(root.join("data")).unwrap();
        let source = "print db.version(\"data/app.sqlite\")\nprint db.apply(\"data/app.sqlite\", [{\"op\": \"put\", \"namespace\": \"tasks\", \"key\": \"one\", \"value\": {\"done\": false}}, {\"op\": \"put\", \"namespace\": \"tasks\", \"key\": \"two\", \"value\": {\"done\": true}}, {\"op\": \"delete\", \"namespace\": \"tasks\", \"key\": \"one\"}])\nprint db.list(\"data/app.sqlite\", \"tasks\", 10)\n";
        let (program, locale) = compile(source).unwrap();
        let mut interpreter = Interpreter::with_project_capabilities(
            locale,
            root.join("main.pd"),
            root.clone(),
            BTreeSet::from(["database:sqlite".into()]),
        );
        interpreter.run(&program).unwrap();
        fs::remove_dir_all(root).unwrap();
        assert_eq!(
            interpreter.output,
            vec!["1", "true", "[{\"key\": two, \"value\": {\"done\": true}}]",]
        );

        let (invalid_program, invalid_locale) = compile(
            "db.apply(\"data/app.sqlite\", [{\"op\": \"drop\", \"namespace\": \"tasks\", \"key\": \"one\"}])\n",
        )
        .unwrap();
        let mut invalid = Interpreter::new(invalid_locale);
        let error = invalid.run(&invalid_program).unwrap_err();
        assert_eq!(error.code, "P1010");
        assert_eq!(static_builtin_arity("db.version"), Some((1, 1)));
        assert_eq!(static_builtin_arity("db.apply"), Some((2, 2)));
    }

    #[test]
    fn project_mode_denies_undeclared_sensitive_capabilities() {
        let capabilities = BTreeSet::new();
        let (program, locale) = compile("print http.get(\"https://example.com\")\n").unwrap();
        let mut interpreter = Interpreter::with_project_capabilities(
            locale,
            PathBuf::from("project/main.pd"),
            PathBuf::from("project"),
            capabilities.clone(),
        );
        let error = interpreter.run(&program).unwrap_err();
        assert_eq!(error.code, "P1034");
        assert!(error.message.contains("network:http"));

        let (program, locale) = compile("দেখাও process.run(\"git\", \"status\")\n").unwrap();
        let mut interpreter = Interpreter::with_project_capabilities(
            locale,
            PathBuf::from("project/main.pd"),
            PathBuf::from("project"),
            capabilities,
        );
        let error = interpreter.run(&program).unwrap_err();
        assert_eq!(error.code, "P1034");
        assert!(error.message.contains("capability"));
    }

    #[test]
    fn project_mode_scopes_declared_file_writes_to_its_root() {
        let root = env::temp_dir().join(format!("padma-capability-root-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let capabilities = BTreeSet::from(["filesystem:write".to_string()]);
        let (program, locale) = compile("file.write(\"inside.txt\", \"safe\")\n").unwrap();
        let mut interpreter = Interpreter::with_project_capabilities(
            locale,
            root.join("src/main.pd"),
            root.clone(),
            capabilities.clone(),
        );
        interpreter.run(&program).unwrap();
        assert_eq!(fs::read_to_string(root.join("inside.txt")).unwrap(), "safe");

        let (program, locale) = compile("file.write(\"@downloads/out.txt\", \"no\")\n").unwrap();
        let mut interpreter = Interpreter::with_project_capabilities(
            locale,
            root.join("src/main.pd"),
            root.clone(),
            capabilities,
        );
        let error = interpreter.run(&program).unwrap_err();
        assert_eq!(error.code, "P1014");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn runs_bangla_program_with_bangla_digits_and_interpolation() {
        let output = run(
            "ধরি নাম = \"রাফি\"\nধরি নম্বর = ৭০ + ২৩\nযদি নম্বর >= 90 {\n  দেখাও \"{নাম}: {নম্বর}\"\n}\n",
        )
        .unwrap();
        assert_eq!(output, vec!["রাফি: 93"]);
    }

    #[test]
    fn runs_english_program() {
        let output = run("let score = 15 * 2\nif score == 30 {\n  print \"passed\"\n}\n").unwrap();
        assert_eq!(output, vec!["passed"]);
    }

    #[test]
    fn supports_mixed_keyword_style() {
        let output = run("ধরি price = 250\nif price > 200 {\n  দেখাও \"discount\"\n}\n").unwrap();
        assert_eq!(output, vec!["discount"]);
    }

    #[test]
    fn reports_missing_bangla_variable_in_bangla() {
        let error = run("ধরি নাম = \"রাফি\"\nদেখাও বয়স\n").unwrap_err();
        assert_eq!(error.code, "P1007");
        assert!(error.message.contains("কোনো variable পাওয়া যায়নি"));
    }

    #[test]
    fn reports_missing_english_variable_in_english() {
        let error = run("let name = \"Rafi\"\nprint age\n").unwrap_err();
        assert_eq!(error.code, "P1007");
        assert!(error.message.contains("Cannot find variable"));
    }

    #[test]
    fn prevents_division_by_zero() {
        let error = run("let total = 10 / 0\n").unwrap_err();
        assert_eq!(error.code, "P1011");
    }

    #[test]
    fn parses_else_block() {
        let output = run(
            "let score = 40\nif score >= 50 {\n print \"pass\"\n} else {\n print \"retry\"\n}\n",
        )
        .unwrap();
        assert_eq!(output, vec!["retry"]);
    }

    #[test]
    fn updates_variables_and_runs_bangla_while_loop() {
        let output = run("ধরি i = ০\nযতক্ষণ i < ৩ {\n দেখাও i\n i = i + ১\n}\n").unwrap();
        assert_eq!(output, vec!["0", "1", "2"]);
    }

    #[test]
    fn keeps_bangla_block_bindings_lexical_without_breaking_outer_assignment() {
        let output = run(
            "ধরি নাম = \"বাইরে\"\nযদি সত্য {\n  ধরি নাম = \"ভেতরে\"\n  দেখাও নাম\n}\nদেখাও নাম\nযদি সত্য {\n  নাম = \"পরিবর্তিত\"\n}\nদেখাও নাম\n",
        )
        .unwrap();
        assert_eq!(output, vec!["ভেতরে", "বাইরে", "পরিবর্তিত"]);
    }

    #[test]
    fn reports_undefined_assignment_target() {
        let error = run("count = 1\n").unwrap_err();
        assert_eq!(error.code, "P1007");
    }

    #[test]
    fn stops_non_terminating_loop_with_safety_error() {
        let error = run("let i = 0\nwhile true {\n i = i + 1\n}\n").unwrap_err();
        assert_eq!(error.code, "P1012");
    }

    #[test]
    fn calls_function_with_parameters_and_return_value() {
        let output =
            run("function add(a, b) {\n return a + b\n}\nlet result = add(2, 3)\nprint result\n")
                .unwrap();
        assert_eq!(output, vec!["5"]);
    }

    #[test]
    fn evaluates_bengali_list_literal() {
        let output = run("ধরি সংখ্যা = [১, ২, ৩]\nদেখাও সংখ্যা\n").unwrap();
        assert_eq!(output, vec!["[1, 2, 3]"]);
    }

    #[test]
    fn indexes_and_mutates_lists_safely() {
        let output = run(
            "let items = [10, 20]\nprint items[1]\nitems.set(0, 9)\nitems.push(30)\nprint items.len()\nprint items.contains(20)\nprint items.remove(1)\nprint items[1]\n",
        )
        .unwrap();
        assert_eq!(output, vec!["20", "3", "true", "20", "30"]);
    }

    #[test]
    fn slices_lists_with_optional_zero_based_bounds() {
        let output = run(
            "let values = [10, 20, 30, 40]\nprint values[1:3]\nprint values[:2]\nprint values[2:]\nদেখাও values[1:1]\n",
        )
        .unwrap();
        assert_eq!(output, vec!["[20, 30]", "[10, 20]", "[30, 40]", "[]"]);
    }

    #[test]
    fn rejects_invalid_list_slice_bounds() {
        let reversed = run("let values = [1, 2]\nprint values[2:1]\n").unwrap_err();
        assert_eq!(reversed.code, "P1027");
        let too_large = run("ধরি মান = [১]\nদেখাও মান[:২]\n").unwrap_err();
        assert_eq!(too_large.code, "P1027");
        assert_eq!(too_large.locale, Locale::Bangla);
    }

    #[test]
    fn iterates_english_lists_text_maps_and_ranges() {
        let output = run(
            "let total = 0\nfor number in range(1, 4) {\n  total = total + number\n}\nprint total\nlet letters = \"\"\nfor letter in \"ab\" {\n  letters = letters + letter\n}\nprint letters\nlet profile = {\"name\": \"Rafi\", \"class\": 6}\nfor key in profile {\n  print key\n}\n",
        )
        .unwrap();
        assert_eq!(output, vec!["6", "ab", "class", "name"]);
    }

    #[test]
    fn iterates_bangla_range_and_restores_loop_variable_scope() {
        let output = run(
            "ধরি যোগফল = ০\nধরি মান = ৯৯\nপ্রতি মান মধ্যে পরিসর(৩) {\n  যোগফল = যোগফল + মান\n}\nদেখাও যোগফল\nদেখাও মান\n",
        )
        .unwrap();
        assert_eq!(output, vec!["3", "99"]);
    }

    #[test]
    fn propagates_return_from_a_for_loop_and_rejects_non_collections() {
        let output = run(
            "function first(values) {\n  for value in values {\n    return value\n  }\n  return none\n}\nprint first([7, 8])\n",
        )
        .unwrap();
        assert_eq!(output, vec!["7"]);

        let error = run("for item in 1 {\n  print item\n}\n").unwrap_err();
        assert_eq!(error.code, "P1010");
    }

    #[test]
    fn indexes_maps_and_supports_bangla_list_code() {
        let output = run(
            "ধরি সংখ্যা = [১০, ২০, ৩০]\nদেখাও সংখ্যা.get(০)\nদেখাও সংখ্যা[২]\nধরি প্রোফাইল = {\"নাম\": \"রাফি\"}\nদেখাও প্রোফাইল[\"নাম\"]\n",
        )
        .unwrap();
        assert_eq!(output, vec!["10", "30", "রাফি"]);
    }

    #[test]
    fn rejects_invalid_or_out_of_range_list_indexes() {
        let invalid_type = run("let items = [1]\nprint items[\"zero\"]\n").unwrap_err();
        assert_eq!(invalid_type.code, "P1026");

        let out_of_range = run("ধরি সংখ্যা = [১]\nদেখাও সংখ্যা[১]\n").unwrap_err();
        assert_eq!(out_of_range.code, "P1027");
        assert_eq!(out_of_range.locale, Locale::Bangla);
    }

    #[test]
    fn supports_english_and_bangla_null_values() {
        let output = run(
            "let result = none\nif result {\n  print \"unexpected\"\n} else {\n  print \"empty\"\n}\nধরি উত্তর = কিছুইনা\nযদি উত্তর {\n  দেখাও \"ভুল\"\n} নইলে {\n  দেখাও \"খালি\"\n}\n",
        )
        .unwrap();
        assert_eq!(output, vec!["empty", "খালি"]);
    }

    #[test]
    fn function_without_return_produces_null() {
        let output = run("function work() {\n print \"done\"\n}\nprint work()\n").unwrap();
        assert_eq!(output, vec!["done", "none"]);
    }

    #[test]
    fn provides_unicode_text_and_deterministic_math_builtins() {
        let output = run(
            "print text.len(\"abc\")\nprint text.trim(\"  padma \")\nprint text.upper(\"padma\")\nprint text.lower(\"PADMA\")\nprint text.contains(\"বাংলা Padma\", \"Padma\")\nprint text.replace(\"a-b\", \"-\", \"+\")\nprint text.join(text.split(\"a,b,c\", \",\"), \"-\")\nprint math.abs(-4)\nprint math.round(2.6)\nprint math.floor(2.9)\nprint math.ceil(2.1)\nprint math.min(7, 2, 5)\nprint math.max(7, 2, 5)\n",
        )
        .unwrap();
        assert_eq!(
            output,
            vec![
                "3", "padma", "PADMA", "padma", "true", "a+b", "a-b-c", "4", "3", "2", "3", "2",
                "7"
            ]
        );
    }

    #[test]
    fn provides_safe_file_read_exists_and_bounded_time_builtins() {
        let path = format!("target/padma-stdlib-test-{}.txt", process::id());
        let source = format!(
            "print file.exists(\"{path}\")\nfile.write(\"{path}\", \"hello\")\nprint file.exists(\"{path}\")\nprint file.read(\"{path}\")\nprint time.now() > 0\nprint time.sleep(0)\n"
        );
        let output = run(&source).unwrap();
        fs::remove_file(&path).unwrap();
        assert_eq!(output, vec!["false", "true", "hello", "true", "none"]);
    }

    #[test]
    fn parses_and_serializes_json_values() {
        let output = run(
            "let data = json.parse(\"{\\\"name\\\":\\\"Rafi\\\",\\\"scores\\\":[1,2],\\\"active\\\":true,\\\"note\\\":null}\")\nprint data[\"name\"]\nprint data[\"scores\"][1]\nprint data[\"note\"]\nprint json.stringify(data)\n",
        )
        .unwrap();
        assert_eq!(
            output,
            vec![
                "Rafi",
                "2",
                "none",
                "{\"active\":true,\"name\":\"Rafi\",\"note\":null,\"scores\":[1.0,2.0]}",
            ]
        );
    }

    #[test]
    fn parses_safe_http_urls_and_rejects_invalid_json_and_urls() {
        let output = run(
            "let info = url.parse(\"https://example.com:8443/api?q=padma#docs\")\nprint info[\"scheme\"]\nprint info[\"host\"]\nprint info[\"path\"]\nprint info[\"query\"]\nprint info[\"fragment\"]\nprint info[\"port\"]\nprint url.is_valid(\"ftp://example.com\")\n",
        )
        .unwrap();
        assert_eq!(
            output,
            vec![
                "https",
                "example.com",
                "/api",
                "q=padma",
                "docs",
                "8443",
                "false"
            ]
        );

        let json_error = run("print json.parse(\"{bad}\")\n").unwrap_err();
        assert_eq!(json_error.code, "P1029");
        let url_error = run("দেখাও url.parse(\"ftp://example.com\")\n").unwrap_err();
        assert_eq!(url_error.code, "P1030");
        assert_eq!(url_error.locale, Locale::Bangla);
    }

    #[test]
    fn formats_from_maps_and_handles_safe_relative_paths() {
        let output = run(
            "let details = {\"name\": \"Rafi\", \"class\": 6}\nprint text.format(\"{name} is in class {class}\", details)\nprint path.basename(\"notes/today.txt\")\nprint path.extension(\"notes/today.txt\")\nprint path.join(\"notes\", \"2026\", \"today.txt\")\n",
        )
        .unwrap();
        assert_eq!(
            output,
            vec![
                "Rafi is in class 6",
                "today.txt",
                "txt",
                "notes/2026/today.txt"
            ]
        );

        let placeholder_error = run("print text.format(\"Hello {name}\", {})\n").unwrap_err();
        assert_eq!(placeholder_error.code, "P1031");
        let path_error = run("দেখাও path.basename(\"../secret.txt\")\n").unwrap_err();
        assert_eq!(path_error.code, "P1014");
        assert_eq!(path_error.locale, Locale::Bangla);
    }

    #[test]
    fn generates_bounded_non_cryptographic_random_values() {
        let output = run(
            "let number = random.int(5, 8)\nprint number >= 5\nprint number < 8\nlet choice = random.pick([\"a\", \"b\"])\nif choice == \"a\" {\n  print true\n} else {\n  print choice == \"b\"\n}\n",
        )
        .unwrap();
        assert_eq!(output, vec!["true", "true", "true"]);

        let range_error = run("print random.int(4, 4)\n").unwrap_err();
        assert_eq!(range_error.code, "P1010");
        let pick_error = run("দেখাও random.pick([])\n").unwrap_err();
        assert_eq!(pick_error.code, "P1012");
        assert_eq!(pick_error.locale, Locale::Bangla);
    }

    #[test]
    fn blocks_unsafe_or_missing_file_reads() {
        let unsafe_error = run("print file.read(\"../secret.txt\")\n").unwrap_err();
        assert_eq!(unsafe_error.code, "P1014");
        let missing_error = run("print file.read(\"missing-padma-file.txt\")\n").unwrap_err();
        assert_eq!(missing_error.code, "P1028");
    }

    #[test]
    fn structured_table_csv_filter_select_count_and_export_are_project_scoped() {
        let root = module_fixture_dir("structured-table-csv");
        fs::create_dir_all(root.join("data")).unwrap();
        fs::create_dir_all(root.join("out")).unwrap();
        fs::write(
            root.join("data/inventory.csv"),
            "name,category,price\nTea,food,40\nNotebook,stationery,70\nCoffee,food,80\n",
        )
        .unwrap();
        let capabilities = BTreeSet::from(["filesystem:read".into(), "filesystem:write".into()]);
        let output = run_bridge_project(
            &root,
            capabilities,
            "let inventory = table.read(\"data/inventory.csv\", \"csv\")\nlet food = table.filter_equal(inventory, \"category\", \"food\")\nlet selected = table.select(food, [\"name\", \"price\"])\nlet rows = table.rows(selected)\nlet counts = table.count_by(inventory, \"category\")\nprint table.headers(selected)[0]\nprint rows[1][\"name\"]\nprint counts[\"food\"]\nprint table.write_csv(\"out/food.csv\", selected)\n",
        )
        .unwrap();
        let exported = fs::read_to_string(root.join("out/food.csv")).unwrap();
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(output, vec!["name", "Coffee", "2", "true"]);
        assert_eq!(exported, "name,price\nTea,40\nCoffee,80\n");
    }

    #[test]
    fn structured_table_supports_tsv_json_and_escaped_csv_cells() {
        let root = module_fixture_dir("structured-table-formats");
        fs::create_dir_all(root.join("data")).unwrap();
        fs::write(
            root.join("data/people.csv"),
            "name,city\nRafi,\"Dhaka, BD\"\n",
        )
        .unwrap();
        fs::write(root.join("data/people.tsv"), "name\tage\nRima\t12\n").unwrap();
        fs::write(
            root.join("data/people.json"),
            "[{\"name\":\"Rafi\",\"active\":true},{\"name\":\"Rima\",\"active\":false}]",
        )
        .unwrap();
        let output = run_bridge_project(
            &root,
            BTreeSet::from(["filesystem:read".into()]),
            "let csv = table.read(\"data/people.csv\", \"csv\")\nlet tsv = table.read(\"data/people.tsv\", \"tsv\")\nlet json = table.read(\"data/people.json\", \"json\")\nprint table.rows(csv)[0][\"city\"]\nprint table.rows(tsv)[0][\"age\"]\nprint table.count_by(json, \"active\")[\"true\"]\n",
        )
        .unwrap();
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(output, vec!["Dhaka, BD", "12", "1"]);
    }

    #[test]
    fn structured_table_rejects_missing_grants_paths_malformed_data_and_unsafe_export() {
        let root = module_fixture_dir("structured-table-safety");
        fs::create_dir_all(root.join("data")).unwrap();
        fs::write(root.join("data/broken.csv"), "name,score\nRafi\n").unwrap();

        let denied = run_bridge_project(
            &root,
            BTreeSet::new(),
            "print table.read(\"data/broken.csv\", \"csv\")\n",
        )
        .unwrap_err();
        assert_eq!(denied.code, "P1034");

        let traversal = run_bridge_project(
            &root,
            BTreeSet::from(["filesystem:read".into()]),
            "print table.read(\"../secret.csv\", \"csv\")\n",
        )
        .unwrap_err();
        assert_eq!(traversal.code, "P1014");

        let malformed = run_bridge_project(
            &root,
            BTreeSet::from(["filesystem:read".into()]),
            "print table.read(\"data/broken.csv\", \"csv\")\n",
        )
        .unwrap_err();
        assert_eq!(malformed.code, "P1069");

        let unsafe_export = run_bridge_project(
            &root,
            BTreeSet::from(["filesystem:write".into()]),
            "let table = {\"format\": \"csv\", \"headers\": [\"name\"], \"rows\": [{\"name\": \"Rafi\"}]}\nprint table.write_csv(\"../outside.csv\", table)\n",
        )
        .unwrap_err();
        fs::remove_dir_all(&root).unwrap();
        assert_eq!(unsafe_export.code, "P1014");
    }

    #[test]
    fn counts_repl_braces_without_counting_strings_or_comments() {
        assert_eq!(brace_delta("if true {"), 1);
        assert_eq!(brace_delta("print \"{not a block}\" # }"), 0);
        assert_eq!(brace_delta("}"), -1);
    }

    #[test]
    fn repl_displays_bare_expression_values_without_changing_statement_output() {
        let mut interpreter = Interpreter::new(Locale::Bangla);

        assert_eq!(
            run_repl_submission(&mut interpreter, "1 + 1\n").unwrap(),
            vec!["2"]
        );
        assert_eq!(
            run_repl_submission(&mut interpreter, "২ + 3\n").unwrap(),
            vec!["5"]
        );
        assert_eq!(
            run_repl_submission(&mut interpreter, "কিছুইনা\n").unwrap(),
            Vec::<String>::new()
        );
        assert_eq!(
            run_repl_submission(&mut interpreter, "ধরি সংখ্যা = ৭\n").unwrap(),
            Vec::<String>::new()
        );
        assert_eq!(
            run_repl_submission(&mut interpreter, "সংখ্যা + 1\n").unwrap(),
            vec!["8"]
        );
        assert_eq!(
            run_repl_submission(&mut interpreter, "দেখাও ২ + ৩\n").unwrap(),
            vec!["5"]
        );

        let error = run_repl_submission(&mut interpreter, "অজানা\n").unwrap_err();
        assert_eq!(error.code, "P1007");
        assert!(error.message.contains("কোনো variable পাওয়া যায়নি"));
    }

    #[test]
    fn creates_reads_and_updates_english_map() {
        let output = run(
            "let profile = {\"name\": \"Rafi\", \"age\": 12}\nprint profile.get(\"name\")\nprofile.set(\"age\", 13)\nprint profile.get(\"age\")\n",
        )
        .unwrap();
        assert_eq!(output, vec!["Rafi", "13"]);
    }

    #[test]
    fn creates_reads_and_updates_bangla_map() {
        let output = run(
            "ধরি প্রোফাইল = {\"নাম\": \"রাফি\", \"শ্রেণি\": ৬}\nদেখাও প্রোফাইল.get(\"নাম\")\nপ্রোফাইল.set(\"শ্রেণি\", ৭)\nদেখাও প্রোফাইল.get(\"শ্রেণি\")\n",
        )
        .unwrap();
        assert_eq!(output, vec!["রাফি", "7"]);
    }

    #[test]
    fn reports_missing_map_key() {
        let error =
            run("let profile = {\"name\": \"Rafi\"}\nprint profile.get(\"age\")\n").unwrap_err();
        assert_eq!(error.code, "P1021");
    }

    #[test]
    fn rejects_non_string_map_key() {
        let error = run("let profile = {\"name\": \"Rafi\"}\nprint profile.get(1)\n").unwrap_err();
        assert_eq!(error.code, "P1020");
    }

    #[test]
    fn check_reports_multiple_syntax_errors_without_stopping_at_the_first() {
        let english_errors = check_source("let = 1\nprint )\nlet valid = 3\n").unwrap_err();
        assert_eq!(english_errors.len(), 2);
        assert!(english_errors.iter().all(|error| error.code == "P1003"));

        let bangla_errors = check_source("ধরি = ১\nদেখাও )\nধরি ঠিক = ৩\n").unwrap_err();
        assert_eq!(bangla_errors.len(), 2);
        assert!(bangla_errors.iter().all(|error| error.code == "P1003"));
    }

    #[test]
    fn imports_english_module_functions_and_values() {
        let directory = module_fixture_dir("english-import");
        let module = directory.join("math.pd");
        let main = directory.join("main.pd");
        fs::write(
            &module,
            "function double(value) {\n  return value * 2\n}\nlet course = \"Padma\"\n",
        )
        .unwrap();
        fs::write(
            &main,
            "import \"math.pd\"\nprint double(21)\nprint course\n",
        )
        .unwrap();
        let output = run_file(&main).unwrap();
        fs::remove_dir_all(directory).unwrap();
        assert_eq!(output, vec!["42", "Padma"]);
    }

    #[test]
    fn imports_bangla_module_functions() {
        let directory = module_fixture_dir("bangla-import");
        let module = directory.join("বার্তা.pd");
        let main = directory.join("main.pd");
        fs::write(&module, "ফাংশন অভিবাদন(নাম) {\n  ফেরত \"হ্যালো {নাম}\"\n}\n").unwrap();
        fs::write(&main, "ইমপোর্ট \"বার্তা.pd\"\nদেখাও অভিবাদন(\"রাফি\")\n").unwrap();
        let output = run_file(&main).unwrap();
        fs::remove_dir_all(directory).unwrap();
        assert_eq!(output, vec!["হ্যালো রাফি"]);
    }

    #[test]
    fn imports_english_module_into_an_isolated_namespace() {
        let directory = module_fixture_dir("namespaced-import");
        let module = directory.join("lesson.pd");
        let main = directory.join("main.pd");
        fs::write(
            &module,
            "let course = \"Padma\"\nfunction double(value) {\n  return value * 2\n}\nfunction course_name() {\n  return course\n}\nfunction describe(value) {\n  return double(value)\n}\n",
        )
        .unwrap();
        fs::write(
            &main,
            "import \"lesson.pd\" as lesson\nprint lesson.course\nprint lesson.double(21)\nprint lesson.course_name()\nprint lesson.describe(3)\n",
        )
        .unwrap();
        let output = run_file(&main).unwrap();
        fs::remove_dir_all(directory).unwrap();
        assert_eq!(output, vec!["Padma", "42", "Padma", "6"]);
    }

    #[test]
    fn imports_bangla_module_with_hisabe_namespace_alias() {
        let directory = module_fixture_dir("bangla-namespaced-import");
        let module = directory.join("বার্তা.pd");
        let main = directory.join("main.pd");
        fs::write(
            &module,
            "ধরি শিরোনাম = \"পদ্ম\"\nফাংশন বার্তা(নাম) {\n  ফেরত \"{শিরোনাম}: {নাম}\"\n}\n",
        )
        .unwrap();
        fs::write(
            &main,
            "ইমপোর্ট \"বার্তা.pd\" হিসেবে লেখা\nদেখাও লেখা.শিরোনাম\nদেখাও লেখা.বার্তা(\"রাফি\")\n",
        )
        .unwrap();
        let output = run_file(&main).unwrap();
        fs::remove_dir_all(directory).unwrap();
        assert_eq!(output, vec!["পদ্ম", "পদ্ম: রাফি"]);
    }

    #[test]
    fn alias_imports_do_not_leak_module_values_to_the_caller() {
        let directory = module_fixture_dir("namespace-isolation");
        let module = directory.join("private.pd");
        let main = directory.join("main.pd");
        fs::write(&module, "let private_value = 7\n").unwrap();
        fs::write(
            &main,
            "import \"private.pd\" as private\nprint private_value\n",
        )
        .unwrap();
        let error = run_file(&main).unwrap_err();
        fs::remove_dir_all(directory).unwrap();
        assert_eq!(error.code, "P1007");
    }

    #[test]
    fn explicit_exports_filter_namespaced_module_symbols() {
        let directory = module_fixture_dir("explicit-exports");
        let module = directory.join("library.pd");
        let public_main = directory.join("public-main.pd");
        let private_main = directory.join("private-main.pd");
        fs::write(
            &module,
            "export let public_name = \"Padma\"\nlet private_name = \"hidden\"\nexport function label() {\n  return public_name\n}\nfunction private_label() {\n  return private_name\n}\n",
        )
        .unwrap();
        fs::write(
            &public_main,
            "import \"library.pd\" as library\nprint library.public_name\nprint library.label()\n",
        )
        .unwrap();
        let output = run_file(&public_main).unwrap();
        assert_eq!(output, vec!["Padma", "Padma"]);

        fs::write(
            &private_main,
            "import \"library.pd\" as library\nprint library.private_name\n",
        )
        .unwrap();
        let error = run_file(&private_main).unwrap_err();
        fs::remove_dir_all(directory).unwrap();
        assert_eq!(error.code, "P1007");
    }

    #[test]
    fn supports_bangla_export_declarations() {
        let directory = module_fixture_dir("bangla-exports");
        let module = directory.join("বই.pd");
        let main = directory.join("main.pd");
        fs::write(
            &module,
            "রপ্তানি ধরি নাম = \"পদ্ম\"\nরপ্তানি ফাংশন শিরোনাম() {\n  ফেরত নাম\n}\n",
        )
        .unwrap();
        fs::write(&main, "ইমপোর্ট \"বই.pd\" হিসেবে বই\nদেখাও বই.শিরোনাম()\n").unwrap();
        let output = run_file(&main).unwrap();
        fs::remove_dir_all(directory).unwrap();
        assert_eq!(output, vec!["পদ্ম"]);
    }

    #[test]
    fn preserves_imported_module_source_context_for_diagnostics() {
        let directory = module_fixture_dir("module-diagnostic-context");
        let module = directory.join("broken.pd");
        let main = directory.join("main.pd");
        fs::write(&module, "print missing_value\n").unwrap();
        fs::write(&main, "import \"broken.pd\"\n").unwrap();
        let error = run_file(&main).unwrap_err();
        fs::remove_dir_all(directory).unwrap();
        assert_eq!(error.code, "P1007");
        assert_eq!(error.source_path.as_deref(), Some(module.as_path()));
        assert_eq!(error.source_text.as_deref(), Some("print missing_value\n"));
        assert_eq!(error.locale, Locale::English);
    }

    #[test]
    fn loads_each_module_only_once() {
        let directory = module_fixture_dir("duplicate-import");
        let module = directory.join("notice.pd");
        let main = directory.join("main.pd");
        fs::write(&module, "print \"loaded\"\n").unwrap();
        fs::write(
            &main,
            "import \"notice.pd\"\nimport \"notice.pd\"\nprint \"done\"\n",
        )
        .unwrap();
        let output = run_file(&main).unwrap();
        fs::remove_dir_all(directory).unwrap();
        assert_eq!(output, vec!["loaded", "done"]);
    }

    #[test]
    fn imports_nested_modules_relative_to_their_importer() {
        let directory = module_fixture_dir("nested-import");
        let helpers = directory.join("helpers");
        fs::create_dir_all(&helpers).unwrap();
        let base = helpers.join("base.pd");
        let tools = helpers.join("tools.pd");
        let main = directory.join("main.pd");
        fs::write(
            &base,
            "function square(value) {\n  return value * value\n}\n",
        )
        .unwrap();
        fs::write(&tools, "import \"base.pd\"\n").unwrap();
        fs::write(&main, "import \"helpers/tools.pd\"\nprint square(5)\n").unwrap();
        let output = run_file(&main).unwrap();
        fs::remove_dir_all(directory).unwrap();
        assert_eq!(output, vec!["25"]);
    }

    #[test]
    fn rejects_module_path_traversal_and_import_cycles() {
        let directory = module_fixture_dir("unsafe-import");
        let unsafe_main = directory.join("unsafe.pd");
        fs::write(&unsafe_main, "import \"../secret.pd\"\n").unwrap();
        let unsafe_error = run_file(&unsafe_main).unwrap_err();
        assert_eq!(unsafe_error.code, "P1022");

        let module_a = directory.join("a.pd");
        let module_b = directory.join("b.pd");
        fs::write(&module_a, "import \"b.pd\"\n").unwrap();
        fs::write(&module_b, "import \"a.pd\"\n").unwrap();
        let cycle_error = run_file(&module_a).unwrap_err();
        fs::remove_dir_all(directory).unwrap();
        assert_eq!(cycle_error.code, "P1024");
    }

    #[test]
    fn identity_primitives_are_capability_gated_and_preserve_security_boundaries() {
        let position = Position::new(1, 1);
        let record =
            password_record_from_secret("test-password", Locale::English, position).unwrap();
        assert!(record.starts_with("$padma-pbkdf2-sha256$600000$"));
        assert!(verify_password_record(&record, "test-password"));
        assert!(!verify_password_record(&record, "wrong-password"));

        let secret = b"0123456789abcdef0123456789abcdef";
        let token = issue_signed_session("rafi", secret, 60, Locale::English, position).unwrap();
        assert_eq!(
            verify_signed_session(&token, secret).map(|value| value.0),
            Some("rafi".into())
        );
        assert!(verify_signed_session(&token, b"abcdef0123456789abcdef0123456789").is_none());
        assert!(constant_time_eq(b"csrf", b"csrf"));
        assert!(!constant_time_eq(b"csrf", b"other"));

        let missing_capability = compile("print auth.csrf_token()\n").unwrap();
        let mut denied = Interpreter::new(missing_capability.1);
        assert_eq!(denied.run(&missing_capability.0).unwrap_err().code, "P1034");

        let root = module_fixture_dir("identity-runtime");
        let environment_name = "PADMA_IDENTITY_TEST_SESSION_SECRET";
        unsafe { env::set_var(environment_name, "0123456789abcdef0123456789abcdef") };
        let source = format!(
            "let token = auth.session_issue(\"rafi\", \"{environment_name}\", 60)\nprint auth.session_verify(\"{environment_name}\", token)[\"subject\"]\nlet csrf = auth.csrf_token()\nprint auth.csrf_verify(csrf, csrf)\nprint auth.cookie(\"padma_session\", token, \"{environment_name}\")\n"
        );
        let (program, locale) = compile(&source).unwrap();
        let mut permitted = Interpreter::with_project_capabilities(
            locale,
            root.join("main.pd"),
            root.clone(),
            BTreeSet::from(["identity:local".into()]),
        );
        permitted.run(&program).unwrap();
        assert_eq!(permitted.output[0], "rafi");
        assert_eq!(permitted.output[1], "true");
        assert!(permitted.output[2].contains("HttpOnly; Secure; SameSite=Strict"));

        let (literal_program, literal_locale) =
            compile("auth.password_hash(\"hard-coded\")\n").unwrap();
        let mut literal = Interpreter::with_project_capabilities(
            literal_locale,
            root.join("main.pd"),
            root.clone(),
            BTreeSet::from(["identity:local".into()]),
        );
        assert_eq!(literal.run(&literal_program).unwrap_err().code, "P1045");
        unsafe { env::remove_var(environment_name) };
        fs::remove_dir_all(root).unwrap();

        assert_eq!(static_builtin_arity("auth.password_hash"), Some((1, 1)));
        assert_eq!(static_builtin_arity("auth.session_issue"), Some((3, 3)));
        assert_eq!(static_builtin_arity("auth.csrf_token"), Some((0, 0)));
    }

    #[test]
    fn ai_workflow_manifest_accepts_one_bounded_provider_neutral_contract() {
        let manifest = parse_ai_workflow_manifest(
            "[workflow]\nversion = \"1\"\nadapter = \"json-http-v1\"\nendpoint = \"https://ai-gateway.example.com/v1/padma\"\nsecret_env = \"PADMA_AI_KEY\"\nmodel = \"reviewed-model-id\"\ntimeout_seconds = 30\nmax_input_bytes = 32768\nmax_response_bytes = 65536\nretry_policy = \"never\"\n",
            Locale::English,
        )
        .unwrap();
        assert_eq!(manifest.endpoint, "https://ai-gateway.example.com/v1/padma");
        assert_eq!(manifest.secret_env, "PADMA_AI_KEY");
        assert_eq!(manifest.timeout_seconds, 30);
        assert_eq!(manifest.max_input_bytes, 32_768);
        assert_eq!(manifest.max_response_bytes, 65_536);
        assert_eq!(manifest.retry_policy, "never");
    }

    #[test]
    fn ai_workflow_manifest_rejects_unsafe_endpoint_secret_and_unknown_fields() {
        let private_endpoint = parse_ai_workflow_manifest(
            "[workflow]\nversion = \"1\"\nadapter = \"json-http-v1\"\nendpoint = \"https://127.0.0.1/v1/padma\"\nsecret_env = \"PADMA_AI_KEY\"\nmodel = \"reviewed-model-id\"\ntimeout_seconds = 30\nmax_input_bytes = 1\nmax_response_bytes = 1\nretry_policy = \"never\"\n",
            Locale::English,
        )
        .unwrap_err();
        assert!(private_endpoint.starts_with("P1050"));

        let unsafe_secret = parse_ai_workflow_manifest(
            "[workflow]\nversion = \"1\"\nadapter = \"json-http-v1\"\nendpoint = \"https://ai-gateway.example.com/v1/padma\"\nsecret_env = \"PADMA_AI_KEY=secret\"\nmodel = \"reviewed-model-id\"\ntimeout_seconds = 30\nmax_input_bytes = 1\nmax_response_bytes = 1\nretry_policy = \"never\"\n",
            Locale::English,
        )
        .unwrap_err();
        assert!(unsafe_secret.starts_with("P1050"));

        let unknown_field = parse_ai_workflow_manifest(
            "[workflow]\nversion = \"1\"\nadapter = \"json-http-v1\"\nendpoint = \"https://ai-gateway.example.com/v1/padma\"\nsecret_env = \"PADMA_AI_KEY\"\nmodel = \"reviewed-model-id\"\ntimeout_seconds = 30\nmax_input_bytes = 1\nmax_response_bytes = 1\nretry_policy = \"never\"\ncommand = \"curl\"\n",
            Locale::Bangla,
        )
        .unwrap_err();
        assert!(unknown_field.starts_with("P1050"));
        assert!(unknown_field.contains("নিরাপদ"));
    }

    #[test]
    fn ai_workflow_plan_is_capability_gated_and_never_reads_or_exposes_secret_values() {
        let root = module_fixture_dir("ai-workflow-plan");
        fs::write(
            root.join("padma.toml"),
            "[padma]\nname = \"study-helper\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\nlocale = \"en\"\n\n[capabilities]\nnetwork = [\"ai\"]\n",
        )
        .unwrap();
        fs::write(
            root.join("padma-ai.toml"),
            "[workflow]\nversion = \"1\"\nadapter = \"json-http-v1\"\nendpoint = \"https://ai-gateway.example.com/v1/padma\"\nsecret_env = \"PADMA_AI_TEST_SECRET\"\nmodel = \"reviewed-model-id\"\ntimeout_seconds = 30\nmax_input_bytes = 32768\nmax_response_bytes = 65536\nretry_policy = \"never\"\n",
        )
        .unwrap();
        let plan = ai_workflow_plan_contents(&root).unwrap();
        let inspect = ai_workflow_inspect_contents(&root).unwrap();
        fs::remove_dir_all(&root).unwrap();
        assert!(plan.contains("\"mode\": \"inspection-only\""));
        assert!(plan.contains("\"value\": \"not-read\""));
        assert!(plan.contains("\"network\": \"disabled\""));
        assert!(plan.contains("\"environmentRead\": \"disabled\""));
        assert!(plan.contains("\"childProcess\": \"disabled\""));
        assert!(!plan.contains("PADMA_AI_TEST_SECRET_VALUE"));
        assert!(inspect.starts_with("Padma AI workflow manifest (inspection-only)"));

        let denied = module_fixture_dir("ai-workflow-capability-denied");
        fs::write(
            denied.join("padma.toml"),
            "[padma]\nname = \"denied\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\nlocale = \"en\"\n\n[capabilities]\nnetwork = [\"http\"]\n",
        )
        .unwrap();
        fs::write(
            denied.join("padma-ai.toml"),
            "[workflow]\nversion = \"1\"\nadapter = \"json-http-v1\"\nendpoint = \"https://ai-gateway.example.com/v1/padma\"\nsecret_env = \"PADMA_AI_TEST_SECRET\"\nmodel = \"reviewed-model-id\"\ntimeout_seconds = 30\nmax_input_bytes = 1\nmax_response_bytes = 1\nretry_policy = \"never\"\n",
        )
        .unwrap();
        let error = ai_workflow_plan_contents(&denied).unwrap_err();
        fs::remove_dir_all(&denied).unwrap();
        assert!(error.starts_with("P1034"));
        assert!(error.contains("network:ai"));
    }

    #[test]
    fn ai_workflow_runtime_uses_strict_inert_data_envelopes() {
        let manifest = parse_ai_workflow_manifest(
            "[workflow]\nversion = \"1\"\nadapter = \"json-http-v1\"\nendpoint = \"https://ai-gateway.example.com/v1/padma\"\nsecret_env = \"PADMA_AI_RUNTIME_TEST_SECRET\"\nmodel = \"reviewed-model-id\"\ntimeout_seconds = 30\nmax_input_bytes = 32768\nmax_response_bytes = 65536\nretry_policy = \"never\"\n",
            Locale::English,
        )
        .unwrap();
        let input = Value::Map(BTreeMap::from([
            ("task".to_string(), Value::String("summarize".to_string())),
            (
                "instruction".to_string(),
                Value::String("Summarize the supplied text only.".to_string()),
            ),
            (
                "data".to_string(),
                Value::Map(BTreeMap::from([(
                    "text".to_string(),
                    Value::String("Padma keeps model output inert.".to_string()),
                )])),
            ),
        ]));
        let request =
            ai_workflow_request_payload(&input, &manifest, Locale::English, Position::new(1, 1))
                .unwrap();
        let request_json: JsonValue = serde_json::from_slice(&request).unwrap();
        assert_eq!(request_json["protocol"], "padma-ai-workflow-v1");
        assert_eq!(request_json["task"], "summarize");
        assert_eq!(request_json["model"], "reviewed-model-id");

        let value = ai_workflow_response_value(
            br#"{"protocol":"padma-ai-workflow-v1","output":{"suggested_code":"file.write(\"outside.txt\", \"never execute this\")"}}"#,
            &manifest,
            Locale::English,
            Position::new(1, 1),
        )
        .unwrap();
        let Value::Map(result) = value else {
            panic!("workflow result must remain a map")
        };
        let Some(Value::Map(output)) = result.get("output") else {
            panic!("workflow output must remain an inert map")
        };
        assert_eq!(
            output.get("suggested_code"),
            Some(&Value::String(
                "file.write(\"outside.txt\", \"never execute this\")".to_string()
            ))
        );
        assert!(!Path::new("outside.txt").exists());

        let invalid = ai_workflow_response_value(
            br#"{"protocol":"wrong","output":{"text":"ignored"}}"#,
            &manifest,
            Locale::English,
            Position::new(1, 1),
        )
        .unwrap_err();
        assert_eq!(invalid.code, "P1052");

        let config = ai_workflow_curl_config(&manifest, "runtime-secret", &request).unwrap();
        let config = String::from_utf8(config).unwrap();
        assert!(config.contains("request = \"POST\""));
        assert!(config.contains("max-time = \"30\""));
        assert!(config.contains("Authorization: Bearer runtime-secret"));
        assert!(!config.contains("location"));
        assert!(!config.contains("retry"));
    }

    #[test]
    fn ai_workflow_runtime_fails_before_transport_when_secret_is_missing() {
        let root = module_fixture_dir("ai-workflow-runtime-missing-secret");
        let secret_name = "PADMA_AI_WORKFLOW_MISSING_SECRET_91C2";
        assert!(env::var(secret_name).is_err());
        fs::write(
            root.join("padma.toml"),
            "[padma]\nname = \"ai-runtime\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\nlocale = \"en\"\n\n[capabilities]\nnetwork = [\"ai\"]\n",
        )
        .unwrap();
        fs::write(
            root.join("padma-ai.toml"),
            format!(
                "[workflow]\nversion = \"1\"\nadapter = \"json-http-v1\"\nendpoint = \"https://ai-gateway.example.com/v1/padma\"\nsecret_env = \"{secret_name}\"\nmodel = \"reviewed-model-id\"\ntimeout_seconds = 30\nmax_input_bytes = 32768\nmax_response_bytes = 65536\nretry_policy = \"never\"\n"
            ),
        )
        .unwrap();
        fs::write(
            root.join("main.pd"),
            "let response = ai.workflow({\"task\": \"summarize\", \"instruction\": \"Summarize only.\", \"data\": {\"text\": \"Padma\"}})\nprint response\n",
        )
        .unwrap();
        let (manifest, entry) = load_project_manifest(&root).unwrap();
        let source = fs::read_to_string(&entry).unwrap();
        let (program, locale) = compile(&source).unwrap();
        let mut interpreter = Interpreter::with_project_capabilities(
            locale,
            entry,
            root.clone(),
            manifest.capabilities,
        );
        let error = interpreter.run(&program).unwrap_err();
        fs::remove_dir_all(root).unwrap();
        assert_eq!(error.code, "P1051");
        assert!(error.message.contains("credential is unavailable"));
    }

    #[test]
    fn ai_workflow_runtime_makes_one_fixed_transport_call_without_network() {
        let _serial = AI_WORKFLOW_CURL_TEST_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap();
        let root = module_fixture_dir("ai-workflow-runtime-one-shot");
        let curl = root.join("mock-curl.sh");
        let count = root.join("transport-count.txt");
        fs::write(
            &curl,
            format!(
                "#!/bin/sh\ncat >/dev/null\nprintf '1' >> '{}'\nprintf '%s' '{{\"protocol\":\"padma-ai-workflow-v1\",\"output\":{{\"summary\":\"mock response\"}}}}'\n",
                count.display()
            ),
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&curl).unwrap().permissions();
            permissions.set_mode(0o700);
            fs::set_permissions(&curl, permissions).unwrap();
        }
        *AI_WORKFLOW_CURL_TEST_PROGRAM
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap() = Some(curl);
        fs::write(
            root.join("padma.toml"),
            "[padma]\nname = \"ai-runtime\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\nlocale = \"en\"\n\n[capabilities]\nnetwork = [\"ai\"]\n",
        )
        .unwrap();
        fs::write(
            root.join("padma-ai.toml"),
            "[workflow]\nversion = \"1\"\nadapter = \"json-http-v1\"\nendpoint = \"https://ai-gateway.example.com/v1/padma\"\nsecret_env = \"PATH\"\nmodel = \"reviewed-model-id\"\ntimeout_seconds = 30\nmax_input_bytes = 32768\nmax_response_bytes = 65536\nretry_policy = \"never\"\n",
        )
        .unwrap();
        fs::write(
            root.join("main.pd"),
            "let response = ai.workflow({\"task\": \"summarize\", \"instruction\": \"Summarize only.\", \"data\": {\"text\": \"Padma\"}})\nprint response\n",
        )
        .unwrap();
        let (manifest, entry) = load_project_manifest(&root).unwrap();
        let source = fs::read_to_string(&entry).unwrap();
        let (program, locale) = compile(&source).unwrap();
        let mut interpreter = Interpreter::with_project_capabilities(
            locale,
            entry,
            root.clone(),
            manifest.capabilities,
        );
        interpreter.run(&program).unwrap();
        *AI_WORKFLOW_CURL_TEST_PROGRAM
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap() = None;
        assert_eq!(fs::read_to_string(&count).unwrap(), "1");
        fs::remove_dir_all(root).unwrap();
    }

    fn valid_browser_plan_manifest() -> &'static str {
        "[browser]\nversion = \"1\"\nintent = \"navigation-review\"\nredirect_policy = \"deny\"\nmax_steps = 2\n\n[allowlist]\norigins = [\n  \"https://www.rust-lang.org\",\n  \"https://docs.python.org\"\n]\n\n[navigation]\nurls = [\n  \"https://docs.python.org/3/tutorial/\",\n  \"https://www.rust-lang.org/learn\"\n]\n"
    }

    fn write_browser_plan_project(root: &Path, granted: bool, manifest: &str) {
        let capabilities = if granted {
            "browser = [\"plan\"]"
        } else {
            "network = [\"http\"]"
        };
        fs::write(
            root.join("padma.toml"),
            format!(
                "[padma]\nname = \"browser-plan-test\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\nlocale = \"en\"\n\n[capabilities]\n{capabilities}\n"
            ),
        )
        .unwrap();
        fs::write(root.join("padma-browser.toml"), manifest).unwrap();
    }

    #[test]
    fn browser_plan_emits_a_deterministic_no_side_effect_descriptor() {
        let root = module_fixture_dir("browser-plan-valid");
        write_browser_plan_project(&root, true, valid_browser_plan_manifest());

        let plan: JsonValue = serde_json::from_str(&browser_plan_json(&root).unwrap()).unwrap();
        let inspect = browser_inspect_contents(&root).unwrap();
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(plan["browserPlanVersion"], 1);
        assert_eq!(plan["mode"], "inspection-only");
        assert!(plan["planDigest"].as_str().is_some_and(is_sha256_digest));
        assert_eq!(plan["intent"], "navigation-review");
        assert_eq!(
            plan["allowlistedOrigins"],
            serde_json::json!(["https://docs.python.org", "https://www.rust-lang.org"])
        );
        assert_eq!(plan["navigation"][0]["method"], "GET");
        assert_eq!(
            plan["navigation"][0]["url"],
            "https://docs.python.org/3/tutorial/"
        );
        assert_eq!(plan["limits"]["redirectPolicy"], "deny");
        assert_eq!(plan["browser"], "not-started");
        assert_eq!(plan["network"], "disabled");
        assert_eq!(plan["dns"], "disabled");
        assert_eq!(plan["cookies"], "not-read");
        assert_eq!(plan["credentials"], "not-read");
        assert_eq!(plan["environmentRead"], "disabled");
        assert_eq!(plan["childProcess"], "disabled");
        assert!(inspect.starts_with("Padma browser plan manifest (inspection-only)"));
    }

    #[test]
    fn browser_plan_requires_the_narrow_browser_plan_capability() {
        let root = module_fixture_dir("browser-plan-capability-denied");
        write_browser_plan_project(&root, false, valid_browser_plan_manifest());

        let error = browser_plan_json(&root).unwrap_err();
        fs::remove_dir_all(&root).unwrap();

        assert!(error.starts_with("P1034"));
        assert!(error.contains("browser:plan"));
    }

    #[test]
    fn browser_plan_rejects_noncanonical_or_unsafe_origins_locally() {
        for origin in [
            "http://docs.python.org",
            "https://*.python.org",
            "https://127.0.0.1",
            "https://localhost",
            "https://user@docs.python.org",
        ] {
            let manifest = valid_browser_plan_manifest().replace("https://docs.python.org", origin);
            let error = parse_browser_plan_manifest(&manifest, Locale::English).unwrap_err();
            assert!(
                error.starts_with("P1053"),
                "origin should be rejected: {origin}"
            );
            assert!(!error.contains(origin));
        }
    }

    #[test]
    fn browser_plan_rejects_navigation_outside_the_exact_allowlist() {
        for navigation_url in [
            "https://sub.docs.python.org/3/tutorial/",
            "https://docs.python.org.attacker.invalid/3/tutorial/",
            "https://user@docs.python.org/3/tutorial/",
        ] {
            let manifest = valid_browser_plan_manifest()
                .replace("https://docs.python.org/3/tutorial/", navigation_url);
            let error = parse_browser_plan_manifest(&manifest, Locale::English).unwrap_err();
            assert!(
                error.starts_with("P1054"),
                "navigation URL should be rejected: {navigation_url}"
            );
            assert!(!error.contains(navigation_url));
        }
    }

    #[test]
    fn browser_plan_rejects_duplicate_policy_fields() {
        let manifest = valid_browser_plan_manifest().replacen(
            "intent = \"navigation-review\"",
            "intent = \"navigation-review\"\nintent = \"navigation-review\"",
            1,
        );
        let error = parse_browser_plan_manifest(&manifest, Locale::English).unwrap_err();
        assert!(error.starts_with("P1053"));
    }

    #[test]
    fn browser_execution_remains_an_explicitly_prohibited_future_boundary() {
        let error = browser_plan_error(Locale::English, "P1055", "");
        assert!(error.starts_with("P1055: Browser execution is prohibited"));
        assert!(error.contains("no browser will be launched"));
        assert!(!usage(Locale::English).contains("browser navigate"));
    }

    fn valid_browser_confirmation_manifest(digest: &str) -> String {
        format!(
            "[confirmation]\nversion = \"1\"\nmode = \"local-session-plan\"\nbrowser_plan_digest = \"{digest}\"\nnavigation_index = 1\nmax_session_seconds = 60\n"
        )
    }

    fn write_browser_confirmation_project(root: &Path, granted: bool, manifest: &str) {
        let capabilities = if granted {
            "browser = [\"plan\", \"confirm-plan\"]"
        } else {
            "browser = [\"plan\"]"
        };
        fs::write(
            root.join("padma.toml"),
            format!(
                "[padma]\nname = \"browser-confirmation-test\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\nlocale = \"en\"\n\n[capabilities]\n{capabilities}\n"
            ),
        )
        .unwrap();
        fs::write(
            root.join("padma-browser.toml"),
            valid_browser_plan_manifest(),
        )
        .unwrap();
        fs::write(root.join("padma-browser-confirm.toml"), manifest).unwrap();
    }

    #[test]
    fn browser_confirmation_plan_binds_one_reviewed_get_destination_without_execution() {
        let root = module_fixture_dir("browser-confirmation-valid");
        let browser_plan =
            parse_browser_plan_manifest(valid_browser_plan_manifest(), Locale::English).unwrap();
        let manifest = valid_browser_confirmation_manifest(&browser_plan_digest(&browser_plan));
        write_browser_confirmation_project(&root, true, &manifest);

        let plan: JsonValue =
            serde_json::from_str(&browser_confirmation_plan_json(&root).unwrap()).unwrap();
        let inspect = browser_confirmation_inspect_contents(&root).unwrap();
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(plan["browserConfirmationPlanVersion"], 1);
        assert_eq!(plan["mode"], "local-confirmation-session-planning");
        assert_eq!(plan["browserPlan"]["method"], "GET");
        assert_eq!(
            plan["browserPlan"]["url"],
            "https://docs.python.org/3/tutorial/"
        );
        assert_eq!(plan["browserPlan"]["redirectPolicy"], "deny");
        assert_eq!(plan["confirmation"]["required"], true);
        assert_eq!(plan["confirmation"]["status"], "not-issued");
        assert_eq!(plan["confirmation"]["singleUse"], true);
        assert_eq!(plan["confirmation"]["modelSupplied"], "rejected");
        assert_eq!(plan["session"], "awaiting-confirmation");
        assert_eq!(plan["browser"], "not-started");
        assert_eq!(plan["network"], "disabled");
        assert_eq!(plan["dns"], "disabled");
        assert_eq!(plan["cookies"], "not-read");
        assert_eq!(plan["credentials"], "not-read");
        assert_eq!(plan["browserProfile"], "not-read");
        assert_eq!(plan["javascriptExecution"], "disabled");
        assert_eq!(plan["formSubmission"], "disabled");
        assert_eq!(plan["payment"], "disabled");
        assert!(inspect.starts_with("Padma browser confirmation session (inspection-only)"));
    }

    #[test]
    fn browser_confirmation_plan_requires_its_own_narrow_capability() {
        let root = module_fixture_dir("browser-confirmation-capability-denied");
        let browser_plan =
            parse_browser_plan_manifest(valid_browser_plan_manifest(), Locale::English).unwrap();
        let manifest = valid_browser_confirmation_manifest(&browser_plan_digest(&browser_plan));
        write_browser_confirmation_project(&root, false, &manifest);

        let error = browser_confirmation_plan_json(&root).unwrap_err();
        fs::remove_dir_all(&root).unwrap();

        assert!(error.starts_with("P1034"));
        assert!(error.contains("browser:confirm-plan"));
    }

    #[test]
    fn browser_confirmation_plan_rejects_unbound_or_unsafe_manifest_data() {
        let browser_plan =
            parse_browser_plan_manifest(valid_browser_plan_manifest(), Locale::English).unwrap();
        let digest = browser_plan_digest(&browser_plan);
        for manifest in [
            valid_browser_confirmation_manifest(
                "sha256:0000000000000000000000000000000000000000000000000000000000000000",
            ),
            valid_browser_confirmation_manifest(&digest)
                .replace("navigation_index = 1", "navigation_index = 3"),
            valid_browser_confirmation_manifest(&digest)
                .replace("mode = \"local-session-plan\"", "mode = \"navigate\""),
            valid_browser_confirmation_manifest(&digest).replace(
                "max_session_seconds = 60",
                "max_session_seconds = 60\nmax_session_seconds = 61",
            ),
            valid_browser_confirmation_manifest(&digest)
                .replace("max_session_seconds = 60", "max_session_seconds = 301"),
        ] {
            let root = module_fixture_dir("browser-confirmation-invalid");
            write_browser_confirmation_project(&root, true, &manifest);
            let error = browser_confirmation_plan_json(&root).unwrap_err();
            fs::remove_dir_all(&root).unwrap();
            assert!(error.starts_with("P1060"));
        }
    }

    #[test]
    fn browser_confirmation_and_navigation_execution_remain_prohibited() {
        let error = browser_confirmation_error(Locale::English, "P1061", "");
        assert!(error.starts_with("P1061: Browser confirmation or navigation action is prohibited"));
        assert!(error.contains("no browser, DNS, network, cookie, or credential will be used"));
        assert!(!usage(Locale::English).contains("browser navigate"));
        assert!(!usage(Locale::English).contains("browser confirm execute"));
    }

    fn write_browser_handoff_project(root: &Path, granted: bool, manifest: &str) {
        let capabilities = if granted {
            "browser = [\"plan\", \"confirm-plan\", \"handoff\"]"
        } else {
            "browser = [\"plan\", \"confirm-plan\"]"
        };
        fs::write(
            root.join("padma.toml"),
            format!(
                "[padma]\nname = \"browser-handoff-test\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\nlocale = \"en\"\n\n[capabilities]\n{capabilities}\n"
            ),
        )
        .unwrap();
        fs::write(
            root.join("padma-browser.toml"),
            valid_browser_plan_manifest(),
        )
        .unwrap();
        fs::write(root.join("padma-browser-confirm.toml"), manifest).unwrap();
    }

    fn valid_browser_handoff_audit_manifest(max_records: usize) -> String {
        format!(
            "[audit]\nversion = \"1\"\nmode = \"redacted-local-v1\"\npath = \"audit/handoff.jsonl\"\nmax_records = {max_records}\n"
        )
    }

    fn write_browser_handoff_audit_project(root: &Path, manifest: &str, audit: &str) {
        fs::write(
            root.join("padma.toml"),
            "[padma]\nname = \"browser-handoff-audit-test\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\nlocale = \"en\"\n\n[capabilities]\nbrowser = [\"plan\", \"confirm-plan\", \"handoff\", \"audit\"]\n",
        )
        .unwrap();
        fs::write(
            root.join("padma-browser.toml"),
            valid_browser_plan_manifest(),
        )
        .unwrap();
        fs::write(root.join("padma-browser-confirm.toml"), manifest).unwrap();
        fs::write(root.join("padma-browser-audit.toml"), audit).unwrap();
    }

    #[test]
    fn browser_handoff_requires_a_separate_capability_and_uses_one_reviewed_destination() {
        let browser_plan =
            parse_browser_plan_manifest(valid_browser_plan_manifest(), Locale::English).unwrap();
        let manifest = valid_browser_confirmation_manifest(&browser_plan_digest(&browser_plan));

        let denied_root = module_fixture_dir("browser-handoff-capability-denied");
        write_browser_handoff_project(&denied_root, false, &manifest);
        let error = load_browser_handoff_context(&denied_root).unwrap_err();
        fs::remove_dir_all(&denied_root).unwrap();
        assert!(error.starts_with("P1034"));
        assert!(error.contains("browser:handoff"));

        let granted_root = module_fixture_dir("browser-handoff-destination");
        write_browser_handoff_project(&granted_root, true, &manifest);
        let context = load_browser_handoff_context(&granted_root).unwrap();
        fs::remove_dir_all(&granted_root).unwrap();
        assert_eq!(context.locale, Locale::English);
        assert_eq!(context.destination, "https://docs.python.org/3/tutorial/");
    }

    #[test]
    fn browser_handoff_uses_only_the_fixed_termux_opener_and_reviewed_url_argument() {
        let command = termux_browser_handoff_command(
            std::ffi::OsStr::new("/data/data/com.termux/files/usr/bin"),
            "https://docs.python.org/3/tutorial/",
        );
        assert_eq!(
            command.get_program(),
            std::ffi::OsStr::new("termux-open-url")
        );
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            vec![std::ffi::OsStr::new("https://docs.python.org/3/tutorial/")]
        );
        assert!(command
            .get_envs()
            .all(|(key, value)| key == std::ffi::OsStr::new("PATH") && value.is_some()));
    }

    #[test]
    fn browser_handoff_cancellation_is_explicit_and_never_becomes_open() {
        assert_eq!(
            browser_handoff_confirmation_decision("OPEN\n"),
            BrowserHandoffDecision::Open
        );
        for answer in ["", "CANCEL", "open", "OPEN now", "\\n"] {
            assert_eq!(
                browser_handoff_confirmation_decision(answer),
                BrowserHandoffDecision::Cancelled,
                "unexpected handoff decision for {answer:?}"
            );
        }
    }

    #[test]
    fn browser_handoff_audit_is_opt_in_bounded_and_redacted() {
        let browser_plan =
            parse_browser_plan_manifest(valid_browser_plan_manifest(), Locale::English).unwrap();
        let confirmation = valid_browser_confirmation_manifest(&browser_plan_digest(&browser_plan));
        let root = module_fixture_dir("browser-handoff-audit-redacted");
        write_browser_handoff_audit_project(
            &root,
            &confirmation,
            &valid_browser_handoff_audit_manifest(2),
        );
        let context = load_browser_handoff_context(&root).unwrap();
        browser_handoff_audit_record(&context, "cancelled", "P1062").unwrap();
        browser_handoff_audit_record(&context, "opener-failed", "P1063").unwrap();
        browser_handoff_audit_record(&context, "opener-requested", "requested").unwrap();
        let contents = fs::read_to_string(root.join("audit/handoff.jsonl")).unwrap();
        let records = contents.lines().collect::<Vec<_>>();
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(records.len(), 2);
        assert!(!contents.contains("https://"));
        assert!(!contents.contains("OPEN"));
        for line in records {
            let record: JsonValue = serde_json::from_str(line).unwrap();
            assert_eq!(record["event"], "android-browser-handoff");
            assert!(record.get("browserPlanDigest").is_some());
            assert!(record.get("navigationIndex").is_some());
            assert!(record.get("state").is_some());
            assert!(record.get("outcome").is_some());
            assert_eq!(record.as_object().unwrap().len(), 7);
        }
    }

    #[test]
    fn browser_handoff_audit_requires_a_narrow_grant_and_rejects_unsafe_data() {
        let browser_plan =
            parse_browser_plan_manifest(valid_browser_plan_manifest(), Locale::English).unwrap();
        let confirmation = valid_browser_confirmation_manifest(&browser_plan_digest(&browser_plan));
        let root = module_fixture_dir("browser-handoff-audit-unsafe");
        write_browser_handoff_project(&root, true, &confirmation);
        fs::write(
            root.join("padma-browser-audit.toml"),
            "[audit]\nversion = \"1\"\nmode = \"redacted-local-v1\"\npath = \"../secrets.jsonl\"\nmax_records = 1\n",
        )
        .unwrap();
        let no_audit_context = load_browser_handoff_context(&root).unwrap();
        assert!(no_audit_context.audit.is_none());
        fs::remove_dir_all(&root).unwrap();

        let unsafe_root = module_fixture_dir("browser-handoff-audit-injected");
        write_browser_handoff_audit_project(
            &unsafe_root,
            &confirmation,
            &valid_browser_handoff_audit_manifest(2),
        );
        fs::create_dir(unsafe_root.join("audit")).unwrap();
        fs::write(
            unsafe_root.join("audit/handoff.jsonl"),
            "{\"version\":1,\"event\":\"android-browser-handoff\",\"timestampEpochSeconds\":1,\"browserPlanDigest\":\"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"navigationIndex\":1,\"state\":\"cancelled\",\"outcome\":\"P1062\",\"rawUrl\":\"https://secret.invalid\"}\n",
        )
        .unwrap();
        let context = load_browser_handoff_context(&unsafe_root).unwrap();
        let error = browser_handoff_audit_record(&context, "cancelled", "P1062").unwrap_err();
        fs::remove_dir_all(&unsafe_root).unwrap();
        assert!(error.starts_with("P1064"));
        assert!(!error.contains("secret.invalid"));
    }

    fn valid_browser_draft_manifest(digest: &str) -> String {
        format!(
            "[draft]\nversion = \"1\"\nmode = \"user-review-only\"\nbrowser_plan_digest = \"{digest}\"\nnavigation_index = 1\naction = \"message-draft\"\ntitle = \"Documentation question\"\nbody = \"Please review this public documentation question before I manually submit it.\"\nattachment_path = \"attachments/context.txt\"\nmax_review_seconds = 60\n"
        )
    }

    fn write_browser_draft_project(root: &Path, granted: bool, manifest: &str) {
        let capabilities = if granted {
            "browser = [\"plan\", \"draft\"]"
        } else {
            "browser = [\"plan\"]"
        };
        fs::write(
            root.join("padma.toml"),
            format!(
                "[padma]\nname = \"browser-draft-test\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\nlocale = \"en\"\n\n[capabilities]\n{capabilities}\n"
            ),
        )
        .unwrap();
        fs::write(
            root.join("padma-browser.toml"),
            valid_browser_plan_manifest(),
        )
        .unwrap();
        fs::write(root.join("padma-browser-draft.toml"), manifest).unwrap();
    }

    #[test]
    fn browser_draft_emits_an_inert_user_takeover_descriptor() {
        let browser_plan =
            parse_browser_plan_manifest(valid_browser_plan_manifest(), Locale::English).unwrap();
        let root = module_fixture_dir("browser-draft-inert-descriptor");
        write_browser_draft_project(
            &root,
            true,
            &valid_browser_draft_manifest(&browser_plan_digest(&browser_plan)),
        );

        let plan: JsonValue =
            serde_json::from_str(&browser_draft_plan_json(&root).unwrap()).unwrap();
        let inspect = browser_draft_inspect_contents(&root).unwrap();
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(plan["browserDraftPlanVersion"], 1);
        assert_eq!(plan["mode"], "inspection-only");
        assert_eq!(
            plan["browserPlan"]["url"],
            "https://docs.python.org/3/tutorial/"
        );
        assert_eq!(plan["draft"]["action"], "message-draft");
        assert_eq!(plan["draft"]["execution"], "disabled");
        assert_eq!(plan["attachment"]["path"], "attachments/context.txt");
        assert_eq!(plan["attachment"]["metadataOnly"], true);
        assert_eq!(plan["attachmentRead"], "disabled");
        assert_eq!(plan["browser"], "not-started");
        assert_eq!(plan["network"], "disabled");
        assert_eq!(plan["dns"], "disabled");
        assert_eq!(plan["formSubmission"], "disabled");
        assert_eq!(plan["credentialAccess"], "disabled");
        assert_eq!(plan["generatedOutputExecution"], "disabled");
        assert_eq!(plan["userTakeover"]["login"], "user-takeover-required");
        assert_eq!(plan["userTakeover"]["payment"], "user-takeover-required");
        assert!(inspect.starts_with("Padma browser interaction draft (inspection-only)"));
    }

    #[test]
    fn browser_draft_requires_its_narrow_capability_and_exact_reviewed_plan_binding() {
        let browser_plan =
            parse_browser_plan_manifest(valid_browser_plan_manifest(), Locale::English).unwrap();
        let digest = browser_plan_digest(&browser_plan);

        let denied_root = module_fixture_dir("browser-draft-capability-denied");
        write_browser_draft_project(&denied_root, false, &valid_browser_draft_manifest(&digest));
        let denied = browser_draft_plan_json(&denied_root).unwrap_err();
        fs::remove_dir_all(&denied_root).unwrap();
        assert!(denied.starts_with("P1034"));
        assert!(denied.contains("browser:draft"));

        for manifest in [
            valid_browser_draft_manifest(
                "sha256:0000000000000000000000000000000000000000000000000000000000000000",
            ),
            valid_browser_draft_manifest(&digest)
                .replace("navigation_index = 1", "navigation_index = 3"),
        ] {
            let root = module_fixture_dir("browser-draft-plan-binding");
            write_browser_draft_project(&root, true, &manifest);
            let error = browser_draft_plan_json(&root).unwrap_err();
            fs::remove_dir_all(&root).unwrap();
            assert!(error.starts_with("P1065"));
        }
    }

    #[test]
    fn browser_draft_rejects_unsafe_fields_actions_paths_and_execution_modes() {
        let browser_plan =
            parse_browser_plan_manifest(valid_browser_plan_manifest(), Locale::English).unwrap();
        let valid = valid_browser_draft_manifest(&browser_plan_digest(&browser_plan));
        let unsafe_manifests = [
            valid.replace("mode = \"user-review-only\"", "mode = \"execute\""),
            valid.replace("action = \"message-draft\"", "action = \"login\""),
            valid.replace("attachments/context.txt", "../secret.txt"),
            valid.replace(
                "max_review_seconds = 60",
                "max_review_seconds = 60\nscript = \"alert(1)\"",
            ),
            valid.replace(
                "max_review_seconds = 60",
                "max_review_seconds = 60\nselector = \"#password\"",
            ),
            valid.replace(
                "max_review_seconds = 60",
                "max_review_seconds = 60\ncookie = \"secret\"",
            ),
            valid.replace(
                "max_review_seconds = 60",
                "max_review_seconds = 60\nheader = \"Authorization\"",
            ),
            valid.replace(
                "max_review_seconds = 60",
                "max_review_seconds = 60\nraw_url = \"https://evil.invalid\"",
            ),
            valid.replace(
                "max_review_seconds = 60",
                "max_review_seconds = 60\nmax_review_seconds = 61",
            ),
        ];
        for manifest in unsafe_manifests {
            let error = parse_browser_draft_manifest(&manifest, Locale::English).unwrap_err();
            assert!(error.starts_with("P1065"));
        }
    }

    #[test]
    fn browser_draft_output_is_text_only_and_execution_stays_prohibited() {
        let browser_plan =
            parse_browser_plan_manifest(valid_browser_plan_manifest(), Locale::English).unwrap();
        let root = module_fixture_dir("browser-draft-generated-output-inert");
        let manifest = valid_browser_draft_manifest(&browser_plan_digest(&browser_plan)).replace(
            "Please review this public documentation question before I manually submit it.",
            "Generated text remains inert and must never run.",
        );
        write_browser_draft_project(&root, true, &manifest);
        let plan = browser_draft_plan_json(&root).unwrap();
        fs::remove_dir_all(&root).unwrap();

        assert!(plan.contains("Generated text remains inert and must never run."));
        assert!(plan.contains("\"generatedOutputExecution\": \"disabled\""));
        let error = browser_confirmation_error(Locale::English, "P1066", "");
        assert!(error.starts_with("P1066: Browser interaction draft execution is prohibited"));
        assert!(!usage(Locale::English).contains("browser draft execute"));
        assert!(!usage(Locale::English).contains("browser draft run"));
    }

    fn valid_browser_takeover_manifest(digest: &str) -> String {
        format!(
            "[takeover]\nversion = \"1\"\nmode = \"visible-user-takeover-only\"\nbrowser_plan_digest = \"{digest}\"\nnavigation_index = 1\nsensitive_action = \"payment\"\nmax_review_seconds = 60\n"
        )
    }

    fn write_browser_takeover_project(root: &Path, granted: bool, manifest: &str) {
        let capabilities = if granted {
            "browser = [\"plan\", \"takeover\"]"
        } else {
            "browser = [\"plan\"]"
        };
        fs::write(
            root.join("padma.toml"),
            format!(
                "[padma]\nname = \"browser-takeover-test\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\nlocale = \"en\"\n\n[capabilities]\n{capabilities}\n"
            ),
        )
        .unwrap();
        fs::write(
            root.join("padma-browser.toml"),
            valid_browser_plan_manifest(),
        )
        .unwrap();
        fs::write(root.join("padma-browser-takeover.toml"), manifest).unwrap();
    }

    #[test]
    fn browser_takeover_emits_a_visible_manual_checklist_without_execution() {
        let browser_plan =
            parse_browser_plan_manifest(valid_browser_plan_manifest(), Locale::English).unwrap();
        let root = module_fixture_dir("browser-takeover-inert-descriptor");
        write_browser_takeover_project(
            &root,
            true,
            &valid_browser_takeover_manifest(&browser_plan_digest(&browser_plan)),
        );

        let plan: JsonValue =
            serde_json::from_str(&browser_takeover_plan_json(&root).unwrap()).unwrap();
        let inspect = browser_takeover_inspect_contents(&root).unwrap();
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(plan["browserTakeoverPlanVersion"], 1);
        assert_eq!(plan["mode"], "inspection-only");
        assert_eq!(
            plan["browserPlan"]["url"],
            "https://docs.python.org/3/tutorial/"
        );
        assert_eq!(plan["takeover"]["sensitiveAction"], "payment");
        assert_eq!(plan["takeover"]["status"], "user-takeover-required");
        assert_eq!(plan["takeover"]["completion"], "not-collected");
        assert_eq!(plan["takeover"]["execution"], "disabled");
        assert_eq!(plan["visibleHandoff"]["status"], "not-started");
        assert_eq!(plan["network"], "disabled");
        assert_eq!(plan["dns"], "disabled");
        assert_eq!(plan["credentialAccess"], "disabled");
        assert_eq!(plan["pageInspection"], "disabled");
        assert_eq!(plan["formFill"], "disabled");
        assert_eq!(plan["payment"], "disabled");
        assert_eq!(plan["userDecision"], "not-collected");
        assert!(inspect.starts_with("Padma visible browser takeover checklist (inspection-only)"));
    }

    #[test]
    fn browser_takeover_requires_its_narrow_capability_and_exact_reviewed_binding() {
        let browser_plan =
            parse_browser_plan_manifest(valid_browser_plan_manifest(), Locale::English).unwrap();
        let digest = browser_plan_digest(&browser_plan);

        let denied_root = module_fixture_dir("browser-takeover-capability-denied");
        write_browser_takeover_project(
            &denied_root,
            false,
            &valid_browser_takeover_manifest(&digest),
        );
        let denied = browser_takeover_plan_json(&denied_root).unwrap_err();
        fs::remove_dir_all(&denied_root).unwrap();
        assert!(denied.starts_with("P1034"));
        assert!(denied.contains("browser:takeover"));

        for manifest in [
            valid_browser_takeover_manifest(
                "sha256:0000000000000000000000000000000000000000000000000000000000000000",
            ),
            valid_browser_takeover_manifest(&digest)
                .replace("navigation_index = 1", "navigation_index = 3"),
        ] {
            let root = module_fixture_dir("browser-takeover-plan-binding");
            write_browser_takeover_project(&root, true, &manifest);
            let error = browser_takeover_plan_json(&root).unwrap_err();
            fs::remove_dir_all(&root).unwrap();
            assert!(error.starts_with("P1067"));
        }
    }

    #[test]
    fn browser_takeover_rejects_unsafe_fields_actions_and_execution_modes() {
        let browser_plan =
            parse_browser_plan_manifest(valid_browser_plan_manifest(), Locale::English).unwrap();
        let valid = valid_browser_takeover_manifest(&browser_plan_digest(&browser_plan));
        let unsafe_manifests = [
            valid.replace(
                "mode = \"visible-user-takeover-only\"",
                "mode = \"execute\"",
            ),
            valid.replace(
                "sensitive_action = \"payment\"",
                "sensitive_action = \"credential-capture\"",
            ),
            valid.replace(
                "max_review_seconds = 60",
                "max_review_seconds = 60\nselector = \"#password\"",
            ),
            valid.replace(
                "max_review_seconds = 60",
                "max_review_seconds = 60\nscript = \"alert(1)\"",
            ),
            valid.replace(
                "max_review_seconds = 60",
                "max_review_seconds = 60\ncookie = \"secret\"",
            ),
            valid.replace(
                "max_review_seconds = 60",
                "max_review_seconds = 60\nraw_url = \"https://evil.invalid\"",
            ),
            valid.replace(
                "max_review_seconds = 60",
                "max_review_seconds = 60\nmax_review_seconds = 61",
            ),
        ];
        for manifest in unsafe_manifests {
            let error = parse_browser_takeover_manifest(&manifest, Locale::English).unwrap_err();
            assert!(error.starts_with("P1067"));
        }
    }

    #[test]
    fn browser_takeover_execution_remains_an_explicitly_prohibited_boundary() {
        let error = browser_confirmation_error(Locale::English, "P1068", "");
        assert!(error.starts_with("P1068: Browser user-takeover execution is prohibited"));
        assert!(!usage(Locale::English).contains("browser takeover execute"));
        assert!(!usage(Locale::English).contains("browser takeover run"));
    }

    fn valid_ai_tool_plan_manifest() -> &'static str {
        "[agent]\nversion = \"1\"\nmode = \"plan-only\"\nmax_steps = 3\nmax_wall_seconds = 45\nretry_policy = \"never\"\n\n[toolset]\ntools = [\n  \"ai-workflow\",\n  \"file-read\",\n  \"http-request\"\n]\n"
    }

    fn write_ai_tool_plan_project(root: &Path, grants: &str, manifest: &str) {
        fs::write(
            root.join("padma.toml"),
            format!(
                "[padma]\nname = \"tool-plan-test\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\nlocale = \"en\"\n\n[capabilities]\nai = [\"tools\"]\n{grants}\n"
            ),
        )
        .unwrap();
        fs::write(root.join("padma-ai-tools.toml"), manifest).unwrap();
    }

    #[test]
    fn ai_tool_plan_emits_a_deterministic_zero_execution_descriptor() {
        let root = module_fixture_dir("ai-tool-plan-valid");
        write_ai_tool_plan_project(
            &root,
            "network = [\"ai\", \"http\"]\nfilesystem = [\"read\"]",
            valid_ai_tool_plan_manifest(),
        );

        let plan: JsonValue = serde_json::from_str(&ai_tool_plan_json(&root).unwrap()).unwrap();
        let inspect = ai_tool_inspect_contents(&root).unwrap();
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(plan["aiToolPlanVersion"], 1);
        assert_eq!(plan["mode"], "inspection-only");
        assert_eq!(plan["agent"]["mode"], "plan-only");
        assert_eq!(plan["agent"]["maxSteps"], 3);
        assert_eq!(plan["tools"][0]["name"], "ai-workflow");
        assert_eq!(plan["tools"][0]["execution"], "disabled");
        assert_eq!(plan["network"], "disabled");
        assert_eq!(plan["environmentRead"], "disabled");
        assert_eq!(plan["childProcess"], "disabled");
        assert_eq!(plan["toolExecution"], "disabled");
        assert_eq!(plan["agentLoop"], "disabled");
        assert_eq!(plan["backgroundExecution"], "disabled");
        assert_eq!(plan["generatedOutputExecution"], "disabled");
        assert!(inspect.starts_with("Padma AI tool manifest (inspection-only)"));
    }

    #[test]
    fn ai_tool_plan_requires_the_tool_and_each_declared_tool_capability() {
        let missing_tools = module_fixture_dir("ai-tool-plan-missing-tools-capability");
        fs::write(
            missing_tools.join("padma.toml"),
            "[padma]\nname = \"tool-plan-test\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\nlocale = \"en\"\n\n[capabilities]\nnetwork = [\"ai\", \"http\"]\nfilesystem = [\"read\"]\n",
        )
        .unwrap();
        fs::write(
            missing_tools.join("padma-ai-tools.toml"),
            valid_ai_tool_plan_manifest(),
        )
        .unwrap();
        let missing_tools_error = ai_tool_plan_json(&missing_tools).unwrap_err();
        fs::remove_dir_all(&missing_tools).unwrap();
        assert!(missing_tools_error.starts_with("P1034"));
        assert!(missing_tools_error.contains("ai:tools"));

        let missing_http = module_fixture_dir("ai-tool-plan-missing-http-capability");
        write_ai_tool_plan_project(
            &missing_http,
            "network = [\"ai\"]\nfilesystem = [\"read\"]",
            valid_ai_tool_plan_manifest(),
        );
        let missing_http_error = ai_tool_plan_json(&missing_http).unwrap_err();
        fs::remove_dir_all(&missing_http).unwrap();
        assert!(missing_http_error.starts_with("P1034"));
        assert!(missing_http_error.contains("network:http"));
    }

    #[test]
    fn ai_tool_plan_rejects_execution_modes_unsafe_tools_and_duplicate_fields() {
        let execute_mode =
            valid_ai_tool_plan_manifest().replace("mode = \"plan-only\"", "mode = \"run\"");
        assert!(parse_ai_tool_plan_manifest(&execute_mode, Locale::English)
            .unwrap_err()
            .starts_with("P1056"));

        let unsafe_tool =
            valid_ai_tool_plan_manifest().replace("\"file-read\"", "\"browser-login\"");
        let unsafe_error = parse_ai_tool_plan_manifest(&unsafe_tool, Locale::English).unwrap_err();
        assert!(unsafe_error.starts_with("P1056"));
        assert!(!unsafe_error.contains("browser-login"));

        let duplicate = valid_ai_tool_plan_manifest().replacen(
            "max_steps = 3",
            "max_steps = 3\nmax_steps = 4",
            1,
        );
        assert!(parse_ai_tool_plan_manifest(&duplicate, Locale::English)
            .unwrap_err()
            .starts_with("P1056"));
    }

    #[test]
    fn ai_tool_and_agent_execution_remain_an_explicitly_prohibited_boundary() {
        let error = ai_tool_plan_error(Locale::English, "P1057", "");
        assert!(error.starts_with("P1057: AI tool or agent execution is prohibited"));
        assert!(error.contains("no tool or agent will be started"));
        assert!(!usage(Locale::English).contains("ai tools run"));
    }

    fn valid_ai_training_plan_manifest() -> &'static str {
        "[training]\nversion = \"1\"\nmode = \"plan-only\"\nbackend = \"local-adapter-v1\"\ndataset_path = \"datasets/study.jsonl\"\nartifact_path = \"artifacts/study.padma-model\"\nmax_epochs = 3\nmax_wall_seconds = 300\nmax_dataset_bytes = 1048576\nmax_memory_mb = 512\nmax_cpu_threads = 2\n"
    }

    fn write_ai_training_plan_project(root: &Path, granted: bool, manifest: &str) {
        let capabilities = if granted {
            "ai = [\"training-plan\"]"
        } else {
            "ai = [\"tools\"]"
        };
        fs::write(
            root.join("padma.toml"),
            format!(
                "[padma]\nname = \"training-plan-test\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\nlocale = \"en\"\n\n[capabilities]\n{capabilities}\n"
            ),
        )
        .unwrap();
        fs::write(root.join("padma-ai-training.toml"), manifest).unwrap();
    }

    #[test]
    fn ai_training_plan_emits_a_deterministic_zero_execution_descriptor() {
        let root = module_fixture_dir("ai-training-plan-valid");
        write_ai_training_plan_project(&root, true, valid_ai_training_plan_manifest());

        let plan: JsonValue = serde_json::from_str(&ai_training_plan_json(&root).unwrap()).unwrap();
        let inspect = ai_training_inspect_contents(&root).unwrap();
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(plan["aiTrainingPlanVersion"], 1);
        assert_eq!(plan["mode"], "inspection-only");
        assert_eq!(plan["backend"], "local-adapter-v1");
        assert_eq!(plan["dataset"]["path"], "datasets/study.jsonl");
        assert_eq!(plan["dataset"]["read"], "disabled");
        assert_eq!(plan["artifact"]["write"], "disabled");
        assert_eq!(plan["limits"]["maxMemoryMb"], 512);
        assert_eq!(plan["training"], "not-started");
        assert_eq!(plan["localBackend"], "not-started");
        assert_eq!(plan["remoteCompute"], "disabled");
        assert_eq!(plan["datasetRead"], "disabled");
        assert_eq!(plan["artifactWrite"], "disabled");
        assert_eq!(plan["childProcess"], "disabled");
        assert_eq!(plan["network"], "disabled");
        assert!(inspect.starts_with("Padma AI training manifest (inspection-only)"));
    }

    #[test]
    fn ai_training_plan_requires_its_narrow_capability() {
        let root = module_fixture_dir("ai-training-plan-capability-denied");
        write_ai_training_plan_project(&root, false, valid_ai_training_plan_manifest());
        let error = ai_training_plan_json(&root).unwrap_err();
        fs::remove_dir_all(&root).unwrap();
        assert!(error.starts_with("P1034"));
        assert!(error.contains("ai:training-plan"));
    }

    #[test]
    fn ai_training_plan_rejects_execution_mode_and_unsafe_paths() {
        let execute_mode =
            valid_ai_training_plan_manifest().replace("mode = \"plan-only\"", "mode = \"run\"");
        assert!(
            parse_ai_training_plan_manifest(&execute_mode, Locale::English)
                .unwrap_err()
                .starts_with("P1058")
        );

        let unsafe_dataset =
            valid_ai_training_plan_manifest().replace("datasets/study.jsonl", "../secret.jsonl");
        let dataset_error =
            parse_ai_training_plan_manifest(&unsafe_dataset, Locale::English).unwrap_err();
        assert!(dataset_error.starts_with("P1058"));
        assert!(!dataset_error.contains("../secret.jsonl"));

        let unsafe_artifact = valid_ai_training_plan_manifest()
            .replace("artifacts/study.padma-model", "outputs/study.padma-model");
        assert!(
            parse_ai_training_plan_manifest(&unsafe_artifact, Locale::English)
                .unwrap_err()
                .starts_with("P1058")
        );
    }

    #[test]
    fn ai_training_execution_remains_an_explicitly_prohibited_boundary() {
        let error = ai_training_plan_error(Locale::English, "P1059", "");
        assert!(error.starts_with("P1059: AI training execution is prohibited"));
        assert!(error.contains("no dataset will be read"));
        assert!(!usage(Locale::English).contains("ai training run"));
    }
}
