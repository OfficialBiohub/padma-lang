# Local Pauli Observable Analysis v1

## Purpose and execution boundary

This feature computes a deterministic local expectation value \(\langle\psi|P|\psi\rangle\) for one full-register Pauli observable over the existing bounded Padma state vector. A Pauli string is a tensor-product observable; Hermitian Pauli terms have real expectation values. [1] [2]

```padma
ধরি bell = {"qubits": 2, "operations": [{"gate": "h", "targets": [0]}, {"gate": "cx", "targets": [0, 1]}], "measurements": [{"qubit": 0, "bit": 0}, {"qubit": 1, "bit": 1}]}
দেখাও quantum.expectation_pauli(bell, "ZZ")
দেখাও quantum.expectation_pauli(bell, "XX")
```

Both expressions return `1` for this Bell-style state.

## API and strict input contract

| API | Result | Capability |
|---|---|---|
| `quantum.expectation_pauli(circuit, observable)` | One normalized deterministic real expectation number in `[-1, 1]`. | None |

`circuit` is exactly the existing strict local circuit map. `observable` must be an ASCII text string whose length exactly equals `qubits`, with every character one of `I`, `X`, `Y`, or `Z`. No coefficient, Pauli sum, shots, seed, parameter, provider, backend, URL, credential, or device input exists in v1.

## Ordering and numerical convention

| Item | Rule |
|---|---|
| State | The existing exact local state vector, initialized to `|00…0⟩`, maximum 12 qubits. |
| String ordering | The **leftmost** Pauli character acts on `q[n-1]`; the **rightmost** character acts on `q[0]`. This matches the fixed-width probability-label convention and the conventional tensor-product string orientation. [2] |
| Observable | One coefficient-`1` Pauli product only. `I` leaves a qubit unchanged; `X`, `Y`, and `Z` use their standard Pauli action. |
| Output | A deterministic `f64` rounded to 12 decimal places. A near-zero imaginary residual or numeric result outside `[-1,1]` is rejected rather than silently reported. |
| Measurement | The circuit’s complete mapping remains validation metadata; the observable calculation uses the pre-measurement local state and does not sample or collapse it. |

## Explicit exclusions

The feature does not perform basis-changing physical measurement, shot sampling, count estimation, Pauli sums/Hamiltonians, coefficients, expectation-value optimisation, gradients, parameter binding, QAOA/QML/Grover algorithms, noise/density-matrix simulation, QPU/provider selection, cloud submission, credential access, network request, or process execution. A real QPU would require separate basis transformations for non-diagonal Pauli measurements; this local evaluator applies the Pauli matrix directly to its in-memory state vector. [3]

## Safety and diagnostics

`P1085` reports empty/wrong-length/non-ASCII/non-Pauli observables or a non-real/out-of-range local expectation invariant. Existing `P1083` validates circuit syntax and `P1084` protects the state-vector resource and normalization bound.

## References

[1] [IBM Quantum: Specify observables in the Pauli basis](https://quantum.cloud.ibm.com/docs/guides/specify-observables-pauli)  
[2] [IBM Quantum: SparsePauliOp](https://quantum.cloud.ibm.com/docs/api/qiskit/qiskit.quantum_info.SparsePauliOp)  
[3] [IBM Quantum: Pauli-basis measurement transformations](https://quantum.cloud.ibm.com/docs/guides/specify-observables-pauli)
