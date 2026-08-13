# Contributing to Padma

Thank you for helping build a practical Bangla-English programming language. Padma is designed for learners in Bangladesh without isolating them from the global software ecosystem. Clarity, safety, compatibility, and reproducibility take priority over adding features quickly.

## Before you start

Please search existing issues and RFCs before proposing a change. Open an issue first for a new keyword, syntax rule, standard-library API, code-generation target, security-sensitive feature, or breaking behavior. Small documentation corrections and isolated test fixes can be submitted directly as pull requests.

| Change type | What is required before merge |
|---|---|
| Lexer/parser bug | Regression test that failed before the fix. |
| New diagnostic | Stable `Pxxxx` code, Bengali and English text, and a snapshot/unit test. |
| New syntax | RFC discussion, parser test, interpreter/type-check test, documentation update. |
| Standard library feature | API rationale, safety review, examples, test coverage. |
| External package bridge | Version pinning policy, error mapping, permission model, security review. |
| Breaking change | Approved RFC, migration note, semantic-versioning review. |

## Local setup

```bash
git clone https://github.com/OfficialBiohub/padma-lang.git
cd padma-lang
cargo test
cargo build --release
```

Before opening a pull request, run:

```bash
cargo fmt --check
cargo test
cargo build --release
```

## Language-design rules

A Bengali and English keyword must map to the same canonical internal token. Do not create language-specific behavior; `ধরি` and `let` must always have identical semantics. The compiler must accept UTF-8 source and avoid leaking user secrets in diagnostics, logs, or generated source maps.

Error messages must be actionable. Each diagnostic needs a stable code, exact source location, a clear explanation in the active locale, and a safe next step. Do not use a live machine-translation service in the compiler; translation must be deterministic and available offline.

## Commit and pull-request guidance

Use small, focused commits with imperative titles, for example `Add Bengali alias for boolean literals` or `Report division by zero as P1011`. Pull requests should describe the user-visible behavior, test command and result, compatibility impact, and any remaining limitations.

## Security

Do not publicly disclose a potential vulnerability before maintainers have reviewed it. Report security-sensitive concerns privately to the repository maintainers. A formal security policy will be added before public package publishing or network-capable official libraries are released.
