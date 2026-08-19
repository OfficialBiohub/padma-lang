# Padma Core Source

`src/` is the stable Rust crate boundary for the Padma interpreter and its public library facade.

| File | Responsibility | Compatibility rule |
|---|---|---|
| `main.rs` | CLI, lexer, parser, AST, evaluator, built-ins, localized diagnostics, and core regression tests | The `padma` binary name, documented commands, stable diagnostic codes, and Termux script execution behavior are public contracts. |
| `lib.rs` | Narrow Rust API consumed by the language server and other tooling | Public functions require compatibility review and matching tooling tests. |

The implementation remains intentionally compact while the language semantics evolve. The single-file core is **not** a permanent architecture decision. Future refactors will move cohesive units into modules only after their internal contracts are covered by regression tests and preserve behavior at the `padma` CLI and `lib.rs` boundaries.

New language work should first add or update tests close to the behavior it protects. Cross-cutting compatibility, corpus, and end-to-end tests belong under [`../tests/`](../tests/README.md) as the test suite is progressively separated from `main.rs`.
