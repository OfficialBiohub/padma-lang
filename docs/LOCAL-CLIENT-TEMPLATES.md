# Local Proposal, Brief, and Message Templates

## Purpose

`client.template_*` turns **explicit user-provided content** into a deterministic escaped local proposal, project brief, or copy-only message-template review artifact. It does not generate a claim about skill, income, job outcome, or client acceptance.

> A `message-template` is text for a person to review and manually copy. It is not a recipient-aware message and has no send, post, or marketplace action.

## Strict draft schema

```padma
ধরি draft = {"templateType": "proposal", "title": "বাংলা portfolio page", "overview": "আমি responsive layout এবং clear handover প্রস্তুত করব", "skills": ["HTML", "CSS"], "requirements": ["Mobile layout"], "deliverables": ["Responsive page"], "reviewSteps": ["Review scope", "Copy manually"], "callToActionLabel": "Reply after review"}
```

| Field | Rule |
|---|---|
| `templateType` | Exactly `proposal`, `brief`, or `message-template`. |
| `title` / `overview` | Explicit bounded human text. The overview becomes the copy-only message text for `message-template`. |
| `skills`, `requirements`, `deliverables`, `reviewSteps` | Each is a non-empty unique list containing one to twenty bounded text items. |
| `callToActionLabel` / `notes` | Optional bounded local review text only. A call-to-action label does not trigger an action. |

All accepted text rejects control characters, raw `<`/`>` delimiters, `://`, `www.`, and `@`. The schema also rejects content that makes income, job, or client-acceptance guarantees. Recipient, email/contact, account, platform, authorization, payment, upload, submit, or send fields are unknown fields and are rejected.

## APIs and output policy

| API | Result | Capability |
|---|---|---|
| `client.template_summary(draft)` | Redacted counts/type and fixed action-boundary markers. | None |
| `client.template_markdown(draft)` | Escaped local Markdown in memory. | None |
| `client.write_template("out/proposal.md", draft)` | Project-local reviewed Markdown plus `true`. | `filesystem:write` |

The writer follows the shared local client-document rule: a project root, an existing non-symlink parent, and a `.md` output path are required. Absolute paths, traversal, `@downloads`, non-Markdown targets, and symlinked output components are rejected.

## Manual workflow and boundaries

First inspect the rendered Markdown. Review every requirement, deliverable, and statement for truthfulness, then decide whether it is suitable for the specific recipient and platform. If you choose to use a message template, copy it manually into your visible application and make any final edit yourself.

| This runtime does | This runtime does not do |
|---|---|
| Render provided proposal/brief/message text, counts, and review checkboxes locally. | Find recipients, collect client data, log in, attach a browser session, inspect a page, or infer a client response. |
| Optionally write one project-local `.md` review artifact. | Send/post/upload/download/submit/sign/pay, access an account/network, or start a process. |

## Diagnostics and runnable example

`P1082` denotes unsafe or invalid template input/output. Missing `filesystem:write` for the writer is `P1034`; shared safe-path diagnostics cover invalid output targets.

Run the Bangla-English example at [`examples/freelancer-client-templates/`](../examples/freelancer-client-templates/):

```sh
cd ~/padma-lang
cargo build --release
export PATH="$HOME/padma-lang/target/release:$PATH"
cd examples/freelancer-client-templates
padma .
cat out/proposal.md
rm out/proposal.md
```
