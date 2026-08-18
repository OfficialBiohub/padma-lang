# Padma language server

`padma-lsp` is the first standards-based editor bridge for Padma. It uses LSP JSON-RPC over standard input/output and has no access to project capabilities, processes, or filesystem paths beyond text supplied by the editor.

Current support is deliberately narrow:

- `initialize` advertises full-document sync and document formatting;
- `textDocument/didOpen`, `didChange`, and `didClose` publish CLI-compatible diagnostics;
- `textDocument/formatting` returns the idempotent Padma formatter output.
- `textDocument/completion` returns a safe static Bangla-English keyword, builtin, and standard-library module catalogue.
- `textDocument/hover` explains supported Bangla-English keywords and selected stable builtins.

The server now consumes a compiler-owned, non-executing local declaration API with parser positions and lexical scope metadata for Bangla-English `let`/`ধরি`, `function`/`ফাংশন`, and loop-variable declarations. This replaces text-prefix scanning and is the tested foundation for scope-aware navigation; it does not yet bind identifier references for rename.

`textDocument/definition` now uses that index for conservative same-document lookup. It returns the nearest declaration visible at the request position and deliberately returns no result for imported symbols, public exports, ambiguous malformed blocks, or cross-file references.

Completion now supplements the static catalogue with local declarations visible before the request position in the current lexical brace scope. Imported, exported, and cross-file names remain intentionally absent until a parser-backed project symbol graph exists.

Diagnostics are generated through the public `padma_lang::check_source_json` API, so their codes, Bangla-English messages, and ranges agree with `padma check --json`. The server converts columns to LSP UTF-16 positions before publishing diagnostics.

Run it during development with:

```bash
cargo run --manifest-path tooling/padma-lsp/Cargo.toml
```

Rename, dynamic symbol completion, and incremental range updates are intentionally deferred until identifier-reference binding and the static-analysis contracts are expanded and tested.
