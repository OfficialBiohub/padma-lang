# Local Portfolio Case Studies and Visible Handoff v1

This local-only toolkit prepares an escaped portfolio case-study draft and a **visible-review handoff**. It does not send a message, upload an attachment, submit a delivery, open a browser, log in, sign, pay, contact a client, or use a marketplace account.

| API | Result | Capability |
|---|---|---|
| `client.case_study_markdown(draft)` | Escaped local portfolio case-study Markdown | None |
| `client.case_study_summary(draft)` | Redacted outcome/link counts and disabled-action markers | None |
| `client.write_case_study(path, draft)` | Project-local non-symlink `.md` draft write | `filesystem = ["write"]` |
| `client.visible_handoff_markdown(draft)` | Manual message/attachment/review checklist Markdown | None |
| `client.visible_handoff_summary(draft)` | Redacted attachment/review counts and disabled send/upload markers | None |

Case studies require `projectTitle`, `challenge`, `solution`, and unique `outcomes`; `publicLinks` and `notes` are optional. Text rejects raw markup, URL/contact delimiters, and income/guarantee claims. Public links, when present, must be unique public `https://` URLs without credentials, query values, fragments, or private-host indicators. Private client/contact/account/payment/authorization/platform fields are rejected with `P1077`.

The visible handoff requires only a destination **label**, a message draft, attachment labels, and review steps. A destination URL, recipient, email, or account is intentionally not accepted. It renders a stop-and-review artifact and returns `P1078` for unsafe data. Sending, posting, upload/download, delivery submission, signing, payment, browser, account, network, and child process are all disabled.

Run [`examples/freelancer-portfolio-handoff`](../examples/freelancer-portfolio-handoff/README.md) in Termux. Review public ownership, link accuracy, attachment selection, destination, message, platform rules, and every final action yourself.
