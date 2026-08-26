# Local Scope-of-Work v1

Local Scope-of-Work v1 একটি **local-only review draft** toolkit। এটি strict map validate করে escaped Markdown তৈরি করে এবং explicit `filesystem:write` grant থাকলে project root-এর non-symlink `.md` file-এ লেখে। এটি contract, legal advice, acceptance, payment request, client contact, marketplace submission, or e-signature নয়।

| API | Result | Capability |
|---|---|---|
| `client.scope_markdown(draft)` | In-memory escaped Markdown | None |
| `client.scope_summary(draft)` | Redacted counts and fixed disabled-action markers | None |
| `client.write_scope(path, draft)` | Project-local `.md` draft write, returns `true` | `filesystem = ["write"]` |

The required fields are `clientLabel`, `projectTitle`, `scopeItems`, `exclusions`, `revisionLimit`, and `deliveryTargetLabel`. Optional `reference` and `notes` are bounded text. Lists are non-empty, unique, and limited to 20 items; revision limit is a whole number from `0` to `10`.

Text is bounded, single-line, and rejects control characters, raw `<`/`>` delimiters, URLs (`://`, `www.`), and `@` contact delimiters. Unknown fields—including payment, account, recipient, authorization, platform, contact, and contract-signing fields—are rejected. P1075 is returned without echoing draft values.

```padma
ধরি scope = {"clientLabel": "Rina Studio", "projectTitle": "Portfolio page", "scopeItems": ["Responsive page"], "exclusions": ["Paid ads"], "revisionLimit": 2, "deliveryTargetLabel": "After manual scope confirmation"}
ধরি summary = client.scope_summary(scope)
দেখাও summary["scopeItemCount"]
client.write_scope("out/scope.md", scope)
```

The summary does not return client/project labels, items, exclusions, reference, or notes. It returns counts and fixed markers for `clientContact: "user-review-required"`, and disabled contract signing, marketplace submission, payment, network, and child process actions.

Run [`examples/freelancer-scope-of-work`](../examples/freelancer-scope-of-work/README.md) from Termux. Before manually sharing a draft, review the scope, exclusions, revision label, delivery label, and legal/tax/contract implications yourself. Padma cannot log in, scrape private data, bypass CAPTCHA, send/post/upload, sign/accept, make/withdraw payment, access an account, open a browser, or auto-submit output.
