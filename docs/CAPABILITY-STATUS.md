# Padma Capability Status

This page is the **truth table** for the current repository. A capability is labelled **Implemented** only when the release binary has a real execution path, automated tests, and public documentation. **Bounded** means it works only inside a stated local/capability/safety contract. **Planned** means it is not available as a completed user-facing feature.

> A document, manifest validator, output plan, or example file is not treated as a complete application, deployment, automation system, browser controller, payment system, or marketplace integration.

## Implemented runtime foundations

| Area | Status | What the release binary really does | Evidence and boundary |
|---|---|---|---|
| Bangla-English language core | Implemented | Executes `.pd` scripts, modules, functions, lists/maps, control flow, REPL, formatter, linter, and localized diagnostics. | `padma file.pd`, `padma`, `padma check`, and `padma fmt`; direct bare REPL expression evaluation is regression-tested. |
| Project workflow | Implemented | Initializes and runs local projects with `padma.toml`, `src/`, `data/`, `out/`, and `tests/`. | `padma init`; direct scripts and legacy flat projects remain supported. |
| Local text/data/files | Bounded | Handles text, JSON, safe project-local file operations, CSV/TSV/object-row JSON tables, checksums, text search, and local Markdown reports. | Explicit project capabilities and path rules apply; no cloud sync, Excel macro, shared-storage write, or background daemon. |
| Local configuration | Bounded | Validates in-memory scalar profile maps and returns redacted summaries. | No account/device/network/process action follows from profile data. |
| Client documents v1 | Bounded | Validates and renders local quote/invoice/scope/delivery/portfolio Markdown, a no-send/no-upload visible handoff review artifact, local reconciliation outputs, and checksum-backed attachment-review manifests; may write a reviewed `.md` file inside a project. | `client.document_*`, `client.scope_*`, `client.delivery_*`, `client.case_study_*`, `client.visible_handoff_*`, `client.reconcile_*`, and `client.attachment_review_*`; client-task schemas remain separate work. |
| SQLite and identity primitives | Bounded | Provides fixed local SQLite record operations and local password/session/CSRF helpers. | It is not a hosted authentication server, ORM, remote database, or account provider. |
| HTTP, AI, and approved bridges | Bounded | Supports explicit HTTP helpers, one bounded provider-neutral AI workflow envelope, and fixed Python/Node bridge paths. | Requires an explicit capability and valid local configuration; no autonomous agent/tool loop or generated-output execution. |
| Authorized media helper | Bounded | Can invoke the fixed `yt-dlp` integration for content the user owns or is authorized to download. | Requires the external tool and explicit grants; no platform bypass or unauthorized use. |

## Intentionally bounded or planning-only areas

| Area | Actual current result | Not provided by Padma |
|---|---|---|
| Browser workflows | Local plan/confirm/draft/takeover checklist generation and a visible Android URL handoff. | Browser control, login/session automation, CAPTCHA bypass, private-data scraping, form fill/submit, posting, upload/download, account action, purchase, or payment. |
| GUI/mobile | Static renderer inspection/plan metadata and Android build-permission planning. | APK build/sign/install, permission elevation, ADB/device control, native code/JNI execution, or a production mobile-app framework. |
| Deployment | Read-only Render/Git-linked release and adapter-plan descriptors. | Provider API deployment, secret transfer, remote mutation, rollback execution, or automatic hosting. |
| Website/backend examples | Safe HTML-file creation and local response-envelope construction. | A full web framework, public production server, user auth service, managed deployment, or SaaS backend. |
| AI training | Resource-bounded training plan validation. | Dataset reading, model training, GPU/remote training execution, artifact writing, or an AI platform. |

## Incomplete daily-use backlog

The following are **not complete** and must not be described as finished. They are the main feature-by-feature delivery backlog.

| Priority | Capability family | Current state | Completion requirement |
|---|---|---|---|
| P0 | Starter project templates | Implemented | `padma init` supports the backward-compatible `basic` default plus `--template data-report` and `--template web-response`; generated projects have minimum capabilities, README commands, and regression/CLI smoke coverage. |
| P0 | Student/family/small-business records | Implemented first foundation | `record.validate` and redacted `record.summary` support strict attendance, expense, and inventory tables; local Markdown output reuses the separately capability-gated report toolkit. Study-note and other record schemas remain separate work. |
| P0 | Freelancer client workflow expansion | Partial | Quote/invoice-draft, scope-of-work, delivery-checklist, portfolio case-study, visible handoff, reconciliation, and checksum-backed attachment review are implemented local foundations. Client-task schemas and reusable proposal/brief templates remain separate work. |
| P1 | Client data delivery | Implemented first foundations | Local table reconciliation produces redacted validation/count/checksum artifacts, while attachment review produces a separate bounded project-local checksum/byte-count manifest. CSV/JSON cleaning remains separate work. |
| P1 | Content/document preparation | Implemented first foundation | `client.template_*` renders explicit-input Bangla-English proposal, brief, and copy-only message-template Markdown with human review and a project-local export option. It has no recipient, platform, or send authority. |
| P2 | Local quantum circuit planning, simulation, observables, sampling, and Hamiltonians | Implemented bounded foundation | `quantum.*` validates local circuit maps including finite numeric `rx`/`ry`/`rz` rotations, emits/writes deterministic OpenQASM 3.0, calculates full-basis state-vector probabilities, evaluates fixed Pauli products and up to 64 unique real-coefficient full-register Pauli Hamiltonian terms, and returns explicit-seed reproducible counts for up to 12 qubits. It does not bind symbolic parameters, use a hidden seed/noise/collapse state, automatically optimise an energy, execute an algorithm, submit, authenticate to, or select a quantum provider/QPU. |
| P2 | Local OpenQASM interchange assessment | Implemented bounded foundation | `quantum.assess_openqasm3` checks only byte-exact equality between explicit ASCII source and Padma’s deterministic renderer for the same validated circuit, then returns a stable local metadata map. It is not a general QASM parser/importer/compiler, file reader, source executor, provider/QPU adapter, or hardware compatibility claim. |
| P2 | Local classical optimisation primitives | Implemented bounded foundation | `optimize.quadratic_value`, `optimize.finite_difference_gradient`, and `optimize.projected_gradient_step` evaluate only an explicit finite separable quadratic, its centered finite-difference gradient, and one bounds-clamped proposal. They do not mutate state, repeat a step, execute callbacks, train a model, connect to a Hamiltonian/circuit, implement VQE/QAOA/QML/Grover, or access a provider/QPU/credential/network/process. |
| P1 | Developer delivery toolkit | Not implemented | Deterministic project task/test/lint/build descriptors and checksum manifests without arbitrary shell/background execution. |
| P1 | Verifiable delivery package | Implemented first foundation | `client.delivery_package_*` validates declared project-local files, creates deterministic checksum/byte-count metadata, and renders/writes a manual folder/review Markdown artifact. It does not copy files, upload, submit, or render a PDF. |
| P1 | Live browser session / human authentication event bridge | Intentionally unavailable | Existing browser work is local planning plus foreground visible handoff only. Live CDP connection, cookie/session/page-state access, CAPTCHA/2FA observation, browser control, and automatic post-authentication resume are not permitted by the current runtime. |
| P1 | API integration templates | Not implemented | Reusable validated request descriptors, safe response-field extraction, bounded retry/timeout policy, and secret-name references. |
| P2 | Creator/document conversion | Not implemented | Authorized local metadata and document/text conversion helpers without upload or hidden process execution. |
| P2 | Privacy-safe local inspection | Partial | Existing checksum/URL/password foundations exist; a unified redacted configuration and secret-strength report toolkit is not complete. |
| P2 | User-owned game project tools | Not implemented | Offline schema validation, fixture balance reports, accessibility templates, and local debugging helpers. Cracking, cheating, anti-cheat bypass, process/memory manipulation, and unfair advantage remain excluded. |
| P3 | Package ecosystem | Planning only | Trusted registry/provenance/offline-cache workflow; no package publishing or unreviewed lifecycle scripts. |

## Required completion gate

Every selected backlog item must pass the same gate before its status changes to **Implemented**: real parser/runtime or CLI path as applicable; Bangla-English diagnostics; narrow capability and project-root boundary; positive and negative tests; a runnable standalone Termux example; public documentation; `cargo fmt`; the root/LSP/release repository verification gate; a focused Git commit; and a successful push.

## Next implementation order

1. Build **P1 explicit-input Bangla-English proposal, brief, and message-template preparation** as the next safe freelancer increment.
2. Continue P1 and P2 items only after their dependencies and safety contracts are complete.

For an overview of long-term work, see [`todo.md`](../todo.md). For the exact public API contract, see [`STANDARD-LIBRARY.md`](STANDARD-LIBRARY.md). For runnable examples, see [`PRACTICAL-PROJECT-EXAMPLES.md`](PRACTICAL-PROJECT-EXAMPLES.md).
