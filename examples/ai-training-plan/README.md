# AI training planning example

This example validates a bounded, project-local training intent only. It does not contain training data, a model artifact, a backend command, a provider, a credential, or a network setting.

Run the commands from this directory with a built or installed Padma binary:

```bash
padma ai training inspect .
padma ai training plan .
padma .
```

The first command begins with `Padma AI training manifest (inspection-only)`. The second prints JSON with `"training": "not-started"`, `"datasetRead": "disabled"`, `"artifactWrite": "disabled"`, `"childProcess": "disabled"`, and `"network": "disabled"`. The final command prints:

```text
AI training planning is local and inspection-only. No dataset will be read and no training will be started.
```

The `dataset_path` and `artifact_path` are validated as local policy metadata; neither file must exist for planning and neither is opened or created. A future separately installed local backend will require explicit confirmation, active resource enforcement, cancellation, and a new security review before Padma can train or write an artifact. Read the full [AI training planning foundation](../../docs/AI-TRAINING-PLANNING.md) for the contract.
