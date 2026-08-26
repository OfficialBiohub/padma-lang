# Freelancer Delivery Checklist Example

This standalone Termux project creates one reviewed local delivery-checklist Markdown file. It has no network, browser, client message, marketplace login, upload/download, delivery submission, contract, payment, account, shared Android storage, or child process.

```bash
cd ~/padma-lang
cargo build --release
export PATH="$HOME/padma-lang/target/release:$PATH"
cd examples/freelancer-delivery-checklist
padma .
cat out/delivery-checklist.md
```

Expected output:

```text
Deliverables: 2
Review items: 2
Handover items: 2
Saved: true
disabled
```

The project grants only `filesystem = ["write"]` for its one project-local Markdown file. Review each deliverable, handover item, ownership question, and delivery method yourself. The generated checklist is not an upload action, delivery submission, client message, acceptance, contract, payment request, or marketplace action.
