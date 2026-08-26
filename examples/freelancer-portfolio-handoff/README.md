# Freelancer Portfolio and Visible Handoff Example

This Termux project creates a local case-study Markdown draft and prepares an in-memory manual handoff checklist. It does not send, post, upload, download, submit delivery, sign, pay, log in, open a browser, or access an account.

```bash
cd ~/padma-lang
cargo build --release
export PATH="$HOME/padma-lang/target/release:$PATH"
cd examples/freelancer-portfolio-handoff
padma .
cat out/case-study.md
```

Expected output:

```text
Outcomes: 2
Public links: 1
Attachments: 1
disabled
Saved: true
```

Only `filesystem = ["write"]` is granted, solely for the local Markdown file. Review permissions, claims, links, attachments, destination, and message before any manual external action.
