# Simple Padma Project Structure

Padma project সহজে শুরু হবে, কিন্তু project বড় হলে একই structure ধরে রাখতে পারবে। নতুন project-এর জন্য একটি canonical layout ব্যবহার করুন:

```text
my-project/
├── padma.toml       # Project name, entry file, locale, explicit capabilities
├── padma.lock       # Local reproducibility record
├── src/
│   └── main.pd      # Main Padma program
├── data/            # Optional local input files
├── out/             # Optional local generated reports/files
├── tests/           # Optional future project tests
└── README.md        # Run command and project notes
```

The layout is small enough for a first school program and consistent enough for a data/report, local web response, or multi-file project. `data/`, `out/`, and `tests/` begin empty; they never grant a capability by themselves.

## Four beginner commands

```bash
padma init my-project
cd my-project
padma .
padma check src/main.pd
```

`padma init` creates only local text/template files and refuses a non-empty directory. It does not download packages, start a server, run a shell command, access Android shared storage, create an account, or use the network. `padma .` runs the manifest entry; `padma check` finds a source error without running the program. Use `padma fmt src/main.pd` to format source and `padma lint src/main.pd` for style warnings.

## Beginner and advanced usage

| Need | Simple choice | When the project grows |
|---|---|---|
| One exercise/script | `padma hello.pd` | Move entry to `src/main.pd` only when a manifest or extra files help |
| One project | `padma init name`, then `padma .` | Add helpers below `src/` and import them relatively |
| Local CSV/report | Keep input in `data/`, output in `out/` | Add only `filesystem = ["read", "write"]` when needed |
| Local web response | Keep response code in `src/` | Add `server` grant only for documented local server flow |
| Reusable learning code | Add `tests/` notes and small `.pd` fixtures | Keep external dependencies out until the audited package ecosystem exists |

## Compatibility rules

Existing flat projects remain valid. A project whose `padma.toml` has `entry = "main.pd"` continues to run with `padma .`; it is not rewritten or migrated. Direct scripts continue to use `padma file.pd`. The canonical new structure is a recommendation and the default new-project starter, not a breaking change.

Project mode remains safer for growing projects because sensitive actions are denied unless `padma.toml` explicitly lists a narrow capability. A directory name, `data/` folder, or `out/` folder never grants filesystem, network, process, media, browser, account, or payment authority.

## Next starter templates

The first generated starter is the basic project. Local data/report and local web-response templates will use the same layout and minimum capabilities, but each will be added only after its generated files, safety tests, documentation, and Termux example are complete. No starter template will silently enable network, process, browser, deployment, payment, cloud, or Android device permissions.
