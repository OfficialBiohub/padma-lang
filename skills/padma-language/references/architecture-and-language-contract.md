# Architecture and Language Contract

## Read This Reference When

Read this file before changing parsing, execution, diagnostics, modules, public CLI behavior, LSP interfaces, or source layout.

## Repository Map

| Location | Responsibility | Compatibility rule |
|---|---|---|
| `src/main.rs` | Lexer aliases, parser, AST, evaluator, builtins, CLI, REPL, tests | Treat as the primary stable implementation surface. |
| `src/lib.rs` | Library-facing helpers for editor tooling | Preserve JSON payload shapes used by LSP. |
| `tooling/padma-lsp/` | Language server, formatting, completion, definition, rename | Run its independent tests after core changes. |
| `tooling/tree-sitter-padma/` | Grammar and syntax tooling | Update corpus/queries when syntax changes. |
| `docs/` | User/security/runtime contracts | Update the relevant document with public behavior. |
| `examples/` | Capability-bounded executable projects | Keep each example runnable from its own directory. |

## Stable User Contract

Maintain these unless a documented compatibility migration exists.

```text
padma --version
padma
padma file.pd
padma check --json file.pd
padma fmt file.pd
padma lint file.pd
```

Padma accepts UTF-8 `.pd` code and supports Bangla, English, and mixed keyword forms. Select diagnostics according to source-language cues; do not replace a localized error with a generic Rust or system error.

## Feature Change Pattern

1. Add or adjust AST/parser behavior only after identifying the current grammar/test convention.
2. Define semantic behavior and exact diagnostic failure modes.
3. Implement evaluator/builtin behavior with bounded inputs.
4. Add tests adjacent to the existing behavior in `src/main.rs`; add LSP tests if editor-facing behavior changes.
5. Update CLI help and relevant docs/examples when users can invoke the feature.

## Source Layout Rule

The interpreter is intentionally single-file while its behavior remains closely coupled. Do not split it solely to make the directory tree look conventional. A future split must first create integration/golden coverage around lexer, parser, diagnostics, REPL, manifests, and builtins; then move one vertical slice at a time without changing public result shapes.

## Diagnostics

Use the existing stable `P####` diagnostic family. Keep codes stable, messages bilingual where appropriate, and `padma check --json` output machine-readable. Never expose raw environment values, full filesystem paths outside the project root, command lines containing secrets, or provider tokens in a diagnostic.
