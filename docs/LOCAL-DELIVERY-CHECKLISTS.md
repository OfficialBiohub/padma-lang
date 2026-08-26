# Local Delivery Checklists v1

Local Delivery Checklists v1 একটি **local-only review preparation** toolkit। এটি explicit project deliverables, review items, ও handover items থেকে escaped Markdown তৈরি করে। এটি কোনো delivery submit, upload, client message, acceptance, contract, payment, or marketplace action করে না।

| API | Result | Capability |
|---|---|---|
| `client.delivery_markdown(draft)` | In-memory escaped checklist Markdown | None |
| `client.delivery_summary(draft)` | Redacted item counts and disabled-action markers | None |
| `client.write_delivery_checklist(path, draft)` | Project-local `.md` review draft write, returns `true` | `filesystem = ["write"]` |

Required fields are `projectTitle`, `deliverables`, `reviewItems`, and `handoverItems`. Each list is non-empty, unique, and limited to 20 text items. Optional `reference` and `notes` are bounded text. The schema rejects unknown payment, recipient, contact, account, platform, authorization, upload, delivery-submission, and contract-signing fields.

Raw `<`/`>` delimiters, control characters, URLs (`://`, `www.`), and `@` contact delimiters are rejected. `P1076` is returned without echoing draft values. The local writer rejects absolute/traversal/`@downloads`/symlink paths and non-`.md` output.

```padma
ধরি checklist = {"projectTitle": "Portfolio page", "deliverables": ["Responsive page"], "reviewItems": ["Mobile layout"], "handoverItems": ["Project archive"]}
ধরি summary = client.delivery_summary(checklist)
দেখাও summary["deliverableCount"]
client.write_delivery_checklist("out/delivery.md", checklist)
```

The summary does not return project titles or list values. It returns counts and fixed markers for user-reviewed contact plus disabled upload, delivery submission, signing, payment, network, and child process actions.

Run [`examples/freelancer-delivery-checklist`](../examples/freelancer-delivery-checklist/README.md) from Termux. Review every item before manually sharing or uploading anything. Padma cannot contact a client, upload/download, submit delivery, sign/accept, request/withdraw payment, access a marketplace account, open a browser, use a network, start a process, or auto-submit generated output.
