# Browser planning foundation

Padma 0.1.0 provides a **local, inspection-only browser planning foundation**. It validates a small reviewed policy file, `padma-browser.toml`, then prints a deterministic description of the permitted GET navigation metadata. It does not inspect a remote webpage: in this release, “inspect” means inspecting the local manifest only.

> **Browser planning is not browser automation.** The `browser:plan` capability grants only local validation and plan output. It does not grant `network:http`, DNS resolution, browser launch, page fetches, JavaScript execution, profile access, cookies, credentials, login/session automation, CAPTCHA bypass, form submission, messaging, posting, upload, download, payment, or device control.

## Create a reviewed project

Create a normal Padma project and grant only `browser:plan`:

```toml
# padma.toml
[padma]
name = "docs-review"
version = "0.1.0"
entry = "main.pd"
locale = "en"

[capabilities]
browser = ["plan"]
```

Add a project-local `padma-browser.toml` file. Version 1 permits only the fixed `navigation-review` intent and a fixed list of exact HTTPS destinations.

```toml
[browser]
version = "1"
intent = "navigation-review"
redirect_policy = "deny"
max_steps = 2

[allowlist]
origins = [
  "https://docs.python.org",
  "https://www.rust-lang.org"
]

[navigation]
urls = [
  "https://docs.python.org/3/tutorial/",
  "https://www.rust-lang.org/learn"
]
```

Run the local review commands from the project directory:

```bash
padma browser inspect .
padma browser plan .
```

`inspect` prints the heading `Padma browser plan manifest (inspection-only)` followed by the plan. `plan` prints only the JSON descriptor. Neither command launches a browser or contacts a destination. The output includes a deterministic `planDigest`, which is the only value a separate local confirmation-session plan may reference.

## Version 1 manifest rules

| Field | Required rule |
|---|---|
| `browser.version` | Quoted string exactly equal to `"1"`. |
| `browser.intent` | Quoted string exactly equal to `"navigation-review"`. |
| `browser.redirect_policy` | Quoted string exactly equal to `"deny"`; redirects are never followed. |
| `browser.max_steps` | Integer from 1 through 16 and not less than the number of declared URLs. |
| `allowlist.origins` | One through 16 distinct lowercase ASCII HTTPS origins with an exact DNS hostname. |
| `navigation.urls` | One through `max_steps` distinct HTTPS GET URLs that match an allowlisted origin exactly. |

An allowlisted origin is an exact capability boundary, not a suffix match. `https://docs.python.org` does not permit `https://sub.docs.python.org`, `https://docs.python.org.attacker.invalid`, or `https://docs.python.org:8443`.

The v1 origin form rejects HTTP, wildcard and `xn--` labels, IP literals, one-label hosts, ports, userinfo, paths, queries, fragments, whitespace, uppercase characters, and trailing dots. Navigation URLs reject queries, fragments, userinfo, ports, percent encoding, non-ASCII data, repeated slashes, and `.` or `..` path segments. Padma does not normalize unsafe input before comparison; it rejects it locally.

## Deterministic no-side-effect output

The JSON plan preserves navigation order and sorts the reviewed origins. It includes the intent, each fixed GET URL, `maxSteps`, and `redirectPolicy`, plus explicit no-side-effect status fields:

| Output field | v1 value |
|---|---|
| `mode` | `"inspection-only"` |
| `browser` | `"not-started"` |
| `network` and `dns` | `"disabled"` |
| `cookies`, `credentials`, and `browserProfile` | `"not-read"` |
| `environmentRead` and `childProcess` | `"disabled"` |
| `redirectFollowing` and `unsafeActionExecution` | `"disabled"` |

## Local confirmation-session planning

An optional, separate `browser:confirm-plan` capability can bind one reviewed navigation URL to a **local, not-yet-issued** confirmation-session descriptor. It is not browser automation and does not issue an approval token. Add `"confirm-plan"` alongside `"plan"` in `padma.toml`, copy the browser plan’s `planDigest` into `padma-browser-confirm.toml`, then run:

```bash
padma browser confirm inspect .
padma browser confirm plan .
```

The resulting descriptor records `session: "awaiting-confirmation"`, `confirmation.status: "not-issued"`, and all browser/network/profile/action fields as disabled or not-read. It is intended to prevent a future runner from changing a reviewed destination after planning, not to authorize execution. See [Browser confirmation-session planning foundation](BROWSER-CONFIRMATION-PLANNING.md) for the strict manifest, digest binding, and privacy contract.

The manifest has no secret, cookie, header, proxy, selector, JavaScript, or action field. Invalid raw origins and URLs are not repeated in diagnostics, so a malformed value containing userinfo or a token-like string is not copied into output.

## Diagnostics and future boundary

| Code | Meaning |
|---|---|
| `P1034` | The project did not declare `browser:plan`. |
| `P1053` | The browser planning manifest is malformed or violates the strict origin/policy contract. |
| `P1054` | A navigation URL violates the reviewed exact-origin/path policy. |
| `P1055` | A browser execution path is unavailable or prohibited in this Padma version. |
| `P1060` | A local browser confirmation-session manifest is missing, unsafe, malformed, or not bound to the reviewed browser plan. |
| `P1061` | Browser confirmation or navigation execution is unavailable or prohibited in this Padma version. |

Browser execution is deliberately not part of this milestone. A future action adapter would require a new capability, a versioned manifest, destination revalidation immediately before a connection, a fresh visible confirmation for one bounded action, and a separate security review. `browser:plan` has no implicit upgrade path to browsing, login, posting, purchase, upload, download, or any other remote action.

## Termux-friendly example

The repository includes [`examples/browser-plan`](../examples/browser-plan/). It needs only a working `padma` binary; it does not need a browser, network package, account, cookie, or credential. From the repository root:

```bash
cd examples/browser-plan
padma browser inspect .
padma browser plan .
padma .
```

The final command prints a local reminder that no browser will be launched. The two planning commands only read `padma.toml` and `padma-browser.toml` beneath the project directory.
