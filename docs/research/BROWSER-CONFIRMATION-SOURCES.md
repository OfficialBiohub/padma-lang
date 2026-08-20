# Browser confirmation-session research notes

This source record informs the planned local-only Padma browser confirmation-session foundation. It records security rationale only; it does not authorize browser execution, authentication, form submission, posting, payment, or any other external side effect.

| Source | Relevant finding | Padma design consequence |
|---|---|---|
| [OWASP Transaction Authorization Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Transaction_Authorization_Cheat_Sheet.html) | A user must be able to identify and acknowledge significant transaction data; authorization must be unique per transaction and state transitions must not be bypassed. | Require a single-use, short-lived confirmation bound to one reviewed plan digest and destination. Never treat a prior confirmation as authority for a changed destination or sensitive action. |
| [OWASP Session Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html) | An authenticated session token is temporarily equivalent to the application’s strongest authentication factor; token disclosure/capture can enable impersonation. | Do not read, export, or reuse browser profiles, cookies, password-manager values, credentials, or session identifiers. Begin future navigation from an empty isolated profile. |
| [OWASP CSRF Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html) | Browsers automatically include session cookies; state-changing requests need server-side authorization/CSRF defenses and user interaction may help for highly sensitive operations. GET must not be used for state change. | Keep the planned foundation local and GET-only. Deny form submission, upload, post, account modification, purchase, payment, and deletion; a future separate adapter would require fresh visible confirmation plus destination-side controls. |

These sources were retrieved on 20 August 2026. The repository’s binding policy remains `docs/BROWSER-ACTION-ADAPTER-DESIGN.md` and the roadmap in `todo.md`.
