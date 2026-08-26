# Local Quantum Circuit Planning

This Termux-first project validates a bounded two-qubit circuit and writes its **OpenQASM 3.0 plan** to `out/bell-plan.qasm`.

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
true
true
```

The `superposition` operation expands to a Hadamard instruction for each listed target and `entangle-linear` expands to a linear `cx` chain. Padma then writes static qubit/bit declarations, reset, gates, and declared measurement mappings in deterministic order.

The `filesystem = ["write"]` grant only permits the project-local `.qasm` output. This example does **not** simulate the circuit, return measurement counts, validate a quantum algorithm, estimate performance, access IBM Quantum/Amazon Braket/any provider, read credentials, send a network request, submit a quantum task, or start a process.

Remove the generated plan when finished:

```sh
rm out/bell-plan.qasm
```
