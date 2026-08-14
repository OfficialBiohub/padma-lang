# Padma Production Language TODO

- [x] Audit current compiler, WASM wrapper, playground additions, tests, and repository layout.
- [x] Expose the existing compiler core through the reusable `src/lib.rs` API used by the WASM wrapper.
- [ ] Define and document the stable Padma language specification for Bengali, English, and mixed source.
- [x] Add first-class functions, parameters, calls, return values, blocks, and loops.
- [ ] Add lists, maps, modules, and structured error handling.
- [ ] Add static type checking and actionable bilingual diagnostics without breaking the interpreter.
- [ ] Build a Termux-friendly `padma` CLI with run, check, format, test, init, and package commands.
- [ ] Add a dependency-free standard library for files, text, collections, JSON, HTTP, and process boundaries.
- [ ] Add safe interoperability paths for Python, JavaScript/TypeScript, C, and shell instead of claiming automatic conversion.
- [ ] Add package metadata, lockfile, registry-ready layout, reproducible builds, and semantic versioning.
- [ ] Add security limits, sandbox boundaries, path validation, dependency checks, and supply-chain guidance.
- [ ] Add unit, integration, golden diagnostic, fuzz, and Termux smoke tests.
- [ ] Document Android/Termux installation, examples, contribution rules, and release process.
- [ ] Remove or isolate non-core playground artifacts if they distract from the language repository.
