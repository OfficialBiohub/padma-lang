# AI workflow planning example

This project demonstrates the strict, provider-neutral `padma-ai.toml` contract. It contains a **placeholder endpoint** and the **name** of a future secret environment variable; neither is contacted or read by the current planning commands.

Run the local checks without setting any credential:

```bash
cd examples/ai-workflow-plan
padma ai inspect .
padma ai plan .
```

The result explicitly reports `network: "disabled"` and `secret.value: "not-read"`. `padma .` only runs `main.pd` and prints the same inspection-only boundary. No `ai.workflow` request, model execution, prompt, token, provider SDK, training action, or generated-output execution exists in this example.
