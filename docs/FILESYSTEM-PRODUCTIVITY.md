# Filesystem Productivity Toolkit v1

Filesystem Productivity Toolkit v1 project-local files inspect করে এবং copy, move, archive operation-এর **dry-run plan** তৈরি করে। এটি Termux project cleanup, report preparation, source review, checksum verification, এবং safe manual file-operation preparation-এর জন্য তৈরি।

> v1 কোনো file copy, move, archive, delete, rename, chmod, shell command, background job, shared-storage write, বা external process চালায় না.

## Capability and scope

সব API project mode-এ `filesystem = ["read"]` চায়। কারণ list, checksum, search, এবং plan সব source metadata বা content inspect করে। Plan command destination validate করলেও কোনো destination file তৈরি বা পরিবর্তন করে না; ফলে `filesystem:write` capability plan-এর জন্য যথেষ্ট নয় বা দরকারও নয়।

Paths must be project-relative, canonical root-এর নিচে, non-empty, and free of `..`, absolute paths, `@downloads`, and symlink escape. v1 symlink path, non-regular source, unsafe destination, unreadable text, oversized source, and unbounded traversal reject করে।

## APIs

| API | Result | Limit |
|---|---|---|
| `fs.list(path, depth)` | Ordered list of `{path, type, size}` entries under one project-local directory | `depth` 0–4; at most 256 entries; symlink rejected |
| `fs.checksum(path)` | `sha256:<hex>` checksum of one regular file | At most 1 MiB; no symlink |
| `fs.search_text(path, query, limit)` | Ordered list of `{line, text}` matching UTF-8 text lines | Source at most 1 MiB; query 1–128 bytes; 1–100 matches |
| `fs.copy_plan(source, destination)` | Deterministic copy descriptor with `execution: "disabled"` | Regular source up to 1 MiB; `.copy` destination rejected only by generic path policy |
| `fs.move_plan(source, destination)` | Deterministic move descriptor with `execution: "disabled"` | Same as copy plan |
| `fs.archive_plan(source, destination)` | Deterministic archive descriptor with `execution: "disabled"` | Regular source up to 1 MiB; destination must end in `.zip` |

All plan result maps contain fixed `operation`, `source`, `destination`, `sourceSize`, `sourceChecksum`, `execution`, `network`, `childProcess`, and `filesystemMutation` fields. They are review artifacts, not mutable task queues, confirmation tokens, or permission upgrades.

## Example flow

```bash
cd ~/padma-lang/examples/filesystem-productivity
padma .
```

The example lists a project folder, creates a SHA-256 checksum, finds selected text, and prints a disabled copy plan. It does not write, copy, move, archive, delete, or contact a network service.

## Failure policy

Normal wrong argument/type failures remain `P1009`/`P1010`; missing `filesystem:read` is `P1034`; unsafe path is `P1014`; unreadable source is `P1028`. Filesystem productivity limit, symlink, binary/text, non-regular source, unsafe plan, or invalid depth/query/match limit errors use `P1070`. Diagnostics do not echo raw file content or filesystem paths outside the project root.

## Non-goals

v1 deliberately excludes recursive mutation, delete/rename/copy/move/archive execution, Android Downloads access, permissions, glob patterns, arbitrary command invocation, file watcher/daemon behavior, compression backend invocation, encrypted archives, malware scanning, and remote file sync. Any future mutating action requires an independent versioned contract, fresh visible confirmation, write capability, bounded operation, cancellation, audit/redaction policy, and security review.
