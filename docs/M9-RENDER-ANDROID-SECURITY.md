# Padma M9: Render and Android Adapter Security Contract

## Purpose

This document defines the boundary for the first provider-specific deployment and mobile-build planning layers. It is an implementation contract, not an instruction to deploy, sign, install, grant permissions, or control a device.

## Render contract

Render exposes a REST API for service and deploy management and authenticates API requests with an API key. Render states that API keys are secret credentials and must not be committed or shared.[1] Therefore, a Padma Render manifest may contain only an uppercase environment-variable **name** for a token, never its value. Planning commands must not read the environment variable.

Render can deploy a specific linked-repository commit through its API. Render documents that API-triggered specific-commit deployment does **not** disable auto-deploy by itself.[2] A plan must therefore disclose the commit identity and auto-deploy state as an operator-reviewed policy item rather than imply a rollback-safe automatic action.

Render's rollback endpoint starts a rollback deployment, but API-triggered rollback does not disable auto-deploy. Further, a rollback reuses a selected build artifact but does not restore every current service setting, such as disks and custom domains.[3] Padma's Render API adapter therefore requires a selected `dep-` rollback target, a fresh action-specific confirmation token, and an explicit separate command; it never chains a rollback automatically after a failed deploy.

| Layer | Local planning mode | Future confirmed action mode |
|---|---|---|
| Git-linked release | Validate repo, branch, commit SHA, service ID, source digest, and dashboard review URL | User confirms release in Render Dashboard; Padma does not send an API request |
| Render API request | Validate service ID, token variable **name**, immutable commit SHA, confirmation token, and rollback deploy ID | After a user types the displayed confirmation token, read the named token and send one bounded provider-defined request for the selected commit |
| Rollback | Validate rollback deploy identifier and provider limitations | Require a separate fresh confirmation; never silently chain rollback after a failed deploy |

## Android contract

Android applications run in a limited-access sandbox. Android documentation states that dangerous permissions need a runtime request, should be requested in context after a user action, and should degrade gracefully if denied.[4] Android also recommends data minimization and requesting the smallest set of permissions needed for a user-invoked task.[5]

Android requires APKs to be signed before device installation or update. Android signing documentation distinguishes private app-signing/upload keys from shareable certificates and says private signing keys must be kept secret.[6] Therefore, Padma's core language does not accept keystore paths, key passwords, signing commands, install commands, ADB commands, native hooks, or automatic permission elevation in project manifests.

| Mobile layer | Allowed in initial build-plan validator | Explicitly excluded |
|---|---|---|
| App identity | Package ID, version code/name, minimum SDK, deterministic source digest | Keystore path, password, private key, signing command |
| Permission declaration | Small reviewed allowlist plus human-readable reason | Permission request execution, special permissions, accessibility service, overlays |
| Artifact declaration | Expected `.apk` or `.aab` metadata and certificate fingerprint field | APK/AAB build, signing, upload, install, or update |
| Device boundary | None | ADB, USB, wireless debugging, device control, native process launch |

## References

[1]: https://render.com/docs/api "Render API documentation"
[2]: https://render.com/docs/deploys "Render deployment documentation"
[3]: https://render.com/docs/rollbacks "Render rollback documentation"
[4]: https://developer.android.com/training/permissions/requesting "Android runtime permissions"
[5]: https://developer.android.com/guide/topics/permissions/overview "Android permissions overview"
[6]: https://developer.android.com/studio/publish/app-signing "Android app signing"
