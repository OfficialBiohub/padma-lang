# Padma capability security model

Padma projects use a **manifest-declared capability model**. The rule is simple: code running through `padma .` receives no sensitive capability unless the project owner declares the smallest matching grant in `padma.toml`. The runtime enforces this rule before calling filesystem, network, process, or media backends.

This document defines the current contract and the approval requirements for any future permission escalation. It is intentionally stricter than direct single-file compatibility mode, because a project manifest is reviewable, version-controlled, and inspectable without executing project code.

## Current enforcement contract

| Resource | Current manifest grant | Runtime behavior | Stable failure |
|---|---|---|---|
| Project files | `filesystem = ["read"]` / `["write"]` | Resolves under the canonical project root. Rejects `..`, absolute paths, `@downloads`, and symlink escapes. | `P1034` when not granted; `P1014` for an unsafe path. |
| HTTP(S) | `network = ["http"]` | Enables bounded `http.get`; URL validation still applies. | `P1034` when not granted. |
| Selected executables | `process = ["git", "yt-dlp", "curl", "ffmpeg", "python", "python3", "node"]` | Enables only the named executable through argument-safe process invocation, never a shell. `bridge.call` accepts only fixed Python/Node runtime selectors and a validated project-local script path. | `P1034` when not granted. |
| Media download | `media = ["download"]` plus `filesystem = ["write"]` | Enables `media.download` with safe output-path checks. | `P1034` when either grant is absent. |

Inspect permissions before execution:

```bash
padma capabilities .
```

This command reads only `padma.toml`; it does not import or execute Padma source. A capability string is part of the project review surface, alongside ordinary source changes.

## Compatibility mode versus project mode

`padma file.pd` is kept for Python-like Termux compatibility. It retains existing narrow path validation and executable allowlists so established single-file scripts do not stop working without a manifest.

`padma .` is the recommended mode for new and multi-file work. It is deny-by-default, validates manifest grants, and scopes declared filesystem operations to the canonical project directory. A project must not rely on its caller's current directory to obtain extra file access.

> A capability grant is not a privilege-escalation mechanism. It only permits a runtime operation that Padma already supports and validates. It cannot enable a shell, arbitrary binaries, absolute paths, arbitrary package installation, hidden background execution, or access to secrets.

## Auditable escalation protocol

No broader capability may be implemented merely by adding a new string that users can place in a manifest. Before enabling a new resource class, Padma maintainers must complete each item below in a public change set.

| Gate | Required evidence |
|---|---|
| Resource definition | A narrowly named grant, a one-sentence threat model, and an explicit list of permitted APIs. Broad grants such as `filesystem:all`, `network:all`, `process:shell`, or `android:all` are prohibited. |
| Path or host boundary | Canonical root/allowlist enforcement is implemented before documentation. The grant cannot rely on caller working directory, environment-variable expansion, or shell parsing. |
| User-visible audit | `padma capabilities <project>` displays the new grant. A documented manifest diff shows what changed and why; future structured output must be additive and versioned. |
| Localized failure | Bangla and English error rendering uses a stable diagnostic code and names the denied grant plus requested operation. |
| Negative tests | Tests cover missing permission, malformed grant, duplicate grant, path/host escape, and imported-module behavior. |
| Human review | The change has a security-review issue or pull request with a migration note. A release note identifies any newly available authority. |
| Rollback | The release can disable the feature or remove the grant without silently widening a prior grant. |

Sensitive values such as API tokens must never appear in a capability listing, diagnostics, command arguments, or a future audit log. Capabilities identify **authority**, not secrets.

## M9 AI workflow and browser planning

Padma implements a narrow provider-neutral AI workflow boundary and an independent browser planning boundary. Projects may declare `network = ["ai"]` and use `padma ai inspect|plan` to validate a strict `padma-ai.toml` manifest; `ai.workflow` uses one bounded, fixed-shape `json-http-v1` request and returns generated data as inert Padma values.

Projects may separately declare `browser = ["plan"]` and use `padma browser inspect|plan` to validate one strict `padma-browser.toml` manifest. These browser commands read only local project files and render a deterministic plan. They do not read environment values, resolve DNS, connect to a network, start a process, launch a browser, fetch a page, access a profile, cookies, or credentials, or execute page/model output.

| Authority | Manifest grant | Current bound | Authoritative contract |
|---|---|---|---|
| AI inspection-only workflow | Existing `network = ["ai"]` | One validated HTTPS JSON workflow descriptor, environment variable name only, bounded metadata, and no network/model/output action. | [AI workflow foundation](AI-WORKFLOW.md) |
| Provider-neutral AI runtime | Existing `network = ["ai"]` | One fixed `json-http-v1` transport with bounded structured input/output; model output remains inert data. | [AI workflow foundation](AI-WORKFLOW.md) |
| Browser navigation planning | Existing `browser = ["plan"]` | Local exact-origin/navigation validation only; no DNS, browser launch, page fetch, login, form submit, upload/download, post, or payment. | [Browser planning foundation](BROWSER-PLANNING.md) |

The browser design intentionally does not widen `network = ["http"]` or create a broad browser privilege. A future `browser:navigate` adapter, if separately approved, requires a fresh grant, strict destination verification, a bounded operation, and visible user confirmation immediately before navigation. The planning contract reserves no implicit upgrade path from `browser:plan` to any action authority.

## Android and Termux shared-storage boundary

Android storage is a platform permission boundary, not an ordinary Padma path feature. Current project-mode behavior deliberately rejects `@downloads` so a manifest cannot quietly extend a project's write authority outside its canonical root. Direct scripts retain the existing `@downloads` compatibility alias only when the Termux host itself has already been granted storage access.

Before Padma adds any project-mode shared-storage capability, all of the following are mandatory:

1. The user must grant Android/Termux storage access outside Padma through the platform's visible consent flow, such as `termux-setup-storage`. Padma must never invoke, automate, or simulate that consent.
2. The capability must be directory-specific—for example a future `storage = ["downloads:write"]`—rather than a device-wide storage grant.
3. The runtime must canonicalize the host-provided directory, reject traversal and symlink escapes, and require the directory to exist before writing.
4. `padma capabilities` and the project documentation must state the exact shared directory, whether reads/writes are allowed, and how the user can revoke host storage permission.
5. The feature requires Termux device smoke tests and a documented failure path for devices where shared storage is unavailable.

Until those gates are complete, project-mode output belongs inside the project directory. Developers needing to place a generated file in Android Downloads should review and copy it explicitly using Termux tools outside the project permission model.

## Non-goals and future work

The current model does not provide sandboxing from malicious code on a device. A script that receives a process grant runs with the operating-system permissions of the user's Termux session. The model makes that authority explicit and constrained; it does not replace Android, Linux, package-manager, or account security.

Future work may add host allowlists for HTTP, bounded response-size limits, capability preflight in `padma check`, structured capability reports, and a separately reviewed secret-reference system. Each addition must follow the escalation protocol above.
