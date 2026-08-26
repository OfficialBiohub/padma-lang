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
true
true
```

The `superposition` operation expands to a Hadamard instruction for each listed target and `entangle-linear` expands to a linear `cx` chain. The bounded local simulator starts from `|00⟩` and returns deterministic all-basis probabilities; for this circuit, only `00` and `11` have probability `0.5`. Padma then writes static qubit/bit declarations, reset, gates, and declared measurement mappings in deterministic order.

The `filesystem = ["write"]` grant only permits the project-local `.qasm` output; `quantum.simulate_probabilities` itself needs no capability. This example does **not** return sampled measurement counts, validate a quantum algorithm, estimate performance, access IBM Quantum/Amazon Braket/any provider, read credentials, send a network request, submit a quantum task, or start a process.

Remove the generated plan when finished:

```sh
rm out/bell-plan.qasm
```
