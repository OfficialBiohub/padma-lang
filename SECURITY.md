# Security Policy

## Supported security boundary

Padma `v0.1.0` is experimental. The maintainers review security-sensitive reports affecting the current `main` branch and the latest published source release. No historical branch receives a compatibility or patch guarantee.

Security-sensitive areas include capability enforcement, project-root path validation, module and asset traversal, process argument handling, external bridge calls, secret redaction, package verification, session primitives, GUI/mobile boundaries, deployment confirmation, Render API adapter behavior, and any action that could write files, contact a network service, or control a device.

## Reporting a vulnerability

Do **not** open a public GitHub issue for a suspected vulnerability. Send a private report to the repository maintainers through the contact route listed in the repository owner profile. Include the affected revision, a minimal reproduction, impact, any known constraints, and a safe way to reproduce the problem.

Maintainers will acknowledge a report, assess reproducibility and impact, coordinate a fix, add a regression test where appropriate, and agree on disclosure timing with the reporter. Please do not publish exploit details until the maintainers have had a reasonable opportunity to investigate and release a fix.

## Scope exclusions

Padma does not claim to provide a hosted service, managed secret vault, Android APK build service, device-management system, browser-bypass system, or production package registry. Findings in third-party tools or services are in scope only when Padma itself causes the unsafe behavior through a documented integration path.

## Safe research expectations

Use isolated test data and accounts you control. Do not access others' data, bypass authentication or CAPTCHA controls, run denial-of-service tests, deploy artifacts to a third-party account, or expose secrets while investigating a report.
