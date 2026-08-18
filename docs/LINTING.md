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

The first lint release deliberately has a fixed, small rule set. Manifest-level rule selection, warning suppression, severity overrides, and semantic lint rules remain future M6 work; no source comment silently disables a warning today.
