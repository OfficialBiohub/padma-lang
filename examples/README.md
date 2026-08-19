# Padma Examples

Every file and folder in this directory is intended to be runnable from the repository root or from its own project folder. Examples use local data and conservative capability manifests; they must not contain real credentials, private URLs, or hidden side effects.

| Location | Demonstrates | Run from repository root |
|---|---|---|
| `hello-bn.pd` / `hello-en.pd` | Basic Bangla and English programs | `padma examples/hello-bn.pd` |
| `collections.pd`, `maps.pd`, `iteration.pd` | Lists, maps, ranges, and loops | `padma examples/collections.pd` |
| `function-bn.pd`, `mixed.pd` | Functions and hybrid syntax | `padma examples/function-bn.pd` |
| `input-demo.pd` | Safe interactive input | `padma examples/input-demo.pd` |
| `standard-library.pd` | Built-in text, math, file, and related helpers | `padma examples/standard-library.pd` |
| `modules/` | Imports, namespaces, and exports | `cd examples/modules && padma main.pd` |
| `capabilities/` | Manifest capability grants | `cd examples/capabilities && padma .` |
| `gui-static/` | Static GUI and Android planning manifests | `cd examples/gui-static && padma gui plan . && padma android plan .` |
| `render-git-linked/` | Render planning manifests | `cd examples/render-git-linked && padma render plan .` |

The `youtube-download.pd` example requires an installed `yt-dlp` backend and must only be used for media a user is authorized to download. It does not bypass DRM or access controls.

When adding an example, document its command here or in its folder, keep it deterministic, and add it to CI only if it needs no credentials, network access, or special device state.
