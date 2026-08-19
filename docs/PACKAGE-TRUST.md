# Padma Local Package Trust Foundation

Padma M9-এর প্রথম package workflow **network registry client নয়**। এটি project-এর নিজস্ব `packages/` directory থেকে locally reviewed source inspect, validate, digest, lock এবং verify করে। কোনো package স্বয়ংক্রিয়ভাবে download, install, build-hook চালানো, বা execute করা হয় না। এই choice supply-chain trust-এর প্রথম boundary: application code import করার আগে source, manifest, digest, declared capability, এবং lockfile explicitly check করা যায়।

> **Trust rule:** `padma package lock` এবং `padma package verify` source যাচাই করে; এগুলো কোনো package code run করে না।

| Command | Effect | Does not do |
|---|---|---|
| `padma package lock [project]` | Verified local dependency metadata দিয়ে canonical `padma.lock` লেখে | Download, execute, or mutate a package |
| `padma package verify [project]` | Source manifest, bounded tree, SHA-256 digest, requested capabilities, এবং current lockfile মিলিয়ে দেখে | Repair or overwrite a mismatched lockfile |
| `padma package inspect <name> [project]` | One direct local package-এর version, source path, digest, exports, এবং declared capabilities দেখায় | Import or run that package |

## Package layout

Project-এর dependency source শুধু `packages/` directory-এর নিচে থাকবে। The package source must be a real directory—not a symbolic link—and it may contain at most **256 regular files** and **5 MiB** of source data. Its `padma-package.toml` is excluded from the hash so that the manifest can record the hash of its payload without a circular self-reference.

```text
my-project/
├── padma.toml
├── padma.lock
├── src/
│   └── main.pd
└── packages/
    └── helper/
        ├── padma-package.toml
        └── main.pd
```

## Project declaration

Add a direct local dependency to `padma.toml`:

```toml
[padma]
name = "my-project"
version = "0.1.0"
entry = "src/main.pd"
locale = "bn"

[dependencies]
helper = "packages/helper"

[capabilities]
filesystem = []
network = []
process = []
media = []
server = []
database = []
```

Dependency names begin with an English letter and may contain only letters, digits, `_`, and `-`. The declared path must be a relative path under `packages/`; absolute paths, `..`, `@downloads`, and symlink escapes are rejected.

## Package manifest

Create `packages/helper/padma-package.toml` after computing the digest of the package payload. A package is deliberately explicit about its public exports and its requested capabilities.

```toml
[package]
name = "helper"
version = "1.0.0"
entry = "main.pd"
exports = ["greet"]
digest = "sha256:replace-with-the-verified-64-character-digest"

[capabilities]
filesystem = []
network = []
process = []
media = []
server = []
database = []
```

The current project must grant every capability requested by a package. A package that asks for a capability not granted by the project is rejected during lock/verify with `P1034`; a package manifest, path, digest, file-tree, or lock mismatch uses `P1044`.

## Termux workflow

After independently reviewing the package source, run:

```bash
cd my-project
padma package inspect helper
padma package lock
padma package verify
```

`padma.lock` is deterministic JSON. Running `padma package lock` twice without changing the project manifest, package content, or package manifest produces the same file. If any package file changes later, `padma package verify` fails rather than silently updating the lockfile. Review the changed code and digest before deliberately rerunning `padma package lock`.

## Integrity format

Padma computes SHA-256 over a canonical file stream: source files are ordered by normalized relative path, and each path plus byte length plus raw bytes contributes to the digest. The implementation includes a standard SHA-256 test vector (`sha256("abc")`) and rejects malformed lowercase `sha256:<64-hex>` values. This mirrors the general integrity-lock principle documented for npm lockfiles, where integrity values are recorded to make installs reproducible.[1]

## Current limits

This milestone provides **metadata verification and reproducible local locking only**. It does not yet provide a hosted registry, remote download, package publishing, scoped names, signature verification, dependency graph resolution, source compilation, module imports from packages, semver range solving, or lifecycle scripts. In particular, Padma does not adopt install-time lifecycle scripts; package lifecycle automation is a supply-chain execution boundary that requires a separate security model.[2]

## References

[1]: https://docs.npmjs.com/cli/v8/configuring-npm/package-lock-json/ "npm package-lock.json documentation"
[2]: https://docs.npmjs.com/cli/v8/using-npm/scripts/ "npm lifecycle scripts documentation"
