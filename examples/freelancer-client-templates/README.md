# Freelancer Client Templates

This project renders one **explicit-input** Bangla-English proposal, a brief, and a copy-only message template. It writes only the proposal review artifact to `out/proposal.md`.

```sh
cd ~/padma-lang
cargo build --release
export PATH="$HOME/padma-lang/target/release:$PATH"
cd examples/freelancer-client-templates
padma .
cat out/proposal.md
```

Expected terminal output:

```text
Template: proposal
Skills: 2
Copy only: user-review-required
true
true
true
```

The project requests only `filesystem = ["write"]` to save its reviewed local Markdown output. `proposal`, `brief`, and `message-template` all render only text you explicitly put in the source. Before using any draft, verify every claim and change it for the actual recipient and context yourself.

Padma does **not** find a client, collect a recipient/contact/account, log in, open or inspect a browser, send/post a message, upload/download files, submit a proposal or delivery, sign a contract, make a payment, access a network, or start a process.

Remove the generated local artifact when finished:

```sh
rm out/proposal.md
```
