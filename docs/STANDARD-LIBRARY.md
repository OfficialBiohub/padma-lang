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

Wrong argument counts use `P1009`; incompatible values use `P1010`; unsafe paths use `P1014`; unreadable files use `P1028`; malformed JSON uses `P1029`; unsupported URLs use `P1030`; invalid format placeholders use `P1031`; and over-limit sleeps or random bounds use `P1012`. Bridge failures use `P1035` through `P1040`. In manifest-run projects, undeclared sensitive operations use `P1034`; see [`PROJECTS.md`](PROJECTS.md) for capability grants and [`DIAGNOSTICS.md`](DIAGNOSTICS.md) for localized messages and stable code meanings.
