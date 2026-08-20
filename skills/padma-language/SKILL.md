---
name: padma-language
description: Build, review, debug, document, package, or extend the Padma Bangla-English programming language and its Termux-first ecosystem. Use when working in OfficialBiohub/padma-lang; editing Rust interpreter or LSP code; creating `.pd` programs, `padma.toml` project manifests, capabilities, standard/domain libraries, examples, installer or release/CI work; or assessing Padma security and deployment/mobile boundaries.
---

# Padma Language Engineering

Use this skill for **language-first** work in `OfficialBiohub/padma-lang`. Preserve the Python-like Termux contract: one installation flow, `padma --version`, `padma` REPL, and `padma file.pd` script execution. Treat the web playground as a separate optional product, not as the language’s primary runtime.

## Start Every Change

1. Locate the repository and run `git status --short`; do not overwrite unrelated work.
2. Read `todo.md`, the relevant document under `docs/`, and the exact affected source or tooling file before editing.
3. Update the active task plan and add specific unchecked work to `todo.md` before implementation when scope changes materially.
4. Preserve the stable public CLI, UTF-8 Bangla/English source support, localized diagnostics, project-root filesystem scoping, and no-new-Rust-crates policy.
5. Make the smallest coherent implementation. Do not claim a capability exists until its parser/runtime/CLI path, documentation, examples, negative tests, and release verification exist.

## Route the Task

| Request category | Read before acting |
|---|---|
| Lexer, parser, AST, evaluator, diagnostic, or CLI change | `references/architecture-and-language-contract.md` |
| Builtin, module, manifest, capability, file/network/process/security work | `references/capabilities-and-security.md` |
| `.pd` tutorial, reusable project, or user-facing example | `references/examples-and-documentation.md` |
| Termux installation, release binary, packaging, CI, GitHub release | `references/termux-distribution-and-release.md` |
| Test failure, refactor, LSP, formatting, final push | `references/verification-and-maintenance.md` |

## Mandatory Engineering Rules

### Preserve public behavior

Keep `padma file.pd` as the normal script command; do not require a `run` keyword. Keep REPL behavior, `--version`, bilingual keyword aliases, and deterministic structured diagnostics compatible unless an intentional versioned migration is documented.

### Keep security explicit

Use `padma.toml` capability grants for every sensitive builtin or integration. Resolve paths beneath the project root; reject absolute paths, `..`, symlinks where the contract requires it, hidden process execution, secret values in manifests/plans, and uncontrolled network/device side effects. Prefer inspect/plan modes before action modes.

### Keep external integration narrow

Do not add Rust crates. Use existing standard library patterns and approved system-tool bridges where the project already uses them. For provider, Android, browser, GUI, AI, or deployment work, implement a versioned manifest, validate inputs, redact secrets, gate capability access, create negative tests, and document both present behavior and exclusions.

### Make examples honest

Every published example needs a project manifest, exact command, expected output, required external dependency, and clear safety/ownership boundary. Do not present a plan/inspect function as a deployed application, or an example as a full framework.

## Implementation Workflow

1. **Classify** the request with the routing table and read only the relevant references.
2. **Inspect** current implementation and tests; reuse established patterns rather than inventing parallel conventions.
3. **Design** data model, diagnostics, capability gate, path/secrets policy, and CLI behavior before adding code.
4. **Implement** parser/runtime/CLI/documentation/examples together when a public feature changes.
5. **Test** positive behavior and realistic negative cases such as denied grants, malformed manifests, path escape, missing tools, malformed JSON, secret exposure, and unauthorized network/device actions.
6. **Verify** with the repository gate in `references/verification-and-maintenance.md`.
7. **Commit** focused changes only after every applicable gate passes; push only after successful verification.

## Definition of Done

Before declaring any Padma feature finished, confirm all applicable conditions.

| Area | Required result |
|---|---|
| Runtime | Correct behavior for Bangla, English, and relevant mixed source forms |
| Safety | Capability denial, project-root, validation, and redaction tests |
| Diagnostics | Stable code, localized message, and structured JSON behavior where applicable |
| CLI | Help text, argument validation, ordinary and error paths |
| Documentation | Contract, limits, Termux command, and expected output |
| Quality | Formatter, root tests, LSP tests, release build, examples, hygiene/docs checks |
| GitHub | Focused commit, clean status, and pushed verified branch |

## Do Not Do These Things

- Do not introduce a remote deployment, browser action, APK build, permission grant, device control, native-code execution, secret transfer, payment, publishing action, or automatic rollback without a reviewed provider/device contract and explicit user confirmation.
- Do not use the package examples to evade platform rules; `media.download` is for authorized content only.
- Do not weaken path boundaries or change capability defaults merely to make an example work.
- Do not move the single-file interpreter into modules as cosmetic cleanup. First establish a test-backed migration boundary and preserve every public behavior.

## Repository Pointers

Read `docs/REPOSITORY-ARCHITECTURE.md` for directory ownership, `docs/PRACTICAL-PROJECT-EXAMPLES.md` for supported examples, and `scripts/verify-repository.sh` for the current quality gate. The references in this skill explain when and how to use these project files without duplicating their full content.
