# Padma VS Code extension

This is the first local VS Code integration for Padma. It associates `.pd` files, highlights Bangla and English keywords, adds bracket/comment behavior, and exposes explicit commands:

- **Padma: Run Current File** runs `padma file.pd` in a visible terminal.
- **Padma: Check Current File** runs `padma check --json file.pd` and renders localized diagnostics in the Problems panel.
- **Padma: Format Current File** runs `padma fmt file.pd` in a visible terminal.
- **Padma: Lint Current File** runs `padma lint file.pd` in a visible terminal.
- **Padma: Start Language Server** explicitly starts `padma-lsp` over standard I/O; it publishes diagnostics as you edit and enables document-formatting requests.
- **Padma: Stop Language Server** stops that editor-side process.

Run/check/format/lint commands never run on open, save, or extension activation. The language server is also **opt-in**: it starts only after the explicit start command, which prevents the extension from silently launching a process. Configure `padma.command` or `padma.languageServer.command` when either executable is not on the VS Code environment's `PATH`.

For Termux, install Padma first and use a VS Code environment that can reach the same Termux shell or a synced repository. The extension does not request Android permissions or bypass Padma project capabilities.

The companion `../tree-sitter-padma` package provides the future structural parser. This extension currently uses a small TextMate grammar for broadly compatible syntax highlighting while the independent Tree-sitter grammar and language-server node contract mature.
