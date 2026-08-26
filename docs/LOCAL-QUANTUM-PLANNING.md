# Local Quantum Circuit Planning v1

## Purpose and execution boundary

Padma Quantum Planning v1 turns an explicit, bounded local circuit map into deterministic **OpenQASM 3.0 text**. It is a circuit-description feature, not a simulator, algorithm library, or quantum-cloud client.

> OpenQASM describes gates with a fixed number of quantum arguments and describes static qubit/bit registers; this v1 emits a deliberately small compatible subset. [1] [2]

## Strict circuit schema

```padma
ধরি circuit = {"qubits": 2, "operations": [{"gate": "superposition", "targets": [0, 1]}, {"gate": "entangle-linear", "targets": [0, 1]}], "measurements": [{"qubit": 0, "bit": 0}, {"qubit": 1, "bit": 1}]}
```

| Field | Rule |
|---|---|
| `qubits` | Whole number from `1` to `20`. This is a v1 planning bound, not a simulator or QPU capacity claim. |
| `operations` | One to `256` strict `{gate, targets}` maps in declared order. |
| `operations[].gate` | Exactly one of `h`, `x`, `z`, `s`, `t`, `cx`, `superposition`, or `entangle-linear`. |
| `operations[].targets` | Bounded distinct whole qubit indexes. Single-qubit gates require one target; `cx` two; high-level `superposition` and `entangle-linear` use the listed targets. |
| `measurements` | Exactly one `{qubit, bit}` entry for every qubit, with a unique qubit and bit index in `0..qubits-1`. |

`superposition` emits an `h` instruction for each target. `entangle-linear` emits a chain of `cx` instructions over the target list. The generator inserts `reset q;` before all requested gates and emits `c[bit] = measure q[qubit];` for every declared mapping. OpenQASM 3 supports static `qubit[size]` and `bit[size]` registers and measurement assignment syntax of this form. [2] [3]

## APIs and capability boundary

| API | Result | Capability |
|---|---|---|
| `quantum.circuit_summary(circuit)` | Redacted circuit/count metadata and fixed disabled-action markers. | None |
| `quantum.openqasm3(circuit)` | Deterministic OpenQASM 3.0 source text. | None |
| `quantum.write_openqasm3("out/circuit.qasm", circuit)` | One project-local non-symlink `.qasm` file and `true`. | `filesystem:write` |

The writer follows the existing project-root policy: target parent must already exist and not be a symlink; absolute paths, traversal, `@downloads`, and other suffixes are rejected.

## Explicitly not included

| Not provided | Reason |
|---|---|
| `quantum { ... }` parser block or QIR | The v1 surface is a runtime map API. A new grammar/AST/QIR compiler requires a separately scoped language evolution. |
| State-vector simulator, result counts, auto-fallback | This release does not implement state evolution or bit-count sampling, so no measurement result or algorithm correctness is claimed. |
| QAOA, QML, Grover, optimisation, noise mitigation | These are algorithms/techniques, not implied by emitting a circuit text format. Their correctness and suitability need separate implementation and verification. |
| IBM Quantum / Amazon Braket / REST / gRPC / provider choice | IBM non-local service access requires an API token, while Braket submission creates remote asynchronous tasks and retrieves results from cloud storage. These are credentialed external actions outside this local-only feature. [4] [5] |

## Safety and diagnostics

`P1083` reports malformed or unsafe circuit data, unsupported gates, invalid indices/mapping, unsupported provider/device/simulator fields, or oversized QASM. The summary and QASM output fixedly report no provider, network, credential, QPU, simulator, or process execution.

## References

[1] [OpenQASM 3.0: Gates](https://openqasm.com/versions/3.0/language/gates.html)  
[2] [OpenQASM 3.0: Types and Casting](https://openqasm.com/versions/3.0/language/types.html)  
[3] [OpenQASM 3.0: Built-in Quantum Instructions](https://openqasm.com/versions/3.0/language/insts.html)  
[4] [IBM Quantum Compute: QiskitRuntimeService](https://quantum.cloud.ibm.com/docs/api/qiskit-ibm-runtime/qiskit-runtime-service)  
[5] [Amazon Braket: Running Quantum Tasks](https://docs.aws.amazon.com/braket/latest/developerguide/braket-using.html)
