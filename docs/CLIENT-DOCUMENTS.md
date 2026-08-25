# Local Client Documents v1

Padma Local Client Documents v1 একটি **local-only preparation toolkit**। এটি explicit quote বা invoice-draft map validate করে, escaped Markdown তৈরি করে, এবং চাইলে project root-এর `out/`-এর মতো existing non-symlink directory-তে একটি `.md` draft লিখে। এটি কাজ, income, client acceptance, contract validity, tax treatment, বা payment outcome guarantee করে না।

> Generated document is a review artifact, not a contract, legal advice, tax calculation, payment request, client message, marketplace submission, or signed agreement.

## APIs

| API | Result | Capability |
|---|---|---|
| `client.document_markdown(draft)` | Escaped deterministic Markdown quote/invoice-draft text | None |
| `client.document_summary(draft)` | Redacted metadata plus fixed action-boundary markers | None |
| `client.write_document(path, draft)` | Writes a project-local non-symlink `.md` document and returns `true` | `filesystem = ["write"]` in project mode |

`client.document_markdown` and `client.document_summary` are in-memory only. They do not read a file, use a network, start a child process, open a browser, contact a client, or create a payment. `client.write_document` is the only mutation: it writes one reviewed Markdown draft to an allowed local path.

## Draft schema

The draft must be one map containing exactly the required fields and, optionally, the listed optional fields.

| Field | Required | Rule |
|---|---:|---|
| `documentType` | Yes | Exactly `quote` or `invoice-draft`. |
| `clientName` | Yes | Single-line text, 1–512 bytes, without raw `<` or `>` delimiters. |
| `projectTitle` | Yes | Single-line text, 1–512 bytes, without raw `<` or `>` delimiters. |
| `currency` | Yes | Three uppercase ASCII letters, for example `BDT` or `USD`. It is a label, not a payment instruction. |
| `amount` | Yes | Finite non-negative number up to `1,000,000,000,000`. It is client-provided draft data, not a calculation or recommendation. |
| `deliverables` | Yes | Non-empty list of 1–20 unique single-line text items, each at most 512 bytes. |
| `reference` | No | Single-line text up to 96 bytes. |
| `validUntil` | No | `YYYY-MM-DD` text label only; Padma does not calculate legal validity. |
| `notes` | No | Single-line text up to 2,048 bytes. |

Unknown fields are rejected. This deliberately rejects fields such as payment URLs, recipient email, account identifiers, payout data, authorization values, or marketplace metadata. `P1073` is returned for malformed, unsafe, oversized, raw-HTML-like, or unsupported draft data; diagnostic details do not echo draft values.

## Bangla-English example

```padma
ধরি draft = {"documentType": "quote", "clientName": "রিমা ডিজাইন স্টুডিও", "projectTitle": "Mobile-friendly portfolio page", "currency": "BDT", "amount": 12500, "deliverables": ["Responsive page", "Source-file handover"], "reference": "Q-2026-07", "validUntil": "2026-12-31"}

ধরি summary = client.document_summary(draft)
ধরি markdown = client.document_markdown(draft)
দেখাও summary["deliverableCount"]
client.write_document("out/portfolio-quote.md", draft)
```

The `summary` never returns the client name, title, amount, reference, notes, or deliverable text. It returns only `documentType`, counts/presence flags, and the fixed markers below.

| Summary marker | Fixed value | Meaning |
|---|---|---|
| `clientContact` | `user-review-required` | You decide whether and how to contact a client. |
| `payment` / `contractSigning` / `marketplaceSubmission` | `disabled` | The runtime cannot request payment, sign, or submit. |
| `network` / `childProcess` | `disabled` | This toolkit cannot connect, fetch, upload, or run a process. |

## Output-path policy

`client.write_document` requires a project root, an existing non-symlink parent directory, and an output path ending in `.md`. The path cannot be absolute, contain `..`, use `@downloads`, or traverse a symlink. Output remains under the project root and under the shared local report output limit.

Use a simple project layout, such as `src/` for code and `out/` for reviewed drafts. The runnable [`freelancer quote example`](../examples/freelancer-quote-draft/README.md) uses the minimum `filesystem = ["write"]` grant.

## Human and marketplace boundary

Before manually using a draft, review its scope, price, currency, dates, ownership, and any legal/tax implications yourself. Padma does not log in to a marketplace; collect private client data; read/export credentials, cookies, or profiles; bypass CAPTCHA; send messages; post proposals; upload or download files; accept/sign contracts; alter an account; initiate payment/withdrawal; manipulate reviews/ratings; or automatically submit generated output. These remain your own visible, informed decisions in the relevant service.

The next future extensions—scope-of-work, delivery checklist, and portfolio case-study schemas—require their own bounded contract, tests, documentation, and review. They are not implemented by Local Client Documents v1.
