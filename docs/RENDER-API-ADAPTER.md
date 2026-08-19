# Render API Adapter

## Purpose

`padma render api-plan` validates a reviewed Git-linked release against a provider-specific Render API request. It creates no build artifact, reads no token, uploads no source file, and sends no network request. The separate deploy and rollback commands require a fresh, action-specific confirmation token calculated from the current project source digest.

This adapter is designed for a **Render Git-connected service**. It asks Render to deploy an immutable commit that the project manifest already identifies. It does not create a service, change service configuration, update environment variables, change auto-deploy settings, upload an image, or modify domains.

## Required files

The project must have `padma.toml`, `padma-deploy.toml`, and `padma-render.toml` from the Git-linked release contract. It then adds `padma-render-api.toml`.

```toml
[render_api]
version = "1"
service = "srv-..."
token_env = "RENDER_API_TOKEN"
commit = "40-or-64-character-immutable-git-sha"
clear_cache = "do_not_clear"
rollback_deploy = "dep-..."
```

| Field | Validation and meaning |
|---|---|
| `service` | Must match the reviewed `srv-` ID in `padma-render.toml`. |
| `token_env` | An uppercase environment-variable **name** only. The token value is rejected from manifests and is not read during planning. |
| `commit` | Must be an immutable 40- or 64-character SHA and exactly match `padma-render.toml`. |
| `clear_cache` | Must be `clear` or `do_not_clear`. |
| `rollback_deploy` | Must be a `dep-` ID and exactly match the reviewed Git-linked rollback target. |

## Termux workflow

First review the plan locally. This is safe to run repeatedly because it does not read credentials or contact Render.

```bash
cd ~/padma-lang/examples/render-git-linked
padma render api-plan .
```

The JSON output contains two different `confirmationToken` values: one for `deploy` and another for `rollback`. Before a real deployment, place a Render API key in the Termux session only; do not put it in a Padma file, Git commit, shell history, or screenshot.

```bash
export RENDER_API_TOKEN='your-render-api-key'
padma render deploy --confirm render-PASTE-DEPLOY-TOKEN-HERE .
```

To roll back, create and review a **new** plan, then use only its rollback token.

```bash
padma render api-plan .
padma render rollback --confirm render-PASTE-ROLLBACK-TOKEN-HERE .
```

> Render documents that a specific-commit deploy and a rollback do not turn off Render auto-deploy. Review the service's auto-deploy setting in Render before using either command. A rollback deploy target returns the selected prior deploy; it does not restore all mutable service configuration. [1] [2]

## Security boundary

| Padma does | Padma does not do |
|---|---|
| Sends a single HTTPS POST only after a matching typed confirmation token and required environment variable are present. | Run a local build, upload an artifact, create services, alter secrets, domains, disks, or auto-deploy settings. |
| Binds the confirmation token to action, service, commit, rollback target, and current source digest. | Store, print, log, or include a token value in the plan. |
| Uses a 60-second bounded provider request and redacts the token from returned diagnostic text. | Execute deploys, rollbacks, or retries in the background. |
| Requires a separate command and separate fresh confirmation for rollback. | Roll back automatically after a failed deployment. |

## References

[1]: https://api-docs.render.com/reference/create-deploy "Render API: Trigger deploy"
[2]: https://api-docs.render.com/reference/rollback-deploy "Render API: Roll back deploy"
