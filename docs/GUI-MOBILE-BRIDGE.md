# Padma GUI/Mobile Renderer Bridge

Padma M9-এর প্রথম GUI/mobile layer কোনো Android app framework, APK builder, WebView wrapper, native bridge, device controller, বা Android permission manager নয়। এটি একটি **project-local renderer contract**: Padma project একটি read-only manifest দিয়ে বলে দেয় যে তার UI entry এবং assets কোথায়; `padma gui inspect` ও `padma gui plan` শুধু সেই declaration validate ও render plan দেখায়। কোনো renderer process চালু হয় না এবং phone permission, package installation, JavaScript execution, local server, network call, native-code execution, or device action ঘটানো হয় না।

> **Security principle:** A renderer plan is data, not executable configuration. Manifestে command, URL, permission, native module, plugin, script hook, or arbitrary backend settings নেই।

## Why the boundary is narrow

Android’s official WebView security guidance warns that broad `file://` access can expose local application files; it recommends scoped asset loading and disabling file/content access where possible. [1] Android also warns that JavaScript-to-native bridges expose objects to WebView frames and should not be present for untrusted content. [2] Therefore Padma does not generate a WebView bridge, does not accept WebView settings, and does not use a `file://` rendering URL in this milestone.

| Included now | Intentionally excluded |
|---|---|
| Versioned local renderer manifest | Android APK generation |
| Project-relative UI and asset path validation | WebView/native JavaScript bridge |
| Fixed `html-static` renderer backend label | JavaScript execution and plugins |
| Read-only inspect/plan commands | Renderer/server/device process launch |
| Source digest and strict path scope | Network URL, remote asset, or download support |

## Renderer manifest

Create `padma-gui.toml` at the project root:

```toml
[gui]
version = 1
backend = "html-static"
entry = "ui/index.html"
assets = "ui/assets"
title = "Padma Example"
```

The only supported backend is `html-static`. `entry` must be a project-relative `.html` regular file and `assets` must be a project-relative real directory. Both are constrained to the project root; absolute paths, `..`, symlink traversal, `@downloads`, external URLs, permissions, executable hooks, unknown fields, and non-HTML entries are rejected.

## Termux workflow

```bash
cd my-project
padma gui inspect
padma gui plan
```

`inspect` reports normalized manifest fields. `plan` additionally reports the UI source digest and a renderer policy record. It never opens a browser or starts a server. A future adapter must consume this validated plan with its own explicit installation, permission, network, and confirmation policy.

## Minimal static example

```text
my-project/
├── padma-gui.toml
└── ui/
    ├── index.html
    └── assets/
        └── logo.svg
```

```html
<!-- ui/index.html -->
<!doctype html>
<html lang="bn">
  <meta charset="utf-8" />
  <title>Padma Example</title>
  <body>
    <h1>পদ্মা GUI পরিকল্পনা</h1>
    <p>এই static page কোনো native permission বা JavaScript bridge ব্যবহার করে না।</p>
  </body>
</html>
```

## Future adapter requirements

An Android/WebView adapter must be a separately reviewed component. It must maintain the least-permission model and use an explicit domain allowlist for any external content. Android recommends carefully restricting WebView navigation and avoiding a JavaScript interface unless the loaded content is entirely controlled and trusted. [2] If a future adapter needs native messaging, it must reject wildcard origins, disclose sensitive operations, and apply a per-operation confirmation model. [2]

## References

[1]: https://developer.android.com/privacy-and-security/risks/webview-unsafe-file-inclusion "Android: WebViews – Unsafe File Inclusion"
[2]: https://developer.android.com/privacy-and-security/risks/insecure-webview-native-bridges "Android: WebView – Native bridges"
[3]: https://developer.android.com/privacy-and-security/security-best-practices "Android security best practices"
