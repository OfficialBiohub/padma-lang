# Local Quantum State-Vector Simulator v1

## Purpose and execution boundary

This feature evaluates a previously validated Padma quantum circuit **inside the local Padma process** and returns exact computational-basis probability data. It is intentionally a small teaching/prototyping simulator, not a QPU executor, cloud backend, noise model, or performance estimator.

> A state-vector simulation convention starts from the all-zero state and evolves the circuit locally; computational-basis probabilities are the squared amplitude magnitudes. [1] [2]

## API

| API | Result | Capability |
|---|---|---|
| `quantum.simulate_probabilities(circuit)` | Deterministic full-register basis probability map and simulator metadata. | None |
| `quantum.expectation_pauli(circuit, observable)` | Deterministic real expectation for one strict full-register Pauli product. | None |

The simulator consumes the same strict circuit map accepted by `quantum.openqasm3`. It does not add `shots`, `seed`, `provider`, `device`, `noiseModel`, `backend`, credentials, URLs, or any other fields.

## Resource and semantic limits

| Area | v1 contract |
|---|---|
| Qubits | Maximum **12**, meaning at most `2^12 = 4096` complex amplitudes. The broader 20-qubit OpenQASM planning limit still applies only to text planning. |
| Operations | Reuses the planner’s maximum of 256 explicit operations. |
| Initial state | Exactly `|00…0⟩`; no user-supplied amplitude vector. |
| Supported lowering | `h`, `x`, `z`, `s`, `t`, finite-angle `rx`/`ry`/`rz`, `cx`, `superposition`, and `entangle-linear`. |
| Bit convention | Qubit index `0` is the least-significant internal bit. Output labels are fixed-width, most-significant-bit-first strings, so `q[0]` is the rightmost label bit. |
| Measurement | The complete declared map only defines output classical-bit placement. There is no sampled measurement, no random seed, no partial-measurement API, and no collapse state. |
| Output | A deterministic lexicographically ordered map of all basis bitstrings to normalized probabilities, rounded to 12 decimal places. |

The `cx` operation uses the first target as control and the second as target, matching the standard controlled-X definition that flips the target when its control is in the one state. [3]

## Result shape

```text
{
  "qubitCount": 2,
  "basisStateCount": 4,
  "probabilities": {"00": 0.5, "01": 0.0, "10": 0.0, "11": 0.5},
  "probabilitySum": 1.0,
  "method": "local-state-vector-exact-probabilities",
  "sampling": "disabled",
  "provider": "not-configured",
  "qpu": "disabled",
  "network": "disabled",
  "childProcess": "disabled"
}
```

The returned numbers are deterministic probability values, **not shot counts** or a claim about noisy-device output. Reference state-vector tooling likewise distinguishes exact local state-vector computation from sampling/noise and real device execution. [2] [4]

## Safety and diagnostics

`P1084` reports a circuit above the local simulation bound, invalid/non-finite state normalization, unsupported simulator inputs, or an internal result-map invariant failure. The simulator has no filesystem, network, process, provider, browser, account, credential, payment, or device capability.

## Explicitly not included

This v1 does not provide amplitude injection/export, density matrices, mid-circuit measurement, collapse, random sampling/counts, symbolic parameter binding, Pauli sums/Hamiltonians, gradients, QAOA/QML/Grover algorithms, noise mitigation, hardware calibration, QPU selection, provider accounts, cloud transport, or results from a real quantum device. Explicit finite numeric rotations are documented in [`LOCAL-QUANTUM-ROTATIONS.md`](LOCAL-QUANTUM-ROTATIONS.md); single Pauli-product expectations are documented separately in [`LOCAL-QUANTUM-OBSERVABLES.md`](LOCAL-QUANTUM-OBSERVABLES.md).

## References

[1] [Qiskit Statevector: construction from an all-zero initialized circuit](https://quantum.cloud.ibm.com/docs/api/qiskit/qiskit.quantum_info.Statevector)  
[2] [Qiskit Statevector: computational-basis probabilities and deterministic probability dictionaries](https://quantum.cloud.ibm.com/docs/api/qiskit/qiskit.quantum_info.Statevector)  
[3] [IBM Quantum: CXGate controlled-X definition](https://quantum.cloud.ibm.com/docs/api/qiskit/qiskit.circuit.library.CXGate)  
[4] [IBM Quantum: Exact local state-vector simulation](https://quantum.cloud.ibm.com/docs/guides/simulate-with-qiskit-sdk-primitives)
