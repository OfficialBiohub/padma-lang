# Android Browser Handoff source record

**Purpose:** This record supports the M10 Termux-first Android Browser Handoff choice. It is not an implementation guide and does not grant browser control.

## Recorded findings

| Source | Finding used in the design | Design consequence |
|---|---|---|
| [Termux official repository](https://github.com/termux/termux-app) | Termux is an Android terminal and Linux environment. Its Android app and plugins must be installed from the same signing source; mixing F-Droid, GitHub, and other signed variants causes compatibility failures. | The Padma handoff does not require a Termux plugin or ask the user to change signing sources. It performs a preflight for one existing local URL opener only. |
| [Termux:X11 official project](https://github.com/termux/termux-x11) | Termux:X11 requires Android 8 or later, an Android companion app, and a Termux companion package; it is a graphical runtime with additional setup. | Termux:X11 is not selected for the first phone-first handoff release. A future isolated visual runner would be a separate reviewed adapter. |
| [Playwright Android documentation](https://playwright.dev/docs/api/class-android) | Playwright Android automation is experimental and requires authenticated ADB. Its Android server can provide OS-user control to a process that knows the endpoint path. | Padma does not use Playwright/ADB for the handoff. It does not add device-control, remote debugging, or browser automation authority. |
| [OWASP Transaction Authorization Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Transaction_Authorization_Cheat_Sheet.html) | Authorization for sensitive actions must bind the user’s awareness to the exact transaction/action data and must not be silently reused. | Padma requires a fresh, foreground confirmation before one fixed URL handoff and excludes all sensitive browser actions. |

## Runtime decision

The first action-layer increment uses **Android Browser Handoff** rather than an embedded browser or ADB-based automation. A fixed, user-installed Termux URL opener receives exactly one already-validated HTTPS URL as one process argument after fresh foreground confirmation. Padma does not pass headers, cookies, credentials, selectors, scripts, browser profile paths, or arbitrary process arguments.

The external browser remains the user’s own visible application. Padma neither receives nor exports its cookie/profile/credential state, and it never performs login, CAPTCHA bypass, form submission, posting, upload/download, account change, purchase, or payment.
