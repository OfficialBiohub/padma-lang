# Git-linked Render Release Plan

Padma-এর `render` command প্রথমে একটি **reviewable, local-only release plan** তৈরি করে। এটি Render-এ login করে না, source upload করে না, local build চালায় না, token পড়ে না, provider API call করে না, এবং deployment বা rollback করে না। Render API key secret credential; তাই manifest-এ token-এর value রাখা নিষিদ্ধ।[1]

## Required project files

The deployment plan uses `padma.toml`, `padma-deploy.toml`, and this provider manifest:

```toml
# padma-render.toml
[render]
version = "1"
mode = "git-linked"
service = "srv-yourrenderid"
repository = "OfficialBiohub/padma-lang"
branch = "main"
commit = "0123456789abcdef0123456789abcdef01234567"
rollback_deploy = "dep-previoussuccessfuldeploy"
```

The commit must be a complete 40- or 64-character SHA, rather than a mutable branch label. This lets the release plan identify a specific source revision. The repository, branch, service ID, deployment target, and source digest are all local validation inputs.

```toml
# padma.toml
[capabilities]
deployment = ["render"]
```

Run these commands from the project root:

```bash
padma render plan .
padma render inspect .
```

| Plan field | Meaning |
|---|---|
| `sourceDigest` | Digest of Padma's bounded deployment source snapshot |
| `localBuild` | Always `disabled`; Padma does not build an artifact locally |
| `providerBuild` | Requires confirmation in the Render Dashboard |
| `providerApi` | Always `disabled` in this Git-linked mode |
| `rollback.execution` | Always `disabled`; a deploy ID is review metadata only |

Render documents that Git-linked services can be deployed from a connected repository and that a specific commit can be selected for a manual deployment. It also notes that automatic deployment behavior matters when using a specific commit.[2] Therefore, inspect the service, branch, commit, auto-deploy setting, build command, environment variables, and rollback target in the Render Dashboard before confirming anything.

## Current boundary

This mode intentionally does **not** transmit any credentials or source contents. The environment list may name `RENDER_API_TOKEN`, but neither the plan nor manifest contains or reads a token value. Remote API operation belongs to the separate explicit-confirmation adapter contract.

## References

[1]: https://render.com/docs/api "Render API documentation"
[2]: https://render.com/docs/deploys "Render deployment documentation"
