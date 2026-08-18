# Padma VS Code extension

This is the first local VS Code integration for Padma. It associates `.pd` files, highlights Bangla and English keywords, adds bracket/comment behavior, and exposes explicit commands:

- **Padma: Run Current File** runs `padma file.pd` in a visible terminal.
- **Padma: Check Current File** runs `padma check --json file.pd` and renders localized diagnostics in the Problems panel.
- **Padma: Format Current File** runs `padma fmt file.pd` in a visible terminal.
- **Padma: Lint Current File** runs `padma lint file.pd` in a visible terminal.
- **Padma: Start Language Server** explicitly starts `padma-lsp` over standard I/O; it publishes diagnostics as you edit and enables document formatting, local completion, hover, go-to-definition, and conservative same-document local-variable rename.
- **Padma: Stop Language Server** stops that editor-side process.

Run/check/format/lint commands never run on open, save, or extension activation. The language server is also **opt-in**: it starts only after the explicit start command, which prevents the extension from silently launching a process. Configure `padma.command` or `padma.languageServer.command` when either executable is not on the VS Code environment's `PATH`.

For Termux, install Padma first and use a VS Code environment that can reach the same Termux shell or a synced repository. The extension does not request Android permissions or bypass Padma project capabilities.

## Local package and sample validation

From this directory, install the pinned development dependencies and produce a local VSIX artifact:

```bash
pnpm install --frozen-lockfile
pnpm run check
pnpm run package:check
code --install-extension dist/padma-vscode-0.1.0.vsix
```

The package validation command fails unless it creates a non-empty `.vsix` file. Open `sample-workspace/` after installation. Its code uses a nested Bangla variable binding so that syntax highlighting, diagnostics, definition, and local rename can be checked without granting sensitive capabilities.

For a source checkout, build the language server before starting it from the Command Palette:

```bash
cargo build --manifest-path tooling/padma-lsp/Cargo.toml --release --locked
```

Set `padma.languageServer.command` to the resulting `padma-lsp` executable if it is not already on VS Code's `PATH`. Starting the server remains an explicit user action; the extension never launches it merely because a `.pd` file was opened.

The companion `../tree-sitter-padma` package provides the future structural parser. This extension currently uses a small TextMate grammar for broadly compatible syntax highlighting while the independent Tree-sitter grammar and language-server node contract mature.
