# AI tool planning foundation

Padma M10 adds a **local AI tool planning foundation**. A project may declare a small reviewed toolset, validate it with `padma ai tools inspect`, and render a deterministic JSON description with `padma ai tools plan`. The foundation exposes neither a callable tool runtime nor an autonomous agent runtime: it reads only local project manifests.

> **A tool plan is not permission to call a tool.** It never starts an agent loop, reads a secret or environment variable, resolves DNS, opens a network connection, starts a child process, invokes `ai.workflow`, reads a file, executes generated output, writes an audit log, or runs in the background.

## Project contract

The project must grant the planning authority and every capability that a declared tool would require if a separate future execution adapter were approved:

```toml
# padma.toml
[padma]
name = "study-tool-plan"
version = "0.1.0"
entry = "main.pd"
locale = "en"

[capabilities]
ai = ["tools"]
network = ["ai", "http"]
filesystem = ["read"]
```

Place one regular, project-local `padma-ai-tools.toml` beside `padma.toml`:

```toml
[agent]
version = "1"
mode = "plan-only"
max_steps = 3
max_wall_seconds = 45
retry_policy = "never"

[toolset]
tools = [
  "ai-workflow",
  "file-read",
  "http-request"
]
```

| Manifest field | Version 1 rule |
|---|---|
| `agent.version` | Quoted string exactly equal to `"1"`. |
| `agent.mode` | Quoted string exactly equal to `"plan-only"`. Any execution mode is rejected. |
| `agent.max_steps` | Integer from 1 through 8. It is a future runbook ceiling, not an active loop. |
| `agent.max_wall_seconds` | Integer from 1 through 600. It is a future runbook ceiling, not an active timeout. |
| `agent.retry_policy` | Quoted string exactly equal to `"never"`; no retry behavior exists. |
| `toolset.tools` | One through three unique tools in a multiline list: `ai-workflow`, `file-read`, or `http-request`. |

Each tool has a distinct capability review requirement. `ai-workflow` requires `network:ai`; `file-read` requires `filesystem:read`; `http-request` requires `network:http`. Missing `ai:tools` or one of those declared capability grants fails locally with **P1034**. Unsupported tool names, malformed lists, duplicate fields, missing sections, missing fields, symbolic links, and non-planning policies fail with **P1056**; raw unsupported tool text is not copied into the diagnostic.

## Commands and output

Run either command in the project directory, or pass the directory as the final argument:

```bash
padma ai tools inspect .
padma ai tools plan .
```

`inspect` begins with `Padma AI tool manifest (inspection-only)` and then prints the plan. `plan` prints JSON only. A successful output lists each reviewed tool with `execution: "disabled"` and explicitly declares its no-side-effect state:

```json
{
  "aiToolPlanVersion": 1,
  "mode": "inspection-only",
  "agent": {"mode": "plan-only", "maxSteps": 3, "maxWallSeconds": 45, "retryPolicy": "never"},
  "network": "disabled",
  "toolExecution": "disabled",
  "agentLoop": "disabled",
  "backgroundExecution": "disabled",
  "generatedOutputExecution": "disabled",
  "auditLog": "not-written"
}
```

## Boundaries and next layer

This foundation deliberately does **not** implement tool calling, model-driven tool selection, autonomous loops, tool retries, scheduling, persistence, agent memory, audit-log writing, code generation/execution, local training, browser control, payments, posting, credential collection, or side effects. The `ai:tools` grant cannot bypass the capability grants already required by each planned tool and cannot activate an undeclared tool.

`P1057` is reserved for an AI tool or agent execution path that is unavailable or prohibited in this Padma version. A future adapter must use a versioned runbook, bounded step/wall-clock limits, cancellation, a user-visible stop control, per-tool schema validation, fresh confirmation before every external side effect, immutable redacted audit events, and new security regression tests. It must not silently upgrade this planning capability to browser navigation, form submission, messaging/posting, upload/download, payment, account changes, or generated-code execution.

For the runnable local example, see [`examples/ai-tools-plan`](../examples/ai-tools-plan/). It needs no account, token, browser, network package, or provider access because it performs no execution.
