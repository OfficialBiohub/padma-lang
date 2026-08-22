# User-Mediated Browser Interaction Drafts

Padma can locally inspect a **reviewable browser interaction draft**. A draft is not browser automation. It binds short text and optional attachment **metadata** to one previously reviewed navigation item, so a person can review or manually copy the content after opening the destination in their own visible browser.

> The draft foundation has no browser control, no page inspection, no form filling, and no action runner. It never decides or records what the person does in the browser.

## Required project files and capability

`padma.toml` must explicitly grant both plan review and draft review:

```toml
[capabilities]
browser = ["plan", "draft"]
```

The project also needs a strict `padma-browser.toml` reviewed navigation plan and a strict `padma-browser-draft.toml`:

```toml
[draft]
version = "1"
mode = "user-review-only"
browser_plan_digest = "sha256:..."
navigation_index = 1
action = "message-draft"
title = "Documentation question"
body = "Please review this message before I manually submit it."
attachment_path = "attachments/context.txt"
max_review_seconds = 60
```

`browser_plan_digest` must exactly match the canonical digest printed by `padma browser plan`, and `navigation_index` must identify an existing reviewed GET URL. The draft cannot specify a raw URL, redirect setting, selector, script, header, cookie, credential, profile, page data, or live browser session.

| Field | Strict policy |
|---|---|
| `version` / `mode` | Exactly `"1"` / `"user-review-only"`. Execution modes are rejected. |
| `action` | One fixed review vocabulary item: `form-draft`, `message-draft`, `upload-draft`, `download-request`, `account-request`, or `payment-request`. It is a label, not an executable operation. |
| `title` / `body` | Local review text only. Title is at most 160 bytes; body is at most 4,096 bytes; control characters are rejected. |
| `attachment_path` | Optional project-relative metadata beneath `attachments/`. Padma does not check, open, hash, read, send, or upload the path. |
| `max_review_seconds` | A local 15–300 second review bound. It expires no browser state because no state is created. |

## Commands

```bash
padma browser draft inspect .
padma browser draft plan .
```

Both commands only read the project manifest files and print deterministic JSON. `inspect` adds a localized heading. The resulting descriptor includes the digest-bound reviewed URL, draft text, attachment metadata state, and the following immutable boundaries:

```json
{
  "mode": "inspection-only",
  "browser": "not-started",
  "network": "disabled",
  "dns": "disabled",
  "attachmentRead": "disabled",
  "upload": "disabled",
  "formSubmission": "disabled",
  "posting": "disabled",
  "payment": "disabled",
  "credentialAccess": "disabled",
  "generatedOutputExecution": "disabled"
}
```

## Visible user takeover

The only practical runtime route remains [Android Browser Handoff](ANDROID-BROWSER-HANDOFF.md): a person first reviews the bound URL and types `OPEN` in the foreground Termux terminal. That handoff opens the one URL in the person’s visible Android browser; it does not transfer draft content to the browser.

After a visible handoff, the person may manually copy or type reviewed draft text. Padma always reports `user-takeover-required` for login, CAPTCHA handling, form completion, posting, upload/download, account change, purchase, and payment. It cannot inject text, click a field, fill a form, collect a decision, read a result, or infer that an action occurred.

## Rejected input and stable diagnostics

`P1065` rejects malformed or unsafe draft manifests, including an unknown or duplicate field, an invalid digest/index, an execution mode, an unsupported action, a raw URL field, a selector/script/header/cookie/credential field, or an attachment traversal path. `P1066` reserves the explicit browser-draft execution boundary: no `padma browser draft execute` or `run` command exists.

The contract intentionally does **not** add CAPTCHA bypass, credential capture, cookie/profile export, JavaScript injection, hidden autonomous workflows, silent form submission, message posting, upload/download, account changes, purchase/payment, or generated-output execution.

## Termux example

Use the safe copy-paste example from the repository root:

```bash
padma browser plan examples/browser-draft
padma browser draft inspect examples/browser-draft
padma browser draft plan examples/browser-draft
```

These commands do not open Android Browser Handoff and do not use an attachment even though the manifest names its metadata path.
