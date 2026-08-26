# Freelancer Tool Inventory

This inventory states only what the current Padma release binary executes, documents, tests, and ships with a runnable Termux example. These tools help prepare local work artifacts; they do **not** guarantee income, jobs, client acceptance, contract validity, or marketplace outcomes.

## Runnable freelancer preparation tools

| # | Tool | Real task result | Evidence |
|---:|---|---|---|
| 1 | Quote draft | Validates a local quote and writes reviewed Markdown. | `client.document_*`; `examples/freelancer-quote-draft` |
| 2 | Invoice draft | Validates a local invoice-draft review artifact; no invoice transmission. | `client.document_*`; `CLIENT-DOCUMENTS.md` |
| 3 | Scope-of-work | Validates scope, exclusions, revision label, and delivery target label. | `client.scope_*`; `examples/freelancer-scope-of-work` |
| 4 | Delivery checklist | Validates deliverable, review, and handover lists. | `client.delivery_*`; `examples/freelancer-delivery-checklist` |
| 5 | Portfolio case study | Validates public project/challenge/solution/outcome material and optional constrained public links. | `client.case_study_*`; `examples/freelancer-portfolio-handoff` |
| 6 | Visible handoff review | Prepares a copy-only message draft, attachment labels, destination label, and manual review steps. | `client.visible_handoff_*`; `examples/freelancer-portfolio-handoff` |
| 7 | Local data/report base | Reads bounded local table data and writes reviewed Markdown reports. | `table.*`, `report.*`; `examples/local-reporting-expense` |
| 8 | Local record base | Validates attendance, expense, and inventory records for local review. | `record.*`; `examples/local-records-household` |
| 9 | Local file/checksum base | Lists/checksums/searches bounded project-local content. | `fs.*`; filesystem toolkit example |
| 10 | Project templates | Creates basic, data-report, and local response projects with minimum grants. | `padma init`; CLI smoke tests |
| 11 | Attachment-review manifest | Reads declared project-local attachments for checksum/byte-count review and writes a local Markdown manifest. | `client.attachment_review_*`; `examples/freelancer-attachment-review` |
| 12 | Verifiable delivery package | Reads declared project-local files for integrity metadata and writes a manual folder/review package manifest. | `client.delivery_package_*`; `examples/freelancer-delivery-package` |
| 13 | Proposal, brief, and copy-only message templates | Renders explicit Bangla-English local preparation drafts with review checklists. | `client.template_*`; `examples/freelancer-client-templates` |
| 14 | Local quantum circuit planning | Validates bounded fixed and finite-angle rotation gates, exports deterministic OpenQASM 3.0 circuit text, simulates exact local basis probabilities, and evaluates one Pauli-product expectation up to 12 qubits. | `quantum.*`; `examples/local-quantum-planning` |

All client-document writers require only `filesystem = ["write"]` and write a project-local non-symlink `.md` file. The APIs return redacted summaries and disabled-action markers rather than performing external actions.

## Requested external actions and safe-replacement status

| Requested action | Automatic action added? | Implemented safe replacement | Still missing |
|---|---:|---|---|
| Client message | No | Copy-only `visible_handoff` review plus bounded proposal/brief/message-template Markdown drafts | Manual recipient/context selection and final copy/send remain the user's visible action by design |
| Upload / attachment delivery | No | Attachment labels, ownership review steps, and a single local checksum/byte-count attachment-review manifest | Manual file selection and upload remain the user's visible action by design |
| Delivery submission | No | Delivery checklist, visible destination-label review, and local checksum/byte-count delivery package | Manual client-side file selection/upload/submission remains the user's visible action by design |
| Payment / withdrawal | No | Quote/invoice-draft review labels; payment disabled marker | No payment replacement beyond manual review by design |
| Contract / e-signature | No | Scope-of-work review draft; contract disabled marker | No contract automation replacement beyond manual review by design |
| Marketplace work | No | Portfolio, quote, scope, delivery, and handoff preparation | No login, scraping, proposal posting, account, or submission system by design |
| Browser use | No hidden browser automation | Existing browser planning/confirmation/visible Android handoff system | No automatic browser control by design |
| Account automation | No | Redacted local profile and review artifacts | No account/session automation by design |

> “No” in the automatic-action column is intentional. These actions can cause irreversible external consequences; Padma leaves them to the user’s visible decision in the relevant service.

## Not made or incomplete

| Priority | Capability | Current state |
|---|---|---|
| P1 | Client-data delivery/reconciliation | Implemented foundations: local table comparison with redacted counts/checksums plus a separate project-local attachment checksum/ownership review manifest. CSV/JSON cleaning remains separate work. |
| P1 | Content/document preparation | Implemented first foundation: explicit-input Bangla-English proposal, brief, and copy-only message-template Markdown with local review/export. Recipient, platform, and sending remain separate by design. |
| P2 | Quantum computing | Implemented local foundations: validated circuit planning/OpenQASM text export, finite-angle rotation gates, bounded exact state-vector probability simulation, and one Pauli-product expectation evaluator. QPU/cloud submission, provider credentials, symbolic parameters, Hamiltonian/algorithm libraries, noise mitigation, sampling, and performance claims are intentionally unavailable. |
| P1 | Developer delivery toolkit | Not made: deterministic task/test/lint/build manifest and combined checksum manifest. |
| P1 | API integration templates | Not made: reusable request descriptors, extraction rules, retry/timeout policy, and secret-name references. |
| P2 | Study-note and extra local record schemas | Not made beyond attendance, expense, and inventory. |

The next selected safe increment is **explicit-input Bangla-English proposal, brief, and message-template preparation**. It will remain project-local and user-reviewed, without sending, upload, submission, payment, browser, account, network, or background authority.
