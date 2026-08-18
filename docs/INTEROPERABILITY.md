# Padma interoperability bridge

## Version 1 contract

Padma provides a deliberately narrow, opt-in bridge for invoking a **reviewed local Python or JavaScript file**. The bridge is intended for existing libraries and small reusable helpers; it is not a shell, a package installer, a background-job system, or a way to import arbitrary foreign source into the Padma interpreter.

```padma
ধরি request = {"name": "রাফি", "scores": [4, 5, 5]}
ধরি result = bridge.call("python", "bridges/average.py", request)
দেখাও result["average"]
```

`bridge.call(runtime, script_path, data)` accepts exactly three arguments. `data` must be JSON-compatible: finite numbers, text, true/false, none, lists, and maps with text keys. Padma writes one UTF-8 JSON document to the child process standard input. The bridge script must write one UTF-8 JSON value to standard output; Padma parses that value into an ordinary Padma value. Output is never evaluated as Padma, Python, or JavaScript source.

| Runtime selector | Executable | Required project grant | Script extension |
|---|---|---|---|
| `"python"` | `python` | `process = ["python"]` | `.py` |
| `"javascript"` | `node` | `process = ["node"]` | `.js` |

In project mode (`padma .`), the bridge requires the exact matching process grant before a child process starts. Script paths must be non-empty relative files inside the canonical project root; absolute paths, `..`, Android shared-storage aliases, missing files, and symlink escapes are rejected. Direct single-file mode preserves the same relative-path and extension checks and permits only the two fixed bridge executables.

The runtime selector is not a command string. Padma constructs a fixed argument vector containing only the selected executable and validated script path. It never invokes a shell, expands environment variables, accepts arbitrary bridge arguments, installs dependencies, or starts a hidden background process. Standard input and standard output are each limited to 256 KiB. The process receives a 10-second execution budget, its exit status is checked, and stderr is captured only for process hygiene; script stderr is not copied into a Padma diagnostic because it may contain sensitive material.

## Bridge scripts

The following Python file is a valid bridge script:

```python
import json
import sys

request = json.load(sys.stdin)
scores = request["scores"]
json.dump({"average": sum(scores) / len(scores)}, sys.stdout)
```

The corresponding JavaScript file uses the same protocol:

```javascript
const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const request = JSON.parse(Buffer.concat(chunks).toString("utf8"));
process.stdout.write(JSON.stringify({ average: request.scores.reduce((a, b) => a + b, 0) / request.scores.length }));
```

Bridge code runs with the operating-system authority of the user’s Termux or desktop session. A Padma capability makes that authority reviewable and limits how Padma launches the child process; it does **not** sandbox a malicious Python or JavaScript program. Review foreign bridge files and their dependencies before granting `process = ["python"]` or `process = ["node"]`.

## Stable diagnostics

| Code | Meaning |
|---|---|
| `P1035` | Unsupported bridge runtime selector. |
| `P1036` | Unsafe or invalid bridge script path. |
| `P1037` | Bridge input or output exceeded the fixed limit. |
| `P1038` | Bridge process could not start or exited unsuccessfully. |
| `P1039` | Bridge process exceeded its execution time budget. |
| `P1040` | Bridge standard output was not valid JSON-compatible data. |

The runtime reports these codes in Bangla when the Padma source is Bangla-first and in English when it is English-first. Future runtime selectors, host services, package integrations, secret references, streaming protocols, and cross-platform sandbox claims require a separate reviewed specification and migration note.
