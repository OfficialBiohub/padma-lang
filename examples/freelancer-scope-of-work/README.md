# Freelancer Scope-of-Work Example

This standalone Termux project prepares one reviewed local scope-of-work Markdown file. It uses no network, browser, client message, marketplace login, payment, contract signing, account, shared Android storage, or child process.

```bash
cd ~/padma-lang
cargo build --release
export PATH="$HOME/padma-lang/target/release:$PATH"
cd examples/freelancer-scope-of-work
padma .
cat out/scope-of-work.md
```

Expected output:

```text
Scope items: 2
Exclusions: 2
Revisions: 2
Saved: true
disabled
```

The project grants only `filesystem = ["write"]` for its one `out/scope-of-work.md` file. Review every label and scope item yourself before manually using it. The generated file is not a contract, legal/tax advice, acceptance, invoice, payment request, proposal submission, or message.
