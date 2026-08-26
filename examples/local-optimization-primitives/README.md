# Local Optimisation Primitives

This Termux-first project evaluates one explicit two-variable weighted quadratic, estimates its gradient with a centered finite difference, and calculates one **projected proposal**. It needs no capability grant because every result is calculated in memory.

```sh
cd ~/padma-lang
cargo build --release
export PATH="$HOME/padma-lang/target/release:$PATH"
cd examples/local-optimization-primitives
padma .
```

Expected terminal output:

```text
10
4
-4
1
0
4.5
true
disabled
not-configured
disabled
```

The objective is `2 × (x₀ - 1)² + 0.5 × (x₁ - 3)²` at `[2, -1]`, so its value is `10`. With epsilon `0.001`, the centered finite-difference gradient is `[4, -4]`. A single learning-rate `0.25` proposal produces `[1, 0]` and an objective value of `4.5`.

`proposalOnly = true` is important: Padma does **not** replace `objective["parameters"]`, repeat the step, choose a stopping condition, run a callback, perform model training, implement VQE/QAOA/QML/Grover, couple this request to a quantum Hamiltonian, access a QPU/provider, read credentials, use the network, or start a process. Review and explicitly use the returned proposal in later Padma code only if it is appropriate for your program.
