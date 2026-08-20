# Changelog

All notable user-visible changes are recorded in this file. Padma follows a pre-1.0 development policy: interfaces may evolve, but breaking changes require a migration note and explicit review.

## Unreleased

### Added

| Area | Change |
|---|---|
| Repository | Language-first repository architecture, documentation index, examples/tooling/test indexes, collaboration policies, issue forms, pull-request template, and repeatable quality scripts. |
| Agent engineering | Versioned `padma-language` Agent Skill with modular architecture, security, Termux, examples, and verification guidance. |
| M9 design | Provider-neutral AI workflow and domain-allowlisted browser-planning contract, including capability boundaries, manifest schemas, redaction, no-side-effect plans, regression matrix, and staged implementation order. No new AI or browser runtime authority is enabled by this documentation change. |
| M9 AI planning | Strict `padma-ai.toml` validation, `network:ai`-gated `padma ai inspect|plan`, bilingual `P1050`, redacted deterministic plans, and a credential-free example. These local commands do not read secrets, resolve DNS, start a process, connect to a provider, invoke a model, or execute generated output. |
| M9 AI runtime | Provider-neutral `ai.workflow` with an inert JSON input/output envelope, one `json-http-v1` curl configuration path, fixed POST request, no redirect/retry, bounded timeout/output, and bilingual `P1051`/`P1052` transport diagnostics. A test-only local mock proves one transport invocation without a network request. Model output remains data and is never executed. |
| Quality | Repository hygiene, documentation-link, and Termux installer-contract checks integrated with CI. |

## 0.1.0

### Added

| Area | Change |
|---|---|
| Language core | Bangla-English lexer aliases, parser, AST, interpreter, functions, collections, modules, exports, localized diagnostics, REPL, formatter, linter, and selected static checks. |
| Runtime and safety | Project manifests, capability model, safe file/process/bridge operations, HTTP/AI/domain helpers, package trust foundation, local server, SQLite persistence, identity/session helpers, and deployment planning boundaries. |
| Tooling | Tree-sitter grammar, Padma LSP, VS Code extension, WASM bridge, and Termux installer. |
| Platform planning | Local GUI manifest bridge, Android build plan, Render Git-linked release plan, and explicit-confirmation Render API adapter. |

The detailed evolution and unfinished milestones remain in `todo.md` and the documents under `docs/`.
