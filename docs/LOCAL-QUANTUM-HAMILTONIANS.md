# Local Pauli Hamiltonian Analysis v1

## Purpose and execution boundary

`quantum.expectation_hamiltonian` calculates the deterministic local energy expectation of a **real linear combination of full-register Pauli products** over the existing bounded Padma state vector. A Pauli-basis operator is naturally represented as a sum of Pauli strings with coefficients. [1] [2]

This is local analysis only. It does not perform gradient evaluation, minimisation, variational parameter binding, VQE, QAOA, QML, noise modelling, QPU execution, provider selection, credential use, network communication, or process execution.

## API

```padma
ধরি hamiltonian = {"terms": [{"coefficient": 1, "pauli": "ZZ"}, {"coefficient": -0.5, "pauli": "XI"}, {"coefficient": 0.25, "pauli": "II"}]}
ধরি analysis = quantum.expectation_hamiltonian(circuit, hamiltonian)
দেখাও analysis["energy"]
```

| API | Result | Capability |
|---|---|---|
| `quantum.expectation_hamiltonian(circuit, hamiltonian)` | Deterministic energy and an ordered bounded term breakdown from the existing local state vector. | None |

## Strict Hamiltonian schema

```text
{
  "terms": [
    {"coefficient": 1.0, "pauli": "ZZ"},
    {"coefficient": -0.5, "pauli": "XI"}
  ]
}
```

| Field | Rule |
|---|---|
| Top-level map | Exactly one `terms` field; `constant`, `provider`, `backend`, `shots`, `seed`, `parameter`, `token`, `url`, and every other field are rejected. A constant can be represented explicitly with a full-register `I...I` term. |
| `terms` | Non-empty ordered list of at most 64 term maps. Input order is preserved in the output breakdown. |
| Term map | Exactly `coefficient` and `pauli`. |
| `coefficient` | Required finite nonzero real number, absolute value at most `1_000_000`. Complex, symbolic, text, `NaN`, infinity, and zero are rejected. |
| `pauli` | Required unique ASCII full-register string with one `I`, `X`, `Y`, or `Z` per circuit qubit. Leftmost character acts on `q[n-1]`; rightmost acts on `q[0]`. |

The v1 real-only policy ensures the Hamiltonian is Hermitian when every term is a real coefficient times a standard Hermitian Pauli product. It is intentionally narrower than general SDK operator classes, which may support complex or parameterized coefficients. [1] [2]

## Energy and output invariants

Padma calculates

\[
E = \langle \psi | H | \psi \rangle = \sum_j c_j\langle\psi|P_j|\psi\rangle.
\]

Each term expectation uses the same pre-measurement in-memory state vector and the single-Pauli convention already used by `quantum.expectation_pauli`. `energy` and every numeric breakdown value are rounded deterministically to 12 decimal places. The term count is at most 64, the circuit limit remains 12 locally simulated qubits, and the coefficient absolute-sum must not exceed `1_000_000`.

The returned map contains `energy`, `termCount`, `coefficientL1Norm`, a declared-order `terms` breakdown (`pauli`, `coefficient`, `expectation`, `contribution`), and fixed local-only status markers. It is not a claim that the circuit has been sampled or executed on hardware.

## Example

For the Bell-style state \((|00\rangle+|11\rangle)/\sqrt2\), the Hamiltonian

```padma
{"terms": [{"coefficient": 1, "pauli": "ZZ"}, {"coefficient": 0.5, "pauli": "XX"}, {"coefficient": 0.25, "pauli": "II"}]}
```

has local energy `1.75`, because the `ZZ`, `XX`, and `II` expectations are all `1`.

## Safety and diagnostics

`P1087` reports invalid Hamiltonian maps, terms, coefficients, Pauli strings, duplicate terms, numeric bounds, or local energy invariants. Existing `P1083` validates circuits and `P1084` remains responsible for the state-vector resource/normalization bound.

## References

[1] [IBM Quantum: Operator classes and `SparsePauliOp`](https://quantum.cloud.ibm.com/docs/guides/operators-overview)  
[2] [IBM Quantum: Specify observables in the Pauli basis](https://qiskit.qotlabs.org/docs/guides/specify-observables-pauli)  
[3] [IBM Quantum: `SparsePauliOp` API](https://quantum.cloud.ibm.com/docs/api/qiskit/qiskit.quantum_info.SparsePauliOp)
