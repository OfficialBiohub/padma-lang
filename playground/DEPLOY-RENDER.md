# Deploy Padma Playground on Render

The repository contains a static React/Vite playground. It runs Padma code in the browser through the generated WebAssembly bundle in `client/public/padma-wasm/`; no backend service, database, or secret is required.

## Render Blueprint

If the playground is inside the `padma-lang` repository under `playground/`, use the root `render.yaml` from the repository. Render will run the build from that directory and publish the static output.

```yaml
services:
  - type: web
    name: padma-playground
    runtime: static
    rootDir: playground
    buildCommand: pnpm install --frozen-lockfile && pnpm build
    staticPublishPath: dist/public
    pullRequestPreviewsEnabled: false
```

## Manual Render setup

Create a new **Static Site** in Render and connect `https://github.com/OfficialBiohub/padma-lang`. Set **Root Directory** to `playground`, **Build Command** to `pnpm install --frozen-lockfile && pnpm build`, and **Publish Directory** to `dist/public`.

After deploy, open the generated Render URL on an Android phone. The Run button, horizontal examples strip, editor, and output panel are designed for narrow screens. The application is client-only, so it can be served from a CDN and does not need a persistent server.

## Local verification

```bash
pnpm install --frozen-lockfile
pnpm check
pnpm build
```

The browser runner uses the Padma WASM package first and automatically falls back to the small TypeScript runner if the WASM asset is unavailable during local development.

