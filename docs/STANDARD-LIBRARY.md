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

Wrong argument counts use `P1009`; incompatible values use `P1010`; unsafe paths use `P1014`; unreadable files use `P1028`; malformed JSON uses `P1029`; unsupported URLs use `P1030`; invalid format placeholders use `P1031`; and over-limit sleeps or random bounds use `P1012`. Malformed or unsafe structured tables use `P1069`; unsafe filesystem productivity input/plan state uses `P1070`; unsafe local reporting policy uses `P1071`. Bridge failures use `P1035` through `P1040`. In manifest-run projects, undeclared sensitive operations use `P1034`; see [`PROJECTS.md`](PROJECTS.md) for capability grants and [`DIAGNOSTICS.md`](DIAGNOSTICS.md) for localized messages and stable code meanings.
