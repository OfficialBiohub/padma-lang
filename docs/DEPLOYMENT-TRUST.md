# Padma Deployment Trust Boundary

Padma M9-এর deployment feature এখন **local, read-only plan validation**। এটি host account-এ login করে না, build চালায় না, files upload করে না, remote server পরিবর্তন করে না, এবং secret value পড়ে না। Its purpose is to give a project a reviewable deployment description and a stable source digest before a future, separately reviewed remote adapter exists.

> **Safety rule:** `padma deploy plan` and `padma deploy inspect` are dry-run-only. Their implementation has no networking, subprocess, archive, shell, credential, or remote-mutation branch.

| Command | Effect | Explicitly not performed |
|---|---|---|
| `padma deploy plan [project]` | Validates `padma-deploy.toml` and prints a deterministic JSON deployment plan | Build, network request, artifact upload, deployment, rollback |
| `padma deploy inspect [project]` | Prints the same validated plan with an inspection heading | Reading secret values or checking whether an environment variable is populated |

## Project layout

The deployment source is intentionally bounded to the regular `src/` tree plus `padma.toml` and an optional `padma.lock`. The project entry must be in `src/`, and no symbolic links are permitted in deployment source. The source snapshot may contain at most **256 files** and **5 MiB**. These bounds are local policy controls, not a claim that every deployment target has the same limits.

```text
my-project/
├── padma.toml
├── padma.lock
├── padma-deploy.toml
├── deploy/
│   └── rollback.json
└── src/
    └── main.pd
```

## Deployment manifest

Create `padma-deploy.toml` beside `padma.toml`:

```toml
[deployment]
version = "1"
entry = "src/main.pd"
target = "static"
base_url = "https://app.example.com"
rollback = "deploy/rollback.json"

[environment]
names = ["PADMA_API_TOKEN", "PUBLIC_API_BASE"]
```

| Field | Required policy |
|---|---|
| `version` | Must be exactly `"1"` |
| `entry` | Must exactly match `[padma].entry` and be a project-relative `.pd` file |
| `target` | One of `static`, `container`, or `loopback` |
| `base_url` | A public `https://` origin only; no path, query, fragment, credentials, localhost, or loopback host |
| `rollback` | Project-relative `.json` descriptor path only; it is metadata, not a remote rollback command |
| `environment.names` | Optional uppercase environment variable **names** only, such as `PADMA_API_TOKEN`; values are never stored or read |

The manifest accepts no provider name, endpoint token, secret literal, remote URL, shell command, arbitrary build command, artifact upload setting, or deployment credential. A manifest failure uses the stable `P1046` diagnostic.

## Termux workflow

Run these commands from the project root:

```bash
padma deploy plan
padma deploy inspect
```

The plan includes the project name and version, selected target, public base URL, declared environment-variable names, deterministic `sha256:` source digest, rollback descriptor, and hard-coded status fields:

```json
{
  "mode": "dry-run-only",
  "network": "disabled",
  "artifactUpload": "disabled",
  "remoteMutation": "disabled"
}
```

Changing any source file in `src/`, `padma.toml`, or `padma.lock` changes the digest. That gives a reviewer a compact identity for the source snapshot. Provenance systems use a similar link between a software artifact and its source/build information to support integrity verification.[1]

## Secret separation

Only names are allowed in the deployment manifest; values must be supplied through a future host-specific secret store or a future explicit user-confirmed operation. Do not place these in `padma-deploy.toml`, `padma.toml`, `padma.lock`, or `.pd` source:

```toml
# Rejected: this is a value, not an environment-variable name.
names = ["token=actual-secret"]
```

OWASP recommends managing application secrets outside source repositories and protecting them through appropriate secret-management controls.[2] The present boundary enforces the first practical part of that recommendation: secrets do not appear in the deployment manifest or plan output.

## Current non-goals

This is not a complete deployment system. It does not publish to Vercel, Render, Netlify, a VPS, Android, a registry, or any other remote provider. It does not create accounts, accept payment, transfer credentials, run shell commands, create Docker images, or provide rollback execution. A future provider adapter needs its own target contract, artifact build reproducibility, explicit user confirmation, secret handling, access policy, audit record, and rollback semantics.

## References

[1]: https://slsa.dev/ "SLSA: Supply-chain Levels for Software Artifacts"
[2]: https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html "OWASP Secrets Management Cheat Sheet"
