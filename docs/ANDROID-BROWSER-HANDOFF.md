# Android Browser Handoff

Padma 0.1.0 includes a narrow **Android Browser Handoff** for Termux. After a project policy, confirmation-session descriptor, capability grant, and foreground terminal confirmation all match, Padma gives exactly one reviewed HTTPS URL to the fixed local Termux opener `termux-open-url`. Android then displays that URL in the user’s own browser application.

> **This is a visible URL handoff, not browser automation.** Padma does not drive the browser after opening it. It does not attach or inspect a browser profile, read/export cookies or credentials, run JavaScript, bypass a CAPTCHA, complete a login, submit a form, post a message, upload/download, modify an account, purchase, or pay.

## Requirements

The command is intended for the same Android phone that runs Termux. The user must have a working local `termux-open-url` command on `PATH`; Padma neither installs a package nor downloads an opener. The Termux ecosystem advises installing the app and its plugins, when used, from one signing source rather than mixing F-Droid, GitHub, and other sources.[1]

The project has three independent browser grants:

```toml
# padma.toml
[padma]
name = "reviewed-browser-handoff"
version = "0.1.0"
entry = "main.pd"
locale = "en"

[capabilities]
browser = ["plan", "confirm-plan", "handoff"]
```

The project must also contain the strict `padma-browser.toml` and `padma-browser-confirm.toml` files described in [Browser planning foundation](BROWSER-PLANNING.md) and [Browser confirmation-session planning](BROWSER-CONFIRMATION-PLANNING.md). The confirmation file binds one one-based reviewed navigation index to the current `planDigest`; it does not accept a raw handoff URL.

## Run one reviewed handoff

From the project root, inspect the local policy first and then run the handoff command:

```bash
padma browser plan .
padma browser confirm inspect .
padma browser handoff .
```

Padma prints the one exact reviewed destination and asks for the uppercase foreground confirmation `OPEN`. If the user types any other value, presses EOF, or cancels the terminal prompt, Padma does not start the opener. When the user types `OPEN`, Padma runs only this fixed process shape, without a shell:

```text
termux-open-url <one-reviewed-https-url>
```

The URL is the one already selected by the digest-bound confirmation manifest. It is not copied from terminal input, an AI response, a webpage, a variable, or an argument. No headers, cookies, credentials, browser profile paths, proxy settings, selectors, scripts, or extra process arguments can be passed to the opener.

| Check before handoff | Required result |
|---|---|
| Capability | `browser:plan`, `browser:confirm-plan`, and `browser:handoff` are all granted. |
| Browser plan | Exact HTTPS origin, fixed GET URL, simple path, and `redirect_policy = "deny"` remain valid. |
| Confirmation plan | Its digest equals the current canonical browser-plan digest and its navigation index selects one approved URL. |
| Foreground decision | A person at the terminal enters the exact value `OPEN` immediately before the opener is called. |
| Local opener | `termux-open-url` is available and returns success. |

## Failure, privacy, and cancellation

If a project grant, manifest, digest, destination, confirmation, or opener check fails, Padma emits a localized diagnostic and makes no fallback browser, network request, retry, or stateful session. The URL is not echoed in a failure diagnostic. If the local opener fails, Padma does not attempt a different executable or remote service.

After the visible Android browser opens, the user—not Padma—controls it. A page that requires login, CAPTCHA, a form, upload, post, payment, or account action remains an application the user operates directly. Closing the Android browser or returning to Termux ends Padma’s involvement; no browser session is retained and no audit content beyond a redacted local outcome is designed for this release.

The `termux-open-url` process only asks Android to display the reviewed URL. It is not an isolated browser profile, a crawler, a WebDriver session, a network proxy, or a generic process runner. A more capable browser navigation/session adapter would require a separately reviewed runtime, destination revalidation immediately before every connection, active session ceilings, isolated state, cancellation, and another security review. The Android/ADB route is intentionally excluded because Playwright describes Android automation as experimental and requires authenticated ADB; this would introduce device-control authority that the handoff does not need.[2]

## Diagnostics

| Code | Meaning |
|---|---|
| `P1034` | One of the required browser capabilities was not declared. |
| `P1060` | The confirmation planning manifest is unsafe, malformed, or no longer matches the reviewed browser plan. |
| `P1062` | The handoff request is unsafe or not freshly confirmed; for example, an unsupported mode, missing `OPEN`, or invalid reviewed binding. |
| `P1063` | The fixed local Termux opener is unavailable or failed. Padma does not retry or select a replacement opener. |

For a copyable phone-only project, see [`examples/browser-handoff`](../examples/browser-handoff/).

## References

[1] [Termux official repository](https://github.com/termux/termux-app)

[2] [Playwright — Android automation API](https://playwright.dev/docs/api/class-android)
