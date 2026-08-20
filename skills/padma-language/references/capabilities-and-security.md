# Capabilities and Security

## Read This Reference When

Read this file before adding or changing any builtin that reads/writes files, uses network/process tools, accepts credentials, starts a server, creates database data, bridges another language, or touches deployment/mobile/browser/GUI behavior.

## Core Rule

Sensitive behavior must be **project-declared, capability-gated, input-validated, project-root scoped, and tested in both allowed and denied modes**. A successful happy-path example is never sufficient.

## Current Capability Families

Use the current parser/manifest code and relevant `docs/` guide as the source of truth. The established families include the following categories.

| Family | Typical boundary |
|---|---|
| `file` | Project-scoped file access only |
| `network:http` | Explicit HTTP request boundary |
| `network:ai` | Explicit AI-provider request boundary |
| `process:python`, `process:node` | Validated bridge scripts/process arguments |
| `media` | Authorized local media workflow with external tool prerequisite |
| `server:local` | Loopback-only local server |
| `database:sqlite` | Project-scoped SQLite via approved system bridge |
| `identity:local` | Local password/session/CSRF primitives |
| `deployment` | Manifest/plan contract; provider actions require extra review |
| `gui:local` | Static local renderer plan only |
| `android:plan` | Read-only Android build/permission declaration plan |

## Mandatory Checks for New Sensitive Features

1. Require the least-privileged capability name and reject missing/incorrect grants.
2. Reject absolute paths, traversal segments, URLs in local-only fields, forbidden symlinks, unsafe filenames, and unknown manifest fields.
3. Keep data exchange typed and bounded; use JSON-only bridge payloads where the existing bridge contract requires it.
4. Store credential **names** in manifests, never credential values. Read a secret only immediately before an explicitly confirmed action; redact it from all output.
5. Separate `inspect`/`plan` from remote, device, deployment, posting, payment, or rollback action commands.
6. Add tests for invalid manifest, missing grant, unsafe path, missing dependency, redaction, and no-side-effect planning behavior.

## High-Risk Boundaries

Do not silently add these to the core runtime: remote deploy, browser login/session automation, CAPTCHA bypass, payment/posting, Android permission elevation, APK signing/install, ADB/device control, JNI/native hooks, or automatic rollback. Require a dedicated versioned adapter contract, explicit action confirmation, a narrow transport, bounded timeout, and documented recovery/rollback limits.

## Documentation Pointers

Choose the matching project guide before implementation: `DOMAIN-LIBRARIES.md`, `SQLITE-PERSISTENCE.md`, `IDENTITY-SESSION.md`, `DEPLOYMENT-TRUST.md`, `GUI-MOBILE-BRIDGE.md`, `ANDROID-BUILD-PLAN.md`, `RENDER-API-ADAPTER.md`, or `M9-RENDER-ANDROID-SECURITY.md`.
