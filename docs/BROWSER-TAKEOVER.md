# Visible Browser Takeover Checklist

Padma can locally inspect a **visible browser takeover checklist** for a sensitive action. The checklist binds one fixed sensitive-action label to one digest-bound reviewed browser-plan URL. It helps a user pause, review the destination, and then complete the sensitive action only in the destination’s own visible browser UI.

> This is not browser automation and it is not approval. The checklist never opens a browser, reads page state, collects a decision, or performs the labelled action.

## Required capability and manifest

The project requires the smallest explicit grants:

```toml
[capabilities]
browser = ["plan", "takeover"]
```

`padma-browser.toml` remains the reviewed exact-origin navigation source. Add a separate project-local `padma-browser-takeover.toml`:

```toml
[takeover]
version = "1"
mode = "visible-user-takeover-only"
browser_plan_digest = "sha256:..."
navigation_index = 1
sensitive_action = "payment"
max_review_seconds = 60
```

The digest must exactly equal the current canonical digest from `padma browser plan`; the one-based index must identify an existing reviewed GET URL. This manifest accepts no raw URL, selector, script, page data, header, cookie, credential, profile, attachment, browser session, approval token, or user decision.

| Field | Rule |
|---|---|
| `version` / `mode` | Exactly `"1"` / `"visible-user-takeover-only"`. All execution modes are rejected. |
| `browser_plan_digest` / `navigation_index` | Must match one current reviewed browser-plan destination exactly. |
| `sensitive_action` | Exactly one label from `login`, `captcha`, `form-completion`, `message-post`, `upload`, `download`, `account-change`, `purchase`, or `payment`. It describes the user’s action; it is not a command. |
| `max_review_seconds` | Local review window from 15 to 300 seconds. No browser session, approval, or completed-action state is created. |

## Commands and practical flow

```bash
padma browser takeover inspect .
padma browser takeover plan .
```

Both commands inspect local manifest files only. Their deterministic descriptor shows the reviewed URL, the sensitive-action label, a manual checklist, and `completion: "not-collected"`.

If a user independently chooses to open the URL on Android, they must use the separate [Android Browser Handoff](ANDROID-BROWSER-HANDOFF.md) project flow. That flow requires a distinct confirmation descriptor and foreground `OPEN` confirmation. A takeover checklist neither opens the browser nor upgrades into handoff authority.

| Descriptor state | Fixed value |
|---|---|
| `takeover.status` | `"user-takeover-required"` |
| `visibleHandoff.status` | `"not-started"` |
| `credentialAccess`, `pageInspection`, `formFill`, `formSubmission`, `posting`, `upload`, `download`, `accountChange`, `purchase`, `payment` | `"disabled"` |
| `browser`, `network`, `dns`, `childProcess` | `"not-started"` or `"disabled"` as applicable |
| `cookies`, `browserProfile`, `userDecision` | `"not-read"` or `"not-collected"` as applicable |

The safe real-world sequence is therefore: first review the local plan and checklist; next, if needed, visibly open the digest-bound URL through the separately confirmed Android handoff; finally, personally log in, handle CAPTCHA, type/fill/submit, upload/download, modify an account, purchase, or pay in the destination-controlled UI. The user may cancel by not running handoff, answering anything other than `OPEN` in the separate handoff flow, closing the browser, or leaving the site. Padma cannot infer, record, or replay that decision.

## Rejection and execution boundary

`P1067` rejects malformed or unsafe takeover manifests: missing capability, unknown/duplicate field, raw browser-control field, invalid digest/index, unsupported action label, unsupported mode, or out-of-policy review window. `P1068` marks the permanent execution boundary for this version: no `padma browser takeover execute` or `run` command exists.

This design follows least-privilege and user-controlled action review: sensitive destination actions remain outside Padma’s authority rather than being converted into automation.[1] [2]

## Termux example

From the repository root:

```bash
padma browser plan examples/browser-takeover
padma browser takeover inspect examples/browser-takeover
padma browser takeover plan examples/browser-takeover
```

All three commands are local-only. They do not open a browser or interact with a payment service.

## References

[1] [OWASP Transaction Authorization Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Transaction_Authorization_Cheat_Sheet.html)

[2] [OWASP AI Agent Security Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/AI_Agent_Security_Cheat_Sheet.html)
