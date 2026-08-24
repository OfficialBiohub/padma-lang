# Filesystem Productivity Example

এই example project-local directory list করে, একটি file-এর SHA-256 checksum নেয়, text search করে, এবং disabled copy plan দেখায়। কোনো extra Termux package, shell command, network, browser, Android permission, বা file mutation লাগে না।

```bash
cd ~/padma-lang
cargo build --release
export PATH="$HOME/padma-lang/target/release:$PATH"
cd examples/filesystem-productivity
padma .
test ! -e workspace/notes-copy.txt && echo "No copy was created"
```

Expected output begins with:

```text
workspace/nested
Search line: 2
Checksum: sha256:...
Copy execution: disabled
No copy was created
```

`fs.list`, `fs.checksum`, `fs.search_text`, এবং all `*_plan` APIs require only `filesystem = ["read"]` because they inspect project-local source files. Copy, move, archive, delete, rename, or write execution does not exist in v1. Paths cannot be absolute, contain `..`, use `@downloads`, or cross a symlink; sources are bounded regular files only.
