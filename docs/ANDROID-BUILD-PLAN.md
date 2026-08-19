# Android Build Plan

## What it provides

`padma android inspect` and `padma android plan` validate an Android application declaration that is attached to a Padma static GUI project. The plan binds a package identifier, permitted SDK range, expected APK output location, public signing-certificate digest, **signing-key environment-variable name**, declared permissions, and the digest of the reviewed static GUI source.

The commands are intentionally planning-only. They do not require Android Studio, Gradle, an Android SDK, a keystore, a USB cable, ADB, or a connected device. They do not build, sign, install, launch, control, or debug an APK.

## Manifest

Add the `android:plan` grant to `padma.toml`, while retaining the `gui:local` grant:

```toml
[capabilities]
gui = ["local"]
android = ["plan"]
```

Then create `padma-android.toml` in the same project.

```toml
[android]
version = "1"
application_id = "org.example.padmaapp"
min_sdk = 26
target_sdk = 35
artifact = "build/padma-release.apk"
signing_key_env = "PADMA_ANDROID_SIGNING_KEY"
signing_cert_sha256 = "sha256:lowercase-64-hex-character-certificate-fingerprint"

[permissions]
names = ["android.permission.POST_NOTIFICATIONS"]
```

| Field | Policy |
|---|---|
| `application_id` | A dotted Android package identifier only. |
| `min_sdk`, `target_sdk` | 23–35, with `target_sdk` not lower than `min_sdk`. |
| `artifact` | A project-relative expected `.apk` path; it is not written by the plan. |
| `signing_key_env` | An environment-variable **name**, never a key value or keystore path. The plan never reads it. |
| `signing_cert_sha256` | A public, lowercase `sha256:` certificate fingerprint used as signing metadata. |
| `permissions.names` | A small reviewed allowlist: `INTERNET`, `ACCESS_NETWORK_STATE`, `POST_NOTIFICATIONS`, `CAMERA`, and `RECORD_AUDIO`. |

## Termux use

```bash
cd ~/padma-lang/examples/gui-static
padma android inspect .
padma android plan .
```

The JSON plan shows which declared permissions would require runtime user consent (`CAMERA`, `RECORD_AUDIO`, and `POST_NOTIFICATIONS`), but it never asks Android for them. Android requires an app to declare permissions and, for applicable dangerous permissions, request them in context at runtime; a future Android adapter must keep that consent screen inside the Android app and must not attempt automatic elevation. [1]

## Explicit exclusions

| Not implemented by Padma core | Reason |
|---|---|
| APK/AAB build, signing, or keystore access | Signing keys need dedicated custody and a reviewed build environment. [2] |
| ADB, USB debugging, install, launch, or device control | A CLI plan must not acquire control over a user device. |
| Automatic permission grant | Android permission consent belongs to the user-facing Android runtime flow. [1] |
| JNI, native hooks, or automatic native-code execution | Native code needs a separately reviewed, signed Android adapter. |

## Next adapter boundary

An eventual Android adapter must be a distinct project with a locked Android Gradle build, reproducible build inputs, protected signing-key handoff, a visible per-permission rationale, a signed artifact review, and user-initiated device actions. That adapter must not be added to the Padma core command without an independent security review.

[1]: https://developer.android.com/training/permissions/requesting "Android: Request app permissions"
[2]: https://developer.android.com/studio/publish/app-signing "Android: App signing"
