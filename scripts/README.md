# Repository Scripts

These scripts are committed quality checks. They do not publish releases, contact deployment providers, read secrets, modify a device, or write into source files.

| Script | Purpose |
|---|---|
| `verify-repository.sh` | Run the full local maintainership check: hygiene, links, Termux contract, Rust formatting, root tests, LSP tests, release build, and safe examples. |
| `verify-repository-hygiene.sh` | Reject tracked generated build output and verify required public repository files. |
| `verify-doc-links.sh` | Verify local Markdown inline links resolve inside the checkout. |
| `verify-termux-contract.sh` | Verify documented source-install assumptions remain compatible with the installer and Termux package recipe. |

Run the full check from the repository root:

```bash
bash scripts/verify-repository.sh
```
