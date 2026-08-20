# Browser planning example

This example demonstrates the **browser planning foundation**, not live browser automation. `padma-browser.toml` records two fixed documentation URLs under exact HTTPS origins. The `browser = ["plan"]` capability allows Padma to validate that local policy and render a deterministic descriptor.

Run these commands from this directory with a built or installed Padma binary:

```bash
padma browser inspect .
padma browser plan .
padma .
```

The first command begins with `Padma browser plan manifest (inspection-only)`. The second prints JSON with `"browser": "not-started"`, `"network": "disabled"`, `"dns": "disabled"`, and `"cookies": "not-read"`. The final command prints:

```text
Browser planning is local and inspection-only. No browser will be launched.
```

No browser, network client, DNS resolver, cookie jar, account, credential, profile, or external package is required for this example. It reads only the two local TOML files and does not fetch either documentation URL. Browser login, CAPTCHA bypass, form submission, posting, payment, upload, download, and JavaScript injection are outside this release.

For the complete policy and diagnostics, see [Browser planning foundation](../../docs/BROWSER-PLANNING.md).
