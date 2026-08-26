# Local OpenQASM Interchange Assessment

This Termux-first project generates one bounded Padma OpenQASM 3.0 program in memory and verifies the generated text against the exact same local renderer. It needs no capability grant because it does not read or write a `.qasm` file, call a provider, or start another process.

```sh
cd ~/padma-lang
cargo build --release
export PATH="$HOME/padma-lang/target/release:$PATH"
cd examples/local-quantum-interchange-assessment
padma .
```

Expected terminal output:

```text
true
2
2
3
2
local-openqasm3-exact-subset-assessment-v1
not-implemented
disabled
not-configured
disabled
```

The source circuit has two high-level operations. `superposition` lowers to two rendered `h` instructions, while `entangle-linear` lowers to one `cx`, so `renderedGateInstructionCount` is `3`. The assessment confirms that the generated text is the exact bounded renderer output and returns stable local metadata.

This does **not** parse arbitrary QASM, import a circuit, execute source, accept comments or another provider’s formatting, read/write files, run a simulator beyond separately requested Padma APIs, submit to a QPU/provider, read credentials, contact a network, or start a process. An altered-but-valid QASM text is rejected because successful output means only exact equality with Padma’s renderer.
