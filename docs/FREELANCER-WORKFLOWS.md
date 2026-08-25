# Freelancer Workflows Roadmap

Padma’s freelancer workflow goal হলো local preparation, validation, and delivery-quality কাজ সহজ করা—client data organized রাখা, quote/invoice **draft** তৈরি করা, data clean করা, report তৈরি করা, API payload validate করা, and project delivery verify করা। It does not guarantee jobs, contracts, platform acceptance, client payment, or income.

> A document draft is not a contract, legal advice, tax calculation, payment request, account action, or marketplace submission.

## Production-oriented workflow path

| Workflow | Practical local result | Required safety boundary |
|---|---|---|
| Client document | Quote, scope, invoice-draft, delivery checklist, or portfolio case-study Markdown | Local render; user reviews every client/payment/contact decision |
| Data delivery | CSV/JSON cleaning, reconciliation, validation, and client review report | Project-root scope, bounded input/output, no cloud transfer |
| API integration | Validated JSON request descriptor and response-field extraction | Explicit `network:http`, bounded timeout/retry, environment variable name only |
| Content preparation | Proposal brief, project checklist, copy template, and reviewed Markdown artifact | No impersonation, automatic posting, or generated-output submission |
| Developer delivery | Check/lint/test/build plan, checksum manifest, local web/static preparation | Fixed tool contracts, no arbitrary shell or remote deploy action |
| Portfolio evidence | Locally generated case study/report from provided project facts | No fabricated reviews, ratings, earnings, client names, or results |

## First increment: client-document foundation

The first implementation is a strict local quote/invoice-draft renderer. It will receive an explicit bounded draft map; validate document type, client-facing labels, currency, non-negative amount, delivery list, and optional reference/date/notes; then produce escaped Markdown. Writing the final `.md` file requires project `filesystem:write`; rendering in memory does not.

The tool will deliberately omit payment collection, invoice transmission, tax/legal calculation, e-signature, client contact, proposal posting, contract acceptance, marketplace login, account change, review/rating action, and platform/browser automation.

## Marketplace and client boundary

Padma may help prepare local material that a freelancer reviews before use. It will not scrape private client data, capture credentials/cookies, bypass CAPTCHA, automate browser login, post a proposal, submit generated content, accept/sign a contract, message a client, alter an account, initiate payment/withdrawal, or manipulate reviews/ratings. Those actions remain the user’s own visible, informed decisions in the relevant service.

## Reliable working style

Use the simple Padma project layout: place inputs in `data/`, source in `src/`, generated review artifacts in `out/`, and exact commands in `README.md`. Keep capability grants as small as possible. A local quote renderer should need only `filesystem:write` when saving the document; it does not need network, process, media, browser, database, identity, Android, GUI, deployment, or payment authority.
