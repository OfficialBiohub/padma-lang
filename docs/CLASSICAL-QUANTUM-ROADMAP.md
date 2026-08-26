# Classical + Quantum Production Roadmap

## Status principle

Padma is a classical Bangla-English programming language with a **bounded local quantum runtime foundation**. It is not yet a universal production quantum SDK, a QPU cloud client, or a guarantee of quantum advantage. Every row below is marked by actual runtime status so that a roadmap is never presented as a finished capability.

The local circuit surface follows a deliberately small OpenQASM 3-compatible subset. OpenQASM standard gates include fixed single-qubit gates, rotations, and controlled operations; Padma adds only the operations whose validation, rendering, local matrix semantics, tests, and Termux example are present. [1]

## Current local hybrid foundation

| Area | Status | Current concrete boundary |
|---|---|---|
| Classical language | Implemented | Bangla-English UTF-8 source, expressions, functions, maps/lists, modules, project manifests, REPL, files, table/report helpers, capability policy, and existing bounded integrations. |
| Circuit maps | Implemented | Strict `1..20` qubit map with at most 256 operations, complete measurements, and only supported fields. |
| Gates | Implemented bounded subset | `h`, `x`, `z`, `s`, `t`, `rx`, `ry`, `rz`, `cx`, `superposition`, `entangle-linear`. Rotations use explicit finite numeric radians, not symbolic parameters. |
| Circuit interchange | Implemented bounded subset | Deterministic OpenQASM 3.0 text and project-local `.qasm` export. |
| Local simulation | Implemented bounded subset | Exact state-vector probabilities for at most 12 qubits, all-zero initial state, no sampling or noise. |
| Observable analysis | Implemented bounded subset | One full-register coefficient-`1` `I`/`X`/`Y`/`Z` Pauli-product expectation. |
| QPU/provider execution | Not implemented | No provider account, credential, network request, job, polling, result retrieval, cost, quota, or cancellation action exists. |
| Local backend response routing | Implemented bounded foundation | `server.route_response` validates explicit method/path route maps, returns deterministic JSON response envelopes, and stays in memory. Existing `padma serve .` remains only a capability-gated loopback health server; custom routes, database/auth/payment/deployment are not connected. |

## Sequential production milestones

| Milestone | Status | Why it comes in this order |
|---|---|---|
| M32: finite numeric rotations | Implemented in current increment | Local rotation matrices and QASM lowering are required before parameter experiments. |
| M33: reproducible local sampling | Implemented bounded foundation | `quantum.sample_counts` uses required explicit integer `seed` and bounded `shots` with a versioned local PRNG to return a deterministic sparse count map. It does not expose collapse, noise, hardware, or hidden randomness. |
| M34: real-coefficient Pauli Hamiltonians | Implemented bounded foundation | `quantum.expectation_hamiltonian` evaluates up to 64 unique ordered real full-register Pauli terms against the local state vector and returns deterministic energy plus contribution breakdown. It does not optimise, bind symbolic values, estimate hardware energy, or execute an algorithm. |
| M35: bounded classical optimisation primitives | Implemented bounded foundation | `optimize.quadratic_value`, `optimize.finite_difference_gradient`, and `optimize.projected_gradient_step` handle only an explicit finite separable quadratic and a one-step local projected proposal. They do not accept a Hamiltonian/circuit/callback, mutate state, loop to convergence, train a model, or implement VQE/QAOA/QML. |
| M36: program tooling/interchange | Implemented bounded foundation | `quantum.assess_openqasm3` proves only byte-exact equality with Padma’s existing bounded renderer and returns stable local artifact metadata. A general parser, QASM round-trip import, compiler, execution path, or provider compatibility claim remains unimplemented. |
| M37: provider integration assessment | Implemented bounded assessment foundation | `quantum.provider_readiness` validates only provider labels, reviewed artifact fingerprints, and redacted public policy notes, then returns required-control markers. A cloud adapter still needs explicit capability grants, secret management, user-confirmed cost/job actions, cancellation, provenance, provider-specific transport, and security regressions. |
| M39: local backend route responses | Implemented bounded foundation | `server.route_response` provides exact method/path matching and finite JSON response envelopes for practice and local application logic. It is not a socket router, backend framework, authentication service, database ORM, payment system, or public deployment. |

## Non-negotiable production boundaries

| Boundary | Reason |
|---|---|
| No “quantum advantage” claim | A small exact simulator does not demonstrate hardware performance, application advantage, or scalability. |
| No hidden cloud actions | QPU submission is a credentialed, cost- and quota-relevant remote operation. It must never occur from a local planning/simulation API. |
| No plan-as-execution claim | OpenQASM output is a portable circuit description, not proof that a provider accepted or ran it. |
| No algorithm label without algorithm | Rotations and Pauli expectations alone are not QAOA, VQE, Grover, QML, error mitigation, or a compiler framework. |
| Deterministic first | Local runtime results, schemas, diagnostics, and artifacts must be reproducible before any sampling or optimiser expands the state space. |

## How to use the current foundation

Use ordinary Padma classical variables to calculate an explicit finite angle, place that numeric value in an `rx`, `ry`, or `rz` operation map, inspect local probabilities/Pauli expectations, and export an OpenQASM plan for manual review. This is a local programming workflow; it neither opens an account nor sends a circuit externally.

```padma
ধরি halfTurn = 1.5707963267948966
ধরি circuit = {"qubits": 1, "operations": [{"gate": "ry", "targets": [0], "angle": halfTurn}], "measurements": [{"qubit": 0, "bit": 0}]}

দেখাও quantum.simulate_probabilities(circuit)["probabilities"]["1"]
দেখাও quantum.expectation_pauli(circuit, "Z")
```

## References

[1] [OpenQASM 3 Standard Library](https://openqasm.com/language/standard_library.html)  
[2] [IBM Quantum: Specify observables in the Pauli basis](https://quantum.cloud.ibm.com/docs/guides/specify-observables-pauli)  
[3] [IBM Quantum: Qiskit Runtime Service](https://quantum.cloud.ibm.com/docs/api/qiskit-ibm-runtime/qiskit-runtime-service)
