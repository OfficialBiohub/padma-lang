# Android Browser Handoff example

This Termux-first example opens **one fixed Python documentation URL** in your visible Android browser after you inspect the local policy and type the foreground confirmation `OPEN`. It is a real URL handoff, but it is not browser automation.

The only required external dependency is a working local `termux-open-url` command. Check it before running the example:

```bash
command -v termux-open-url
```

Run these commands from this directory with a built or installed Padma binary:

```bash
padma browser plan .
padma browser confirm inspect .
padma browser handoff .
```

The first two commands only inspect local files. The third prints the reviewed destination and waits for `OPEN`. If you type `OPEN`, Padma invokes the fixed one-argument command `termux-open-url <reviewed-url>`, and Android displays the URL in your normal browser. If you type anything else, Padma exits without opening the browser.

The project grants `browser = ["plan", "confirm-plan", "handoff", "audit"]`. The raw URL comes only from the digest-bound local manifests; it cannot be supplied through terminal input, a script, an AI response, a header, a cookie, a browser profile, or an arbitrary command argument.

This example enables a small local audit at `audit/handoff.jsonl`. If you type anything other than `OPEN`, Padma cancels without starting `termux-open-url` and records only the fixed cancellation outcome. If you type `OPEN`, it records that the local opener request succeeded or failed. The file never contains the URL, query string, confirmation input, cookies, credentials, browser profile data, page content, or browser output, and it keeps at most 16 records.

After Android opens the browser, you remain in control. This example does not read cookies or credentials, automate login, bypass CAPTCHA, run JavaScript, submit forms, post messages, upload/download, change an account, purchase, or pay. See the full [Android Browser Handoff guide](../../docs/ANDROID-BROWSER-HANDOFF.md).
