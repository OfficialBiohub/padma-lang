# Padma Identity and Session Foundation

Padma M9-এর identity layer-এর উদ্দেশ্য এখনই hosted authentication service চালু করা নয়। উদ্দেশ্য হলো এমন একটি **reviewable, local, opt-in contract** বানানো যেখানে plaintext password, embedded secret, predictable token, URL token, implicit cookie, এবং automatic external login flow-এর কোনো স্থান নেই। Network server, user database provisioning, account recovery, OAuth, MFA, and remote identity-provider integration are explicitly outside this first foundation.

> **Security position:** Padma must not claim password authentication until a supported cryptographic backend can create and verify a slow, salted password hash on the target device. Fast SHA-256 is not an acceptable password-hash substitute. [1]

## Evidence-based rules

| Concern | Padma contract | Reason |
|---|---|---|
| Password storage | Store only an algorithm-tagged salted record from a supported slow password-hashing backend; never plaintext, reversible encryption, or a fast hash | OWASP recommends Argon2id, scrypt, bcrypt, or PBKDF2 and explains why fast hashes are unsuitable for password storage. [1] |
| Secret supply | APIs take a validated environment-variable **name**, never a literal signing key in Padma source | This avoids accidentally committing keys; the secret remains outside project code. |
| Session identifier | Use an OS/CSPRNG-backed value with at least 128 bits when a backend is introduced | OWASP recommends a CSPRNG and at least 128 bits for custom session IDs. [2] |
| Session envelope | Versioned signed payload with subject, issued-at, expiry, session nonce, and signature; reject unknown fields, expired values, and a wrong signing key | A session token is equivalent to the active authentication method and must be protected against hijacking. [2] |
| Browser transport | No default cookie emission. Future HTTP adapter must explicitly opt into `HttpOnly`, `Secure`, `SameSite`, `Path=/`, and narrow host-only scope | Cookie-based sessions require constrained transport; broad domain cookies may expose a session to uncontrolled subdomains. [2] [3] |
| CSRF | State-changing browser requests require an independent synchronizer token or session-bound signed double-submit token; never GET or URL token transport | OWASP recommends validating tokens on state-changing requests and warns that URL transport leaks through history, logs, and referrers. [3] |

## Proposed Padma-facing boundary

The first source-level boundary will be intentionally narrow and local. It will parse and verify public, versioned record formats; it will generate no account database, launch no web service, send no `Set-Cookie` header, and perform no login flow. A future adapter can bind this contract to the loopback server only after it has a reviewed CSPRNG and slow password-hash backend on Termux.

| Record | Canonical fields | Non-negotiable validation |
|---|---|---|
| Password record | `v`, `algorithm`, `salt`, `digest`, algorithm parameters | Exact known algorithm; lowercase encoded bytes; bounded fields; no plaintext field; no unknown field |
| Session envelope | `v`, `sub`, `iat`, `exp`, `sid`, `sig` | Subject/nonce format; expiry after issue; bounded lifetime; future constant-time signature check |
| CSRF request token | `v`, session-bound nonce, signature | Never a URL parameter; associated with one session; a backend validates it before state change |
| Cookie policy | name, path, max-age, secure-only flag, HttpOnly, SameSite | Host-only; no user-provided raw header fragments; no `Domain` attribute in default policy |

## Explicit non-goals

The following are deferred so this contract is not mistaken for a production identity server: user registration, persistent user tables, password reset, password recovery, OAuth/OIDC, MFA, remote session stores, cross-device login, rate limiting, email/SMS delivery, and deployment of a public endpoint. Each requires a separate threat model and operational controls.

## References

[1]: https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html "OWASP Password Storage Cheat Sheet"
[2]: https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html "OWASP Session Management Cheat Sheet"
[3]: https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html "OWASP Cross-Site Request Forgery Prevention Cheat Sheet"
