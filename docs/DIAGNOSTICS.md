# Padma diagnostics reference

Padma diagnostics use a stable `P` code, a source position, a localized title, and a practical hint. Bengali source defaults to Bengali diagnostics; English source defaults to English diagnostics. Add `# padma:locale=bn` or `# padma:locale=en` to override automatic detection.

`padma check file.pd` is intended for editing and CI. It recovers at statement boundaries where safe, so it can report multiple independent syntax errors in a single run. It also performs non-executing static checks where an error is provable from source alone: literal `number / 0` reports `P1011`, while a stable builtin or top-level function call with a provably wrong argument count reports `P1009`. It does not guess at imported, dynamic, or collection method calls. Running a file stops at the first runtime error because continuing after a failed side effect could be unsafe.

| Code | Category | Meaning |
|---|---|---|
| P1001 | Lexer | Unsupported character. |
| P1002 | Lexer | Unterminated text string. |
| P1003 | Parser | Missing expected syntax, such as a name, delimiter, or new line. |
| P1004 | Parser | Invalid statement start. |
| P1007 | Runtime | Variable is not declared. |
| P1008 | Runtime | Function does not exist. |
| P1009 | Runtime | Function was called with the wrong number of arguments. |
| P1010 | Runtime | Operation received incompatible value types. |
| P1011 | Runtime | Division by zero. |
| P1012 | Safety | Loop exceeded the configured safety limit. |
| P1013 | Runtime | Interactive input could not be read. |
| P1014 | Safety | File output path is not allowed. |
| P1015 | Runtime | File could not be written. |
| P1016 | Safety | URL is not permitted. |
| P1017 | Safety | External process is not permitted. |
| P1018 | Runtime | External process could not be started. |
| P1019 | Runtime | External process returned failure. |
| P1020 | Type | Map key is not text. |
| P1021 | Runtime | Requested map key does not exist. |
| P1022 | Safety | Module path is unsafe. |
| P1023 | Runtime | Module file could not be read. |
| P1024 | Safety | Circular module import detected. |
| P1025 | Compile/runtime | Imported module has an error; the diagnostic points to that module's source. |
| P1026 | Type | Collection index is not a non-negative whole number. |
| P1027 | Runtime | Collection index is outside the list bounds. |
| P1028 | Runtime | Safe file path could not be read. |
| P1029 | Runtime | JSON text could not be parsed, or a Padma value could not be represented as JSON. |
| P1030 | Safety/type | URL is not a supported absolute HTTP or HTTPS URL. |
| P1031 | Runtime | `text.format` placeholder is missing or malformed. |
| P1032 | Project | Project manifest or entrypoint is invalid or cannot be read. |
| P1033 | Project safety | Third-party dependencies are not supported until a trusted registry policy exists. |
| P1034 | Capability safety | A manifest-run project attempted a sensitive builtin without its explicit capability grant. |
| P1035 | Bridge safety | Requested bridge runtime is not supported. |
| P1036 | Bridge safety | Bridge script path is unsafe, missing, outside the project root, or has the wrong extension. |
| P1037 | Bridge safety | Bridge JSON input or output exceeded the 256 KiB limit. |
| P1038 | Bridge runtime | Bridge process could not start or completed with a non-zero exit status. |
| P1039 | Bridge safety | Bridge process exceeded the 10-second execution limit. |
| P1040 | Bridge runtime | Bridge standard output was not one valid JSON-compatible value. |
| P1050 | AI workflow safety | `padma-ai.toml` or the structured workflow request is missing, unsafe, malformed, or outside the strict v1 contract. |
| P1051 | AI workflow transport | The one-shot reviewed transport could not obtain a usable credential, start, complete, or stay within its configured bound. Secret values are never included in this diagnostic. |
| P1052 | AI workflow response safety | The provider response is missing, oversized, malformed, too deeply nested, or outside the strict v1 structured-response schema. |
| P1053 | Browser planning safety | `padma-browser.toml` is missing, unsafe, malformed, or outside the strict v1 exact-HTTPS-origin policy. Rejected raw origins and URLs are not echoed. |
| P1054 | Browser navigation policy | A fixed navigation URL does not match a reviewed exact HTTPS origin or violates the simple-path policy. The command remains local and performs no navigation. |
| P1055 | Browser execution boundary | A browser execution path is unavailable or prohibited in this Padma version. Only local `padma browser inspect` and `padma browser plan` commands are supported. |
| P1056 | AI tool planning safety | `padma-ai-tools.toml` is missing, malformed, unsafe, or outside the strict local v1 planning contract. Unsupported raw tool names are not echoed. |
| P1057 | AI tool and agent execution boundary | A tool or agent execution path is unavailable or prohibited in this Padma version. Only local `padma ai tools inspect` and `padma ai tools plan` commands are supported. |
| P1058 | AI training planning safety | `padma-ai-training.toml` is missing, malformed, unsafe, outside resource limits, or requests an execution mode. Unsafe raw paths are not echoed. |
| P1059 | AI training execution boundary | A training execution path is unavailable or prohibited in this Padma version. Only local `padma ai training inspect` and `padma ai training plan` commands are supported. |
| P1060 | Browser confirmation planning safety | `padma-browser-confirm.toml` is missing, malformed, unsafe, uses an unsupported mode, has an invalid/mismatched digest, or references an unavailable reviewed URL. Raw sensitive values are not echoed. |
| P1061 | Browser confirmation and navigation boundary | A browser confirmation or navigation execution path is unavailable or prohibited in this Padma version. Only local `padma browser confirm inspect` and `padma browser confirm plan` commands are supported. |
| P1062 | Android Browser Handoff safety | A handoff request is unsafe, unsupported, not freshly confirmed, or no longer bound to one reviewed browser-plan destination. Raw URLs and approval input are not echoed. |
| P1063 | Android Browser Handoff runtime | The fixed local `termux-open-url` opener was unavailable or failed. Padma does not retry, use another executable, or fall back to a remote browser service. |
| P1064 | Android Browser Handoff audit safety | The opt-in local audit manifest, path, existing JSONL record, or bounded atomic write is unsafe or failed. Raw URLs, query strings, approval input, cookies, credentials, profiles, page data, and opener output are not written or echoed. |
| P1065 | Browser interaction draft safety | `padma-browser-draft.toml` is missing, malformed, unsafe, uses an unsupported mode/action, has an invalid/mismatched reviewed plan binding, or uses unsafe attachment metadata. Raw sensitive values are not echoed. |
| P1066 | Browser interaction draft execution boundary | A browser draft execution path is unavailable or prohibited in this Padma version. Only local `padma browser draft inspect` and `padma browser draft plan` commands are supported. |
| P1067 | Visible browser takeover safety | `padma-browser-takeover.toml` is missing, malformed, unsafe, uses an unsupported mode/action, has an invalid/mismatched reviewed plan binding, or includes a browser-control/user-decision field. Raw sensitive values are not echoed. |
| P1068 | Visible browser takeover execution boundary | A browser takeover execution path is unavailable or prohibited in this Padma version. Only local `padma browser takeover inspect` and `padma browser takeover plan` commands are supported. |
| P1069 | Structured data table safety | A CSV/TSV/JSON table is malformed, exceeds bounded data policy, has an unsafe schema/cell shape, or cannot be handled safely. Raw table source and file content are not echoed. |
| P1070 | Filesystem productivity safety | A filesystem productivity input, source, directory entry, search, or dry-run plan is unsafe, malformed, over bounded limits, symlinked, non-regular, or otherwise unavailable under the project-root policy. Raw content and paths outside the project root are not echoed. |
| P1071 | Local reporting safety | A report title, rendered output, or Markdown export path is unsafe, malformed, exceeds bounds, uses raw HTML delimiters, a symlinked component, an invalid suffix, or violates the project-root report policy. Raw source table content and paths outside the project root are not echoed. |
| P1072 | Local profile safety | A profile/schema is malformed, exceeds bounded field/key/text policy, has an unknown/missing field, uses an unsupported scalar type, mismatched value/default, nested value, or unsafe rule. Raw profile values are not echoed. |
| P1073 | Local client-document safety | A quote/invoice-draft map is malformed, has a missing/unknown/unsafe field, invalid type/currency/amount/deliverable/date/text, raw HTML delimiter, oversized output, or invalid local Markdown export path. Draft values and paths outside the project root are not echoed. |
| P1074 | Local record safety | An attendance, expense, or inventory table has an invalid kind/exact header/required text/date/status/currency/amount/quantity/duplicate identity/raw markup or record-specific bound. Record values and paths outside the project root are not echoed. |
| P1075 | Local scope-of-work safety | A scope-of-work map has missing/unknown fields, invalid text/list/revision bound, duplicate item, raw markup, URL/contact delimiter, oversized output, or unsafe local Markdown export path. Draft values and paths outside the project root are not echoed. |
| P1076 | Local delivery-checklist safety | A delivery-checklist map has missing/unknown fields, invalid text/list bound, duplicate item, raw markup, URL/contact delimiter, oversized output, or unsafe local Markdown export path. Draft values and paths outside the project root are not echoed. |
| P1077 | Local portfolio case-study safety | A case-study map has missing/unknown/private-data fields, invalid text/link/list, raw markup, URL/contact injection, duplicate/unsafe public link, unverified income/guarantee claim, oversized output, or unsafe export path. Draft values are not echoed. |
| P1078 | Visible handoff safety | A manual handoff map has missing/unknown fields, invalid message/destination/attachment/review text, duplicate item, raw markup, URL/contact injection, or action-oriented field. It never sends, uploads, submits, signs, pays, or opens a browser. |
| P1079 | Local reconciliation safety | Two local tables lack a shared safe key, contain duplicate/unsafe key values, or reconciliation output exceeds local policy. The summary redacts row identifiers and disables contact/upload/submission/payment/network/process actions. |
| P1080 | Local attachment-review safety | A local attachment-review map has missing/unknown/unsafe fields, duplicate labels or paths, URL/contact/raw-markup text, unsafe/non-regular/symlinked attachment source, or oversized output. It only computes local checksums and never sends, uploads, submits, signs, pays, opens a browser, accesses an account/network, or starts a process. |
| P1081 | Local delivery-package safety | A local delivery-package map has missing/unknown/unsafe fields, duplicate labels/paths/review steps, URL/contact/raw-markup text, unsafe/non-regular/symlinked source, or oversized output. It only computes local checksums and renders a manual review artifact; file copy, PDF rendering, send/upload/submission/payment/browser/account/network/process action are unavailable. |
| P1082 | Local template safety | A proposal, brief, or copy-only message-template map has a missing/unknown field, unsupported type, invalid/duplicate/empty list, raw-markup/URL/contact text, income/client-acceptance guarantee, or oversized output. It only renders explicit local draft content; send/post/upload/submission/payment/browser/account/network/process actions are unavailable. |

## Example

```text
ত্রুটি[P1003]: এখানে `variable name` প্রত্যাশিত ছিল
  --> broken.pd:1:5
   |
1 | ধরি = ১
   |     ^ এই স্থানে
   = পরামর্শ: আগের statement ও বন্ধনীগুলো পরীক্ষা করুন।
```

Consumers that parse diagnostics should use the `P` code, not the localized message text.

## JSON diagnostics for editors and CI

Use `padma check --json file.pd` (or `padma check file.pd --json`) when an editor, CI workflow, or script needs machine-readable output. The command writes one JSON object to standard output and still exits with `0` for a clean file and `1` when diagnostics are reported.

```json
{
  "status": "error",
  "path": "broken.pd",
  "diagnostics": [
    {
      "code": "P1002",
      "message": "...",
      "hint": null,
      "locale": "en",
      "path": "broken.pd",
      "range": {
        "start": { "line": 1, "column": 5 },
        "end": { "line": 1, "column": 6 }
      },
      "source_line": "let = 3"
    }
  ]
}
```

The stable fields are `status`, `path`, `diagnostics[].code`, `diagnostics[].message`, `diagnostics[].hint`, `diagnostics[].locale`, `diagnostics[].path`, and one-based `range` coordinates. Later releases may add fields; consumers should ignore unknown fields.
