# Freelancer Quote Draft Example

এই standalone Termux project একটি local-only client quote draft তৈরি করে। এটি `out/portfolio-quote.md`-এ review করার জন্য escaped Markdown লেখে; কোনো extra Termux package, network, browser, client contact, marketplace login, payment, email, upload, shared Android storage, বা background process লাগে না।

```bash
cd ~/padma-lang
cargo build --release
export PATH="$HOME/padma-lang/target/release:$PATH"
cd examples/freelancer-quote-draft
padma .
cat out/portfolio-quote.md
```

Expected terminal output:

```text
Document: quote
Deliverables: 2
Payment: disabled
Saved: true
```

The generated Markdown begins with:

```markdown
# Client Quote (Draft)

**Status:** User review required. This is not a contract, payment request, tax calculation, or marketplace submission.
```

`client.document_summary` returns only redacted metadata and disabled-action markers. `client.write_document` needs the project’s minimal `filesystem = ["write"]` grant, and writes only an `.md` file inside the project root. Review every client label, scope, amount, currency, deliverable, and legal/tax consideration yourself before manual use. The program cannot contact a client, submit a proposal, sign/accept a contract, request/withdraw payment, access a marketplace account, open a browser, run a process, or submit generated output.
