# Padma AI Workflow Foundation

## Status and purpose

Padma M9 now provides an **inspection-only AI workflow foundation**. A project can declare one reviewed, provider-neutral workflow in `padma-ai.toml`, then validate it locally with `padma ai inspect` or render its deterministic machine-readable plan with `padma ai plan`.

This increment **does not send an AI request**. It neither reads the secret value nor resolves a hostname, opens a network connection, starts a child process, invokes a model, or executes generated output. The future `ai.workflow(...)` runtime call remains a separate, security-reviewed implementation step.

> A plan describes reviewed intent; it is never permission to contact an AI provider.

## Project contract

The project must explicitly grant the existing AI network authority in `padma.toml`:

```toml
[padma]
name = "study-helper"
version = "0.1.0"
entry = "main.pd"
locale = "bn"

[capabilities]
network = ["ai"]
```

Place exactly one `padma-ai.toml` next to that file. The first version has one fixed, reviewed workflow:

```toml
[workflow]
version = "1"
adapter = "json-http-v1"
endpoint = "https://ai-gateway.example.com/v1/padma"
secret_env = "PADMA_AI_KEY"
model = "reviewed-model-id"
timeout_seconds = 30
max_input_bytes = 32768
max_response_bytes = 65536
retry_policy = "never"
```

`secret_env` is an environment-variable **name**, never a token, `NAME=value` string, source interpolation, or command fragment. The planning commands only disclose that name and always render the value as `not-read`.

| Field | M9 v1 acceptance rule |
|---|---|
| `version` | Quoted string exactly equal to `"1"`. |
| `adapter` | Quoted string exactly equal to `"json-http-v1"`. |
| `endpoint` | Public HTTPS ASCII DNS URL with optional normalized path; no IP literal, credentials, port, query, fragment, traversal, private host suffix, or whitespace. |
| `secret_env` | A safe uppercase environment-variable name. |
| `model` | A bounded printable identifier, treated only as public metadata. |
| `timeout_seconds` | Integer from 1 through 30. |
| `max_input_bytes` | Integer from 1 through 32,768. |
| `max_response_bytes` | Integer from 1 through 65,536. |
| `retry_policy` | Quoted string exactly equal to `"never"`; automatic retries are not enabled. |

Unknown sections, unknown fields, duplicated fields, missing fields, malformed values, symbolic-link manifests, and unsafe values fail locally with **P1050**. Missing `network:ai` follows the established localized **P1034** capability diagnostic.

## Commands

Run these commands from the project folder, or pass the project folder explicitly:

```bash
padma ai inspect .
padma ai plan .
```

`inspect` prints one concise localized heading followed by the same JSON object as `plan`. `plan` prints JSON only, making it suitable for local checks and editor tooling. A successful plan intentionally declares the disabled boundaries:

```json
{
  "aiWorkflowPlanVersion": 1,
  "mode": "inspection-only",
  "secret": {"environmentName": "PADMA_AI_KEY", "value": "not-read"},
  "network": "disabled",
  "environmentRead": "disabled",
  "dnsResolution": "disabled",
  "childProcess": "disabled",
  "modelExecution": "disabled",
  "generatedOutputExecution": "disabled"
}
```

## Security boundary

The manifest is deliberately strict because a future AI request can transmit user data and incur provider-side cost. The plan therefore locks the endpoint, adapter, model metadata, timeout, byte limits, and retry policy before any transport exists. It does not support provider SDK selection, arbitrary headers, proxy configuration, TLS bypass, custom shell arguments, redirects, tool use, prompts that trigger commands, model training, browser access, database access, file writes, or automatic retries.

Future runtime work must keep the request envelope versioned, use a fixed argument vector, prevent secret values from appearing in command arguments or user-facing output, bound request/response size and JSON depth, and treat all model output as untrusted data. The wider design and its browser-planning counterpart are documented in [M9 AI and Browser Design](M9-AI-BROWSER-DESIGN.md) and the current capability contract is in [Capability Security](CAPABILITY-SECURITY.md).

## Next implementation boundary

The next AI increment may introduce `ai.workflow(input)` only after its structured input/output validation, one-shot transport, no-redirect policy, secret handoff, response schema enforcement, localized `P1051`/`P1052` paths, and security-negative regression coverage are implemented and reviewed. Until then, `padma ai inspect` and `padma ai plan` remain intentionally local and side-effect free.
