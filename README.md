# পদ্মা / Padma

> **Padma** is an experimental Bangla-English hybrid programming language for learning, automation, web work, and future production tooling in Bangladesh.

Padma allows Bengali and English keywords to represent the **same internal syntax**. A learner can write `ধরি` and `দেখাও`; an experienced developer can write `let` and `print`; a team can temporarily mix them while the future formatter/linter guides the project toward a consistent style.

**Current status:** `v0.1.0` is a runnable, dependency-free Rust MVP. It is not yet suitable for production systems. It exists to validate the core language experience before expanding the compiler, standard library, editor extension, package manager, and backend targets.

| What works in v0.1.0 | Bangla keyword | English keyword |
|---|---|---|
| Declare a variable | `ধরি` | `let` |
| Show output | `দেখাও` | `print` |
| Conditional branch | `যদি` / `নইলে` | `if` / `else` |
| Boolean values | `সত্য` / `মিথ্যা` | `true` / `false` |
| Number, string, arithmetic, comparison | Yes | Yes |
| String interpolation | `"নাম: {নাম}"` | `"Name: {name}"` |
| Bengali digits | `০`–`৯` | `0`–`9` |
| Localized diagnostics | Bengali source → Bengali error | English source → English error |

## Quick start

The compiler is written in Rust. Install a stable Rust toolchain, clone this repository, then build it locally.

```bash
git clone https://github.com/OfficialBiohub/padma-lang.git
cd padma-lang
cargo run -- run examples/hello-bn.pd
```

For an optimized binary:

```bash
cargo build --release
./target/release/padma run examples/hello-en.pd
./target/release/padma check examples/mixed.pd
```

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

## Commands

| Command | Purpose |
|---|---|
| `padma run <file.pd>` | Parse and interpret a Padma source file. |
| `padma check <file.pd>` | Check the source syntax without executing it. |
| `padma ast <file.pd>` | Show the compiler’s current abstract syntax tree. |
| `padma --version` | Print the installed compiler version. |

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

The MVP has no third-party compiler dependencies. Unit tests cover Bengali, English, mixed syntax, Bengali digits, conditional execution, string interpolation, localization, and division-by-zero handling.

## Scope and roadmap

| Release | Planned outcome |
|---|---|
| `v0.1` | Interpreter MVP: variables, output, arithmetic, conditions, diagnostics. |
| `v0.2` | Functions, lists, maps, modules, assignments, formatter, stronger type errors. |
| `v0.3` | TypeScript code generator, browser playground, Node.js package bridge. |
| `v0.4` | Python bridge for data/AI workflows, project manifest and lockfile. |
| `v0.5` | VS Code extension, Tree-sitter grammar, language server. |
| `v1.0` | Stable language specification, package governance, security model, reproducible releases. |

AI training, full web frameworks, database connectors, quantum SDK wrappers, native LLVM output, and a public package registry are deliberately **not** part of `v0.1`. They will be added only after the language core, testing, security boundaries, and interoperability model are stable.

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request. In particular, every change to syntax or semantics needs a test and an RFC discussion where appropriate.

## License

Padma is released under the [MIT License](LICENSE).
