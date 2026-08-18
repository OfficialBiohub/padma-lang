# Padma linting

`padma lint file.pd` checks source style without running the program. Lint is intentionally separate from `padma check`: `check` reports parser diagnostics; `lint` reports reviewable style warnings only after the file passes syntax checking.

```bash
padma lint examples/hello-en.pd
padma lint --json examples/hello-en.pd
```

Lint exits with status `0` when no warnings are found and `1` when it reports warnings. This makes the default suitable for CI. It never changes source files; run `padma fmt file.pd` to apply the currently supported layout normalization.

| Rule | Default | Meaning | Suggested fix |
|---|---|---|---|
| `L1001` | warning | A line ends with spaces or tabs. | Remove the trailing whitespace or run `padma fmt`. |
| `L1002` | warning | Indentation begins with a tab. | Use four spaces for indentation or run `padma fmt`. |
| `L1003` | warning | Bangla and English language keywords are used together in code. | Prefer one keyword style per file. Mixed syntax remains valid Padma. |

`padma lint --json` writes one JSON object to standard output. Its stable fields are `status`, `path`, `locale`, `warnings[].code`, `warnings[].message`, `warnings[].hint`, and one-based `warnings[].range` positions. Unknown future fields must be ignored by clients.

## Reviewed project suppression

For a project source file, Padma searches from that file's directory upward for the nearest `padma.toml`. A project owner can suppress a narrow, known rule only through a reviewed manifest entry:

```toml
[lint]
disable = ["L1003"]
```

Only `L1001`, `L1002`, and `L1003` are accepted. Unknown or duplicate entries fail manifest validation with `P1032`; a source comment cannot silently disable a warning. Run lint against the project source path—for example, `padma lint src/main.pd`—so its nearest project manifest is found.

Severity overrides and semantic lint rules remain future developer-workflow work.
