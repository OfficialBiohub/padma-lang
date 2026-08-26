# Local OpenQASM Interchange Assessment v1

## Purpose and execution boundary

`quantum.assess_openqasm3(circuit, source)` verifies that explicit in-memory ASCII text is **exactly equal** to the deterministic OpenQASM 3.0 text emitted by `quantum.openqasm3(circuit)`. It then returns stable metadata about that same Padma-rendered artifact.

> This is an exact-renderer consistency check for one deliberately small Padma subset. It is **not** a general OpenQASM parser, importer, compiler, circuit executor, file reader, provider adapter, or QPU submission API.

OpenQASM 3 supports static quantum/classical register declarations, standard gate calls, and measurement assignment syntax; Padma v1 only assesses its existing renderer output within that narrower surface. [1] [2]

## API

| API | Input | Result | Capability |
|---|---|---|---|
| `quantum.assess_openqasm3(circuit, source)` | Existing strict Padma circuit map plus exact renderer text. | Deterministic match status and artifact metadata. | None |

The circuit is validated by the existing `P1083` local planning contract. `source` must be non-empty ASCII text at most `1,048,576` bytes and must match the generated text byte-for-byte, including headers, whitespace, instruction order, numeric rotation formatting, and final newline.

## Deterministic result

| Field | Meaning |
|---|---|
| `sourceMatchesRenderer` | Always `true` on success because mismatch is rejected. |
| `sourceBytes` / `sourceSha256` | Byte count and deterministic SHA-256 identifier of the passed text. |
| `qubitCount` / `operationCount` | Original validated circuit-map counts. |
| `renderedGateInstructionCount` | Count after deterministic high-level lowering: `superposition` expands to `h`; `entangle-linear` expands to a `cx` chain. |
| `measurementInstructionCount` | Count of the complete declared measurement mapping. |
| `parser` | Fixed `not-implemented`; no general source parser exists. |
| `import` / `execution` | Fixed `disabled`; no source-to-circuit import or execution occurs. |

The result also fixes `provider = "not-configured"`, `qpu = "disabled"`, `credential = "not-read"`, `network = "disabled"`, and `childProcess = "disabled"`.

## Rejections and limits

`P1089` rejects empty, non-ASCII, oversized, or non-canonical source text, including comments, altered declarations, unsupported gates, changed measurements, or even otherwise-valid QASM whose bytes do not exactly match Padma’s renderer. This strict comparison prevents an inspection result from being mistaken for a broad syntax/parser compatibility claim.

> A successful assessment proves only equality with the current local renderer. It does not prove acceptance by an OpenQASM implementation, hardware compatibility, simulation correctness outside Padma’s bounded runtime, or execution by any provider.

## Explicit exclusions

No `.qasm` file is read or written by this API. It has no lexer/parser for arbitrary OpenQASM source, no round-trip import, no QIR/AST generation, no symbolic parameters, no callback, no automatic optimisation, no VQE/QAOA/QML/Grover framework, no credentials, no network/process action, and no cloud/QPU/provider job.

For the circuit-map/export contract, see [`LOCAL-QUANTUM-PLANNING.md`](LOCAL-QUANTUM-PLANNING.md). For the runnable Termux project, see [`../examples/local-quantum-interchange-assessment`](../examples/local-quantum-interchange-assessment).

## References

[1] [OpenQASM 3: Types and Casting](https://openqasm.com/versions/3.0/language/types.html)  
[2] [OpenQASM 3: Built-in Quantum Instructions](https://openqasm.com/versions/3.0/language/insts.html)
