# Local Attachment-Review Manifest

## Purpose

`client.attachment_review_*` prepares a **local review artifact** before a person manually attaches files in a client-facing application. It reads only declared project-local files, computes SHA-256 checksums and byte counts, and gives the operator visible destination and ownership labels to review.

> It is not a delivery channel. It has no URL, recipient, account, browser, network, upload, message, submission, signature, payment, or process authority.

## Strict draft schema

```padma
let draft = {
  "destinationLabel": "Client compose screen",
  "ownershipLabel": "I confirm authority to share",
  "attachments": [
    {"path": "data/project-brief.txt", "label": "Project brief"},
    {"path": "data/design-note.txt", "label": "Design note"}
  ]
}
```

| Field | Contract |
|---|---|
| `destinationLabel` | Non-empty bounded human label. It must not be a URL, recipient, or contact value. |
| `ownershipLabel` | Non-empty bounded human confirmation/review label, not evidence of authorization. It must not contain a URL or contact value. |
| `attachments` | One to twenty strict maps. Every entry has exactly `path` and `label`; both path and label must be unique. |
| `attachments[].path` | Project-relative existing regular non-symlink file. Absolute paths, `..`, `@downloads`, directories, and symlinks are rejected. |
| `attachments[].label` | Non-empty bounded human label without raw HTML, URL, or contact delimiter. |

The attachment bytes are **not copied into the report**. The Markdown report contains only human labels, SHA-256 checksums, and byte counts. The redacted summary contains no label, path, or checksum value.

## APIs and project capabilities

| API | Output | Required project capability |
|---|---|---|
| `client.attachment_review_summary(draft)` | Redacted attachment/checksum counts and fixed disabled-action markers. | `filesystem:read` |
| `client.attachment_review_markdown(draft)` | Review Markdown in memory. | `filesystem:read` |
| `client.write_attachment_review("out/review.md", draft)` | Local reviewed Markdown file and `true`. | `filesystem:read`, `filesystem:write` |

The writer follows the existing client-document output boundary: `.md` only, within the project root, with an existing non-symlink parent. It cannot write to an absolute, traversal, `@downloads`, shared-storage, or symlinked location.

## Manual workflow

First run the example's `src/main.pd`. Then inspect `out/attachment-review.md`, compare each displayed digest against the files you intend to select, confirm that the destination label is the currently visible application screen, and decide whether your ownership label is true. Only after those checks may **you** choose files manually in your own browser or application.

| Explicitly unavailable | Reason |
|---|---|
| Sending messages, posts, or email | No client-contact authority exists. |
| Uploading/downloading or delivery submission | A local manifest cannot operate a remote form or transfer files. |
| Contract signing or payment | No authorization, account, contract, or payment authority exists. |
| Browser, account, network, child process | These APIs never initialize those subsystems. |

## Diagnostics

`P1080` identifies an unsafe attachment-review schema, text field, attachment path, duplicate entry, or rendered local manifest. Missing `filesystem:read` or `filesystem:write` capability is `P1034`. Unsafe output paths retain the shared path diagnostics such as `P1014` or the local client-document `.md` policy diagnostic.

## Runnable Termux example

Use [`examples/freelancer-attachment-review/`](../examples/freelancer-attachment-review/). From the repository root:

```sh
cargo run --quiet -- examples/freelancer-attachment-review
cat examples/freelancer-attachment-review/out/attachment-review.md
```

Remove the generated report when finished:

```sh
rm examples/freelancer-attachment-review/out/attachment-review.md
```
