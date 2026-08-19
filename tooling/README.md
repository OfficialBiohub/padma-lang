# Padma Tooling

This directory contains optional developer integrations. None of these tools is required to run `padma` on Termux.

| Component | Purpose | Documentation |
|---|---|---|
| `padma-lsp/` | Language Server Protocol implementation for diagnostics, formatting, completion, hover, definition, and rename | [README](padma-lsp/README.md) |
| `tree-sitter-padma/` | Tree-sitter grammar and corpus tests | [README](tree-sitter-padma/README.md) |
| `vscode-padma/` | VS Code extension and packaged VSIX validation | [README](vscode-padma/README.md) |

Tooling must depend on documented Padma library APIs and CLI behavior. A tooling change must not redefine language syntax or runtime semantics; changes to those contracts require matching updates in `src/`, tests, editor documentation, and release notes.
