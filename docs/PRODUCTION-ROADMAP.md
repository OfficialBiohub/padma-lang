# Padma Production-Readiness Roadmap

**Status:** Draft for implementation

**Language:** Bangla-first, English-compatible

**Primary target:** Termux on Android, followed by Linux desktop and CI environments

> **Production ready** does not mean that every possible field, framework, or AI model is built into the language. It means that Padma has a stable and documented core, predictable security boundaries, reproducible releases, strong automated tests, and an honest compatibility policy.

## 1. Product definition

Padma is a lightweight, Bangla-English hybrid scripting language designed for Bangladeshi learners and practical Termux automation. A Class 6 learner should be able to begin with Bangla keywords, while an experienced programmer should be able to use conventional English syntax and familiar tools. Both forms must compile to exactly the same language semantics.

The project will not claim production readiness until the following guarantees are true.

| Area | Production requirement | Acceptance evidence |
|---|---|---|
| Language core | Versioned grammar, stable AST semantics, predictable values and control flow | Conformance test suite, syntax reference, compatibility fixtures |
| Diagnostics | Every user-visible compiler/runtime failure has a stable P-code and Bangla-English message | Snapshot tests for both source locales |
| Security | Filesystem, process, network, media, and future bridge access follow explicit capability rules | Negative security tests and documented permissions |
| Reliability | No panic from valid or malformed source; bounded resource-sensitive operations | Fuzzing, property tests, regression tests, benchmarks |
| Distribution | Released binaries are attributable to source and have checksums, notes, and tested installers | Signed release artifacts, SBOM, provenance, reproducible-build comparison |
| Tooling | Format, check, lint, editor syntax support, and machine-readable diagnostics exist | CLI integration tests and editor extension tests |
| Documentation | Bangla-first tutorials, English reference, API examples, contribution guidance | Documentation build/link checks and runnable examples |

## 2. Current baseline

Padma currently has a Rust interpreter, Bangla and English aliases for core keywords, Bengali digits, variables, conditionals, bounded `while` loops, functions, null values, safely indexed/mutable lists, text-key maps, input, safe file output, HTTP GET, constrained process execution, media download integration, an interactive shell, and safe local imports. The current language remains an **early interpreter release**, not a production-ready universal platform. Important missing foundations include slicing and iteration, JSON, a standard library, namespaces, static checks, formatting, a manifest, editor tooling, typed bridges, reproducible release artifacts, and a comprehensive security policy.

## 3. Delivery principles

Every milestone must retain four principles.

| Principle | Implementation rule |
|---|---|
| Bangla-first without isolation | Bengali and English forms share one grammar, runtime, test suite, and documentation set. |
| Termux-first without lock-in | Core compiler behaviour must work offline; Android-specific capabilities stay behind explicit APIs. |
| Secure by default | No arbitrary shell, unrestricted path, secret, package, or network escalation through ordinary language syntax. |
| Small, tested increments | A feature is not complete until documented, tested in both locales, fuzzed or negatively tested when security-sensitive, and released with a compatibility note. |

## 4. Ordered implementation milestones

### M1 — Compiler and diagnostics stabilization

**Objective:** Make source handling reliable before increasing language surface area.

| Deliverable | Required work | Release gate |
|---|---|---|
| Source model | Explicit source IDs, byte spans, line/column spans, and file-aware diagnostics | Errors show the actual imported file and exact span |
| Parser quality | Newline/block recovery and multiple syntax diagnostics where safe | Malformed programs never panic and tests cover recovery |
| Locale system | Directive-driven locale, keyword detection improvements, locale inheritance for modules | Bilingual snapshot coverage |
| Error catalogue | Stable P1000-series public codes with remediation hints | One catalogue page and code-level tests |
| CLI stability | `run`, direct `.pd`, `check`, `ast`, REPL, `--version`, `--help` contract | Shell smoke tests on Linux and Termux-compatible CI |

### M2 — Core values and collections

**Objective:** Complete the everyday language primitives needed for scripts.

| Deliverable | Required work | Release gate |
|---|---|---|
| Values | `null` / `শূন্য`, integer-safe numeric policy, escaped text, equality rules | Conformance tests for every value pair |
| Lists | Index read/write, negative-index policy, slices, append/remove/length/contains | Bounds and mutation error tests |
| Maps | Key existence, delete, keys/values/items, nested values, stable display | Deterministic behaviour tests |
| Iteration | `for` / `প্রতি` over lists, maps, text, and ranges | Loop termination and scope tests |
| Operators | Modulo, exponentiation decision, logical operations, precedence table | Parser and runtime precedence matrix |

### M3 — Safe standard library

**Objective:** Enable useful phone automation without forcing external Python code.

| Module | Initial APIs | Safety boundaries |
|---|---|---|
| `text` | split, join, trim, replace, case conversion, contains, length | Unicode-preserving behaviour and size limits |
| `math` | abs, round, min, max, floor, random with documented seed policy | No cryptographic claim for normal randomness |
| `time` | now, sleep, duration, formatting | Bounded sleep and explicit timezone handling |
| `path` / `file` | read, write, exists, list, mkdir, move, delete | Capability-scoped roots and no path traversal |
| `json` | parse, stringify, typed value conversion | Depth and input-size limits |
| `url` | encode/decode, query composition | No implicit network action |

### M4 — Modules, projects, and packages

**Objective:** Turn files into maintainable projects without creating an unsafe registry.

| Deliverable | Required work | Release gate |
|---|---|---|
| Imports | Explicit exports, namespaces, aliases, import cache, cycle reporting | Multi-file fixture suite |
| Project manifest | `padma.toml` or equivalent with name, version, entrypoint, locale, capabilities | Manifest schema and migration policy |
| Dependencies | Immutable version constraints, lockfile, checksums, local-path development dependencies | Tampering and offline-resolution tests |
| Registry policy | Signed metadata, maintainer identity, security reporting, takedown policy | Governance document before any public registry |

### M5 — Capability security model

**Objective:** Replace fixed internal allowlists with user-visible, least-privilege grants.

| Capability | Example | Required protection |
|---|---|---|
| Filesystem | `file.read("notes/todo.txt")` | Project root plus user-approved roots |
| Network | `http.get("https://...")` | Scheme policy, timeout, size ceiling, optional host allowlist |
| Process | `process.run("git", ["status"])` | Per-project command grants and no shell interpolation |
| Media | `media.download(url)` | Explicit output location, tool discovery, user-visible metadata |
| Secrets | API tokens for future AI clients | Encrypted/local secret store; values never printed by default |

### M6 — Developer workflow and static analysis

**Objective:** Make errors easy to find before scripts run.

| Tool | Required capability | Release gate |
|---|---|---|
| `padma check` | Parse, module resolution, unused names, unreachable code, capability preflight | Exit-code and JSON diagnostic tests |
| `padma fmt` | Idempotent formatter with stable layout | Formatter golden tests and second-run equality |
| `padma lint` | Configurable beginner and strict rules | Versioned lint-code catalogue |
| Type checking | Gradual annotations for function params/returns, collections, and JSON values | Type-check fixtures and clear opt-in policy |
| Diagnostics API | JSON output for editors and CI | Schema version and integration examples |

### M7 — Editor and learning tooling

**Objective:** Support both mobile learners and professional editors.

| Deliverable | Required work | Release gate |
|---|---|---|
| Tree-sitter grammar | Independent grammar, corpus tests, highlights, injections where needed | Generated parser and corpus CI |
| VS Code extension | Syntax highlighting, snippets, run/check tasks, diagnostics wiring | Extension packaging and sample workspace test |
| Language server | Go-to definition, completion, hover, diagnostics, formatting, rename | LSP conformance-oriented integration tests |
| Termux mobile guide | nano shortcuts, REPL, project folders, backups, troubleshooting | Tested copy-paste commands |

### M8 — Interoperability

**Objective:** Add bridges only after the Padma core and security model are stable.

| Bridge | Scope | Non-negotiable boundary |
|---|---|---|
| Python | Explicit subprocess/JSON bridge first; embedded runtime only after threat review | No automatic import of arbitrary Python packages |
| JavaScript | Node subprocess/JSON bridge first; web build separately | No hidden Node shell execution |
| Native extensions | Stable C ABI design only after semantic versioning is established | Versioned ABI and unsafe-code review |
| WebAssembly | Compile pure Padma core or use interpreter binding for sandboxed playgrounds | Browser APIs require explicit host capabilities |

### M9 — Domain libraries

**Objective:** Add useful domain modules without pretending that the language itself provides every capability.

| Domain | Safe first release | Deferred work |
|---|---|---|
| Web/backend | HTTP client/server routing, JSON responses, environment config | Full web framework and ORM |
| Automation | Scheduling integration, structured HTTP/API clients, file workflows | Background daemon without consent |
| AI | Provider-agnostic HTTP clients, prompt/result types, local secret references | Model training runtime or hidden credential use |
| Security education | Encoding, hashing, log parsing, authorized inventory checks | Exploitation modules or offensive automation |
| Data | CSV, JSON, basic tables and charts through explicit libraries | Large distributed data runtime |

### M10 — Release engineering and governance

**Objective:** Ensure that downloaded Padma binaries correspond to reviewed source and remain supportable.

| Deliverable | Required work | Release gate |
|---|---|---|
| Versioning | Semantic versioning, deprecation policy, compatibility matrix | Release note template and migration notes |
| Build integrity | Locked dependencies, repeatable environments, artifact checksums, provenance | Rebuild comparison against a release artifact |
| Supply chain | SBOM, vulnerability monitoring, signed tags/artifacts, incident process | Published security policy and disclosure path |
| Quality | Unit/integration/regression/fuzz/property/performance tests | Defined minimum gate before release |
| Governance | CONTRIBUTING, code of conduct, maintainer policy, issue templates | Public decision-making and release ownership rules |

## 5. Release sequence

The realistic sequence is intentionally narrow. A single language cannot become a secure web, AI, quantum, backend, and security platform in one release. Padma will grow through compatible releases:

| Release | Scope | Exit criteria |
|---|---|---|
| `0.2` | M1 diagnostics and M2 collections | Stable core test suite and no known parser panics |
| `0.3` | M3 standard library and M4 project structure | Safe file/JSON/text workflows and documented manifests |
| `0.4` | M5 capability model and M6 developer workflow | Explicit permission UX, formatter, check/lint, JSON diagnostics |
| `0.5` | M7 editor tooling | Tree-sitter grammar, VS Code extension, initial language server |
| `0.6` | M8 bridges and first domain libraries | Opt-in subprocess JSON bridges and audited HTTP/automation modules |
| `0.9` | M9 ecosystem beta and M10 release candidate practices | Signed artifacts, SBOM, fuzzing, compatibility matrix |
| `1.0` | Stable core language | No unresolved release-blocking defects; published long-term support policy |

## 6. Immediate next implementation increment

**M1 diagnostics and M2 core collections are complete for the current release line:** Padma has multi-error syntax checks, module-aware localized diagnostics, a stable diagnostic registry, null values, safe list/map indexing, mutation, slicing, collection utilities, and bounded deterministic iteration. The active implementation work is **M3 safe standard library**. Its first verified slice provides `text.*`, `math.*`, bounded `time.*`, and safe relative-path `file.read`/`file.write`/`file.exists`; JSON, URL, richer path operations, formatting, and randomness remain next.

## 7. Security and release references

Reproducible Builds defines reproducibility as independent parties recreating bit-identical artifacts from the same source, build environment, and instructions; Padma will adopt this as a release-engineering target rather than relying only on a successful CI build.[1]

The Language Server Protocol defines editor communication through JSON-RPC and negotiable client/server capabilities. Padma’s future language server will implement a constrained initial subset rather than inventing an incompatible editor protocol.[2]

Tree-sitter documents a grammar-and-generated-parser workflow with corpus testing; Padma’s highlighting grammar will remain an independently tested artifact rather than duplicating lexer logic inside every editor.[3]

SLSA describes incrementally adoptable supply-chain guidelines and distinguishes build-integrity evidence from ordinary code quality. Padma will use it to guide provenance and artifact handling, while retaining separate secure-coding and test gates.[4]

## References

[1]: https://reproducible-builds.org/docs/definition/ "Reproducible Builds — Definition"

[2]: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/ "Language Server Protocol Specification 3.17"

[3]: https://tree-sitter.github.io/tree-sitter/creating-parsers/1-getting-started.html "Tree-sitter — Getting Started"

[4]: https://slsa.dev/spec/v1.1/about "SLSA — About"
