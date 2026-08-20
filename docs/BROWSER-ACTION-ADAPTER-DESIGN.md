# Browser navigation action adapter design

**Status: design only; no browser action adapter is implemented in Padma 0.1.0.** The existing `browser:plan` capability remains local, inspection-only, and cannot start a browser. This document records the minimum security contract required before any future browser navigation implementation can be proposed.

> **Design principle:** a reviewed plan is not an execution grant. A separate, fresh, user-visible confirmation must bind one bounded navigation session to one exact reviewed destination. There is no hidden, background, or autonomous browser mode.

## Why the execution boundary must be separate

SSRF weaknesses arise when a program fetches a remote resource without strict URL validation; OWASP recommends allowlisting identified trusted destinations and disabling automatic redirect following in the client.[1] Unvalidated redirects can move a user from a trusted-looking URL to a malicious destination; OWASP recommends avoiding user-controlled destinations, using an allowlist, and clearly showing the destination before the user confirms.[2] Authenticated session tokens are equivalent to the strongest authentication used by a web application, so an automation layer must not capture, export, or silently reuse a user session.[3]

The future Padma adapter therefore begins with **anonymous, GET-only navigation**. Login automation, credential entry, browser profile access, cookie export, CAPTCHA bypass, JavaScript injection, upload/download, form completion, posting/messaging, account changes, deletion, purchase, and payment remain excluded. A later request to add one of those sensitive actions requires a separate capability, a new reviewed manifest version, and fresh confirmation immediately before the action; it cannot be enabled by `browser:plan` or the initial navigation adapter.

## Proposed authority model

| Layer | Proposed authority | What it can do | What it cannot do |
|---|---|---|---|
| Existing foundation | `browser:plan` | Validate and display a local exact-origin navigation policy. | Launch a browser, resolve DNS, fetch a page, or access a profile. |
| Future action session | `browser:navigate` | Start one isolated, confirmation-bound, GET-only navigation session for already-planned URLs. | Follow an unreviewed redirect, submit a form, read/export cookies, use credentials, run scripts, download/upload, or perform a side effect. |
| Future sensitive action | Separate capability per action class | Only a specifically reviewed action after a new, fresh confirmation. | Inherit permission from navigation, an agent plan, or an earlier confirmation. |

The manifest for the action layer must reference an already valid `padma-browser.toml` plan by canonical plan digest, not repeat or expand origin and URL data. The action layer cannot accept raw URLs, arbitrary headers, proxy settings, cookies, selectors, JavaScript, or command arguments. The browser runner must be a separately installed local adapter, not an implicit external binary.

## Required session state machine

The adapter must expose a small, observable state machine. It may not continue after `Cancelled`, `Expired`, `Denied`, or `Completed`.

```text
Planned → AwaitingConfirmation → Navigating → Viewing → Completed
                  │                    │
                  └→ Denied/Expired    └→ Blocked/Cancelled
```

| State | Required controls |
|---|---|
| `Planned` | Load only a valid local browser plan; no DNS, browser, or network action. |
| `AwaitingConfirmation` | Show the exact URL, origin, `GET` method, maximum navigation count, session deadline, and clear notice that no login/form/action is permitted. Bind confirmation to the plan digest and one origin. |
| `Navigating` | Start one isolated ephemeral browser profile only after confirmation. Immediately revalidate scheme, origin, port, userinfo, path form, and resolved address before every connection. |
| `Viewing` | Permit inspection of the rendered page only within the confirmed session ceiling. No script injection, DOM-triggered action, credential collection, or cookie/profile export. |
| `Completed` | Destroy the isolated session and write a redacted immutable action record. |
| `Denied`, `Expired`, `Blocked`, or `Cancelled` | Stop immediately; destroy session state; retain only the redacted outcome record. |

The confirmation must expire quickly, be single-use, and be invalidated if the project manifest, browser plan, destination, or action class changes. It must be created by the local runner from cryptographically strong randomness; it must never be supplied by generated model output or accepted from a webpage.

## Network and destination controls

The planning foundation already accepts only exact lowercase HTTPS DNS origins and simple GET paths. At execution time, that syntactic validation is necessary but insufficient. The runner must parse the target with one standards-conformant URL parser; reject IP literals, single-label hosts, userinfo, non-default ports, percent-encoded ambiguity, DNS rebinding indicators, loopback, private, link-local, multicast, and cloud-metadata address ranges; and re-check every resolved address immediately before connecting. Redirect following is disabled by default. If a future release permits a redirect, it must stop at every redirect response and return to `AwaitingConfirmation` after exact target revalidation; a redirect is never silently followed.

This design intentionally uses exact origin equality rather than suffix or subdomain matching. The reviewed origin `https://docs.example.org` does not authorize `https://sub.docs.example.org`, `https://docs.example.org.attacker.invalid`, `https://docs.example.org:8443`, or an address literal. The runner must use a separate DNS/address policy in addition to the manifest’s hostname comparison, providing defense in depth consistent with OWASP SSRF guidance.[1]

## Privacy, authentication, and audit record

The initial session must start with an empty ephemeral browser profile. It must not attach the user’s regular browser profile, inspect a password manager, read/copy cookies, prompt for passwords, or export any session identifier. If a webpage needs authentication, the session stops with a localized “authentication required” outcome; Padma does not implement login automation or bypass controls.

The future local audit record must be append-only, redacted, and user-readable. It may include a timestamp, plan digest, approved origin, action class, state transition, and non-sensitive outcome code. It must never include query values, cookies, authentication headers, HTML content, screenshots containing sensitive page data, tokens, credentials, form values, or user identifiers. A user must be able to cancel an active navigation locally and inspect the redacted record.

## Implementation gates and test matrix

No code implementation should begin until the following conditions are reviewed together: a versioned action manifest, a local runner identity and installation model, canonical plan hashing, real active time/navigation ceilings, cancellation semantics, audit retention/deletion policy, and a Termux compatibility plan. The action runner must not be developed as a wrapper around arbitrary command-line arguments or a generic remote browser service.

| Test group | Required negative cases |
|---|---|
| Confirmation | Missing, expired, reused, mismatched plan digest, mismatched origin, cancel-before-start, and model-supplied token. |
| Destination | HTTP, IP literal, private/loopback/link-local resolution, userinfo, non-default port, suffix/subdomain escape, percent encoding, and redirect target mismatch. |
| Session privacy | Existing browser profile, cookie read/export, credential prompt/capture, authentication-required page, and persistent state after completion. |
| Actions | Form submit, upload, download, post/message, delete, account change, purchase/payment, script injection, and CAPTCHA bypass requests. |
| Reliability | Browser start failure, DNS change before connect, timeout, navigation ceiling, user cancellation, runner crash, and audit redaction. |

Only a reviewed implementation that passes this full test matrix may expose a new `browser:navigate` capability. Until then, users should use [`BROWSER-PLANNING.md`](BROWSER-PLANNING.md) for the implemented local planning commands.

## References

[1] [OWASP, *Server-Side Request Forgery Prevention Cheat Sheet*](https://cheatsheetseries.owasp.org/cheatsheets/Server_Side_Request_Forgery_Prevention_Cheat_Sheet.html)

[2] [OWASP, *Unvalidated Redirects and Forwards Cheat Sheet*](https://cheatsheetseries.owasp.org/cheatsheets/Unvalidated_Redirects_and_Forwards_Cheat_Sheet.html)

[3] [OWASP, *Session Management Cheat Sheet*](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html)
