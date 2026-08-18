# Padma VS Code sample workspace

This is a runnable, capability-free Padma project for validating `.pd` file association, Bangla-English highlighting, commands, diagnostics, completion, definition, and conservative local rename.

Open this folder in VS Code after installing a locally built Padma extension. The included `.vscode/settings.json` assumes `padma` and `padma-lsp` are already on the VS Code environment's `PATH`; change those two settings to absolute executable paths when necessary.

Run the project in a visible terminal with:

```bash
padma .
padma check src/main.pd
padma fmt --check src/main.pd
padma lint src/main.pd
```

In VS Code, open `src/main.pd`, run **Padma: Check Current File**, then run **Padma: Start Language Server** explicitly. Try go-to-definition or rename on the outer `greeting`: its two references change together, while the nested `greeting` remains a separate local binding.

The sample intentionally grants no filesystem, network, process, or media capability. It is safe for editor packaging tests and does not replace the Termux-first command-line workflow.
