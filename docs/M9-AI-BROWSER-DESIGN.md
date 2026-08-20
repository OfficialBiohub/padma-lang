# M9 AI Workflow and Browser Planning Design

**Status:** Draft design contract before implementation.

## 1. Design decision summary

| Area | M9 v1 decision | Explicitly deferred |
|---|---|---|
| AI | One project-local, provider-neutral JSON workflow gateway | Embedded vendor SDKs, model training, streaming, tool execution, conversation memory, automatic retries |
| Browser | A project-local navigation **plan**, validated without opening a browser or network connection | Browser control, login/session use, CAPTCHA handling, uploads, downloads, posting, payments, form submission |
| Authority | Existing `network:ai` gates AI requests; a new `browser:plan` grant gates only browser-plan inspection | Broad `network:all`, `browser:all`, shell, persistent services, background jobs |

Both additions are manifest-first. A plan describes intent and validates policy; it is not an action token and cannot itself perform a network request or operate a browser.

## 2. Provider-neutral AI workflow helper

### 2.1 Scope and compatibility

The existing `ai.request(endpoint, secret_env_name, payload)` remains available for narrow advanced use. It is an explicit endpoint-and-payload escape hatch. The new `ai.workflow(input)` is deliberately stricter: it is available only in project mode, reads a reviewed `padma-ai.toml` beside `padma.toml`, allows one fixed endpoint, and speaks one Padma-defined JSON protocol. Existing scripts keep their behavior; new projects receive a safer, portable contract.

> **Provider-neutral does not mean that all external AI vendors share a wire format.** It means Padma source targets a stable Padma request/response envelope rather than a vendor SDK. A provider-specific translation, if necessary, belongs in a separately reviewed external gateway or a future versioned adapter; it does not become implicit interpreter behavior.

M9 v1 does not train, fine-tune, host, select, benchmark, or download models. It sends one bounded request to an already selected, public HTTPS endpoint. It does not fetch web content, call a tool, run returned code, write a file, start a process, or mutate a browser.

### 2.2 Capability model

`ai.workflow` requires the existing project grant:

```toml
[capabilities]
network = ["ai"]
```

No new `ai:workflow` capability is added. `network:ai` already represents the sensitive authority: transmitting data to an AI endpoint. Splitting that same authority across two grants would make review less clear while not narrowing the actual network boundary. The workflow manifest narrows the grant further by fixing one reviewed endpoint and one adapter contract.

Absent `network:ai`, `ai.workflow` and `padma ai inspect|plan` fail through the existing localized `P1034` capability path. Inspection deliberately remains capability-gated: an undeclared integration should not acquire a first-class command surface merely because it is described as a plan.

### 2.3 `padma-ai.toml` v1 contract

The initial release supports exactly one workflow per project. This preserves a small parser, avoids profile selection semantics, and gives users a reviewable one-endpoint policy. Named multi-workflow profiles are a later compatibility-reviewed extension.

```toml
[workflow]
version = "1"
adapter = "json-http-v1"
endpoint = "https://ai-gateway.example.com/v1/padma"
secret_env = "PADMA_AI_KEY"
model = "reviewed-model-id"
timeout_seconds = 30
max_input_bytes = 32768
max_response_bytes = 65536
retry_policy = "never"
```

| Field | Required v1 validation |
|---|---|
| `version` | Exactly the quoted string `"1"`. |
| `adapter` | Exactly `json-http-v1`; no arbitrary provider plugin, executable, URL, or shell selector. |
| `endpoint` | Public `https://` ASCII DNS host and optional normalized path. It rejects credentials, fragments, queries, explicit ports, IP literals, loopback/link-local/private names, whitespace, traversal segments, and non-HTTPS schemes. |
| `secret_env` | An uppercase ASCII environment-variable **name** accepted by the existing safe-name policy; a token, `name=value`, or secret-like literal is invalid. |
| `model` | A bounded printable identifier used as public request metadata, never an environment expansion or command fragment. |
| `timeout_seconds` | Integer from 1 to 30; default not permitted, so latency is always reviewable. |
| `max_input_bytes` | Integer from 1 to 32,768. |
| `max_response_bytes` | Integer from 1 to 65,536. |
| `retry_policy` | Exactly `never` in v1. An AI request may incur cost or create provider-side state, so automatic retry is not safe without an explicit idempotency contract. |

The parser rejects unknown sections, unknown fields, duplicated fields, comments containing no semantic values, malformed quoted values, and all missing required fields. It must never resolve environment-variable expressions, open a network connection, execute an adapter, or read the secret while parsing.

### 2.4 Stable Padma JSON protocol

`ai.workflow` accepts exactly one JSON-compatible Padma map. The call is intentionally explicit about data and instruction separation:

```padma
ফল = ai.workflow({
  "task": "summarize",
  "instruction": "বাংলায় তিনটি সংক্ষিপ্ত পয়েন্টে সারাংশ দাও।",
  "data": {"text": নিবন্ধ}
})
লিখি(ফল["output"]["summary"])
```

The runtime accepts only string map keys, a `task` identifier, a bounded `instruction` string, and JSON-compatible `data`. It enforces a maximum nesting depth of 16 and the configured serialized-input byte limit. It then sends one fixed `POST` body:

```json
{
  "protocol": "padma-ai-workflow-v1",
  "model": "reviewed-model-id",
  "task": "summarize",
  "instruction": "...",
  "data": {"text": "..."}
}
```

The `json-http-v1` endpoint must answer with bounded JSON in this shape:

```json
{
  "protocol": "padma-ai-workflow-v1",
  "output": {"summary": "..."}
}
```

The runtime rejects a wrong protocol, missing `output`, non-object output, invalid JSON, an over-limit response, or over-depth output. A successful call returns only:

```json
{
  "output": {"summary": "..."},
  "meta": {"adapter": "json-http-v1", "model": "reviewed-model-id", "attempts": 1}
}
```

It does not return HTTP headers, raw transport diagnostics, provider credentials, internal system prompts, or a mutable response object. The model output is ordinary untrusted Padma data, not code, HTML, Markdown to render, shell input, SQL, a URL to fetch, or a browser command. OWASP recommends treating model output as zero-trust input and validating it before passing it to downstream systems; it specifically identifies code execution, XSS, SSRF, path traversal, and SQL injection as risks of improper output handling.[3]

### 2.5 Transport and secret handling

The runtime makes exactly one `curl` request using a fixed argument vector: `POST`, JSON content type, configured 1–30 second timeout, configured response size, and **no redirect following**. It has no user-controlled curl flag, header list, proxy value, TLS bypass, custom CA path, output file, or retry option. The endpoint comes exclusively from the verified manifest.

Before action, the runtime reads `secret_env` only from the current process environment. An absent, empty, or control-character-containing value fails locally. `padma ai inspect` and `padma ai plan` do not call `env::var`, so they cannot distinguish an absent credential from a populated one.

The implementation must also remove bearer credentials from the child-process argument vector. The recommended no-new-crate approach is a short-lived, mode-`0600` JSON request file inside an internal project-scoped temporary directory plus `curl --config -`, sending the authorization header only through the child standard input. The temporary input file is deleted after child completion on both success and ordinary failure; it is never named in user-facing output. This is an implementation requirement, not a claim that a local operating system can provide a general secret sandbox.

### 2.6 AI CLI contract

```text
padma ai inspect [project]
padma ai plan [project]
```

Both commands validate the project manifest, the `network:ai` grant, and `padma-ai.toml`; then they print deterministic JSON. Neither starts `curl`, reads a secret, resolves DNS, follows a URL, mutates a file, or calls `ai.workflow`.

```json
{
  "aiWorkflowPlanVersion": 1,
  "mode": "inspection-only",
  "project": {"name": "study-helper", "version": "0.1.0"},
  "adapter": "json-http-v1",
  "endpoint": "https://ai-gateway.example.com/v1/padma",
  "secret": {"environmentName": "PADMA_AI_KEY", "value": "not-read"},
  "limits": {"timeoutSeconds": 30, "maxInputBytes": 32768, "maxResponseBytes": 65536, "retryPolicy": "never"},
  "network": "disabled",
  "modelExecution": "disabled",
  "generatedOutputExecution": "disabled"
}
```

`inspect` may add a one-line human heading before the same JSON. `plan` prints JSON only. The help text and every failure are localized from the project locale. The implementation reserves `P1050` for an invalid AI workflow manifest or request descriptor, `P1051` for a failed bounded AI transport, and `P1052` for a malformed AI workflow response; existing `P1034` remains the missing-capability diagnostic.

## 3. Domain-allowlisted browser planning

### 3.1 The first browser increment is a plan, not browser automation

The commands below read one project-local policy file and render a deterministic navigation descriptor:

```text
padma browser inspect [project]
padma browser plan [project]
```

They do **not** start a browser, resolve a hostname, open a TCP/TLS connection, fetch a URL, read a page, create a profile, use cookies, access a saved login, solve or bypass a CAPTCHA, submit a form, upload or download a file, post content, make a payment, or send a message. “Inspect” refers to inspecting the manifest, never inspecting a remote website.

This deliberately small first increment establishes an auditable policy before any browser adapter is considered. URL mishandling can turn an application into a network proxy; OWASP therefore recommends strict allowlisting for identified trusted destinations and disabling automatic redirect following to prevent validation bypasses.[1]

### 3.2 Capability decision

Browser planning introduces a new narrow project capability:

```toml
[capabilities]
browser = ["plan"]
```

It normalizes to `browser:plan`. The grant authorizes only local validation and plan emission. It does not imply `network:http`, `network:ai`, browser launch, network activity, page retrieval, storage, login, or any future browser action. A missing grant produces the standard localized `P1034` error naming `browser:plan`.

The capability parser must allow `browser = ["plan"]` and reject every other browser grant in M9 v1. Future `browser:navigate` would be a separate reviewed capability; it must not silently activate because an old project already holds `browser:plan`.

### 3.3 `padma-browser.toml` v1 contract

The v1 manifest uses a fixed navigation list rather than a general action language. There are no selectors, JavaScript snippets, form fields, cookies, credentials, request headers, proxy settings, extensions, downloads, or user-supplied browser command options.

```toml
[browser]
version = "1"
intent = "navigation-review"
redirect_policy = "deny"
max_steps = 4

[allowlist]
origins = [
  "https://docs.python.org",
  "https://www.rust-lang.org"
]

[navigation]
urls = [
  "https://docs.python.org/3/tutorial/",
  "https://www.rust-lang.org/learn"
]
```

| Section and field | Required v1 policy |
|---|---|
| `browser.version` | Exactly quoted string `"1"`. |
| `browser.intent` | Exactly `navigation-review`; no generic `automation` intent. |
| `browser.redirect_policy` | Exactly `deny`. There is no implicit or configured redirect following. |
| `browser.max_steps` | Integer from 1 through 16 and greater than or equal to the number of URLs. |
| `allowlist.origins` | One to 16 exact canonical HTTPS origins. Each origin is a lowercase ASCII DNS hostname with no path, query, fragment, userinfo, IP literal, port, trailing dot, wildcard, or `xn--` IDN label. A trailing `/` is invalid rather than normalized. |
| `navigation.urls` | One to `max_steps` exact HTTPS URLs. Each must match one allowlisted origin exactly and may contain only a simple absolute path. Query, fragment, userinfo, explicit port, encoded characters, `.` or `..` segments, repeated slash, control/whitespace, and non-ASCII characters are rejected in v1. |

The ASCII-only policy is intentionally conservative because the Rust core has no additional IDNA/URL-normalization crate. It is a **documented v1 limitation**, not a claim that internationalized domains are unsafe. A future IDN feature requires a tested canonicalization library or a reviewed standards-compliant adapter before it can safely compare visually similar hostnames.

The parser rejects unknown sections/fields, repeated values, empty arrays, malformed quoted lists, and navigation URLs that do not match the allowlist. It reports only the field location or list index—never the rejected raw string—so a bad URL containing credentials or tokens cannot be copied into a diagnostic or log.

### 3.4 Exact-origin matching and future transport policy

An allowlist origin is a capability boundary, not a text suffix test. `https://example.com` matches only URLs whose scheme is `https`, host is exactly `example.com`, and port is the implicit HTTPS port. It does not match `example.com.attacker.invalid`, `sub.example.com`, `example.com:8443`, an IP resolving from the name, or a URL with look-alike/unicode encoding.

M9 planning performs no DNS lookup. A later `browser:navigate` adapter must resolve each permitted hostname immediately before every socket connection and deny all loopback, unspecified, multicast, link-local, private, carrier-grade NAT, documentation, and other non-public IPv4/IPv6 address ranges. It must connect only to a validated returned address, revalidate every redirect target against the same exact-origin rule, and keep redirect following disabled by default. DNS resolution alone is not an allowlist proof because hostname-to-address bindings can change.[1]

The future adapter uses a fresh, ephemeral browser profile with persistent cookie, history, password manager, extension, native messaging, filesystem, credential-store, and cross-origin shared-state access disabled. It must not accept a user’s existing browser profile or browser debugging port as input. Page text, DOM content, screenshots, and downloaded resources are treated as untrusted data; they cannot modify the plan, invoke Padma code, or cause an AI workflow call.

### 3.5 Deterministic planning output and redaction

`padma browser plan` prints only validated metadata, preserving navigation order while sorting exact origins for stable review:

```json
{
  "browserPlanVersion": 1,
  "mode": "inspection-only",
  "project": {"name": "docs-review", "version": "0.1.0"},
  "capability": "browser:plan",
  "intent": "navigation-review",
  "allowlistedOrigins": [
    "https://docs.python.org",
    "https://www.rust-lang.org"
  ],
  "navigation": [
    {"step": 1, "method": "GET", "url": "https://docs.python.org/3/tutorial/"},
    {"step": 2, "method": "GET", "url": "https://www.rust-lang.org/learn"}
  ],
  "redirectPolicy": "deny",
  "browserExecution": "disabled",
  "network": "disabled",
  "browserState": "not-read",
  "credentials": "prohibited",
  "captcha": "prohibited",
  "formSubmission": "prohibited",
  "posting": "prohibited",
  "payment": "prohibited",
  "uploads": "disabled",
  "downloads": "disabled"
}
```

Because accepted v1 URLs prohibit query strings, fragments, and userinfo, the plan does not expose tokens in a URL. The browser manifest has no secret/environment field, and both commands must not read the environment. Invalid values are not echoed in output. No plan receives a confirmation token because no action exists to confirm.

### 3.6 Future action-adapter boundary

No browser execution command is part of this milestone. If Padma later adds one, it must use a new manifest version and a new `browser:navigate` grant; show the exact canonical plan and its digest; require explicit, fresh human confirmation for a single bounded GET navigation session; and maintain an audit descriptor without secret values or page contents.

That adapter must preserve the following hard stops. Browser login/session automation, CAPTCHA or anti-bot bypass, password/one-time-code capture, sensitive form handling, arbitrary JavaScript injection, posting, messaging, social-media publishing, purchase/payment, account changes, file upload, download, clipboard access, device control, and remote debugging connection remain outside the core Padma runtime. Any proposed sensitive action needs a target-specific protocol, visible confirmation immediately before the action, secure credential custody outside Padma source/manifests, recovery semantics, and a dedicated security review.

### 3.7 Browser diagnostics

The design reserves `P1053` for an invalid browser planning manifest, `P1054` for a navigation descriptor that violates the reviewed exact-origin/path policy, and `P1055` for an unavailable or prohibited browser execution path. Diagnostics must be generated in Bangla for `locale = "bn"` projects, English for `locale = "en"`, and use the existing automatic locale selection for mixed source. Existing `P1034` remains the capability-denial code.

## 4. Shared security invariants

The following rules apply across both components and are release blockers. They make the authority reduction visible in code, tests, plans, and documentation rather than relying on a user’s prompt or a provider’s behavior.

| Invariant | AI workflow | Browser planning |
|---|---|---|
| Project declaration | Requires `padma.toml`, project-root-scoped `padma-ai.toml`, and `network:ai`. | Requires `padma.toml`, project-root-scoped `padma-browser.toml`, and `browser:plan`. |
| Manifest authority | Fixed protocol, endpoint, limits, model metadata, and environment-variable name. | Fixed intent, exact HTTPS origins, fixed GET navigation descriptors, and no redirect policy. |
| Planning side effects | `inspect` and `plan` read text manifests only. | `inspect` and `plan` read text manifests only. |
| Secret boundary | Names only in manifest/plan; secret read immediately before one bounded request; never logged, returned, or inherited by child process. | No secret field, environment read, cookie jar, or browser profile. |
| Untrusted data boundary | Input and output are typed JSON data; output can never execute, render with authority, or select a tool/action. | A URL is data validated before planning; future page content is untrusted and cannot alter policy. |
| Side-effect boundary | One explicit request only; no automatic retry, process execution, file write, or output action. | No process, browser, DNS, network, read/write page operation, or state mutation in M9 v1. |
| Human boundary | Generated content never produces a follow-on action. | A future browsing action needs a separate capability, plan digest, and fresh visible confirmation. |

The threat model assumes that prompt text, AI output, URLs, manifest contents supplied by another party, remote provider responses, webpages, and local environment values can be malformed or hostile. It does not claim to protect a user from an adversary who already controls their Padma project directory, interpreter binary, operating-system account, or trusted HTTPS endpoint; those are separate supply-chain and device-security concerns. The contract reduces what an untrusted input can cause **through Padma**.

Provider-neutral AI workflows must keep instructions and data structurally separate, treat external or retrieved text as data rather than command authority, and validate every response before downstream use. OWASP identifies direct and indirect prompt injection—including hidden content in webpages and documents—as a risk when natural-language instructions and data are processed together; it recommends structured separation and human oversight for high-risk actions.[2] This is why the v1 workflow has no tool-call, code-execution, browser, file, database, deployment, or agent loop capability.

## 5. Required regression and negative-security tests

The following matrix is part of the feature definition of done. The exact diagnostic text may be localized, but test assertions must include the stable code, the denied authority, and the no-side-effect claim where applicable.

| Area | Test condition | Expected result |
|---|---|---|
| AI manifest | A complete valid `padma-ai.toml` with a public HTTPS endpoint and `network:ai`. | `inspect` and `plan` emit deterministic JSON with `network: "disabled"`, `secret.value: "not-read"`, and all declared limits. |
| AI capability | `network:ai` missing or misspelled. | Localized `P1034`; no environment read, child process, DNS, or network. |
| AI parser | Missing/duplicate/unknown section or field; wrong version/adapter/retry policy; invalid integer; invalid model. | `P1050`; no secret or transport access. |
| AI endpoint | HTTP, IP literal, loopback/private name, userinfo, query, fragment, port, whitespace, encoded or traversal form. | `P1050`; rejected before transport construction. |
| AI secret policy | Token literal, unsafe environment name, absent/empty secret at action time. | Invalid manifest is `P1050`; absent/empty runtime secret is `P1051`; plan output never reveals whether the secret exists. |
| AI plan isolation | Environment contains a sentinel secret and test `PATH` contains a failing fake `curl`. | Plan succeeds without reading the sentinel or invoking the fake executable. |
| AI request envelope | Valid bounded JSON request and a mock `curl` response conforming to the protocol. | Exactly one fixed POST is prepared; returned Padma value contains only `output` and safe `meta`. |
| AI child isolation | Mock transport captures args and environment. | No bearer token in args, URL, stderr, output, or inherited environment; config/payload temporary artifacts are cleaned. |
| AI response | Invalid JSON, wrong protocol, non-object output, over-depth/oversize payload, nonzero transport, timeout. | `P1052` for response schema failures or `P1051` for transport failures; no partial output and no retry. |
| AI output safety | Response holds source-looking text, a URL, SQL, HTML, shell syntax, or browser-looking instructions. | Returned only as inert string/map data; no process, file, HTTP, browser, SQL, or renderer action occurs. |
| Browser manifest | Valid v1 exact origins and navigation URLs under `browser:plan`. | Deterministic plan preserves step order, lists sorted origins, and declares every execution channel disabled. |
| Browser capability | `browser:plan` absent, unknown, duplicated, or replaced by a future-looking value. | `P1034` or `P1032`; no URL resolution, network, or process. |
| Browser origin parser | Non-HTTPS scheme, IP literal, port, userinfo, wildcard, suffix look-alike, subdomain, IDN, path, query, fragment, trailing slash/dot. | `P1053`; rejected locally. |
| Browser navigation parser | Origin mismatch, `..`/`.` segment, duplicate slash, encoded/non-ASCII data, query/fragment/userinfo/port, empty or excess steps. | `P1054`; the invalid raw value is redacted from the diagnostic. |
| Browser plan isolation | Test environment includes secrets and `PATH` holds fake browser/network executables. | `inspect` and `plan` neither inspect environment nor invoke an executable, DNS resolver, browser, or network. |
| Browser action boundary | Attempted nonexistent `padma browser navigate`, `login`, `post`, `submit`, `download`, or `run` command. | Usage failure or `P1055`; there is no fallback to an external tool. |
| Locale | The same denied/invalid AI and browser cases in Bengali, English, and mixed projects. | Stable codes remain identical; user-facing message/hint follows established locale selection. |
| Compatibility | Existing `ai.request`, existing examples, package/deploy/gui/android/render commands, LSP tests, and installer. | No behavior change except accepted `[capabilities].browser = ["plan"]` and new documented commands. |

## 6. Implementation sequence

Implementation should proceed in narrow, testable commits. No source feature may be advertised as working until its own tests, full repository gate, and documentation links pass.

| Commit order | Change | Security completion criterion |
|---|---|---|
| 1 | Add bilingual `P1050`–`P1055` diagnostics, `browser` capability parsing restricted to `plan`, shared strict scalar/list validators, and unit tests. | New codes do not conflict with P1001–P1049; no command behavior changes beyond parsing. |
| 2 | Add `AiWorkflowManifest`, strict `padma-ai.toml` parser, capability helper, deterministic `ai inspect|plan` builders, usage text, and parser/no-side-effect tests. | Planning has no `curl`, environment read, DNS, or network path. |
| 3 | Add `ai.workflow` JSON validation and a fixed, bounded `json-http-v1` transport with test-only mocked child process. | Secret is absent from child args/environment/output; exactly one request; no retries; protocol response is validated as data. |
| 4 | Add `BrowserPlanManifest`, exact-origin and URL validators, deterministic `browser inspect|plan` builders, usage text, and exhaustive allowlist/no-side-effect tests. | No browser adapter or network API is linked, invoked, or documented as active. |
| 5 | Add user guides `AI-WORKFLOW.md` and `BROWSER-PLANNING.md`, safe examples, documentation-index links, and updated capability security table. | Guides state prerequisites, output, data-handling limits, and every explicit non-goal. |
| 6 | Run `bash scripts/verify-repository.sh`, execute the dedicated plan-isolation tests, inspect the final diff for secrets/unsafe sample values, commit each feature separately, and push only after success. | Clean worktree and reproducible green gate before publication. |

The documentation and roadmap changes in this design commit are intentionally separate from the runtime-feature commits. They do not enable external AI access or browser operation. The next implementation session should start with commit 1, not jump directly to a transport or browser executable.

## 7. Security research anchors

The browser component is a planning-only contract in this increment. Its future navigation adapter must use only explicit HTTPS allowlist entries, reject every other scheme, and not automatically follow redirects. OWASP identifies URL mishandling as an SSRF enabler and recommends strict allowlisting for identified/trusted destinations; it also recommends disabling automatic redirect following to prevent validation bypasses.[1]

The present planning command does not resolve DNS, connect to a host, inspect a page, read browser state, or exchange credentials. Any future navigation adapter must separately address DNS rebinding, private/local address rejection, redirect-by-redirect allowlist validation, and a transport that cannot be redirected to a disallowed target.

## References

[1]: https://cheatsheetseries.owasp.org/cheatsheets/Server_Side_Request_Forgery_Prevention_Cheat_Sheet.html "OWASP Server-Side Request Forgery Prevention Cheat Sheet"
[2]: https://cheatsheetseries.owasp.org/cheatsheets/LLM_Prompt_Injection_Prevention_Cheat_Sheet.html "OWASP LLM Prompt Injection Prevention Cheat Sheet"
[3]: https://genai.owasp.org/llmrisk/llm052025-improper-output-handling/ "OWASP LLM05:2025 Improper Output Handling"
