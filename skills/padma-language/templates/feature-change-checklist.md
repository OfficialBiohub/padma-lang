# Padma Feature Change Checklist

## Request and Scope

- [ ] Identify user goal, supported milestone, and non-goal.
- [ ] Read `todo.md`, relevant `docs/`, source, and tests.
- [ ] Update plan and add an unchecked `todo.md` item before implementation.

## Design

- [ ] Preserve `padma file.pd`, REPL, `--version`, Bangla-English source, and localization contracts.
- [ ] Define parser/runtime/CLI behavior and diagnostics.
- [ ] Define capability, project-root, secret-redaction, and side-effect policy.
- [ ] Define plan/inspect versus action boundary when external effects are possible.

## Implementation and Documentation

- [ ] Implement runtime/CLI and positive/negative regression tests.
- [ ] Update help text, public docs, and runnable example when behavior is user-facing.
- [ ] Record external prerequisites and security limits honestly.

## Verification and Delivery

- [ ] Run targeted tests, then `bash scripts/verify-repository.sh`.
- [ ] Run `git diff --check` and verify example outputs from a temporary copy.
- [ ] Mark TODO sub-items completed only after verification.
- [ ] Commit, push, verify clean status, and report what is and is not implemented.
