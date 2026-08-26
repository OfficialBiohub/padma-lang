# Padma Documentation

This index is the maintained entry point for detailed Padma documentation. The root [README](../README.md) is for installation and first use; this directory records language contracts, security limits, platform plans, and contributor-facing architecture.

## Learn and build

| Document | Use it for |
|---|---|
| [Termux-first guide (Bangla)](TERMUX-FIRST-GUIDE-BN.md) | Phone-only installation, editing, projects, and safe workflows |
| [Language specification](LANGUAGE-SPEC.md) | Syntax, semantics, and bilingual keyword rules |
| [Standard library](STANDARD-LIBRARY.md) | Supported built-in APIs and limits |
| [Diagnostics](DIAGNOSTICS.md) | Stable bilingual `Pxxxx` codes |
| [Projects](PROJECTS.md) | Project manifests, modules, exports, and lockfiles |
| [Linting](LINTING.md) | Formatter, linter, and static-check behavior |
| [Editor tooling](EDITOR-TOOLING.md) | Tree-sitter, VS Code, and LSP use |
| [Interoperability](INTEROPERABILITY.md) | Python/JavaScript bridge boundaries |
| [Practical project examples](PRACTICAL-PROJECT-EXAMPLES.md) | Runnable media, website, backend, SQLite, and defensive-security walkthroughs |
| [Daily-use tool roadmap](DAILY-USE-TOOL-ROADMAP.md) | Bangla-first capability matrix and priority order for practical Termux tooling |
| [Structured data toolkit](STRUCTURED-DATA.md) | Local CSV/TSV/JSON table processing with bounded project-root file access |
| [Filesystem productivity toolkit](FILESYSTEM-PRODUCTIVITY.md) | Bounded project-local listing, checksum, text search, and disabled copy/move/archive plans |
| [Local reporting toolkit](LOCAL-REPORTING.md) | Validated table-to-Markdown summaries and capability-gated project-local report export |
| [Simple project structure](SIMPLE-PROJECTS.md) | Beginner-friendly canonical layout, four Termux commands, and flat-project compatibility rules |
| [Local profile toolkit](LOCAL-PROFILES.md) | Bounded in-memory profile validation, explicit defaults, and redacted summaries without action authority |
| [Local records](LOCAL-RECORDS.md) | Strict project-local attendance, expense, and inventory table validation with redacted summaries |
| [Freelancer workflows roadmap](FREELANCER-WORKFLOWS.md) | Production-oriented local preparation path and explicit marketplace/client-action boundaries |
| [Local client documents](CLIENT-DOCUMENTS.md) | Strict local quote/invoice-draft Markdown preparation with user-reviewed client/payment boundaries |
| [Local scope-of-work](LOCAL-SCOPE-OF-WORK.md) | Strict local scope/exclusion/revision draft with redacted summary and project-local review export |
| [Local delivery checklists](LOCAL-DELIVERY-CHECKLISTS.md) | Strict local deliverable/review/handover draft with redacted summary and project-local review export |
| [Capability status](CAPABILITY-STATUS.md) | Evidence-based distinction between implemented runtime, bounded behavior, and remaining delivery backlog |

## Security and local runtime

| Document | Use it for |
|---|---|
| [Capability security](CAPABILITY-SECURITY.md) | Deny-by-default permissions and project filesystem scope |
| [Package trust](PACKAGE-TRUST.md) | Local package locks, verification, and current limitations |
| [SQLite persistence](SQLITE-PERSISTENCE.md) | Local persistence API and system-tool boundary |
| [Identity and session](IDENTITY-SESSION.md) | Local password, session, CSRF, and cookie primitives |
| [Deployment trust](DEPLOYMENT-TRUST.md) | Provider-independent dry-run deployment boundary |
| [AI workflow foundation](AI-WORKFLOW.md) | Strict provider-neutral AI workflow inspection and no-side-effect planning |
| [Browser planning foundation](BROWSER-PLANNING.md) | Strict domain-allowlisted local navigation planning with no browser execution |
| [Browser confirmation-session planning foundation](BROWSER-CONFIRMATION-PLANNING.md) | Digest-bound local confirmation-session descriptor with no issued approval or browser action |
| [Android Browser Handoff](ANDROID-BROWSER-HANDOFF.md) | Foreground-confirmed one-URL Termux handoff to the user’s visible Android browser |
| [Browser interaction drafts](BROWSER-DRAFTS.md) | Digest-bound local review drafts with metadata-only attachments and mandatory visible user takeover |
| [Visible browser takeover checklist](BROWSER-TAKEOVER.md) | Digest-bound sensitive-action checklist with no browser/action authority or user-decision collection |
| [Browser navigation action-adapter design](BROWSER-ACTION-ADAPTER-DESIGN.md) | Security-gated future execution contract; no action authority is implemented |
| [AI tool planning foundation](AI-TOOLS-PLANNING.md) | Capability-gated local AI toolset and bounded agent-runbook planning with no tool execution |
| [AI training planning foundation](AI-TRAINING-PLANNING.md) | Resource-bounded project-local training planning with no dataset read, training, or artifact write |

## Platform and application planning

| Document | Use it for |
|---|---|
| [M9 application platform](M9-APPLICATION-PLATFORM.md) | Current application-platform roadmap |
| [GUI/mobile bridge](GUI-MOBILE-BRIDGE.md) | Static renderer manifest and Android WebView boundary |
| [Android build plan](ANDROID-BUILD-PLAN.md) | Read-only APK metadata and permission validation |
| [Render Git-linked release](RENDER-GIT-LINKED-RELEASE.md) | Render dashboard release metadata contract |
| [Render API adapter](RENDER-API-ADAPTER.md) | Explicit confirmation, secret-handling, deploy, and rollback boundary |
| [Render and Android security](M9-RENDER-ANDROID-SECURITY.md) | Security rationale for these provider/mobile contracts |
| [M9 AI and browser design](M9-AI-BROWSER-DESIGN.md) | AI runtime and browser-planning security design beyond the implemented local AI inspection foundation |

## Maintain the project

| Document | Use it for |
|---|---|
| [Repository architecture](REPOSITORY-ARCHITECTURE.md) | Directory ownership, stable contracts, and source-layout policy |
| [Padma Agent Skill](AGENT-SKILL.md) | Reusable agent guidance for safe, language-first Padma engineering |
| [Production roadmap](PRODUCTION-ROADMAP.md) | Major future implementation milestones |
| [Domain libraries](DOMAIN-LIBRARIES.md) | HTTP, AI, backend, and automation helper boundaries |

Every document must describe implemented behavior precisely. New public features require documentation, a safe example when applicable, capability notes, explicit limitations, and automated test coverage.
