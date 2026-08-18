// Padma v0.1.0 — a small, dependency-free Bangla-English language MVP.
//
// This executable intentionally implements a narrow but complete vertical slice:
// UTF-8 source, Bangla/English keyword aliases, expressions, variables, print,
// conditionals, string interpolation, and localized diagnostics.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::env;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::Value as JsonValue;

static RANDOM_COUNTER: AtomicU64 = AtomicU64::new(0);

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
        collection: Expr,
        body: Vec<Stmt>,
        position: Position,
    },
    Function {
        name: String,
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
        let name = match self.advance().clone() {
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
        Ok(Stmt::Let { name, value })
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
        let name = match self.advance().clone() {
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
            collection,
            body,
            position,
        })
    }

    fn function_statement(&mut self) -> Result<Stmt, PadmaError> {
        let name = match self.advance().clone().kind {
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
        Ok(Stmt::Function { name, params, body })
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

    fn run(&mut self, program: &[Stmt]) -> Result<(), PadmaError> {
        for statement in program {
            self.execute(statement)?;
            if self.return_value.is_some() {
                break;
            }
        }
        Ok(())
    }

    fn execute(&mut self, statement: &Stmt) -> Result<(), PadmaError> {
        match statement {
            Stmt::Let { name, value, .. } => {
                let value = self.evaluate(value)?;
                self.environment.insert(name.clone(), value);
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
                if !self.environment.contains_key(name) {
                    return Err(error_for(self.locale, "P1007", *position, name));
                }
                let value = self.evaluate(value)?;
                self.environment.insert(name.clone(), value);
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
                self.run(branch)?;
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
                    self.run(body)?;
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
                let previous_value = self.environment.remove(name);
                let result = (|| {
                    for value in values {
                        self.environment.insert(name.clone(), value);
                        self.run(body)?;
                        if self.return_value.is_some() {
                            break;
                        }
                    }
                    Ok(())
                })();
                if let Some(value) = previous_value {
                    self.environment.insert(name.clone(), value);
                } else {
                    self.environment.remove(name);
                }
                result?;
            }
            Stmt::Function { name, params, body } => {
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
                self.environment
                    .get(name)
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
                if name == "http.get" {
                    if arguments.len() != 1 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let url = self.evaluate(&arguments[0])?;
                    let url = expect_string(&url, self.locale, *position, "url")?;
                    self.require_capability("network:http", name, *position)?;
                    if !(url.starts_with("https://") || url.starts_with("http://")) {
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
                        .environment
                        .get(list_name)
                        .is_some_and(|value| matches!(value, Value::List(_)))
                    {
                        if arguments.len() != 1 {
                            return Err(error_for(self.locale, "P1009", *position, name));
                        }
                        let index = self.evaluate(&arguments[0])?;
                        let index = expect_collection_index(&index, self.locale, *position)?;
                        let Value::List(values) = self.environment.get(list_name).unwrap() else {
                            unreachable!("list check above guarantees a list")
                        };
                        return values.get(index).cloned().ok_or_else(|| {
                            error_for(self.locale, "P1027", *position, &index.to_string())
                        });
                    }
                }
                if let Some(list_name) = name.strip_suffix(".set") {
                    if self
                        .environment
                        .get(list_name)
                        .is_some_and(|value| matches!(value, Value::List(_)))
                    {
                        if arguments.len() != 2 {
                            return Err(error_for(self.locale, "P1009", *position, name));
                        }
                        let index = self.evaluate(&arguments[0])?;
                        let index = expect_collection_index(&index, self.locale, *position)?;
                        let value = self.evaluate(&arguments[1])?;
                        let Value::List(values) = self.environment.get_mut(list_name).unwrap()
                        else {
                            unreachable!("list check above guarantees a list")
                        };
                        let slot = values.get_mut(index).ok_or_else(|| {
                            error_for(self.locale, "P1027", *position, &index.to_string())
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
                    let list = self
                        .environment
                        .get_mut(list_name)
                        .ok_or_else(|| error_for(self.locale, "P1007", *position, list_name))?;
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
                    let list = self
                        .environment
                        .get_mut(list_name)
                        .ok_or_else(|| error_for(self.locale, "P1007", *position, list_name))?;
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
                    let collection = self.environment.get(collection_name).ok_or_else(|| {
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
                    let collection = self.environment.get(collection_name).ok_or_else(|| {
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
                        .environment
                        .get(map_name)
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
                    let map = self
                        .environment
                        .get_mut(map_name)
                        .ok_or_else(|| error_for(self.locale, "P1007", *position, map_name))?;
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
                let previous_return = self.return_value.take();
                for (parameter, value) in parameters.iter().zip(values) {
                    self.environment.insert(parameter.clone(), value);
                }
                self.run(&body)?;
                let result = self.return_value.take().unwrap_or(Value::Null);
                self.environment = previous_environment;
                self.return_value = previous_return;
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
                .environment
                .get(name)
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
}

const PROCESS_CAPABILITIES: [&str; 6] = ["git", "yt-dlp", "curl", "ffmpeg", "python", "python3"];

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
        "filesystem" => &["read", "write"],
        "network" => &["http"],
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
    for (line_number, raw_line) in source.lines().enumerate() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            if section != "padma" && section != "dependencies" && section != "capabilities" {
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
        if section == "dependencies" || key == "dependencies" {
            return Err(format!(
                "P1033: dependencies are not supported yet (line {})",
                line_number + 1
            ));
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
    };
    if !matches!(manifest.locale.as_str(), "auto" | "bn" | "en") {
        return Err("P1032: `locale` must be `auto`, `bn`, or `en`".to_string());
    }
    Ok(manifest)
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
    };
    fs::create_dir_all(directory.join("src"))
        .map_err(|error| format!("P1032: cannot create project source directory: {error}"))?;
    fs::write(
        directory.join("padma.toml"),
        format!(
            "[padma]\nname = \"{}\"\nversion = \"{}\"\nentry = \"{}\"\nlocale = \"{}\"\n\n# Project mode denies sensitive actions until they are granted below.\n[capabilities]\nfilesystem = []\nnetwork = []\nprocess = []\nmedia = []\n",
            manifest.name, manifest.version, manifest.entry, manifest.locale
        ),
    )
    .map_err(|error| format!("P1032: cannot write manifest: {error}"))?;
    fs::write(
        directory.join("padma.lock"),
        format!(
            "# Padma lockfile v1\nname = \"{}\"\nversion = \"{}\"\n",
            manifest.name, manifest.version
        ),
    )
    .map_err(|error| format!("P1032: cannot write lockfile: {error}"))?;
    fs::write(
        directory.join("src/main.pd"),
        "# padma:locale=bn\nদেখাও \"পদ্ম project শুরু হয়েছে\"\n",
    )
    .map_err(|error| format!("P1032: cannot write starter source: {error}"))?;
    Ok(manifest)
}

fn project_source_with_locale(source: String, locale: &str) -> String {
    match locale {
        "bn" => format!("# padma:locale=bn\n{source}"),
        "en" => format!("# padma:locale=en\n{source}"),
        _ => source,
    }
}

fn check_source(source: &str) -> Result<Locale, Vec<PadmaError>> {
    let locale = Locale::from_source(source);
    let tokens = Lexer::new(source, locale)
        .tokenize()
        .map_err(|error| vec![error])?;
    let (_, errors) = Parser::new(tokens, locale).parse_recovering();
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

fn usage(locale: Locale) -> &'static str {
    match locale {
        Locale::Bangla => {
            "ব্যবহার: padma [file.pd|.] অথবা padma <run|check|fmt|ast> <file.pd>\n\nকমান্ড:\n  padma                 interactive shell চালু করুন\n  padma <file.pd>       Padma script চালান\n  padma .               padma.toml project চালান\n  padma init [folder]   নতুন Padma project তৈরি করুন\n  padma capabilities <project>  project permission দেখুন\n  padma check --json <file.pd>  JSON diagnostic দিন\n  padma fmt <file.pd>   source format করুন\n  padma fmt --check <file.pd>  source পরিবর্তন দরকার কি না দেখুন\n  padma --version       version দেখুন\n  padma --help          এই help দেখুন\n\nউদাহরণ:\n  padma init আমার-project\n  padma examples/hello-bn.pd\n  padma check examples/hello-en.pd\n  padma check --json examples/hello-en.pd\n  padma fmt examples/hello-en.pd\n  padma ast examples/mixed.pd\n"
        }
        Locale::English => {
            "Usage: padma [file.pd|.] or padma <run|check|fmt|ast> <file.pd>\n\nCommands:\n  padma                 open the interactive shell\n  padma <file.pd>       run a Padma script\n  padma .               run a padma.toml project\n  padma init [folder]   create a new Padma project\n  padma capabilities <project>  inspect project permissions\n  padma check --json <file.pd>  emit JSON diagnostics\n  padma fmt <file.pd>   format a source file in place\n  padma fmt --check <file.pd>  report whether formatting is needed\n  padma --version       show the installed version\n  padma --help          show this help\n\nExamples:\n  padma init my-project\n  padma examples/hello-en.pd\n  padma check examples/hello-en.pd\n  padma check --json examples/hello-en.pd\n  padma fmt examples/hello-en.pd\n  padma ast examples/mixed.pd\n"
        }
    }
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
    let (command, path, json_diagnostics, format_check) = match arguments.as_slice() {
        [_, path] if path.ends_with(".pd") => ("run", path.as_str(), false, false),
        [_, command, path] => (command.as_str(), path.as_str(), false, false),
        [_, command, flag, path] if command == "check" && flag == "--json" => {
            ("check", path.as_str(), true, false)
        }
        [_, command, path, flag] if command == "check" && flag == "--json" => {
            ("check", path.as_str(), true, false)
        }
        [_, command, flag, path] if command == "fmt" && flag == "--check" => {
            ("fmt", path.as_str(), false, true)
        }
        [_, command, path, flag] if command == "fmt" && flag == "--check" => {
            ("fmt", path.as_str(), false, true)
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
    if command == "fmt" {
        if let Err(errors) = check_source(&source) {
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
                println!("Examples: দেখাও ২ + ৩ | print \"hello\" | ধরি x = 10");
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
        match compile(&source) {
            Ok((program, locale)) => {
                interpreter.locale = locale;
                interpreter.return_value = None;
                let output_start = interpreter.output.len();
                match interpreter.run(&program) {
                    Ok(()) => {
                        for output in &interpreter.output[output_start..] {
                            println!("{output}");
                        }
                    }
                    Err(error) => eprintln!("{}", error.message),
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
    fn formatter_is_idempotent_and_ignores_braces_inside_strings() {
        let source =
            "function greeting() {\nprint \"{name}\"   \nif true {\nprint \"ok\"\n}\n}\n\n";
        let expected = "function greeting() {\n    print \"{name}\"\n    if true {\n        print \"ok\"\n    }\n}\n";
        let formatted = format_source(source);
        assert_eq!(formatted, expected);
        assert_eq!(format_source(&formatted), formatted);
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
        assert!(dependencies.starts_with("P1033"));

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
    fn parses_and_validates_manifest_capabilities() {
        let manifest = parse_project_manifest(
            "[padma]\nname = \"demo\"\nversion = \"0.1.0\"\nentry = \"main.pd\"\n\n[capabilities]\nfilesystem = [\"read\", \"write\"]\nnetwork = [\"http\"]\nprocess = [\"git\", \"yt-dlp\"]\nmedia = [\"download\"]\n",
        )
        .unwrap();
        assert!(manifest.capabilities.contains("filesystem:read"));
        assert!(manifest.capabilities.contains("process:git"));
        assert_eq!(manifest.capabilities.len(), 6);

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
    fn counts_repl_braces_without_counting_strings_or_comments() {
        assert_eq!(brace_delta("if true {"), 1);
        assert_eq!(brace_delta("print \"{not a block}\" # }"), 0);
        assert_eq!(brace_delta("}"), -1);
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
}
