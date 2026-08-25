# Local Profile Validator Example

এই example `data/profile.json` থেকে local preference পড়ছে, schema অনুযায়ী validate করছে, এবং missing `theme`-এ explicit `light` default ব্যবহার করছে।

```bash
cd ~/padma-lang
cargo build --release
export PATH="$HOME/padma-lang/target/release:$PATH"
cd examples/local-profile-validator
padma .
```

Expected output:

```text
Theme: light
Explicit fields: 2
Defaulted fields: 1
Network: disabled
```

Only `filesystem = ["read"]` is needed because `file.read` loads a project-local JSON file. `profile.validate` and `profile.summary` themselves are in-memory and do not write a file, contact a network, use an account, start a process, inspect a game, change a game account/item, bypass anti-cheat, or create any gameplay advantage.
