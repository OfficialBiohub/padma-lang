# Local Parameterized Rotation Gates v1

## Purpose and execution boundary

Padma quantum circuits can use explicit numeric `rx`, `ry`, and `rz` single-qubit rotations in addition to the existing fixed gate subset. A rotation operation is still a local circuit description: Padma lowers it to OpenQASM 3.0 and applies its matrix inside the bounded local state-vector simulator. It does not create symbolic parameters, invoke an SDK, choose a provider, submit a job, or access credentials.

> OpenQASM 3 standard gates define `rx(θ)`, `ry(θ)`, and `rz(θ)` as rotations by a real angle in radians. [1]

## Strict operation shape

```padma
{"gate": "ry", "targets": [0], "angle": 1.5707963267948966}
```

| Field | v1 rule |
|---|---|
| `gate` | Exactly one of `rx`, `ry`, or `rz`. |
| `targets` | Exactly one declared qubit index. |
| `angle` | Required finite numeric radians with absolute value at most `1_000_000`. No text expression, variable name, symbolic parameter, unit suffix, or implicit default is accepted. |
| Other fields | Rejected. A non-rotation gate must not include `angle`. |

The bounded numeric limit prevents misleading large-angle precision claims while retaining ordinary local experimentation. The local matrices are \(RX(\theta)=e^{-i\theta X/2}\), \(RY(\theta)=e^{-i\theta Y/2}\), and \(RZ(\theta)=e^{-i\theta Z/2}\). [1]

## Runtime and OpenQASM behaviour

For a validated circuit with at most 12 qubits, `quantum.simulate_probabilities` and `quantum.expectation_pauli` use the same rotation matrix implementation as the regular local simulator. The OpenQASM renderer writes a deterministic decimal angle using the standard-library syntax:

```openqasm
ry(1.57079632679489656) q[0];
```

`rx`, `ry`, and `rz` all act on one qubit. `rz` may change relative phase without changing computational-basis probability; use `quantum.expectation_pauli` with `X` or `Y` to inspect phase-sensitive consequences.

## Example

```padma
ধরি halfTurn = 1.5707963267948966
ধরি circuit = {"qubits": 1, "operations": [{"gate": "ry", "targets": [0], "angle": halfTurn}], "measurements": [{"qubit": 0, "bit": 0}]}

ধরি probabilities = quantum.simulate_probabilities(circuit)
দেখাও probabilities["probabilities"]["0"]
দেখাও probabilities["probabilities"]["1"]
দেখাও quantum.expectation_pauli(circuit, "Z")
```

The expected local values are `0.5`, `0.5`, and `0` up to the documented deterministic rounding.

## Explicit exclusions and next milestones

This release has no symbolic parameters, parameter binding, gradient, automatic differentiation, optimizer, sampling/counts, Pauli sum/Hamiltonian, QAOA/VQE/QML algorithm, noise model, density matrix, cloud QPU, provider account, network, credential, or process action. The next roadmap increments are reproducible local sampling, multi-term Pauli Hamiltonian expectation, then bounded classical optimisation; none is implemented merely by adding rotations.

## Safety and diagnostics

Malformed, absent, extra, nonnumeric, non-finite, or oversized angles use the existing local quantum circuit diagnostic `P1083`. State-vector resource and normalization safety remain `P1084`; Pauli-observable validation remains `P1085`.

## References

[1] [OpenQASM 3 Standard Library: `rx`, `ry`, and `rz`](https://openqasm.com/language/standard_library.html)  
[2] [IBM Quantum: RXGate](https://qiskit.qotlabs.org/docs/api/qiskit/qiskit.circuit.library.RXGate)  
[3] [IBM Quantum: RZGate](https://quantum.cloud.ibm.com/docs/api/qiskit/qiskit.circuit.library.RZGate)
