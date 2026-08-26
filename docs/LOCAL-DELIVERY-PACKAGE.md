# Verifiable Local Delivery Package

## Purpose

`client.delivery_package_*` prepares a deterministic **local integrity and manual-review package**. It checks explicitly declared project-local files, calculates SHA-256 digests and byte counts, then renders a Markdown manifest with human review steps and a suggested manual folder layout.

> This is not a one-click delivery system. The folder layout is a checklist only: Padma does not create a folder, copy files, render a PDF, open a browser, send, upload, submit, sign, or pay.

## Strict draft schema

```padma
let package = {"packageLabel": "Website delivery", "destinationLabel": "Client compose screen", "ownershipLabel": "I confirm authority to share", "files": [{"path": "data/project-brief.txt", "label": "Project brief"}], "reviewSteps": ["Compare checksum", "Confirm destination and ownership"]}
```

| Field | Contract |
|---|---|
| `packageLabel` | Non-empty bounded human package label; no raw HTML, URL, or contact delimiter. |
| `destinationLabel` | A human-visible manual destination label, not a URL, recipient, or account identifier. |
| `ownershipLabel` | A human review/confirmation label. It is not evidence of authorization. |
| `files` | One to twenty unique strict `{path, label}` maps. |
| `files[].path` | Existing project-relative regular non-symlink file; absolute paths, `..`, `@downloads`, directories, and symlinks are rejected. |
| `reviewSteps` | One to twenty unique bounded review instructions; a person decides whether each is satisfied. |

The output exposes file labels, SHA-256 checksums, and byte counts, but it never copies file bytes to a delivery directory. Its redacted summary exposes only counts and fixed boundary markers.

## APIs, capabilities, and output

| API | Result | Required project capability |
|---|---|---|
| `client.delivery_package_summary(draft)` | Redacted count/status map. | `filesystem:read` |
| `client.delivery_package_markdown(draft)` | In-memory Markdown manifest. | `filesystem:read` |
| `client.write_delivery_package("out/delivery-package.md", draft)` | Local Markdown manifest and `true`. | `filesystem:read`, `filesystem:write` |

The writer uses the client-document output policy: its target must stay within the project root, end in `.md`, and have an existing non-symlink parent. The output is a review artifact only.

## Manual delivery procedure

Run the example, inspect `out/delivery-package.md`, verify every checksum against each file you intend to attach, confirm the visible destination and ownership labels, then choose and copy/select the files yourself in the application you are using. The suggested layout in the manifest is:

```text
delivery/
  delivery-package.md
  selected-files/
```

`selected-files/` is deliberately not created or populated by Padma. This prevents accidental copying, packaging, upload, or submission under a client’s name.

| Explicitly unavailable | Reason |
|---|---|
| PDF summary | No audited local PDF renderer, font policy, artifact limit, or rendering security contract exists in this release. |
| File/folder creation or copy | The manifest describes a manual folder layout; it does not duplicate deliverables. |
| Send, upload, delivery submission, contract, payment | These are external consequential actions and remain the user’s visible decision. |
| CDP/browser/session/CAPTCHA/2FA event handling | The package never starts or attaches to a browser and cannot access a session, page state, or human authentication event. |

## Diagnostics and Termux example

`P1081` reports unsafe package fields, labels, paths, file/review duplicates, source state, or rendered output. Missing capabilities use `P1034`; unsafe writer path boundaries retain shared output diagnostics such as `P1014` and `P1073`.

Run [`examples/freelancer-delivery-package/`](../examples/freelancer-delivery-package/):

```sh
cd ~/padma-lang
cargo build --release
export PATH="$HOME/padma-lang/target/release:$PATH"
cd examples/freelancer-delivery-package
padma .
cat out/delivery-package.md
rm out/delivery-package.md
```
