# Padma Packaging

This directory contains downstream distribution recipes. Packaging must build the same `padma` CLI that users can compile from the root crate and must not introduce a second interpreter implementation.

| Location | Target | Current status |
|---|---|---|
| `termux/` | Termux package recipe | Source-build recipe; maintainers must validate it against the installer contract. |

Every new package target must document its supported architecture, required toolchain, installation command, update path, uninstall path, offline assumptions, and test command. Packages must not bundle secrets, silently start services, request Android permissions, or download executable dependencies without a documented integrity policy.
