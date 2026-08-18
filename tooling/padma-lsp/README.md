# Padma language server

`padma-lsp` is the first standards-based editor bridge for Padma. It uses LSP JSON-RPC over standard input/output and has no access to project capabilities, processes, or filesystem paths beyond text supplied by the editor.

Current support is deliberately narrow:

- `initialize` advertises full-document sync and document formatting;
- `textDocument/didOpen`, `didChange`, and `didClose` publish CLI-compatible diagnostics;
- `textDocument/formatting` returns the idempotent Padma formatter output.
- `textDocument/completion` returns a safe static Bangla-English keyword, builtin, and standard-library module catalogue.
- `textDocument/hover` explains supported Bangla-English keywords and selected stable builtins.
- `textDocument/definition` resolves the nearest visible same-document local declaration.
- `textDocument/prepareRename` and `textDocument/rename` provide conservative local-variable rename edits.

The server consumes compiler-owned, non-executing declaration and reference-binding APIs with parser positions and lexical scope metadata. This replaces text-prefix scanning. Same-document local variable declarations and references have stable binding IDs; nested shadowed names receive distinct IDs.

`textDocument/definition` now uses that index for conservative same-document lookup. It returns the nearest declaration visible at the request position and deliberately returns no result for imported symbols, public exports, ambiguous malformed blocks, or cross-file references.

Completion now supplements the static catalogue with local declarations visible before the request position in the current lexical brace scope. Imported, exported, and cross-file names remain intentionally absent until a parser-backed project symbol graph exists.

Diagnostics are generated through the public `padma_lang::check_source_json` API, so their codes, Bangla-English messages, and ranges agree with `padma check --json`. The server converts columns to LSP UTF-16 positions before publishing diagnostics.

Run it during development with:

```bash
cargo run --manifest-path tooling/padma-lsp/Cargo.toml
```

Rename is intentionally narrow: it only edits a compiler-resolved same-document local variable binding and rejects keywords, invalid identifiers, same-scope collisions, imports, exports, functions, members, unresolved names, and malformed source. Strings and comments are not parsed as references. Cross-file rename, document symbols, and incremental range updates remain deferred.
