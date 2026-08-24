# Padma Production Language TODO

- [x] Audit current compiler, WASM wrapper, playground additions, tests, and repository layout.
- [x] Expose the existing compiler core through the reusable `src/lib.rs` API used by the WASM wrapper.
- [x] Define and document the stable Padma language specification for Bengali, English, and mixed source.
- [x] Add first-class functions, parameters, calls, return values, blocks, and loops.
- [x] Add list literals and list display values.
- [x] Add list indexing/mutation, modules, and structured error handling.
- [x] Add maps/dictionaries with text keys and `map.get`/`map.set`.
- [x] Add safe `import` / `ইমপোর্ট` modules with relative `.pd` path validation.
- [x] Prevent module traversal, duplicate loading, and import cycles with localized diagnostics.
- [x] Add English, Bengali, nested-module, and invalid-import regression tests.
- [x] Document reusable Padma modules with runnable Termux examples.

## Production-readiness roadmap

- [x] Publish a versioned production-readiness definition with supported platforms, compatibility guarantees, and explicit non-goals.
- [x] Add richer locale detection and a complete bilingual diagnostics catalogue.
- [x] Preserve source file, source line, and locale context for imported-module diagnostics.
- [x] Add parser recovery so `padma check` reports multiple independent syntax errors in one run.
- [x] Implement null values, indexing, slicing, list mutation, iteration, and collection utility APIs.
- [x] Add `none` / `কিছুইনা` null values with stable truthiness and display semantics.
- [x] Add list indexing syntax, bounds diagnostics, and a zero-based non-negative index policy.
- [x] Add zero-based bounded list slices with inclusive start and exclusive end semantics.
- [x] Add list mutation APIs (`push`, `set`, `remove`) with type and bounds validation.
- [x] Add collection length and membership APIs for lists and maps.
- [x] Add collection regression tests, Bengali-English examples, and specification updates.
- [x] Add `for item in collection` / `প্রতি item মধ্যে collection` iteration with bounded execution.
- [x] Add safe `range` generation and deterministic iteration order for list, map, and text values.
- [x] Define loop-variable scope and error behavior, with Bengali-English regression tests and examples.
- [x] Add safe path operations, formatting, and documented non-cryptographic randomness to the standard library.
- [x] Add relative-only `path.basename`, `path.extension`, and `path.join` helpers with traversal rejection.
- [x] Add map-driven `text.format` with explicit placeholder validation.
- [x] Add bounded, explicitly non-cryptographic `random.int` and `random.pick` helpers with deterministic safety limits.
- [x] Add standard-library integration tests, examples, and bilingual documentation for the remaining M3 APIs.
- [x] Add deterministic JSON conversion and safe HTTP/HTTPS URL inspection APIs.
- [x] Add text, deterministic math, bounded time, and safe relative-file read/write/exists standard-library APIs.
- [x] Add module namespaces, public exports, a project manifest, lockfile design, and a no-registry-yet trust policy.
- [x] Add `padma init` with a minimal `padma.toml`, `padma.lock`, and Bengali-English starter source.
- [x] Add manifest-driven project execution through `padma .`, with validated relative entrypoints and locale overrides.
- [x] Define and enforce a no-registry-yet dependency policy that rejects untrusted dependency declarations.
- [x] Add import aliases and public export rules so modules stop leaking all internal names into callers.
- [x] Document local module/project layout, lockfile schema, compatibility policy, and future registry trust requirements.
- [x] Add explicit user-granted capability policies for project process, network, media, and storage access while retaining safe single-file compatibility mode.
- [x] Parse explicit `[capabilities]` grants from `padma.toml` and reject unknown or duplicate grants.
- [x] Enforce capability grants for project file access, HTTP, process execution, and media download.
- [x] Add `padma capabilities <project>` to inspect declared grants without executing project code.
- [x] Add capability-denied diagnostics, negative security tests, and Bangla-English Termux examples.
- [x] Define audited capability escalation and Android shared-storage boundaries before enabling broader permissions.
- [x] Add `padma check`, `padma fmt`, `padma lint`, JSON diagnostics, conservative static checking, and reviewed warning suppression.
- [x] Add stable JSON diagnostic output through `padma check --json <file.pd>` for editors and CI.
- [x] Add an idempotent `padma fmt` formatter with newline/indentation golden tests.
- [x] Add `padma lint` with documented rule identifiers, warning exit policy, and manifest-configurable reviewed rule suppression.
- [x] Add a deterministic `padma lint` CLI with localized `L1001`–`L1003` warnings and JSON output.
- [x] Add manifest-configurable reviewed lint suppression syntax.
- [ ] Add lint severity overrides and conservative semantic lint rules.
- [x] Extend `padma check` from parser recovery to non-executing static semantic checks for provable literal division by zero.
- [x] Extend static checks conservatively with top-level function and stable builtin call-arity validation.
- [ ] Add scope-aware name resolution and safe literal-type static rules without guessing about imports or dynamic values.
- [x] Create a Tree-sitter grammar, VS Code extension, language server protocol implementation, and mobile-editor guidance.
- [x] Create an independent ABI-15 Tree-sitter grammar with Bangla-English corpus tests, highlight queries, generated artifacts, and CI.
- [x] Create an initial VS Code extension with `.pd` association, bilingual highlighting, explicit run/check/format/lint commands, and JSON diagnostic rendering.
- [x] Implement an opt-in stdio Padma language server for diagnostics and document formatting, including UTF-16 position conversion tests.
- [x] Add a safe static Bangla-English LSP completion catalogue with regression tests.
- [x] Add UTF-16-aware static LSP hover help for Bangla-English keywords and selected builtins.
- [x] Add a Bangla Termux-first guide for installation, Nano editing, project capability review, diagnostics, formatting, linting, and Android storage boundaries.
- [x] Add conservative same-document LSP definitions, dynamic local completion, and Bangla-English hover help.
- [x] Complete VS Code extension packaging, sample-workspace validation, and fuller mobile-editor guidance.
- [x] Add a minimal safe sample Padma workspace with bilingual source, project manifest, and editor settings for the local LSP command.
- [x] Add a deterministic extension package validation command and CI check for its produced `.vsix` artifact.
- [x] Document Android-friendly editing choices and the explicit desktop VS Code LSP setup without treating an editor as a replacement for Termux CLI workflows.
- [x] Add an end-to-end LSP JSON-RPC smoke test covering capability negotiation and conservative Bangla local rename.
- [ ] Build a parser-backed document symbol index that records lexical scopes, declarations, references, and UTF-16 locations without executing code.
- [x] Build a tested non-executing local declaration index with lexical scope depth and UTF-16 declaration positions.
- [x] Add scope-aware go-to-definition and dynamic in-document completion from the symbol index, with Bangla-English regression fixtures.
- [x] Make interpreter block-local bindings agree with the compiler’s lexical binding model, including Bangla shadowing and assignment lookup regression tests.
- [x] Add dynamic in-document completion for visible local declarations with Bangla-English regression fixtures.
- [x] Add conservative same-document go-to-definition for the nearest visible local declaration, with shadowing regression tests.
- [x] Add conservative rename preparation and edits for compiler-bound same-document local variables, rejecting imports, public exports, functions, members, unresolved identifiers, malformed source, and invalid replacement identifiers.
- [x] Bind identifier references to the nearest visible same-document local declaration using lexical scope; nested shadowed names receive distinct binding IDs.
- [x] Implement `textDocument/prepareRename` and `textDocument/rename` only for deterministic local variable bindings with validated replacement identifiers.
- [x] Reject unsafe rename requests for imports, exports, functions, members, unresolved identifiers, and malformed source; preserve shadowed locals as separate bindings and add Bangla regression tests.
- [x] Expose a compiler-owned parsed-document analysis API with stable local declaration/reference IDs and source positions for editor consumers; the LSP converts those positions to UTF-16 ranges.
- [x] Expose a compiler-owned parsed declaration API with local declaration kind, scope depth, and one-based source positions for editor consumers.
- [x] Bind same-document local identifier references to lexical declarations without evaluating source or treating strings/comments as references.
- [x] Return LSP rename edits only when every bound local reference is deterministic and the requested replacement is a valid Padma identifier.
- [x] Define safe, opt-in Python and JavaScript bridge interfaces with typed data exchange and subprocess isolation.
- [x] Define a versioned `bridge.call` contract that accepts JSON-compatible data only and returns decoded JSON values without evaluating returned source code.
- [x] Require explicit `process` capability allowlist entries for each Python or JavaScript executable before any bridge child process starts.
- [x] Execute bridge programs with fixed argument vectors, project-root-scoped script paths, bounded input/output, captured stderr, and localized failure diagnostics.
- [x] Add Bangla-English regression tests for successful typed exchange, denied capabilities, invalid JSON, non-zero exit, missing runtime, and unsafe paths.
- [x] Document a minimal Termux installation and use workflow for optional Python and Node.js bridges, including capability manifest examples and security boundaries.
- [x] Add safe domain libraries for HTTP APIs, web services, automation, and AI-provider clients; publish defensive security-tooling boundaries.
- [x] Extend the network library with bounded `http.post` and `http.json` JSON workflows that use validated HTTP(S) URLs, timeouts, response limits, and `network` capability grants.
- [x] Add provider-neutral `ai.request` with an explicit endpoint, caller-supplied environment variable name instead of literal secrets, and decoded JSON data rather than executable code.
- [x] Add local backend response and JSON-file automation primitives without exposing a general public server or arbitrary shell command API.
- [x] Add capability, URL, JSON-boundary, and safe-path regression coverage; stable existing bilingual diagnostics apply to denied access and malformed data.
- [x] Publish Termux-first HTTP, AI, and automation examples with manifest capability grants, secret-handling guidance, and an explicit warning against embedding API keys in Padma source.
- [ ] Add semantic versioning, release notes, signed artifacts, reproducible builds, compatibility fixtures, fuzzing, benchmarks, and SBOM generation.
- [ ] Publish Bangla-first tutorials, a reference manual, API documentation, contribution rules, a code of conduct, and maintainership policy.
- [ ] Add static type checking and actionable bilingual diagnostics without breaking the interpreter.
- [ ] Build a Termux-friendly `padma` CLI with run, check, format, test, init, and package commands.
- [ ] Add dependency-free standard library for files, text, collections, JSON, HTTP, and process boundaries.
- [x] Make `padma file.pd` the primary CLI invocation, with `padma run file.pd` retained only as a compatibility alias.
- [x] Add interactive `input`.
- [x] Add safe file read/write APIs with validated relative output paths.
- [ ] Add argument-safe process execution, timeout, stdout/stderr capture, and exit-code values.
- [x] Add HTTP GET with bounded timeout and localized network errors.
- [x] Add `media.download(url, output)` wrapper backed by an installed `yt-dlp` executable.
- [ ] Add an authorized media-download wrapper with clear platform-terms and ownership boundaries.
- [ ] Add Python-versus-Padma downloader examples and Termux smoke tests.
- [ ] Add `@downloads` alias that resolves to Termux shared storage Download folder.
- [ ] Make the minimal downloader example runnable from any working directory with one `padma file.pd` command.
- [ ] Verify installer behavior on a fresh Termux shell and ensure `$PREFIX/bin/padma` is on PATH.
- [ ] Add a clear post-install command check and recovery message for `padma: command not found`.
- [ ] Add a Termux fallback that runs Padma directly from `~/padma-lang/target/release/padma` when `$PREFIX/bin` installation is unavailable.
- [ ] Provide a diagnostic command that reports the exact installer failure instead of silently proceeding.
- [ ] Add `padma --version` and `padma --help` behavior matching Termux CLI expectations.
- [x] Add `padma` no-argument interactive REPL with Bengali-English input and localized errors.
- [ ] Submit the deterministic Termux package recipe upstream and wait for repository publication before claiming `pkg install padma -y` support.
- [ ] Add package install, version, REPL, and script execution smoke tests.
- [x] Match Python-style interactive shell banner and `padma>` prompt behavior.
- [x] Support REPL commands `help`, `copyright`, `credits`, `license`, `exit()`, `quit()`, and `বের হও`.
- [x] Add persistent REPL examples and command-level smoke tests.
- [x] Make the interactive REPL display a non-null bare expression value, matching Python-style `1 + 1` evaluation while preserving explicit `print`/`দেখাও` statements and localized errors.
  - [x] Add English, Bangla-digit, mixed-expression, null-result, assignment, and error-path REPL regression tests.
- [x] Improve installer PATH detection and print actionable recovery instructions.
- [ ] Submit and track the Padma recipe in an actual Termux package repository before claiming `pkg install padma -y` availability.
- [ ] Add safe interoperability paths for Python, JavaScript/TypeScript, C, and shell instead of claiming automatic conversion.
- [ ] Add package metadata, lockfile, registry-ready layout, reproducible builds, and semantic versioning.
- [ ] Add security limits, sandbox boundaries, path validation, dependency checks, and supply-chain guidance.
- [ ] Add unit, integration, golden diagnostic, fuzz, and Termux smoke tests.
- [ ] Document Android/Termux installation, examples, contribution rules, and release process.
- [ ] Remove or isolate non-core playground artifacts if they distract from the language repository.

## M9 — Application Platform and Distribution

- [ ] Deliver the remaining production platform capabilities through independently shippable, security-reviewed milestones; do not expose unrestricted servers, databases, package installs, credentials, deployment credentials, device permissions, or browser control by default.
- [ ] Add a Termux-first local web server framework with fixed listen policy, explicit `server` capability grant, bounded request sizes, safe route matching, JSON responses, and graceful shutdown.
- [ ] Add a local SQLite persistence layer with project-root database paths, prepared statements, typed JSON rows, explicit `database` capability grant, migrations, and transaction safety; do not create a raw SQL string-evaluation API.
  - [x] Add a fixed-command, project-root-scoped SQLite foundation with explicit `database:sqlite` capability checks and localized diagnostics.
  - [ ] Add typed parameter binding, deterministic JSON row decoding, transaction boundaries, and schema migration metadata without exposing raw SQL evaluation.
    - [x] Add a versioned, fixed migration metadata record and a read-only `db.version` API without accepting executable user schema text.
    - [x] Add an atomic bounded batch API for fixed Padma record operations; reject nesting, callbacks, arbitrary SQL, and cross-database batches.
- [ ] Add a signed package registry client and deterministic package lock workflow with integrity verification, version resolution, package-root isolation, offline cache, and explicit trust policy.
  - [x] Define and validate a versioned package manifest with explicit exports, capability declarations, source digest, and no lifecycle scripts.
  - [x] Add deterministic local dependency resolution and a canonical `padma.lock` writer that records resolved source paths and digests.
  - [x] Add an opt-in project-local cache inspector with strict path scope and digest verification; do not download or execute packages automatically.
- [ ] Add an optional identity/session layer using password hashing, signed expiring sessions, CSRF controls, secure cookie defaults, and secret names sourced from environment variables rather than source code.
  - [ ] Define a local, deterministic password-record validation and creation contract that stores an algorithm-tagged salted digest, never a plaintext password.
  - [ ] Add signed, versioned, expiring session envelopes and explicit verification without a network auth server or default cookie emission.
  - [ ] Define CSRF token and secure-cookie construction policy as pure, reviewed helpers; reject secret literals and untrusted environment-variable names.
- [ ] Add deployment manifests and dry-run adapters before any remote deployment integration; preserve clear ownership, secret isolation, build reproducibility, rollback metadata, and explicit user confirmation for remote actions.
  - [x] Define and validate a versioned deployment manifest with project-relative entry, bounded target metadata, public base URL policy, and approved environment-variable names only.
  - [x] Add deterministic `padma deploy plan` and read-only `padma deploy inspect` commands; never transmit credentials, build artifacts, or application data.
  - [x] Record a source digest and rollback descriptor in a local deployment plan, while rejecting secret values, shell commands, remote URLs, and unbounded providers.
  - [ ] Define provider-specific remote deployment contracts, isolated artifact build inputs, credential handoff by environment-variable name only, a user-visible confirmation token, and rollback execution semantics before enabling any remote action.
  - [ ] Add an explicit adapter boundary that cannot send artifacts, invoke provider APIs, or execute a rollback unless a reviewed provider module and an interactive user confirmation path are present.
  - [x] Add a Render Git-linked release contract that validates repository identity, branch, immutable commit SHA, service identifier, build isolation record, and provider dashboard confirmation without sending a provider request.
  - [x] Add a Render API adapter plan that validates only a Render service ID, a token environment-variable name, an immutable commit SHA, confirmation token, and explicit rollback deploy ID; no secret value or provider request is permitted in planning mode.
- [ ] Define GUI/mobile application bridges as opt-in adapter contracts with no automatic Android permission elevation, then validate a small Termux-friendly renderer example.
  - [x] Define a versioned project-scoped renderer manifest with fixed local backends, project-relative entry and asset roots, and no executable hooks or native permission requests.
  - [x] Add read-only `padma gui inspect` and deterministic `padma gui plan` commands that validate manifest constraints without starting a renderer or device process.
  - [x] Add a small Termux-friendly HTML renderer example and negative tests for unsafe paths, unknown backends, external URLs, permissions, and command injection fields.
  - [ ] Define separately reviewed Android adapter contracts for explicit permission declarations, signed APK build inputs, device transport consent, and native-code boundaries; do not add automatic elevation, APK build, device control, or native-code execution to the core CLI.
  - [x] Add a read-only Android build-plan manifest validator with approved Android permissions, signed artifact metadata fields, and strict rejection of install, ADB/device commands, native hooks, and automatic permission elevation.
- [ ] Add provider-neutral AI workflow helpers for structured responses and local model adapters, bounded by explicit network/process capabilities and never executing model output as code.
- [ ] Add bounded browser automation using reviewed action plans, domain allowlists, confirmation before side effects, redaction of sensitive fields, and no CAPTCHA or login bypass behavior.
- [ ] Add Bangla-English diagnostics, negative security tests, Termux copy-paste examples, and release gates for every M9 component before marking it stable.

## Repository Professionalization

- [ ] Reorganize the repository into a language-first, Termux-friendly, contributor-ready structure without breaking the stable `padma` CLI, interpreter, LSP, installer, examples, or CI contracts.
  - [x] Audit public files, build entry points, CI assumptions, documentation discoverability, and repository hygiene before moving any source or tooling paths.
  - [x] Add a clear root README, contribution guide, security policy, code-of-conduct policy, issue/PR templates, and release-maintenance documentation appropriate for an open-source language project.
  - [x] Establish stable directories for specifications, examples, tooling, tests, scripts, and release artifacts, then document ownership and supported compatibility boundaries.
  - [x] Add repeatable quality commands and CI checks for formatting, tests, release build, documentation links, repository hygiene, and Termux install smoke coverage.

## Practical Project Examples

- [ ] Publish a Termux-first practical project guide that explains runnable Padma examples, exact expected output, capability manifests, and current security limitations without overstating unsupported features.
  - [x] Add a capability-gated authorized media-download example using `media.download`, with a clear ownership and platform-terms boundary.
  - [x] Add static website, local backend response, SQLite persistence, local-server plan, and defensive security-inspection examples with line-by-line walkthroughs.
  - [x] Add corresponding Bengali-English output samples and verify every documented command against the release binary before publishing.

## Padma Agent Skill

- [x] Package Padma project knowledge as a reusable Agent Skill so future feature work follows the language-first, Termux-first, capability-safe, GitHub-reviewed engineering contract.
  - [x] Define the skill trigger, supported request categories, stable public contracts, non-goals, and mandatory safety boundary for Padma language work.
  - [x] Add modular reference guides for architecture, syntax/API verification, Termux distribution, capability security, examples, testing, and release procedure.
  - [x] Add a skill validation checklist and repository documentation that explain how contributors use and maintain the skill without treating it as a runtime language package.

## M9 — AI Workflow and Browser Planning

- [x] Define and implement a provider-neutral AI workflow helper with project-local provider metadata, explicit `network:ai` capability gating, environment-variable-only secrets, bounded request/response JSON, and no automatic execution of generated output.
  - [x] Add a versioned `padma-ai.toml` contract plus `padma ai inspect` and `padma ai plan` commands that validate configuration without making network requests or reading secret values.
    - [x] Implement reserved bilingual diagnostics `P1050`–`P1052`, strict AI manifest data structures/parser, and capability-gated local manifest loading.
    - [x] Implement deterministic inspection-only plan JSON with `network: "disabled"`, `secret.value: "not-read"`, and no environment, DNS, child-process, or network access.
  - [x] Add a structured `ai.workflow` runtime contract with explicit provider selection, input/output schemas, bounded retry policy, localized diagnostics, redaction, positive tests, and security-negative tests.
    - [x] Implement a strict JSON request envelope and bounded structured response validator that expose model output as inert Padma data only.
    - [x] Implement one `json-http-v1` transport path with a fixed request shape, timeout/size limits, no retry, sanitized diagnostics, and secret exclusion from command arguments, output, logs, and child environment.
    - [x] Add transport mock tests for exactly-one request, missing/empty secret, timeout, non-zero exit, invalid response, redaction, and prohibition on generated-output execution.
  - [x] Publish an AI workflow security guide and a runnable local planning example with provider setup and data-handling limits.
- [x] Define and implement domain-allowlisted browser automation planning with explicit `browser:plan` capability gating, project-local manifest, no login or CAPTCHA bypass, no payment/posting, and no browser execution in the initial milestone.
  - [x] Add a versioned `padma-browser.toml` contract plus `padma browser inspect` and `padma browser plan` commands that validate allowlisted HTTPS origins, navigation-only intent, and redacted request descriptors.
    - [x] Implement bilingual `P1053`–`P1055` diagnostics, strict manifest data structures/parser, and exact HTTPS-origin validation that rejects credentials, fragments, path/query values, IP/private-network targets, and suffix matching.
    - [x] Implement deterministic read-only plan JSON with `browser: "not-started"`, `network: "disabled"`, `dns: "disabled"`, `cookies: "not-read"`, and no URL fetch, environment read, child process, or browser-profile access.
  - [x] Add browser-plan policy tests for domain/subdomain matching, redirect boundaries, credentials, private-network targets, unsafe actions, malformed manifests, missing capability, secret redaction, and zero side effects.
  - [x] Publish the browser planning security guide, safe example, explicit confirmation boundary, and future execution-adapter requirements.

## M10 — Confirmed AI and Browser Action Adapters

> Selected sequencing: complete and verify the local AI tool/training planning foundation before beginning any browser navigation action-adapter implementation.

- [ ] Define a versioned, provider-neutral AI tool-contract layer that keeps generated output inert until a project-declared tool schema and one explicit action request are validated.
  - [ ] Design bounded tool descriptors, JSON input/output schemas, least-privilege capabilities, per-tool timeout/output limits, immutable audit records, and bilingual diagnostics without secret leakage.
  - [ ] Implement a local-only `padma ai tools inspect|plan` foundation and regression coverage for missing grants, unsafe schemas, unknown tools, redaction, and zero execution during planning.
  - [ ] Define a bounded agent-runbook state machine with maximum steps, maximum wall-clock duration, no background persistence, no hidden retries, no generated-code execution, and a mandatory user-visible stop control.
- [x] Define a safe local model training adapter contract that plans and validates user-owned datasets, resource limits, model artifacts, and subprocess handoff without claiming a built-in universal training engine.
  - [x] Require an explicit project capability, local-only dataset/artifact paths, declarative hardware/runtime limits, no secret values in manifests, and a no-training `inspect|plan` mode before any training execution adapter.
  - [x] Document that training execution needs a separately installed, reviewed local backend and cannot silently use remote compute, device controls, or unbounded data collection.
- [x] Define a separately reviewed browser navigation action-adapter contract that begins only after an exact-origin local plan and a fresh, user-visible confirmation for each bounded navigation session.
  - [x] Implement a local-only browser confirmation-session manifest and `inspect|plan` command that bind one exact existing browser plan digest to one GET-only reviewed destination without starting a browser.
  - [x] Reserve a short-lived, single-use, locally generated confirmation-challenge contract for a future runner; the current descriptor marks model-supplied approval rejected and issues no token, while reading no profile, cookie, credential, or environment value.
  - [x] Emit deterministic redacted session descriptors with `browser: "not-started"`, `network: "disabled"`, `dns: "disabled"`, `session: "awaiting-confirmation"`, cancellation support, and no action executor.
  - [x] Compare and select one independently installed, user-controlled local browser runner with a Termux-compatible installation model; do not fall back to a generic remote browser service or arbitrary command execution.
  - [x] Implement `browser:handoff` and a fixed Android Browser Handoff command that accepts only a validated exact-origin confirmation-session descriptor and opens its one approved HTTPS URL through a reviewed Termux URL opener.
  - [x] Require a foreground, user-visible confirmation immediately before handoff, reject noninteractive/expired/mismatched requests, and record only a redacted local result; never pass cookies, headers, credentials, profile paths, scripts, selectors, or arbitrary arguments to the opener.
  - [x] Fail safely with localized diagnostics when the Termux URL opener is unavailable or fails, without retrying, scraping, launching a fallback browser service, or persisting session state.
  - [x] Define an opt-in project-local redacted handoff audit format that records only timestamp, plan digest, navigation index, state, and outcome code; prohibit raw URLs, query strings, cookies, credentials, headers, browser data, terminal approval text, and page content.
  - [x] Add an explicit pre-handoff `CANCEL` path and immediate stdin/EOF failure cleanup, ensuring no opener process starts and no reusable confirmation/session state remains after cancellation.
  - [x] Define bounded audit retention, atomic local write behavior, restricted project-relative audit paths, and localized failure diagnostics before any audit persistence is enabled.
  - [ ] Define the runner handshake, immutable plan-digest binding, isolated ephemeral profile, fixed GET-only interface, cancellation, active time/navigation ceilings, failure cleanup, and redacted local audit format before adding any network operation.
  - [ ] Implement confirmed navigation only after the user explicitly selects the reviewed runtime and per-session confirmation interface; require an immediate user-visible confirmation before browser start and prohibit sensitive actions entirely in the first runner release.
  - [ ] Begin browser action-adapter implementation only after the preceding AI tool and training planning layers pass their security regression tests and repository verification.
  - [ ] Preserve exact HTTPS origin matching, revalidate each redirect and destination immediately before use, prohibit login credential capture, CAPTCHA bypass, JavaScript injection, cookie/profile exfiltration, unsafe downloads, and automatic form submission.
  - [ ] Require explicit confirmation before any external side effect such as form submission, upload, message/post, purchase, payment, account change, or data deletion; do not provide a silent or autonomous execution path.
  - [ ] Add browser action tests for missing confirmation, allowlist escapes, redirects, credential-bearing URLs, expired confirmation, cancellation, action audit redaction, and denied sensitive operations.

## M11 — User-Mediated Browser Interaction Drafts

- [x] Define a strict local `padma-browser-draft.toml` contract for reviewable browser interaction drafts that can describe text, attachment metadata, and a reviewed destination without collecting credentials, cookies, page data, selectors, scripts, or a raw live-session state.
  - [x] Add `browser:draft` capability-gated `padma browser draft inspect|plan` commands that emit inert, deterministic draft descriptors and never start a browser, resolve DNS, read a file, upload data, submit a form, post a message, or execute generated content.
  - [x] Require draft attachment paths to be project-relative metadata only; no file read/upload action is permitted in the draft foundation.
  - [x] Add explicit `user-takeover-required` state for login, CAPTCHA, form completion, post/message, upload/download, account change, purchase/payment, or any sensitive external action.
- [x] Document and test that a draft may be copied or reviewed by the user after a visible Android Browser Handoff, but Padma cannot inject it into a webpage, fill a form, or infer/record the user’s decision.

## M12 — Visible Browser Takeover Workflow

- [x] Define a strict local takeover checklist manifest that binds one reviewed browser-plan digest and navigation index to user-visible, non-sensitive review steps without reading live browser state or collecting a user decision.
  - [x] Add a separate capability-gated local `inspect|plan` descriptor that tells the user when to take over for login, CAPTCHA, form completion, posting, upload/download, account change, purchase, payment, or other sensitive destination-controlled actions.
  - [x] Keep the descriptor non-executing: no browser launch, DNS/network activity, form filling, page inspection, credential/cookie/profile access, JavaScript injection, attachment read/upload, post, payment, or generated-output execution.
  - [x] Add cancellation guidance, redaction rules, Bangla-English diagnostics, negative security tests, a Termux example, and documentation before marking the workflow stable.

## M13 — Daily-Use Practical Tool Roadmap

- [ ] Publish a capability matrix that separates currently executable tools, local planning-only tools, external prerequisites, and intentionally unsupported high-risk actions for Termux-first Padma users.
- [x] Structured-data toolkit: capability-gated project-local CSV/TSV/JSON read, validation, filtering, mapping, aggregation, and safe bounded file output with Bangla-English diagnostics and examples.
- [ ] HTTP API toolkit: typed JSON request templates, validated response extraction, timeout/retry limits, secret-environment-name handling, and explicit network capability grants without hidden request loops.
- [ ] Filesystem productivity toolkit: bounded recursive listing, safe copy/move/archive plans, checksums, text search, and dry-run-before-write interfaces scoped to the project root.
- [ ] Developer workspace toolkit: project task aliases, deterministic test/build/lint plans, safe process allowlists, exit-code summaries, and no-shell argument handling for common Termux workflows.
- [ ] Web/backend practical toolkit: reusable local-server routes, request validation, JSON response helpers, static-site generation, and deployment-plan contracts while keeping remote deployment separately confirmed.
- [ ] SQLite application toolkit: schema migration plans, query parameter validation, export/import contracts, backup descriptors, and bounded project-local data maintenance.
- [ ] AI productivity toolkit: provider-neutral structured prompt templates, bounded JSON-schema extraction, explicit user-reviewed outputs, and no autonomous tool/browser/output execution.
- [ ] Media and document toolkit: authorized local media metadata/transcode plans, text/PDF/CSV reporting helpers, and prerequisite validation without unauthorized downloading or background processing.
- [ ] Package ecosystem foundation: registry protocol design, package provenance/digest verification, dependency resolution limits, offline cache rules, and package publication review before any remote registry mutation.
- [ ] Add each increment only after narrow capability design, negative security tests, standalone Termux example, bilingual documentation, and full repository verification.
