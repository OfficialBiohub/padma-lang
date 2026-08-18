# Padma projects

Padma projects are intentionally local-first. A project is a folder containing `padma.toml`, `padma.lock`, and its `.pd` source files. The initial design supports reproducible single-project execution without downloading code from an untrusted registry.

## Create and run

```bash
padma init my-project
cd my-project
padma .
```

`padma init` refuses to overwrite a non-empty directory. It creates this layout:

```text
my-project/
├── padma.toml
├── padma.lock
└── src/
    └── main.pd
```

## Manifest format

```toml
[padma]
name = "my-project"
version = "0.1.0"
entry = "src/main.pd"
locale = "bn"
```

| Field | Requirement |
|---|---|
| `name` | Non-empty project name. |
| `version` | Non-empty project version string. Semantic versioning is recommended. |
| `entry` | Relative `.pd` path without `..` or an absolute path. |
| `locale` | `bn`, `en`, or `auto`. It controls diagnostic language for the entry source. |

`padma .` reads the manifest from the provided directory, validates the entry path, then runs the entry file. Imports remain relative to the importing file and keep the same path-safety restrictions.

## Modules, aliases, and exports

An ordinary import keeps the established shared-module behavior:

```padma
import "helpers.pd"
print double(21)
```

Use an alias to keep a module in its own namespace:

```padma
import "library.pd" as library
print library.title()
```

Bangla has the equivalent `ইমপোর্ট "library.pd" হিসেবে library` syntax. Existing alias modules without export declarations remain compatible and expose their declared names within the alias. A module that contains one or more explicit `export` / `রপ্তানি` declarations switches to a public API: only the marked variables and functions are visible through the alias.

```padma
# library.pd
export let name = "Padma"
let internal_note = "not public"

export function title() {
  return name
}
```

```padma
# main.pd
import "library.pd" as library
print library.title()
print library.name
# library.internal_note is rejected
```

Exports may wrap only `let` / `ধরি` and `function` / `ফাংশন` declarations. They execute normally in the module, but no unexported symbol leaks through a namespace alias.

## Lockfile and dependencies

`padma.lock` records the project name and version in the current lockfile v1 format. It is reserved for an audited future dependency graph and should be committed to source control.

Remote dependencies, package registries, Git URLs, and arbitrary dependency declarations are deliberately **rejected** today with `P1033`. This is a safety boundary, not a missing silent feature: Padma will not download or execute third-party code until registry identity, package integrity, signatures, lockfile resolution, review workflows, and revocation policy are designed and tested.

## Compatibility policy

The initial project schema is `padma.lock` v1. New compiler releases should preserve existing `padma.toml` behavior within the same Padma major version. If a future schema change is required, the compiler will add an explicit migration command rather than modifying user files during `padma .` execution.
