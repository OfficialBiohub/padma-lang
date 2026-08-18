# Padma editor tooling plan

Padma's first editor-tooling release is designed around a **separate Tree-sitter grammar** rather than reusing the interpreter parser directly. The interpreter remains authoritative for execution and diagnostics; the grammar provides fast, tolerant structural parsing for syntax highlighting, navigation, and future editor integrations.

## M7 architecture

| Layer | Repository location | Responsibility | Must not do |
|---|---|---|---|
| Tree-sitter grammar | `tooling/tree-sitter-padma` | Parse `.pd` source, expose stable node names, support incomplete code, and provide highlight queries. | Execute code or decide runtime semantics. |
| Padma CLI | `src/main.rs` | Run, check, format, lint, and emit localized diagnostics. | Depend on a generated editor parser at runtime. |
| VS Code extension | `tooling/vscode-padma` | Associate `.pd`, install grammar assets, expose run/check tasks, and render diagnostics. | Run programs automatically. |
| Language server | `tooling/padma-lsp` | Current stdio LSP bridge for CLI-compatible diagnostics and document formatting; later definitions, completion, hover, and rename. | Invent diagnostics that disagree with the CLI or execute Padma code. |

The grammar follows the Tree-sitter conventions of an intuitive `source_file` root, explicit statement/expression nodes, an `identifier` word token, comment extras, and corpus tests. Tree-sitter's own documentation recommends direct, recognisable syntax-tree nodes rather than mechanically copying an implementation grammar, and it requires corpus tests for grammar rules.[1] The initial grammar must cover the Padma syntax already accepted by the Rust interpreter, including English/Bangla aliases and Bengali digits.

## Delivery status and sequence

The first increment is complete: `tooling/tree-sitter-padma` contains an ABI-15 Tree-sitter grammar, generated parser artifacts, corpus fixtures, highlighting queries, pinned CLI dependency, and CI coverage. The second baseline is also complete: `tooling/vscode-padma` associates `.pd` files, provides Bangla-English TextMate highlighting and language configuration, and supplies explicit run/check/format/lint commands. Its check command renders the stable `padma check --json` diagnostics in VS Code's Problems panel.

The initial language-server baseline is now complete in `tooling/padma-lsp`: it uses standard LSP JSON-RPC on stdin/stdout, publishes diagnostics generated from the public CLI-compatible JSON API, converts Padma source positions to LSP UTF-16 columns, returns full-document formatter edits, and offers a safe static Bangla-English completion catalogue. The VS Code extension starts it only through an explicit user command.

Definitions, hover, rename, dynamic symbol completion, incremental range updates, and semantic document analysis remain later work. The extension must not duplicate those semantic decisions.

This order matters for Termux-first users: command-line editing and `padma check --json` remain useful even where a desktop editor is unavailable. No editor integration may grant capabilities, execute a `.pd` file, or open an external process without an explicit user command. The initial extension intentionally runs all commands in a visible terminal and only after the user selects a Padma command.

## Node-name stability

Node names used by highlighting or future editor features are treated as a public tooling contract. The initial stable set is `source_file`, `comment`, `identifier`, `number`, `string`, `let_declaration`, `function_definition`, `parameter_list`, `block`, `if_statement`, `while_statement`, `for_statement`, `return_statement`, `import_statement`, `export_statement`, `assignment_statement`, `call_expression`, `binary_expression`, `unary_expression`, `list`, `map`, `index_expression`, and `slice_expression`.

Breaking a published node name requires a migration note, updated corpus tests, and synchronized query updates. The grammar should prefer an explicit conflict or recovery node over silently changing the interpretation of valid source.

## References

[1] [Tree-sitter, “Writing the Grammar”](https://tree-sitter.github.io/tree-sitter/creating-parsers/3-writing-the-grammar.html) explains intuitive concrete syntax tree design, precedence handling, and corpus tests.

[2] [tree-sitter-haskell](https://github.com/tree-sitter/tree-sitter-haskell) documents the standard `grammar.js` → generated parser → `test/corpus` workflow and query-based tooling.

[3] [tree-sitter-rust](https://github.com/tree-sitter/tree-sitter-rust) describes Tree-sitter's incremental parsing characteristic for editor use.
