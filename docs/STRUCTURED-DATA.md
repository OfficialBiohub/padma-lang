# Structured Data Toolkit v1

Padma Structured Data Toolkit v1 local CSV, TSV, এবং JSON table file পড়তে, filter করতে, selected column নিতে, category count করতে, এবং deterministic CSV export করতে পারে। এটি inventory, survey, school records, small-business lists, freelancing report preparation, এবং local API export-এর মতো দৈনন্দিন কাজের জন্য তৈরি।

> এটি local data processing toolkit। এটি spreadsheet application, remote database, arbitrary file-system access, background job, বা cloud sync নয়।

## Capability and file boundary

Project mode-এ table file পড়া ও CSV লেখা explicit filesystem capability চায়:

```toml
[capabilities]
filesystem = ["read", "write"]
```

সব table path canonical project root-এর নিচে থাকতে হবে। Absolute path, `..`, `@downloads`, এবং project-root escape rejected হয়। `table.read` সর্বোচ্চ 1 MiB UTF-8 source, 4,096 data row, 64 column, 128-byte header, এবং 4,096-byte single-line cell গ্রহণ করে। `table.write_csv`-ও সর্বোচ্চ 1 MiB output এবং `.csv` output path গ্রহণ করে।

## APIs

| API | Result |
|---|---|
| `table.read(path, format)` | `format` হলো `"csv"`, `"tsv"`, অথবা `"json"`; returns validated table value. |
| `table.headers(table)` | Ordered text header list. |
| `table.rows(table)` | Text-key row map list. |
| `table.filter_equal(table, column, text)` | Exact text cell match থাকা rows-এর নতুন table. |
| `table.select(table, ["column", ...])` | Ordered selected columnsসহ নতুন table. |
| `table.count_by(table, column)` | Cell value থেকে deterministic numeric count map. |
| `table.write_csv(path, table)` | Validated tableকে project-local CSV হিসেবে লেখে এবং `true` return করে. |

Table value একটি validated map: `format`, `headers`, এবং `rows`। প্রতিটি row-এর key headers-এর সঙ্গে exactly match করে এবং প্রতিটি cell text। CSV quoted cell-এ doubled quote (`""`) সমর্থিত; multiline CSV cell intentionalভাবে rejected। JSON input অবশ্যই object-row array হতে হবে এবং cell scalar (`text`, number, boolean, null) হতে হবে; nested object/list cell rejected। Null JSON cell empty text হয়।

## Termux inventory example

```bash
cd ~/padma-lang
cargo build --release
export PATH="$HOME/padma-lang/target/release:$PATH"
cd examples/structured-data-inventory
padma .
cat out/food-inventory.csv
```

Exampleটি `data/inventory.csv` থেকে `food` rows নেয়, শুধু name/price রাখে, category count দেখায়, এবং `out/food-inventory.csv` লেখে। এটি network, process, database, browser, Android storage, বা cloud service ব্যবহার করে না।

## Errors

Wrong argument count `P1009`, wrong ordinary value type `P1010`, denied filesystem grant `P1034`, unsafe project path `P1014`, unreadable path `P1028`, এবং unsafe/malformed table data `P1069` দেয়। Error messages source locale অনুযায়ী Bangla বা English হয়। Table error detail raw source row বা sensitive file content echo করে না।

## Non-goals

v1 এখন numeric coercion, formulas, joins, sort, arbitrary delimiter, multi-line CSV field, Excel/XLSX parsing, shared Downloads write, remote spreadsheet sync, automatic cloud upload, বা spreadsheet macro execution করে না। এগুলোর জন্য আলাদা bounded design, capability, tests, and documentation প্রয়োজন।
