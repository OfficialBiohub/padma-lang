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

## Errors

Wrong argument counts use `P1009`; incompatible values use `P1010`; unsafe paths use `P1014`; unreadable files use `P1028`; and over-limit sleeps use `P1012`. See [`DIAGNOSTICS.md`](DIAGNOSTICS.md) for localized messages and stable code meanings.
