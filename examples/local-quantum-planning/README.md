# Local Quantum Circuit Planning

This Termux-first project validates a bounded two-qubit circuit, computes exact local basis probabilities, and writes its **OpenQASM 3.0 plan** to `out/bell-plan.qasm`.

```sh
cd ~/padma-lang
cargo build --release
export PATH="$HOME/padma-lang/target/release:$PATH"
cd examples/local-quantum-planning
padma .
cat out/bell-plan.qasm
```

Expected terminal output:

```text
Qubits: 2
Operations: 2
Provider: not-configured
0.5
0.5
1
1
0.5
0
64
64
local-seeded-cdf-sampler-v1
true
true
```

The `superposition` operation expands to a Hadamard instruction for each listed target and `entangle-linear` expands to a linear `cx` chain. The bounded local simulator starts from `|00⟩` and returns deterministic all-basis probabilities; for this circuit, only `00` and `11` have probability `0.5`. The next two output lines are the `ZZ` and `XX` Pauli-product expectations, both `1` for this Bell-style state. A normal Padma numeric variable, `halfTurn`, then feeds a one-qubit `ry` operation; its `1` probability is `0.5` and its Z expectation is `0`. The next three lines show the explicit-seed local sampler’s requested 64 shots, verified count total 64, and versioned method marker. Padma writes static qubit/bit declarations, reset, gates, and declared measurement mappings in deterministic order.

The `filesystem = ["write"]` grant only permits the project-local `.qasm` output; `quantum.simulate_probabilities`, `quantum.expectation_pauli`, and `quantum.sample_counts` need no capability. This example’s sampler requires an explicit seed and returns local pseudo-random counts only; it does **not** bind a symbolic parameter, use hidden randomness, expose a collapse state, evaluate a Pauli sum/Hamiltonian or quantum algorithm, estimate performance, access IBM Quantum/Amazon Braket/any provider, read credentials, send a network request, submit a quantum task, or start a process.

Remove the generated plan when finished:

```sh
rm out/bell-plan.qasm
```
