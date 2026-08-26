# Freelancer Client Reconciliation Example

This project compares two local CSV tables by `id`, produces only counts and checksums, and writes a reviewed local Markdown artifact. It does not reveal identifiers in its summary or send/upload/submit anything.

```bash
cd ~/padma-lang
cargo build --release
export PATH="$HOME/padma-lang/target/release:$PATH"
cd examples/freelancer-client-reconciliation
padma .
cat out/reconciliation.md
```

Expected output:

```text
Matched: 1
Local-only: 1
Client-only: 1
disabled
Saved: true
```

`filesystem = ["read", "write"]` is used only for project-local CSV read and Markdown write. Review the underlying rows, recipients, attachments, and any external decision manually.
