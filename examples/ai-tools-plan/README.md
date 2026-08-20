# AI tool planning example

This example declares a small reviewed AI toolset and runs only local manifest validation. The `ai = ["tools"]` capability permits the planning command, while the other grants document the narrow authority each named tool would require in a separate future execution adapter.

Run the following commands from this directory using a built or installed Padma binary:

```bash
padma ai tools inspect .
padma ai tools plan .
padma .
```

The first command begins with `Padma AI tool manifest (inspection-only)`. The second renders JSON with `"toolExecution": "disabled"`, `"agentLoop": "disabled"`, `"backgroundExecution": "disabled"`, and `"network": "disabled"`. The final command prints:

```text
AI tool planning is local and inspection-only. No tool or agent will be started.
```

No network request, model call, file read, environment read, browser launch, subprocess, secret, agent loop, generated-code execution, or audit-log write occurs. The example has no provider, account, credential, or Termux package requirement beyond the normal Padma binary. For the full contract, see [AI tool planning foundation](../../docs/AI-TOOLS-PLANNING.md).
