# Padma Test Strategy

The current core regression suite is colocated with the single-file interpreter implementation in `src/main.rs`. This was practical while the compiler was intentionally compact. New end-to-end, compatibility, corpus, and golden-file tests should be added beneath this directory as the implementation is modularized.

| Test layer | Current location | Required command |
|---|---|---|
| Core interpreter and CLI regressions | `src/main.rs` test module | `cargo test --locked` |
| Language server | `tooling/padma-lsp/` | `cargo test --manifest-path tooling/padma-lsp/Cargo.toml --locked` |
| Tree-sitter grammar corpus | `tooling/tree-sitter-padma/` | `pnpm test` from that directory |
| VS Code extension checks | `tooling/vscode-padma/` | `pnpm run check` from that directory |
| Termux smoke coverage | `scripts/verify-termux-contract.sh` | `bash scripts/verify-termux-contract.sh` |

Every bug fix must add a regression test. Every diagnostic change must preserve its stable code, locale behavior, and source position behavior. Every capability or platform feature must include a negative test proving that unsafe input is rejected.
