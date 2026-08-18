# Padma language server

`padma-lsp` is the first standards-based editor bridge for Padma. It uses LSP JSON-RPC over standard input/output and has no access to project capabilities, processes, or filesystem paths beyond text supplied by the editor.

Current support is deliberately narrow:

- `initialize` advertises full-document sync and document formatting;
- `textDocument/didOpen`, `didChange`, and `didClose` publish CLI-compatible diagnostics;
- `textDocument/formatting` returns the idempotent Padma formatter output.
- `textDocument/completion` returns a safe static Bangla-English keyword, builtin, and standard-library module catalogue.
- `textDocument/hover` explains supported Bangla-English keywords and selected stable builtins.

The server now also builds a non-executing local declaration index with lexical brace-depth metadata for Bangla-English `let`/`ধরি` and `function`/`ফাংশন` declarations. This is an internal, tested foundation for future scope-aware navigation; it does not yet expose a definition or rename request.

`textDocument/definition` now uses that index for conservative same-document lookup. It returns the nearest declaration visible at the request position and deliberately returns no result for imported symbols, public exports, ambiguous malformed blocks, or cross-file references.

Diagnostics are generated through the public `padma_lang::check_source_json` API, so their codes, Bangla-English messages, and ranges agree with `padma check --json`. The server converts columns to LSP UTF-16 positions before publishing diagnostics.

Run it during development with:

```bash
cargo run --manifest-path tooling/padma-lsp/Cargo.toml
```

Go-to-definition, rename, dynamic symbol completion, and incremental range updates are intentionally deferred until the AST/node and static-analysis contracts are expanded and tested.
