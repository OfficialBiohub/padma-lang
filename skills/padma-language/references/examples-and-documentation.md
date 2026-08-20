# Examples and Documentation

## Read This Reference When

Read this file before creating, revising, or publishing a Padma example, tutorial, code snippet, or public capability statement.

## Example Project Standard

Each example must be a standalone project directory containing at least:

```text
example-name/
├── padma.toml
├── main.pd
└── README.md or a linked section in docs
```

Use only the capability grants required by that example. Keep output and generated paths inside its project root. Put mutable output folders behind `.gitkeep` where appropriate, and run the example from a temporary copy during verification.

## Required Public Explanation

Document the exact command, expected output, generated files, required Termux packages/tools, and one paragraph explaining how the code works. Include the safety boundary in plain language.

| Example type | Must state explicitly |
|---|---|
| Media | Authorization/ownership requirement and external downloader prerequisite |
| Website | Whether code only generates files, serves locally, or actually deploys |
| Backend | Whether code creates a response payload, starts local loopback service, or calls remote API |
| Database | SQLite dependency, project-local database location, and capability grant |
| Security | Defensive/local-only purpose; no login bypass, scanning without authorization, or exploitation |
| AI | Provider/network requirement, data handling, and that generated output is not automatically executed |

## Current Practical Examples

Use `docs/PRACTICAL-PROJECT-EXAMPLES.md` as the public guide and keep it synchronized with executable projects under `examples/`. It includes authorized media download, static site file generation, backend response construction, SQLite records, defensive URL inspection, and local password verification.

## Honesty Rule

Never call a plan, manifest validator, project scaffold, or structured response a complete hosted framework. Describe only what the release binary currently executes.
