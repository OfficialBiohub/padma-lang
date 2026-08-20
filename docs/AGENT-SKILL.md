# Padma Agent Skill

## Purpose

`skills/padma-language/` is a reusable agent-engineering package for Padma contributors. It gives an agent the project-specific routing, compatibility rules, capability-security boundaries, Termux distribution contract, example standard, and verification gate needed to extend Padma safely. It is **not** a Padma runtime module, package-registry entry, or a new user installation requirement.

## What it covers

| Contribution or task | Skill guidance supplied |
|---|---|
| Parser, interpreter, CLI, diagnostics, LSP | Repository map and stable language contract |
| Builtins, manifests, bridges, database, server, AI, GUI, Android, or deployment plans | Capability gating, project-root validation, secret redaction, and side-effect boundaries |
| Tutorials and example projects | Runnable example structure, prerequisite disclosure, expected output, and honesty rules |
| Termux installer, CI, release, refactor, or final review | Supported command contract, release checks, and commit procedure |

## Package layout

```text
skills/padma-language/
├── SKILL.md
├── references/
│   ├── architecture-and-language-contract.md
│   ├── capabilities-and-security.md
│   ├── examples-and-documentation.md
│   ├── termux-distribution-and-release.md
│   └── verification-and-maintenance.md
└── templates/
    └── feature-change-checklist.md
```

The concise `SKILL.md` is the entry point. It routes a task to only the needed reference modules so an agent does not load unrelated material. The version-controlled repository copy is the canonical source. A managed environment may mirror it to its active local skills directory when making the package available to an agent.

## Maintain the skill

Update the skill in the same focused change as any newly stable Padma contract. For example, a new capability family needs the corresponding capability-reference update, security limitations, tests, documentation, and an example if users can invoke it. Do not put general README material, generated artifacts, credentials, private URLs, or user data inside the skill package.

Validate the package from a managed development environment and then run the normal repository gate:

```bash
python /home/ubuntu/skills/skill-creator/scripts/quick_validate.py "$(pwd)/skills/padma-language"
bash scripts/verify-repository.sh
```

The validator confirms required skill metadata and frontmatter format. The repository gate verifies that adding or revising the guidance has not damaged the interpreter, installer, tools, examples, or authored-document links.

## Boundaries

The skill reinforces current Padma limits. It does not authorize remote deployment, login or CAPTCHA bypass, posting or payment actions, Android permission elevation, APK signing or installation, device control, native-code execution, automatic rollback, or secret disclosure. A feature may be documented as an inspect or plan contract only when that is the actual implementation state.
