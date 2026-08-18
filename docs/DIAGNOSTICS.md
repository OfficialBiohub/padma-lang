# Padma diagnostics reference

Padma diagnostics use a stable `P` code, a source position, a localized title, and a practical hint. Bengali source defaults to Bengali diagnostics; English source defaults to English diagnostics. Add `# padma:locale=bn` or `# padma:locale=en` to override automatic detection.

`padma check file.pd` is intended for editing and CI. It recovers at statement boundaries where safe, so it can report multiple independent syntax errors in a single run. It also performs non-executing static checks where an error is provable from source alone; the first rule reports literal `number / 0` as `P1011`. Running a file stops at the first runtime error because continuing after a failed side effect could be unsafe.

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
