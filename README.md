# পদ্মা / Padma

[![CI](https://github.com/OfficialBiohub/padma-lang/actions/workflows/ci.yml/badge.svg)](https://github.com/OfficialBiohub/padma-lang/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

> **Padma** is a runnable Bangla-English hybrid programming language for learning, local automation, and practical Termux-first development in Bangladesh.

Padma lets Bengali and English keywords express the same language semantics. Learners can write `ধরি` and `দেখাও`; developers can write `let` and `print`; a project can transition between both while the formatter and linter keep code consistent. Bangla source receives Bangla diagnostics, English source receives English diagnostics, and mixed source remains supported.

## Project status

Padma `v0.1.0` is an experimental but executable Rust interpreter. It is appropriate for learning, local scripts, examples, and controlled project experiments. It is **not yet a production-grade replacement for Python, Node.js, or a hardened web framework**. The roadmap and limitations are documented openly in [the production roadmap](docs/PRODUCTION-ROADMAP.md) and [the repository architecture](docs/REPOSITORY-ARCHITECTURE.md).

| Available now | Deliberately limited or planned |
|---|---|
| Bangla-English syntax, UTF-8 source, Bengali digits, REPL, scripts, functions, collections, modules, formatter, linter, JSON diagnostics | Full static type system, published package registry, official cloud runtime, unrestricted browser control, Android APK build adapter |
| Termux installation, files, input, safe process bridge, `yt-dlp` media helper, HTTP/AI/domain helpers, SQLite plan, local server foundation | Automatic Android permission grants, native-code execution, device control, automatic remote deployment |
| Capability grants, package lock/verify, identity/session helpers, GUI/Android plans, Render deployment planning | Production security certification, formal stable-release support policy |

## Start on Termux

Padma is designed to work from an Android phone with Termux. The reproducible source-install path is:

```bash
pkg update
pkg install -y git rust python
git clone https://github.com/OfficialBiohub/padma-lang.git
cd padma-lang
cargo build --release --locked
install -m755 target/release/padma "$PREFIX/bin/padma"
padma --version
```

The repository also provides `install-termux.sh` for the supported source-installer workflow. Obtain an inspectable source checkout first, inspect/check the installer, then run its explicit install action:

```bash
pkg install -y git
git clone https://github.com/OfficialBiohub/padma-lang.git "$HOME/padma-lang"
cd "$HOME/padma-lang"
sed -n '1,240p' install-termux.sh
bash install-termux.sh --check
bash install-termux.sh
padma --version
```

The installer builds a locked local release and installs only `$PREFIX/bin/padma`; it does not edit shell profiles, install optional `yt-dlp`, read secrets, or take browser/device/provider actions. Use `bash install-termux.sh uninstall` to remove only that command. For a phone-only Bangla tutorial covering install, upgrade, recovery, `nano`, project setup, checking, formatting, and Android storage boundaries, read [the Termux-first guide](docs/TERMUX-FIRST-GUIDE-BN.md) and [the release-hardening contract](docs/TERMUX-RELEASE-HARDENING.md).

## First program

Create a file named `hello.pd`:

```padma
ধরি নাম = "রাফি"
ধরি নম্বর = ৭০ + ২৩

যদি নম্বর >= 90 {
    দেখাও "{নাম}, তুমি পেয়েছ: {নম্বর}"
} নইলে {
    দেখাও "{নাম}, আবার চেষ্টা করো।"
}
```

Run it like Python:

```bash
padma hello.pd
```

You can also open the interactive shell:

```text
$ padma
Padma 0.1.0 (Bangla-English hybrid programming language)
padma> দেখাও ২ + ৩
5
padma> exit()
```

## Core commands

| Command | Purpose |
|---|---|
| `padma <file.pd>` | Run one Padma source file. |
| `padma` | Open the interactive REPL. |
| `padma init <folder>` | Create a manifest-based local project. |
| `padma .` | Run the project entrypoint declared in `padma.toml`. |
| `padma check [--json] <file.pd>` | Run non-executing syntax and selected semantic checks. |
| `padma fmt [--check] <file.pd>` | Format valid source or report needed changes. |
| `padma lint [--json] <file.pd>` | Run non-executing source-style checks. |
| `padma capabilities [project]` | Inspect project capability grants without running code. |
| `padma gui inspect\|plan [project]` | Validate a local static GUI manifest without launching a renderer. |
| `padma android inspect\|plan [project]` | Validate Android build metadata without building, signing, or installing an APK. |
| `padma render plan\|api-plan [project]` | Validate Render release metadata; planning mode sends no provider request. |

Read [`padma --help`](src/main.rs) in a local build for the authoritative CLI output.

## Language snapshot

| Concept | Bangla form | English form |
|---|---|---|
| Variable | `ধরি নাম = "রাফি"` | `let name = "Rafi"` |
| Output | `দেখাও নাম` | `print name` |
| Condition | `যদি সত্য { ... } নইলে { ... }` | `if true { ... } else { ... }` |
| Function | `ফাংশন যোগ(a, b) { ফেরত a + b }` | `function add(a, b) { return a + b }` |
| Boolean/null | `সত্য`, `মিথ্যা`, `কিছুইনা` | `true`, `false`, `none` |

The detailed grammar and runtime behavior live in [the language specification](docs/LANGUAGE-SPEC.md). Runnable programs are in [`examples/`](examples/README.md).

## Security model

Padma local projects use explicit capability grants in `padma.toml`. Sensitive operations such as file access, HTTP, AI requests, process bridges, media tools, SQLite, identity helpers, local server planning, GUI planning, Android planning, and deployment planning are intentionally capability-gated. A plan command validates metadata; it does not silently gain device access, approve permissions, or perform remote work.

| Read next | Subject |
|---|---|
| [Capability security](docs/CAPABILITY-SECURITY.md) | Manifest grants, filesystem scope, and deny-by-default model |
| [Diagnostics](docs/DIAGNOSTICS.md) | Stable bilingual diagnostic codes |
| [Standard library](docs/STANDARD-LIBRARY.md) | Text, math, file, JSON, URL, path, time, and random helpers |
| [Projects](docs/PROJECTS.md) | `padma.toml`, lockfile, modules, and local project boundaries |
| [Editor tooling](docs/EDITOR-TOOLING.md) | Tree-sitter, VS Code extension, and LSP |
| [Platform guides](docs/README.md#platform-and-application-planning) | GUI, Android, identity, SQLite, Render, and deployment boundaries |
| [M9 AI and browser design](docs/M9-AI-BROWSER-DESIGN.md) | Planned provider-neutral AI workflow and browser-plan security boundaries; no new runtime action enabled |
| [Padma Agent Skill](docs/AGENT-SKILL.md) | Reusable language-first engineering guidance for agents and maintainers |

## Repository map

```text
src/        Padma interpreter and public Rust library API
examples/   Runnable copy-pasteable Padma projects
docs/       Language, platform, security, and contributor documentation
tooling/    LSP, Tree-sitter grammar, and VS Code extension
wasm/       Optional browser bridge
playground/ Optional demonstration client
packaging/  Downstream distribution recipes
scripts/    Reproducible quality and maintenance commands
skills/     Reusable agent-engineering guidance; not a runtime dependency
```

The complete responsibility and compatibility map is in [Repository Architecture](docs/REPOSITORY-ARCHITECTURE.md).

## Contribute safely

Padma welcomes contributors. Begin with [CONTRIBUTING.md](CONTRIBUTING.md), then use the issue forms for bugs or language proposals. The versioned [Padma Agent Skill](docs/AGENT-SKILL.md) provides project-specific workflow guidance for agent-assisted contributions. Security reports must follow [SECURITY.md](SECURITY.md), not public issues. Community expectations are documented in [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), while issue-routing guidance is in [SUPPORT.md](SUPPORT.md).

Before submitting a pull request, run:

```bash
./scripts/verify-repository.sh
```

## License

Padma is released under the [MIT License](LICENSE). User-visible release history is maintained in [CHANGELOG.md](CHANGELOG.md).
