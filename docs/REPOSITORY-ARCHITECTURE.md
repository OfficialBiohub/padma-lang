# Padma Repository Architecture

## Purpose

This document defines the repository layout and maintenance boundaries for Padma. The repository is a **language implementation first**, not a web playground first. Its primary supported experience is the `padma` command on Termux and other Rust-supported systems.

The current Rust interpreter remains at the root crate (`src/main.rs` and `src/lib.rs`) because its public binary path, Termux installer, package recipe, WASM bridge, LSP crate, and CI already depend on that contract. Large source moves must be proposed through an RFC and completed only with compatibility tests.

## Canonical layout

```text
padma-lang/
├── src/                      # Canonical Padma interpreter crate and public library API
├── tests/                    # Future integration, golden, fuzz, and compatibility test suites
├── examples/                 # Runnable language, capability, GUI, Android, and deployment examples
├── docs/                     # Versioned language, security, platform, and contributor documentation
├── tooling/                  # Optional developer tools: LSP, Tree-sitter grammar, VS Code extension
├── wasm/                     # Thin browser/WASM bridge; not the primary runtime
├── playground/               # Optional demonstration client; never required for the core language
├── packaging/                # Downstream packaging recipes, including future Termux publication files
├── scripts/                  # Reproducible repository verification and maintenance commands
├── .github/                  # CI, issue forms, pull-request template, and project automation
├── install-termux.sh         # Supported Termux source installer entry point
├── Cargo.toml                # Canonical root Rust package manifest for the `padma` CLI
├── CONTRIBUTING.md           # Contribution and compatibility policy
├── SECURITY.md               # Vulnerability reporting and security boundaries
├── CODE_OF_CONDUCT.md        # Community behavior policy
├── SUPPORT.md                # User support and issue-routing guidance
├── CHANGELOG.md              # User-visible release history
└── todo.md                   # Maintainer implementation roadmap, not the public language specification
```

## Ownership and compatibility boundaries

| Area | Primary responsibility | Stable contract |
|---|---|---|
| `src/` | Lexer, parser, AST, interpreter, runtime, diagnostics, CLI | `padma` commands, `.pd` behavior, diagnostic codes, library API used by LSP/WASM |
| `tests/` | End-to-end and regression coverage | Future tests must run with `cargo test --locked` and use stable public behavior |
| `examples/` | Copy-pasteable learning and platform examples | Examples must be safe by default and remain compatible with the documented CLI |
| `docs/` | Specification, security policies, guides, roadmap | Document current behavior and clear limitations; do not claim unimplemented features |
| `tooling/` | Editor and parser integrations | Optional components must not become runtime dependencies of `padma` |
| `wasm/`, `playground/` | Optional browser experiences | Must not redefine language semantics or block Termux/core releases |
| `packaging/` | Maintained downstream distribution recipes | Root crate layout and `$PREFIX/bin/padma` installation contract are preserved |
| `scripts/` | Repeatable maintainer checks | Scripts may validate but must not silently publish, deploy, or mutate user credentials |
| `.github/` | CI and contributor collaboration | CI runs without secrets and validates a clean, reproducible checkout |

## Source-layout policy

Padma currently has a deliberately compact Rust implementation. A future compiler modularization may split `src/main.rs` into focused modules such as `lexer`, `parser`, `ast`, `runtime`, `diagnostics`, `project`, `capabilities`, and `cli`. That work is not a formatting-only change. It must first preserve all of the following:

1. The root `Cargo.toml` package name and `padma` binary name.
2. The root `src/lib.rs` APIs consumed by Padma LSP and the WASM bridge.
3. The Termux installer and package recipe assumption that `cargo build --release --locked` produces `target/release/padma`.
4. Existing localized diagnostic codes and externally documented CLI syntax.
5. Root, LSP, Tree-sitter, VS Code extension, WASM, and Termux smoke verification.

## Documentation policy

The root README is the public entry point. It must answer: what Padma is, what works today, how to install it in Termux, how to run the first program, where to find examples, and which features are deliberately not yet available.

Detailed material belongs in `docs/`. New platform features must include one guide, one safe example when applicable, capability requirements, explicit security exclusions, and test evidence before they are advertised in the root README.

## Release and artifact policy

Release builds are reproducible from the root with `cargo build --release --locked`. Generated directories such as Rust targets, Node modules, VSIX archives, WASM artifacts, and playground distributions are not source-of-truth repository content. CI validates the source checkout; a future release workflow must attach versioned artifacts rather than commit build output.

## Non-goals

This structure does not create a package registry, guarantee an Android APK build, convert Padma into an unrestricted deployment system, or make the optional playground the product. Those features require separate technical and security milestones.
