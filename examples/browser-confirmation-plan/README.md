# Browser confirmation-session planning example

This example binds the first fixed URL in `padma-browser.toml` to a local confirmation-session descriptor. It does not generate a confirmation token or launch an action session.

Run these commands from this directory with a built or installed Padma binary:

```bash
padma browser plan .
padma browser confirm inspect .
padma browser confirm plan .
padma .
```

The browser plan includes `planDigest`. That digest is copied exactly into `padma-browser-confirm.toml`; change any reviewed browser plan policy and Padma will reject the now-stale confirmation manifest.

The confirmation plan returns `"session": "awaiting-confirmation"`, `"confirmation": {"status": "not-issued"}`, `"browser": "not-started"`, `"network": "disabled"`, `"dns": "disabled"`, `"cookies": "not-read"`, and `"credentials": "not-read"`. It also marks JavaScript execution, form submission, posting, payment, upload, and download as disabled.

No account, login, CAPTCHA bypass, cookie/profile access, password manager, generated output execution, form submit, post, purchase/payment, upload, download, or browser action is supported by this example. The planned session is local metadata only. Read [Browser confirmation-session planning foundation](../../docs/BROWSER-CONFIRMATION-PLANNING.md) for the full contract.
