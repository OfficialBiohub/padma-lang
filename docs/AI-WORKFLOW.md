# Padma AI Workflow Foundation

## Status and purpose

Padma M9 now provides a **provider-neutral AI workflow foundation**. A project can declare one reviewed workflow in `padma-ai.toml`, validate it locally with `padma ai inspect`, render its deterministic machine-readable plan with `padma ai plan`, and make one explicit structured request with `ai.workflow(...)`.

The inspection commands remain local-only: they neither read a secret value nor resolve a hostname, open a network connection, start a child process, invoke a model, or execute generated output. In contrast, `ai.workflow(...)` is an explicit one-shot runtime call. It may read the named secret and contact the reviewed HTTPS endpoint, but it never retries, follows redirects, exposes the secret in command arguments or user-facing output, or executes generated output.

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

The manifest is deliberately strict because an AI request can transmit user data and incur provider-side cost. The plan and runtime lock the endpoint, adapter, model metadata, timeout, byte limits, and retry policy. The runtime uses one fixed `curl --config -` invocation: the configuration is delivered through standard input, the child environment is cleared except for the OS command path and locale, and the only accepted operation is one JSON `POST` to the reviewed endpoint. It does not support provider SDK selection, arbitrary headers, proxy configuration, TLS bypass, custom shell arguments, redirects, tool use, prompts that trigger commands, model training, browser access, database access, file writes, or automatic retries.

`ai.workflow(...)` accepts exactly one map containing `task`, `instruction`, and `data`. It sends a versioned JSON envelope with the declared model metadata. A valid provider response must be a bounded JSON object containing exactly `protocol: "padma-ai-workflow-v1"` and `output`. Padma returns that output as inert Padma data; it does not parse it as source code, execute it, write it to a file, send it to a browser, or invoke another capability. Missing or unusable credentials report localized **P1051**; a timeout, missing `curl`, non-zero transport result, or bounded-stream failure also reports P1051 without disclosing the secret. Invalid or oversized provider response data reports localized **P1052**.

## Current and next implementation boundary

The current runtime deliberately stops at one reviewed `json-http-v1` request. It has no agent loop, tool calling, streaming, provider failover, dynamic endpoint selection, response-driven capability grant, generated-code execution, or model-training function. Future work must keep this zero-trust boundary while adding only separately reviewed features. `padma ai inspect` and `padma ai plan` remain intentionally local and side-effect free.
