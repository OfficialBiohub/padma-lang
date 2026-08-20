# Browser confirmation-session planning foundation

Padma provides a **local confirmation-session planning foundation** that binds one already reviewed browser-plan destination to a short, future confirmation session descriptor. It does not issue a confirmation token, start a browser, resolve DNS, contact a URL, or create an execution session by itself.

> **A confirmation-session plan is not confirmation and is not broad navigation authority.** `browser:confirm-plan` only validates two local manifest files and produces a deterministic descriptor with `session: "awaiting-confirmation"`. The separately gated Android Browser Handoff may use that descriptor only after a fresh terminal `OPEN` confirmation to pass one reviewed URL to the fixed visible Termux opener; it does not control the browser.

## Project setup

The project must retain the existing browser planning capability and add the independent confirmation-planning capability:

```toml
# padma.toml
[padma]
name = "documentation-review"
version = "0.1.0"
entry = "main.pd"
locale = "en"

[capabilities]
browser = ["plan", "confirm-plan"]
```

Create a strict `padma-browser.toml` policy as described in [Browser planning foundation](BROWSER-PLANNING.md). Run its local plan command and copy its `planDigest` value into the confirmation file:

```bash
padma browser plan .
```

```toml
# padma-browser-confirm.toml
[confirmation]
version = "1"
mode = "local-session-plan"
browser_plan_digest = "sha256:ea28e85f828890c644a52a8b2867a9f2b92bc303ed0f03950f1706736d1769c0"
navigation_index = 1
max_session_seconds = 60
```

The digest is an immutable binding to the canonical local browser plan. If the policy, origins, URL ordering, redirect policy, or navigation limit changes, Padma rejects the old confirmation manifest rather than silently changing its destination.

## Inspect the local descriptor

```bash
padma browser confirm inspect .
padma browser confirm plan .
```

`inspect` begins with `Padma browser confirmation session (inspection-only)` and then prints the JSON descriptor. `plan` prints JSON only. Both commands read only `padma.toml`, `padma-browser.toml`, and `padma-browser-confirm.toml` inside the project root; the confirmation manifest must be a regular file rather than a symlink.

| Field | Version 1 rule |
|---|---|
| `confirmation.version` | Quoted string exactly equal to `"1"`. |
| `confirmation.mode` | Quoted string exactly equal to `"local-session-plan"`. Any run or navigation mode is rejected. |
| `confirmation.browser_plan_digest` | Exact lowercase `sha256:` digest for the current canonical `padma-browser.toml` plan. |
| `confirmation.navigation_index` | One-based index of an already reviewed navigation URL; it must not exceed the browser plan’s URL count. |
| `confirmation.max_session_seconds` | Integer from 15 through 300. It is planning metadata; no live session timer exists because no session is created. |

The manifest accepts no raw URL, headers, cookies, credentials, proxy, selector, script, form, payment, post, upload, download, account, or action field. Duplicate or unknown fields, unsafe modes, mismatched digests, and invalid destination indexes are rejected locally.

## Deterministic no-side-effect descriptor

The plan chooses exactly one existing reviewed GET destination and retains the browser plan’s `deny` redirect policy. Its explicit boundaries are:

| Field | v1 value |
|---|---|
| `session` | `"awaiting-confirmation"` |
| `confirmation.required` and `confirmation.singleUse` | `true` |
| `confirmation.status` | `"not-issued"` |
| `confirmation.challenge` | `"local-runner-required"` |
| `confirmation.modelSupplied` | `"rejected"` |
| `browser`, `network`, and `dns` | `"not-started"`, `"disabled"`, and `"disabled"` |
| `cookies`, `credentials`, and `browserProfile` | `"not-read"` |
| `javascriptExecution`, `formSubmission`, `posting`, `payment`, `upload`, and `download` | `"disabled"` |
| `cancellation` | `"available-before-execution"` |

The descriptor deliberately does not contain a valid approval token. A future separately installed local runner would need to generate an unpredictable, short-lived, single-use challenge locally, bind it to the exact plan digest and approved origin, and honour user cancellation. A model, website, or manifest cannot provide that challenge.

## Privacy and sensitive-action boundary

OWASP recommends server-side allowlisting and disabling automatic redirects to reduce server-side request forgery risk.[1] OWASP also treats transaction authorization as a distinct security property: the user must be able to verify exactly what they authorize, and a prior authorization must not be silently reused for a changed transaction.[2] Padma applies the same conservative principle here: an old plan cannot become a new destination, and a future action cannot inherit broad authority from a prior planning step.

The initial future browser session is designed to be anonymous and isolated. It cannot attach the user’s ordinary browser profile, read/export cookies, access a password manager, capture credentials, bypass a CAPTCHA, inject JavaScript, execute generated output, or operate as a hidden autonomous agent. Any webpage requiring login ends in an authentication-required result; it does not trigger login automation.

Form submission, post/message, upload, download, account modification, deletion, purchase, and payment are not implemented. If any such action is ever considered, it requires a separate capability, a new manifest version, independently reviewed semantics, and a fresh user-visible confirmation immediately before that specific action. This local planning capability has no implicit upgrade path to external execution.

## Diagnostics

| Code | Meaning |
|---|---|
| `P1034` | The project did not declare `browser:confirm-plan`. |
| `P1060` | The confirmation manifest is missing, malformed, unsafe, uses an unsupported mode, has an invalid/mismatched digest, or references an unavailable reviewed URL. Raw sensitive values are not echoed. |
| `P1061` | Browser confirmation or navigation execution is unavailable or prohibited in this Padma version. |

For a credential-free planning example, see [`examples/browser-confirmation-plan`](../examples/browser-confirmation-plan/). For the implemented visible one-URL Termux handoff, see [`ANDROID-BROWSER-HANDOFF.md`](ANDROID-BROWSER-HANDOFF.md) and [`examples/browser-handoff`](../examples/browser-handoff/). The accompanying [browser navigation action-adapter design](BROWSER-ACTION-ADAPTER-DESIGN.md) lists the additional session, revalidation, cancellation, and audit requirements required before any future browser-control implementation can be reviewed.

## References

[1] [OWASP, *Server-Side Request Forgery Prevention Cheat Sheet*](https://cheatsheetseries.owasp.org/cheatsheets/Server_Side_Request_Forgery_Prevention_Cheat_Sheet.html)

[2] [OWASP, *Transaction Authorization Cheat Sheet*](https://cheatsheetseries.owasp.org/cheatsheets/Transaction_Authorization_Cheat_Sheet.html)
