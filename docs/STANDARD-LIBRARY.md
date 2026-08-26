# Padma standard library

The standard library uses explicit module-style function names such as `text.trim(value)`. These are built into the Padma runtime; no import is needed. Names are currently written in English so that programs remain interoperable and documentation stays concise. Bangla variables, statements, and comments work normally around every API.

## Text

| API | Result |
|---|---|
| `text.len(text)` | Unicode character count. |
| `text.trim(text)` | Text without leading/trailing whitespace. |
| `text.upper(text)` / `text.lower(text)` | Unicode case conversion. |
| `text.contains(text, query)` | Boolean substring check. |
| `text.replace(text, search, replacement)` | Replaces all matching text. |
| `text.split(text, separator)` | List of text pieces. |
| `text.join(items, separator)` | Joins a list of text values. |

## Math and time

| API | Result and boundary |
|---|---|
| `math.abs`, `math.round`, `math.floor`, `math.ceil` | One finite number in, one number out. |
| `math.min(a, ...)` / `math.max(a, ...)` | Minimum or maximum of one or more finite numbers. |
| `time.now()` | Unix timestamp in seconds as a number. |
| `time.sleep(seconds)` | Waits from `0` to `60` seconds and returns `none`. |

`time.sleep` is deliberately bounded so accidental programs cannot hold a Termux session indefinitely with a single call. It does not make a scheduling guarantee; automation scheduling is a future capability-layer feature.

## Files

| API | Result and safety rule |
|---|---|
| `file.write(path, text)` | Writes text to a safe relative path and returns `true`. |
| `file.read(path)` | Reads UTF-8 text from a safe relative path. |
| `file.exists(path)` | Reports whether a safe relative path is an existing regular file. |

Paths cannot be absolute and cannot contain `..`. The `@downloads/` alias remains available for Termux download-folder output. File reads and writes therefore stay inside the process working directory or the explicit download alias.

## JSON and URLs

| API | Result and boundary |
|---|---|
| `json.parse(text)` | Converts JSON object/array/text/number/boolean/null to Padma map/list/text/number/boolean/`none`. |
| `json.stringify(value)` | Produces compact deterministic JSON from Padma values. Map keys are emitted in text order. |
| `url.is_valid(text)` | Returns whether text is a supported HTTP or HTTPS URL. |
| `url.parse(text)` | Returns a map with `url`, `scheme`, `host`, `path`, `query`, `fragment`, and `port`. Missing optional parts are `none`. |

`url.parse` deliberately accepts only absolute `http://` and `https://` URLs, rejects whitespace and embedded credentials, and does not perform any network request. It is an inspection utility; `http.get` remains the network API and applies its own network safeguards.

## Filesystem productivity

| API | Result and boundary |
|---|---|
| `fs.list(path, depth)` | Lists bounded project-local regular file/directory entries from depth `0` to `4`. Requires `filesystem:read` in project mode. |
| `fs.checksum(path)` | Returns `sha256:<hex>` for one bounded project-local regular file. Requires `filesystem:read` in project mode. |
| `fs.search_text(path, query, limit)` | Returns bounded UTF-8 matching `{line, text}` rows. Requires `filesystem:read` in project mode. |
| `fs.copy_plan(source, destination)` / `fs.move_plan(source, destination)` | Returns a deterministic disabled action descriptor after safe project-local source/destination validation. Requires `filesystem:read` in project mode. |
| `fs.archive_plan(source, destination)` | Returns the equivalent disabled archive descriptor; destination must end in `.zip`. Requires `filesystem:read` in project mode. |

Filesystem productivity APIs are project-only, never start a shell or child process, and never mutate a file. Sources are limited to 1 MiB regular non-symlink files; list output is limited to 256 entries and depth `4`; search query/match/text-line bounds apply. Absolute paths, traversal, `@downloads`, symlinks, non-regular sources, binary text search, and oversized content are rejected. See [`FILESYSTEM-PRODUCTIVITY.md`](FILESYSTEM-PRODUCTIVITY.md) for the complete contract.

## Structured data tables

| API | Result and boundary |
|---|---|
| `table.read(path, format)` | Reads bounded project-local UTF-8 `csv`, `tsv`, or object-row `json` into a validated table value. Requires `filesystem:read` in project mode. |
| `table.headers(table)` / `table.rows(table)` | Returns validated headers or text-cell row maps. |
| `table.filter_equal(table, column, text)` | Returns rows whose named text cell exactly matches `text`. |
| `table.select(table, columns)` | Returns a table containing declared, unique selected columns in the requested order. |
| `table.count_by(table, column)` | Returns deterministic text-cell counts as a map. |
| `table.write_csv(path, table)` | Writes a bounded deterministic project-local `.csv` file and returns `true`. Requires `filesystem:write` in project mode. |

Tables are limited to 1 MiB, 4,096 data rows, 64 columns, 128-byte unique headers, and 4,096-byte single-line text cells. CSV supports doubled quotes in quoted single-line cells; JSON must be an array of object rows with scalar cells. Absolute paths, traversal, `@downloads`, nested JSON cells, malformed rows, duplicate headers, and oversized data are rejected. See [`STRUCTURED-DATA.md`](STRUCTURED-DATA.md) for the complete contract.

## Local reporting

| API | Result and boundary |
|---|---|
| `report.markdown(title, table)` | Renders a bounded escaped Markdown table/report string from an existing validated table. No file/network capability is required. |
| `report.summary(title, table)` | Returns deterministic report metadata: title, table format, row count, column count, and column list. No file/network capability is required. |
| `report.write_markdown(path, title, table)` | Writes a bounded project-local non-symlink `.md` report and returns `true`. Requires `filesystem:write` in project mode. |

Report titles are single-line bounded text and reject raw `<`/`>` HTML delimiters. Table cells and headers are escaped before Markdown emission. Markdown export must remain inside the project root, use an existing non-symlink parent, end in `.md`, and stay under 1 MiB. It never uploads, emails, publishes, renders HTML/JavaScript, starts a job, creates a payment, or performs account action. See [`LOCAL-REPORTING.md`](LOCAL-REPORTING.md) for the complete contract.

## Local profiles

| API | Result and boundary |
|---|---|
| `profile.validate(profile, schema)` | Validates a bounded in-memory profile map against explicit scalar rules and returns declared defaults only where allowed. No capability is required. |
| `profile.summary(profile, schema)` | Returns redacted validation metadata and fixed disabled-action markers without returning profile values. No capability is required. |

Profile schemas accept only bounded key names and scalar `text`, `number`, `boolean`, or `null` rules, with optional boolean `required` or matching scalar `default`. No nested value/list/map profile data is accepted. Use `json.parse(file.read("data/profile.json"))` for a project-local JSON file; only `file.read` needs `filesystem:read`. These APIs never use a profile to trigger network, account, browser, device, process, game, payment, or output execution. See [`LOCAL-PROFILES.md`](LOCAL-PROFILES.md) for the full contract.

## Local records

| API | Result and boundary |
|---|---|
| `record.validate(kind, table)` | Validates one bounded `attendance`, `expense`, or `inventory` table with an exact ordered schema and returns the validated table. No capability is required. |
| `record.summary(kind, table)` | Returns redacted record counts/totals plus fixed disabled-action markers; it does not return row labels or notes. No capability is required. |

Attendance requires unique `(date, student)` entries and status `present`, `absent`, or `late`. Expense requires a real date, non-negative decimal amount with at most two fractional digits, one three-uppercase-letter currency, and bounded note. Inventory requires unique item names plus non-negative whole-number quantity/reorder level. Input is an existing table value, so `table.read` retains its own project-local `filesystem:read` boundary; use `report.write_markdown` separately for a reviewed local output. These APIs do not use account, cloud, network, payment, process, or device authority. See [`LOCAL-RECORDS.md`](LOCAL-RECORDS.md) for exact schemas, limits, and Termux example.

## Local client documents

| API | Result and boundary |
|---|---|
| `client.document_markdown(draft)` | Renders a deterministic escaped local Markdown `quote` or `invoice-draft` from one strict in-memory map. No capability is required. |
| `client.document_summary(draft)` | Returns only document type/count/presence metadata and fixed disabled-action markers; it does not return draft values. No capability is required. |
| `client.write_document(path, draft)` | Writes a bounded project-local non-symlink `.md` document and returns `true`. Requires `filesystem:write` in project mode. |

Drafts require `documentType`, `clientName`, `projectTitle`, three-letter uppercase `currency`, finite non-negative `amount`, and a bounded unique `deliverables` text list. Optional `reference`, `validUntil`, and `notes` are bounded single-line text. Unknown fields—including URLs, recipient/contact, account, payout, or authorization fields—are rejected. Raw `<`/`>` delimiters and control characters are rejected; accepted Markdown-sensitive characters are escaped. Documents cannot write to absolute/traversal/`@downloads`/symlink paths. This toolkit does not contact a client, make a payment, sign a contract, submit to a marketplace, use a network/browser/account, or start a process. See [`CLIENT-DOCUMENTS.md`](CLIENT-DOCUMENTS.md) for the complete contract and Termux example.

## Local scope-of-work

| API | Result and boundary |
|---|---|
| `client.scope_markdown(draft)` | Escaped local scope-of-work Markdown from one strict map. No capability is required. |
| `client.scope_summary(draft)` | Redacted counts and fixed disabled-action markers only. No capability is required. |
| `client.write_scope(path, draft)` | Writes one bounded project-local non-symlink `.md` review draft. Requires `filesystem:write` in project mode. |

Required fields are client/project labels, unique scope/exclusion lists, a `0`–`10` whole revision limit, and a non-contractual delivery target label. URL/contact/raw-HTML delimiters and unknown payment, account, recipient, authorization, platform, or contract fields are rejected. The APIs do not contact a client, sign, submit, pay, use a marketplace/network/browser/account, or start a process. See [`LOCAL-SCOPE-OF-WORK.md`](LOCAL-SCOPE-OF-WORK.md).

## Local delivery checklists

| API | Result and boundary |
|---|---|
| `client.delivery_markdown(draft)` | Escaped local delivery-checklist Markdown from one strict map. No capability is required. |
| `client.delivery_summary(draft)` | Redacted counts and fixed disabled upload/submission/payment markers only. No capability is required. |
| `client.write_delivery_checklist(path, draft)` | Writes one bounded project-local non-symlink `.md` review draft. Requires `filesystem:write` in project mode. |

Required fields are project title plus unique deliverable, review, and handover lists. URL/contact/raw-HTML delimiters and unknown recipient, payment, account, authorization, platform, upload, delivery-submission, or contract fields are rejected. The APIs cannot contact, upload/download, submit delivery, sign, pay, use a marketplace/network/browser/account, or start a process. See [`LOCAL-DELIVERY-CHECKLISTS.md`](LOCAL-DELIVERY-CHECKLISTS.md).

## Local portfolio case studies and visible handoff

| API | Result and boundary |
|---|---|
| `client.case_study_markdown(draft)` / `client.case_study_summary(draft)` | Escaped local public case-study Markdown or redacted counts. No capability is required. |
| `client.write_case_study(path, draft)` | Writes one bounded project-local non-symlink `.md` draft. Requires `filesystem:write`. |
| `client.visible_handoff_markdown(draft)` / `client.visible_handoff_summary(draft)` | Produces a user-mediated message/attachment/destination review artifact and redacted disabled-action summary. No capability is required. |

Portfolio drafts reject private client/contact/account/payment fields, unsafe links, raw markup, and income/guarantee claims. Visible handoff drafts accept only a destination label, a message draft, attachment labels, and review steps; they cannot send, post, upload/download, submit, sign, pay, open a browser, access an account, or use a network/process. See [`LOCAL-PORTFOLIO-HANDOFF.md`](LOCAL-PORTFOLIO-HANDOFF.md).

## Local client-data reconciliation

| API | Result and boundary |
|---|---|
| `client.reconcile_summary(left, right, key)` | Returns only local row/match/mismatch counts and deterministic table checksums. |
| `client.reconcile_markdown(title, left, right, key)` | Renders a redacted local reconciliation Markdown artifact. |
| `client.write_reconciliation(path, title, left, right, key)` | Writes one project-local non-symlink `.md` artifact; requires `filesystem:write`. |

The input tables use the existing table contract and must have one shared key header with unique safe values per table. Reconciliation does not expose identifiers in its summary, contact a client, upload, submit delivery, process payment, use a browser/account/network, or start a process. See [`LOCAL-CLIENT-RECONCILIATION.md`](LOCAL-CLIENT-RECONCILIATION.md).

## Local attachment-review manifests

| API | Result and boundary |
|---|---|
| `client.attachment_review_summary(draft)` | Reads declared project-local attachments and returns only count/checksum-count plus disabled-action markers. Requires `filesystem:read` in project mode. |
| `client.attachment_review_markdown(draft)` | Renders a checksum and byte-count review manifest for declared project-local attachments. Requires `filesystem:read` in project mode. |
| `client.write_attachment_review(path, draft)` | Writes one project-local non-symlink `.md` review artifact; requires both `filesystem:read` and `filesystem:write`. |

The strict draft has `destinationLabel`, `ownershipLabel`, and a non-empty bounded `attachments` list. Each attachment has exactly a unique `path` and unique `label`; paths are read only to produce a SHA-256 checksum and byte count. Absolute/traversal/`@downloads`/symlink/non-regular paths, HTML/URL/contact text, unknown action fields, and duplicate labels/paths are rejected. The summary deliberately omits label, path, and checksum data. This toolkit cannot send, upload, submit, sign, pay, open a browser, access an account, use a network, or start a process. See [`LOCAL-ATTACHMENT-REVIEW.md`](LOCAL-ATTACHMENT-REVIEW.md).

## Verifiable local delivery packages

| API | Result and boundary |
|---|---|
| `client.delivery_package_summary(draft)` | Reads declared project-local files and returns redacted file/checksum/review-step counts plus fixed disabled-action markers. Requires `filesystem:read` in project mode. |
| `client.delivery_package_markdown(draft)` | Renders a deterministic manual folder/checksum/review-step Markdown package. Requires `filesystem:read` in project mode. |
| `client.write_delivery_package(path, draft)` | Writes one project-local non-symlink `.md` manual-submission draft; requires `filesystem:read` and `filesystem:write`. |

The strict draft has `packageLabel`, `destinationLabel`, `ownershipLabel`, unique `files` entries, and unique `reviewSteps`. Each file map has exactly a project-relative `path` and a human `label`; bytes are read only for SHA-256 and byte-count metadata. The rendered manual folder layout is an instruction, not a directory/file-copy action. PDF rendering, file copying, sending, uploading, delivery submission, signing, payment, browser/account/network access, and process execution are not provided. See [`LOCAL-DELIVERY-PACKAGE.md`](LOCAL-DELIVERY-PACKAGE.md).

## Local proposal, brief, and message templates

| API | Result and boundary |
|---|---|
| `client.template_summary(draft)` | Returns the draft type, redacted counts/presence state, copy-only review marker, and fixed disabled-action markers. No capability is required. |
| `client.template_markdown(draft)` | Renders explicit input as an escaped proposal, brief, or copy-only message Markdown draft. No capability is required. |
| `client.write_template(path, draft)` | Writes one project-local non-symlink `.md` review draft. Requires `filesystem:write` in project mode. |

The strict draft accepts `templateType` (`proposal`, `brief`, or `message-template`), `title`, `overview`, unique `skills`, `requirements`, `deliverables`, and `reviewSteps`, with optional `callToActionLabel` and `notes`. The message type creates text to review and manually copy; it has no recipient, platform, or send action. Unknown recipient/contact/account/platform/authorization/payment/upload/send fields, raw HTML, URL/contact delimiters, duplicates, empty lists, and income or client-acceptance guarantees are rejected. This toolkit cannot send/post/upload/submit/sign/pay, or access browser/account/network/process authority. See [`LOCAL-CLIENT-TEMPLATES.md`](LOCAL-CLIENT-TEMPLATES.md).

## Local quantum circuit planning

| API | Result and boundary |
|---|---|
| `quantum.circuit_summary(circuit)` | Returns redacted qubit/operation/measurement counts, a bounded-local simulator availability marker, and fixed disabled provider/QPU/credential/network/process markers. No capability is required. |
| `quantum.openqasm3(circuit)` | Generates a deterministic bounded OpenQASM 3.0 text program in memory. No capability is required. |
| `quantum.write_openqasm3(path, circuit)` | Writes one project-local non-symlink `.qasm` plan. Requires `filesystem:write` in project mode. |
| `quantum.simulate_probabilities(circuit)` | Evaluates a maximum 12-qubit state vector locally and returns a deterministic, full computational-basis probability map. No capability is required. |
| `quantum.expectation_pauli(circuit, observable)` | Evaluates one fixed-length `I`/`X`/`Y`/`Z` Pauli product against the bounded local state vector and returns a deterministic real expectation number. No capability is required. |

The strict map accepts only a `1..20` qubit count, supported gate maps (`h`, `x`, `z`, `s`, `t`, `rx`, `ry`, `rz`, `cx`, `superposition`, `entangle-linear`), and a complete unique measurement mapping. Each rotation needs one finite numeric `angle` in radians with absolute value at most `1_000_000`; no symbolic parameter exists yet. The simulator, Pauli observable evaluator, seeded sampler, and Hamiltonian evaluator further limit themselves to 12 qubits. `expectation_pauli` accepts exactly one ASCII Pauli string with one `I`, `X`, `Y`, or `Z` character per qubit; its leftmost character applies to the highest qubit index. `sample_counts(circuit, {"shots": 1..100000, "seed": 0..2^53-1})` returns reproducible local sparse counts with a required explicit seed. `expectation_hamiltonian(circuit, {"terms": [{"coefficient": finite-nonzero-real, "pauli": full-register-I/X/Y/Z}]})` returns deterministic energy for at most 64 unique ordered terms. There is no hidden seed, collapsed-state reuse, noise model, symbolic coefficient, optimiser, QPU/provider/device selection, cloud transport, credential use, QAOA/QML/Grover implementation, noise mitigation, or real-device result data. See [`LOCAL-QUANTUM-PLANNING.md`](LOCAL-QUANTUM-PLANNING.md), [`LOCAL-QUANTUM-SIMULATOR.md`](LOCAL-QUANTUM-SIMULATOR.md), [`LOCAL-QUANTUM-OBSERVABLES.md`](LOCAL-QUANTUM-OBSERVABLES.md), [`LOCAL-QUANTUM-ROTATIONS.md`](LOCAL-QUANTUM-ROTATIONS.md), [`LOCAL-QUANTUM-SAMPLING.md`](LOCAL-QUANTUM-SAMPLING.md), and [`LOCAL-QUANTUM-HAMILTONIANS.md`](LOCAL-QUANTUM-HAMILTONIANS.md).

## Interoperability bridge

| API | Result and boundary |
|---|---|
| `bridge.call("python", script_path, data)` | Runs a reviewed project-local `.py` file through the fixed `python` executable and decodes its one JSON output value. Requires `process = ["python"]` in project mode. |
| `bridge.call("javascript", script_path, data)` | Runs a reviewed project-local `.js` file through the fixed `node` executable and decodes its one JSON output value. Requires `process = ["node"]` in project mode. |

`data` may contain only JSON-compatible Padma values: finite numbers, text, `true`/`false`, `none`, lists, and maps with text keys. Padma writes exactly one UTF-8 JSON document to standard input and accepts exactly one UTF-8 JSON value from standard output. Returned text is data, never foreign source code to evaluate.

The bridge accepts no arbitrary executable or argument string. Its script path must be a `.py` or `.js` relative file inside the canonical project root, and Padma invokes only the fixed runtime plus that verified path with a cleared child environment. Input and output are each limited to 256 KiB; execution is limited to 10 seconds. A bridge program still has the operating-system authority of the Termux or desktop user, so inspect its source and dependencies before granting `process` capability. See [`INTEROPERABILITY.md`](INTEROPERABILITY.md) for the full versioned contract.

## Paths, formatting, and randomness

| API | Result and boundary |
|---|---|
| `path.basename(path)` | Last component of a relative safe path. |
| `path.extension(path)` | File extension without the dot, or empty text. |
| `path.join(part, ...)` | Combines one or more safe relative path components using `/`. |
| `text.format(template, values)` | Replaces `{key}` placeholders with values from a text-key map. Use `{{` for a literal `{`. |
| `random.int(start, end)` | Non-cryptographic whole number from `start` inclusive to `end` exclusive. The span is limited to one billion. |
| `random.pick(items)` | Non-cryptographic selection from a non-empty list containing at most one million items. |

Path helpers reject absolute paths, `..`, and the special `@downloads` alias because they describe paths rather than performing output. `text.format` rejects an absent or malformed placeholder instead of leaving a misleading marker in user-facing text.

> `random.int` and `random.pick` are **not cryptographically secure**. Do not use them for passwords, tokens, authentication, lotteries, gambling, or security-sensitive decisions. Cryptographic random generation belongs in a future vetted capability layer.

## Errors

Wrong argument counts use `P1009`; incompatible values use `P1010`; unsafe paths use `P1014`; unreadable files use `P1028`; malformed JSON uses `P1029`; unsupported URLs use `P1030`; invalid format placeholders use `P1031`; and over-limit sleeps or random bounds use `P1012`. Malformed or unsafe structured tables use `P1069`; unsafe filesystem productivity input/plan state uses `P1070`; unsafe local reporting policy uses `P1071`; unsafe local profile policy uses `P1072`; unsafe client-document drafts use `P1073`; unsafe local record policy uses `P1074`; unsafe local scope-of-work drafts use `P1075`; unsafe local delivery-checklist drafts use `P1076`; unsafe portfolio case-study data uses `P1077`; unsafe visible handoff data uses `P1078`; unsafe local reconciliation data uses `P1079`; unsafe local attachment-review data uses `P1080`; unsafe local delivery-package data uses `P1081`; unsafe local template data uses `P1082`; unsafe local quantum-circuit data uses `P1083`; unsafe local quantum simulator limits or state invariants use `P1084`; unsafe local Pauli observable data or invariants use `P1085`. Bridge failures use `P1035` through `P1040`. In manifest-run projects, undeclared sensitive operations use `P1034`; see [`PROJECTS.md`](PROJECTS.md) for capability grants and [`DIAGNOSTICS.md`](DIAGNOSTICS.md) for localized messages and stable code meanings.
