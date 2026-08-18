# Tree-sitter Padma grammar

This package provides the editor-facing parser for Padma `.pd` files. It supports the language's current Bangla-English keyword aliases, Bengali digits, comments, declarations, blocks, expressions, lists, maps, imports, exports, indexing, and slicing.

```bash
pnpm install
pnpm generate
pnpm test
```

`grammar.js` is the source of truth. Generation creates parser artifacts under `src/`; corpus tests in `test/corpus` establish the public node contract. Highlight captures are in `queries/highlights.scm`.

The grammar is deliberately separate from the Rust interpreter. It must tolerate partial editor buffers and never execute Padma code. For execution semantics and localized diagnostics, use the root project's `padma check --json` interface.
