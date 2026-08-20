# Verification and Maintenance

## Read This Reference When

Read this file before committing, pushing, refactoring, resolving CI failure, changing LSP/tree-sitter, or declaring a Padma feature complete.

## Standard Gate

Run the repository quality gate from the repository root:

```bash
bash scripts/verify-repository.sh
```

It is the canonical combined check for repository hygiene, authored documentation links, Termux installer contract, Rust formatting/tests/release build, editor tooling checks, and safe project examples. For a targeted change, run the relevant narrower command first, then run the full gate before commit.

## Required Commands When the Combined Script Is Unavailable

```bash
git diff --check
cargo fmt --check
cargo test --locked
cargo test --manifest-path tooling/padma-lsp/Cargo.toml --locked
cargo build --release --locked
```

Run tree-sitter and VS Code extension checks when grammar or extension files change. Execute revised examples with `target/release/padma` from temporary copies. Do not run a real provider deployment, rollback, browser action, or device operation as a smoke test.

## Commit and Push

1. Check `git status --short` and make sure only intended files are staged.
2. Use a focused conventional-style message such as `feat(db): ...`, `fix(cli): ...`, `docs(examples): ...`, or `chore(repo): ...`.
3. Push after every applicable verification passes.
4. Re-check clean status and branch parity after push.

## Maintenance Signals

Treat a stale README claim, an example that does not run, a missing negative test, undocumented capability, secret-bearing diagnostic, or CI-only failure as a release blocker. Update this skill’s references when the project changes a stable contract, capability family, directory owner, or release gate.
