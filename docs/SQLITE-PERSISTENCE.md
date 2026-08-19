# Padma Local SQLite Persistence

Padma-এর প্রথম persistence layer ছোট project-এর local structured data রাখার জন্য তৈরি। এটি raw SQL execution API নয়। বরং `db.put`, `db.get`, `db.list`, এবং `db.delete`—এই চারটি fixed operation দিয়ে namespace এবং key-এর অধীনে JSON-compatible Padma value সংরক্ষণ করে। এই সীমা ইচ্ছাকৃত: user input কখনো SQL text হিসেবে যুক্ত হয় না; fixed statements এবং SQLite parameter binding ব্যবহার করা হয়। SQLite CLI-তে `.parameter`, `.mode json`, `.bail`, এবং `.timeout` command রয়েছে। [1]

| বিষয় | Contract |
|---|---|
| Database engine | ফোনে install করা local `sqlite3` CLI |
| Permission | project manifest-এ explicit `database = ["sqlite"]` |
| File location | শুধুই current Padma project root-এর নিচে relative `.sqlite` path |
| Storage model | `namespace` + `key` + JSON-compatible value |
| Path restrictions | Absolute path, `..`, symbolic-link escape, এবং `@downloads` নিষিদ্ধ |
| Query scope | Fixed key-value operations; arbitrary SQL বা extension loading নেই |
| Limits | Key ও namespace সর্বোচ্চ 256 bytes; JSON value ও child I/O সর্বোচ্চ 256 KiB; 5-second process limit |

## Termux setup

প্রথমে Termux-এ SQLite CLI install করুন:

```bash
pkg install sqlite -y
```

Project-এর ভেতরে database folder তৈরি করুন এবং `padma.toml`-এ only-needed grant দিন:

```bash
mkdir -p data
```

```toml
[capabilities]
database = ["sqlite"]
```

`database = []` থাকলে, অথবা single-file mode-এ চালালে, Padma `P1034` capability diagnostic দেখাবে। Database grant শুধু local project database access দেয়; এটি filesystem, network, process, server, বা arbitrary SQL permission দেয় না।

## API

| Function | Arguments | Returns |
|---|---|---|
| `db.version` | `database_path` | fixed managed schema version (`1`) |
| `db.put` | `database_path, namespace, key, value` | `true` after an insert or replacement |
| `db.get` | `database_path, namespace, key` | stored Padma value, or `none` if no key exists |
| `db.list` | `database_path, namespace, limit` | ordered list of `{ "key": text, "value": value }` maps |
| `db.delete` | `database_path, namespace, key` | `true` after the requested deletion operation |
| `db.apply` | `database_path, operations` | `true` after one atomic fixed-operation batch |

`value` may be any JSON-compatible Padma value: number, text, list, map, `true`, `false`, or `none`. A map’s keys must remain text, consistent with the rest of Padma’s JSON contract.

## English example

Create `main.pd`:

```padma
let saved = db.put("data/profile.sqlite", "user", "rafi", {
  "name": "Rafi",
  "level": 6,
  "skills": ["web", "AI"]
})
print saved

print db.get("data/profile.sqlite", "user", "rafi")
print db.list("data/profile.sqlite", "user", 10)

db.delete("data/profile.sqlite", "user", "rafi")
print db.get("data/profile.sqlite", "user", "rafi")
```

## Fixed version and atomic batches

`db.version` exposes metadata for Padma-managed storage only. The initial version is **1**; it is not a user-defined schema migration engine. This deliberate constraint preserves upgrade ownership in the language runtime rather than accepting executable schema text from an application.

`db.apply` makes up to 32 fixed `put` or `delete` operations to **one database file** atomically. Each operation is a map. No function callbacks, nested batches, arbitrary operation names, extra fields, or cross-database writes are accepted.

```padma
print db.version("data/tasks.sqlite")

let saved = db.apply("data/tasks.sqlite", [
  {
    "op": "put",
    "namespace": "tasks",
    "key": "first",
    "value": {"title": "Learn Padma", "done": false}
  },
  {
    "op": "put",
    "namespace": "tasks",
    "key": "second",
    "value": {"title": "Build safely", "done": true}
  },
  {
    "op": "delete",
    "namespace": "tasks",
    "key": "old-task"
  }
])

print saved
print db.list("data/tasks.sqlite", "tasks", 10)
```

Internally, Padma validates the complete batch before opening SQLite, then emits a single `BEGIN IMMEDIATE` and `COMMIT` sequence. SQLite’s documented transaction model does not allow nested `BEGIN...COMMIT` transactions and permits only one simultaneous writer, so a lock conflict fails rather than silently producing a partial success. [2]

Run it with normal project workflow:

```bash
padma .
```

## Bangla example

```padma
ধরি সংরক্ষণ = db.put("data/ছাত্র.sqlite", "শ্রেণি", "রিমা", {
  "নাম": "রিমা",
  "ক্লাস": 6,
  "বিষয়": ["গণিত", "বিজ্ঞান"]
})
দেখাও সংরক্ষণ
দেখাও db.get("data/ছাত্র.sqlite", "শ্রেণি", "রিমা")
```

## Safety behaviour

The adapter invokes `sqlite3` with a minimal environment, standard input only, and no shell. It prepares a fixed schema named `padma_records`; user values are hexadecimal parameter payloads, so a text value cannot become a SQL command. It sets a 5-second SQLite lock wait and terminates an overlong child process. SQLite’s CLI documentation confirms that dot commands are interpreted by the CLI and that parameter management and a lock timeout are provided there. [1]

This M9 foundation deliberately does **not** claim an ORM, user-defined schema, arbitrary migrations, SQL console, replication, remote database, or managed deployment. Those need separate contracts, schema-version design, transaction APIs, backup policy, and security review before they are added.

## Diagnostics

| Code | Meaning |
|---|---|
| `P1014` | Database path is unsafe or not a project-relative `.sqlite` file |
| `P1034` | `database:sqlite` was not explicitly granted by a project manifest |
| `P1041` | `sqlite3` is missing or the bounded CLI process failed |
| `P1042` | SQLite returned unexpected managed-record JSON |
| `P1043` | A byte-size or 5-second execution safety limit was exceeded |

## References

[1]: https://sqlite.org/cli.html "SQLite Command Line Shell"
[2]: https://sqlite.org/lang_transaction.html "SQLite Transaction documentation"
