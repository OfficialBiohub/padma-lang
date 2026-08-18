# পদ্মা / Padma

> **Padma** is an experimental Bangla-English hybrid programming language for learning, automation, web work, and future production tooling in Bangladesh.

Padma allows Bengali and English keywords to represent the **same internal syntax**. A learner can write `ধরি` and `দেখাও`; an experienced developer can write `let` and `print`; a team can temporarily mix them while the future formatter/linter guides the project toward a consistent style.

**Current status:** `v0.1.0` is a runnable, dependency-free Rust MVP. It is not yet suitable for production systems. It exists to validate the core language experience before expanding the compiler, standard library, editor extension, package manager, and backend targets.

| What works in v0.1.0 | Bangla keyword | English keyword |
|---|---|---|
| Declare a variable | `ধরি` | `let` |
| Show output | `দেখাও` | `print` |
| Conditional branch | `যদি` / `নইলে` | `if` / `else` |
| Repeating loop and assignment | `যতক্ষণ`, `=` | `while`, `=` |
| Function and return | `ফাংশন`, `ফেরত` | `function`, `return` |
| Boolean values | `সত্য` / `মিথ্যা` | `true` / `false` |
| Empty/null value | `কিছুইনা` | `none` |
| Number, string, arithmetic, comparison | Yes | Yes |
| String interpolation | `"নাম: {নাম}"` | `"Name: {name}"` |
| Bengali digits | `০`–`৯` | `0`–`9` |
| Lists | Indexing plus `.get()`, `.set()`, `.push()`, `.remove()`, `.len()`, and `.contains()` | Same APIs |
| Maps/dictionaries | Text-key maps with `.get()` and `.set()` | Text-key maps with `.get()` and `.set()` |
| Localized diagnostics | Bengali source → Bengali error | English source → English error |

## Quick start

The compiler is written in Rust. Install a stable Rust toolchain, clone this repository, then build it locally.

```bash
git clone https://github.com/OfficialBiohub/padma-lang.git
cd padma-lang
cargo run -- examples/hello-bn.pd
```

For an optimized binary:

```bash
cargo build --release
./target/release/padma examples/hello-en.pd
./target/release/padma check examples/mixed.pd
```

### Install directly in Termux

From Termux, the repository includes a reproducible installer:

```bash
curl -fsSL https://raw.githubusercontent.com/OfficialBiohub/padma-lang/main/install-termux.sh | bash
padma examples/hello-bn.pd
```

After installation, `padma --version` prints the installed version and `padma` opens an interactive REPL.

### Python-style interactive shell

When you type `padma` without a file, Padma opens an interactive shell. Each complete line is compiled and executed immediately, and variables remain available for later lines:

```text
$ padma
Padma 0.1.0 (Bangla-English hybrid programming language)
Interactive shell: help, copyright, credits, license; exit with exit() or বের হও.
padma> দেখাও ২ + ৩
5
padma> ধরি নাম = "রাফি"
padma> দেখাও "হ্যালো {নাম}"
হ্যালো রাফি
padma> help
padma> exit()
```

The shell accepts `help` or `সাহায্য`, `copyright`, `credits`, and `license`. To leave it, use `exit()`, `quit()`, `exit`, or `বের হও`.

For a multi-line `if`, `while`, or function block, continue entering lines after Padma changes the prompt to `...`; it runs the buffered block after the closing `}`.

### Why `pkg install padma -y` is not available yet

`pkg` and `apt` do not install arbitrary GitHub repositories. They install only signed packages that have been built and published in the Termux repositories configured on the device. The `packaging/termux/packages/padma/build.sh` file is a **recipe for a future Termux submission**; placing a recipe in this repository does not make `padma` appear in `pkg search`.

If `pkg update` or `apt update` itself fails, first check the Termux installation and mirror. The deprecated Google Play build and old Bintray mirrors can produce repository errors. In a current F-Droid or GitHub Termux installation, run `termux-info`, then use `termux-change-repo` to select a working main mirror and run `pkg upgrade`. Until an upstream package is accepted and published, use the installer above; it builds Padma directly and installs the binary into `$PREFIX/bin`.

For language rules, see [the specification draft](docs/LANGUAGE-SPEC.md). For stable error-code meanings and `padma check` behavior, see [the diagnostics reference](docs/DIAGNOSTICS.md). The current release is an executable interpreter core; functions, collections, modules, and static type checking remain active implementation milestones rather than undocumented claims.

## Your first Padma program

```padma
# hello-bn.pd
ধরি নাম = "রাফি"
ধরি নম্বর = ৭০ + ২৩

যদি নম্বর >= 90 {
    দেখাও "{নাম}, তুমি পেয়েছ: {নম্বর}"
} নইলে {
    দেখাও "{নাম}, আবার চেষ্টা করো।"
}
```

The equivalent English program has identical meaning:

```padma
# hello-en.pd
let name = "Rafi"
let score = 70 + 23

if score >= 90 {
    print "{name}, your score is: {score}"
} else {
    print "{name}, try again."
}
```

### Updating values and repeating work

Padma variables can be updated after declaration, and both Bengali and English loop keywords are accepted:

```padma
ধরি i = ০
যতক্ষণ i < ৩ {
    দেখাও i
    i = i + ১
}
```

The interpreter enforces a loop-iteration safety limit so an accidental infinite loop cannot run forever on Termux.

### Functions

Functions accept parameters and return a value:

```padma
ফাংশন যোগ(a, b) {
    ফেরত a + b
}
ধরি ফল = যোগ(২, ৩)
দেখাও ফল
```

### Maps / dictionaries

Padma maps use text keys. Read a value with `.get(key)` and update or add a value with `.set(key, value)`:

```padma
let profile = {"name": "Rafi", "class": 6}
print profile.get("name")
profile.set("class", 7)
print profile.get("class")
```

The Bengali and mixed forms work the same way:

```padma
ধরি তথ্য = {"নাম": "রাফি", "বিষয়": "Padma"}
দেখাও তথ্য.get("নাম")
তথ্য.set("স্তর", ৬)
দেখাও তথ্য.get("স্তর")
```

### Lists and collection access

Lists and maps can be read with bracket syntax. Lists use a zero-based, non-negative integer index; maps use a text key. Invalid indexes and missing keys stop safely with a localized diagnostic instead of silently producing an incorrect value.

```padma
let tasks = ["learn", "practice"]
tasks.push("build")
tasks.set(0, "study")

print tasks[0]                       # study
print tasks.len()                    # 3
print tasks.contains("practice")    # true
print tasks.remove(1)                # practice

let profile = {"name": "Rafi"}
print profile["name"]
```

The same collection APIs work in Bangla source code:

```padma
ধরি সংখ্যা = [১০, ২০, ৩০]
দেখাও সংখ্যা[২]
দেখাও সংখ্যা.get(০)
```

### Empty values

Use `none` in English source or `কিছুইনা` in Bangla source when a value is intentionally absent. It is false in an `if` condition. A function that reaches the end without `return` / `ফেরত` also returns `none`.

```padma
ধরি নোট = কিছুইনা
যদি নোট {
  দেখাও নোট
} নইলে {
  দেখাও "এখনো কোনো নোট নেই"
}
```

### Reusable modules

Split a larger Padma program into nearby `.pd` files. Use `import "file.pd"` or `ইমপোর্ট "file.pd"`; imported variables and functions become available to the current file. A module is loaded only once, even if it is imported repeatedly.

```padma
# math.pd
function double(value) {
  return value * 2
}
```

```padma
# main.pd, in the same folder
import "math.pd"
print double(21)
```

For safety, module paths must remain relative to the current folder tree, must end in `.pd`, and cannot use `..` or absolute paths. Circular imports are rejected with a localized diagnostic. Run the English and Bangla examples with:

```bash
cd examples/modules
padma main.pd
padma মডিউল-demo.pd
```

### Interactive input

Termux scripts can read a line from the user with the built-in `input()` function:

```padma
ধরি url = input("ভিডিও URL দিন: ")
দেখাও "আপনি দিয়েছেন: {url}"
```

Run it in the Python-style form:

```bash
padma examples/input-demo.pd
```

### Padma media downloader

Padma can call an installed `yt-dlp` backend through a restricted, argument-safe builtin. The URL and output path are passed as separate arguments; shell interpolation is not used.

```padma
ধরি url = input("YouTube URL দিন: ")
ধরি result = media.download(url, "downloads/%(title)s.%(ext)s")
দেখাও result
```

Run it from the repository directory:

```bash
mkdir -p downloads
padma examples/youtube-download.pd
```

Use this only for content you own or are authorized to download and in compliance with the source platform's terms. Padma does not bypass DRM or access controls. If `yt-dlp` is missing, the program returns a localized process-start error instead of executing an arbitrary command.

## Commands

| Command | Purpose |
|---|---|
| `padma <file.pd>` | Parse and interpret a Padma source file, like `python file.py`. |
| `padma` | Open the interactive Padma REPL. |
| `padma run <file.pd>` | Backward-compatible explicit run form. |
| `padma check <file.pd>` | Check the source syntax without executing it. |
| `padma ast <file.pd>` | Show the compiler’s current abstract syntax tree. |
| `padma --version` | Print the installed compiler version. |
| `padma --help` | Show CLI usage and interactive-shell commands. |

## Diagnostic language

Padma detects the source language from its keywords. A Bengali source file receives Bengali diagnostics, while an English source file receives English diagnostics. Add one of the following comments at the top of a file when you need to override automatic detection:

```padma
# padma:locale=bn
# padma:locale=en
```

Example Bengali diagnostic:

```text
ত্রুটি[P1007]: `বয়স` নামে কোনো variable পাওয়া যায়নি
  --> student.pd:2:8
   |
2 | দেখাও বয়স
   |        ^ এই স্থানে
   = পরামর্শ: আগে এটি ঘোষণা করুন: `ধরি বয়স = ...`
```

## Architecture in this MVP

```text
UTF-8 .pd source
  → Bangla/English lexer aliases
  → parser
  → abstract syntax tree
  → small interpreter
  → output or structured diagnostic
```

The language intentionally maps `ধরি` and `let` to one internal token, rather than maintaining separate Bengali and English grammars. This keeps semantics identical and makes future formatters, type checkers, and target generators substantially more reliable.

## Development

```bash
cargo fmt
cargo test
cargo build --release
```

The MVP has no third-party compiler dependencies. Unit tests cover Bengali, English, mixed syntax, Bengali digits, conditional execution, string interpolation, localization, diagnostics recovery, module loading, null values, safe collection indexing/mutation, assignment, while loops, and loop safety limits.

## Scope and roadmap

The full production-readiness plan, release gates, security boundaries, and ordered implementation work are documented in [docs/PRODUCTION-ROADMAP.md](docs/PRODUCTION-ROADMAP.md).

| Release | Planned outcome |
|---|---|
| `v0.1` | Interpreter MVP: variables, output, arithmetic, conditions, diagnostics. |
| `v0.2` | Functions, lists, interactive input, maps, modules, formatter, stronger type errors. |
| `v0.3` | TypeScript code generator, browser playground, Node.js package bridge. |
| `v0.4` | Python bridge for data/AI workflows, project manifest and lockfile. |
| `v0.5` | VS Code extension, Tree-sitter grammar, language server. |
| `v1.0` | Stable language specification, package governance, security model, reproducible releases. |

AI training, full web frameworks, database connectors, quantum SDK wrappers, native LLVM output, and a public package registry are deliberately **not** part of `v0.1`. They will be added only after the language core, testing, security boundaries, and interoperability model are stable.

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request. In particular, every change to syntax or semantics needs a test and an RFC discussion where appropriate.

## License

Padma is released under the [MIT License](LICENSE).
