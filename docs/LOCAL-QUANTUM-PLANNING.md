# Local Quantum Circuit Planning v1

## Purpose and execution boundary

Padma Quantum Planning v1 turns an explicit, bounded local circuit map into deterministic **OpenQASM 3.0 text**. Its companion local state-vector feature can calculate bounded exact probability data; neither feature is a quantum-cloud client, algorithm library, or real-device executor.

> OpenQASM describes gates with a fixed number of quantum arguments and describes static qubit/bit registers; this v1 emits a deliberately small compatible subset. [1] [2]

## Strict circuit schema

```padma
ধরি circuit = {"qubits": 2, "operations": [{"gate": "superposition", "targets": [0, 1]}, {"gate": "entangle-linear", "targets": [0, 1]}], "measurements": [{"qubit": 0, "bit": 0}, {"qubit": 1, "bit": 1}]}
```

| Field | Rule |
|---|---|
| `qubits` | Whole number from `1` to `20`. This is a v1 planning bound, not a simulator or QPU capacity claim. |
| `operations` | One to `256` strict `{gate, targets}` maps in declared order, except rotation maps which additionally require `angle`. |
| `operations[].gate` | Exactly one of `h`, `x`, `z`, `s`, `t`, `rx`, `ry`, `rz`, `cx`, `superposition`, or `entangle-linear`. |
| `operations[].targets` | Bounded distinct whole qubit indexes. Single-qubit and rotation gates require one target; `cx` two; high-level `superposition` and `entangle-linear` use the listed targets. |
| `operations[].angle` | Required only by `rx`, `ry`, and `rz`: finite numeric radians with absolute value at most `1_000_000`. Symbolic parameters, text expressions, units, and extra angle fields are rejected. |
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
| Sampled result counts, auto-fallback, advanced simulation | The companion simulator returns deterministic exact probabilities only. It does not perform random sampling/counts, collapse, noise simulation, fallback simulation, or produce real-device outcomes. |
| QAOA, QML, Grover, optimisation, noise mitigation | These are algorithms/techniques, not implied by emitting a circuit text format. Their correctness and suitability need separate implementation and verification. |
| IBM Quantum / Amazon Braket / REST / gRPC / provider choice | IBM non-local service access requires an API token, while Braket submission creates remote asynchronous tasks and retrieves results from cloud storage. These are credentialed external actions outside this local-only feature. [4] [5] |

## Safety and diagnostics

`P1083` reports malformed or unsafe circuit data, unsupported gates, missing/extra/non-finite/oversized rotation angles, invalid indices/mapping, unsupported provider/device/simulator fields, or oversized QASM. The circuit summary declares bounded local simulator availability, while the planning/QASM feature itself never accesses a provider, network, credential, QPU, or process. `P1084` covers local state-vector resource and numeric invariants.

## References

[1] [OpenQASM 3.0: Gates](https://openqasm.com/versions/3.0/language/gates.html)  
[2] [OpenQASM 3.0: Types and Casting](https://openqasm.com/versions/3.0/language/types.html)  
[3] [OpenQASM 3.0: Built-in Quantum Instructions](https://openqasm.com/versions/3.0/language/insts.html)  
[4] [IBM Quantum Compute: QiskitRuntimeService](https://quantum.cloud.ibm.com/docs/api/qiskit-ibm-runtime/qiskit-runtime-service)  
[5] [Amazon Braket: Running Quantum Tasks](https://docs.aws.amazon.com/braket/latest/developerguide/braket-using.html)
