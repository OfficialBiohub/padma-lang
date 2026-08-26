# Freelancer Attachment Review

This project creates a **local attachment-review manifest** for two declared project files. It records only their labels, SHA-256 checksums, and byte counts, alongside visible destination and ownership labels for a person to review.

```sh
cd ~/padma-lang
cargo build --release
export PATH="$HOME/padma-lang/target/release:$PATH"
cd examples/freelancer-attachment-review
padma .
cat out/attachment-review.md
```

Expected terminal output:

```text
Attachments: 2
Destination review: user-review-required
Upload: disabled
Saved: true
```

The project requests `filesystem = ["read", "write"]` only to checksum project-local regular files and to write `out/attachment-review.md`. Before manually choosing a file in any application, inspect the manifest, compare checksums, verify the visible destination, and decide whether the ownership label is true.

This program does **not** send a message, upload/download a file, submit delivery, sign a contract, request/accept payment, open or control a browser, access an account, use a network, or start a process. Delete the generated local artifact after review if you do not need it:

```sh
rm out/attachment-review.md
```
