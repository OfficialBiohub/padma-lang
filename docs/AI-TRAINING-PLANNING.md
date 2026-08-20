# AI training planning foundation

Padma M10 provides a **local, bounded AI training planning foundation**. It validates a project-owned training intent, normalizes only the manifest’s relative paths, and emits a deterministic resource descriptor. It does not include a built-in universal training engine and does not inspect the dataset or create a model artifact.

> **A training plan is not permission to train.** The plan never reads dataset bytes, writes artifacts, launches a local backend, starts a subprocess, resolves DNS, uses remote compute, reads environment values, sends telemetry, or executes generated output.

## Project contract

Grant only the narrow planning authority in `padma.toml`:

```toml
[padma]
name = "study-model-plan"
version = "0.1.0"
entry = "main.pd"
locale = "en"

[capabilities]
ai = ["training-plan"]
```

Add a regular, project-local `padma-ai-training.toml` file:

```toml
[training]
version = "1"
mode = "plan-only"
backend = "local-adapter-v1"
dataset_path = "datasets/study.jsonl"
artifact_path = "artifacts/study.padma-model"
max_epochs = 3
max_wall_seconds = 300
max_dataset_bytes = 1048576
max_memory_mb = 512
max_cpu_threads = 2
```

| Field | Version 1 rule |
|---|---|
| `version` | Quoted string exactly equal to `"1"`. |
| `mode` | Quoted string exactly equal to `"plan-only"`; any execution mode is rejected. |
| `backend` | Quoted string exactly equal to `"local-adapter-v1"`; it names a future reviewed local handoff, not a binary or command. |
| `dataset_path` | Project-relative path ending in `.jsonl` or `.csv`; absolute paths, `..`, and the Termux shared-storage alias are rejected. The file is not opened. |
| `artifact_path` | Project-relative path below `artifacts/` ending in `.padma-model`. No directory or artifact is created. |
| `max_epochs` | Integer from 1 through 64. |
| `max_wall_seconds` | Integer from 1 through 3,600. |
| `max_dataset_bytes` | Integer from 1,024 through 1,073,741,824. |
| `max_memory_mb` | Integer from 64 through 4,096. |
| `max_cpu_threads` | Integer from 1 through 8. |

Only the manifest is read. The dataset/artifact values are policy metadata, not file handles. Path checks are lexical project-relative validation; the planning command does not test whether either path currently exists.

## Commands and deterministic output

Run the local inspection commands from a project directory, or provide the directory explicitly:

```bash
padma ai training inspect .
padma ai training plan .
```

`inspect` starts with `Padma AI training manifest (inspection-only)` and then prints the JSON plan. `plan` prints JSON only for local editor tooling. A successful plan explicitly records the disabled boundary:

```json
{
  "aiTrainingPlanVersion": 1,
  "mode": "inspection-only",
  "backend": "local-adapter-v1",
  "dataset": {"path": "datasets/study.jsonl", "read": "disabled"},
  "artifact": {"path": "artifacts/study.padma-model", "write": "disabled"},
  "training": "not-started",
  "localBackend": "not-started",
  "remoteCompute": "disabled",
  "datasetRead": "disabled",
  "artifactWrite": "disabled",
  "childProcess": "disabled",
  "network": "disabled"
}
```

The plan also declares the reviewed epoch, time, dataset-size, memory, and CPU-thread ceilings. They are not active process limits in this release because no training process is launched.

## Diagnostics and execution boundary

| Code | Meaning |
|---|---|
| `P1034` | The project did not declare `ai:training-plan`. |
| `P1058` | The training planning manifest is missing, malformed, unsafe, outside resource limits, or requests a non-planning mode. Unsafe raw paths are not echoed. |
| `P1059` | AI training execution is unavailable or prohibited in this Padma version. |

A future training execution adapter must be separately reviewed and explicitly installed by the user. It must accept only the validated local manifest, re-check canonical project paths immediately before use, enforce active CPU/memory/time/dataset limits at the backend boundary, record redacted local audit events, expose cancellation, and request fresh confirmation before creating or overwriting an artifact. It cannot silently select remote compute, collect data, elevate Android permissions, access device controls, inspect credentials, or use browser automation.

The current foundation deliberately has no command to run training. For a credential-free Termux example, see [`examples/ai-training-plan`](../examples/ai-training-plan/).
