# Freelancer Delivery Package

This project builds a **local integrity package** from declared project files. Its Markdown output lists SHA-256 checksums, byte counts, manual review steps, and a suggested delivery folder layout.

```sh
cd ~/padma-lang
cargo build --release
export PATH="$HOME/padma-lang/target/release:$PATH"
cd examples/freelancer-delivery-package
padma .
cat out/delivery-package.md
```

Expected terminal output:

```text
Files: 2
Review steps: 3
PDF: not-provided
Upload: disabled
Saved: true
```

The project requests `filesystem = ["read", "write"]` only to checksum its project-local files and write `out/delivery-package.md`. First review the checksums, destination label, ownership label, and steps. Then, if appropriate, you manually create/select files in the destination application.

Padma does **not** create the suggested folder, copy files, render a PDF, send a message, upload/download, submit delivery, sign a contract, make a payment, open/control a browser, access an account/network, or start a process.

Delete the generated review artifact when finished:

```sh
rm out/delivery-package.md
```
