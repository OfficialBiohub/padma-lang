# Padma language server

`padma-lsp` is the first standards-based editor bridge for Padma. It uses LSP JSON-RPC over standard input/output and has no access to project capabilities, processes, or filesystem paths beyond text supplied by the editor.

Current support is deliberately narrow:

- `initialize` advertises full-document sync and document formatting;
- `textDocument/didOpen`, `didChange`, and `didClose` publish CLI-compatible diagnostics;
- `textDocument/formatting` returns the idempotent Padma formatter output.

Diagnostics are generated through the public `padma_lang::check_source_json` API, so their codes, Bangla-English messages, and ranges agree with `padma check --json`. The server converts columns to LSP UTF-16 positions before publishing diagnostics.

Run it during development with:

```bash
cargo run --manifest-path tooling/padma-lsp/Cargo.toml
```

Go-to-definition, completion, hover, rename, incremental range updates, and VS Code client registration are intentionally deferred until the AST/node and static-analysis contracts are expanded and tested.
