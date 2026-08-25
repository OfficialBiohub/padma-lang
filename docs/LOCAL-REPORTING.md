# Local Reporting Toolkit v1

Padma Local Reporting Toolkit v1 একটি validated CSV/TSV/JSON table থেকে deterministic Markdown report ও concise local summary তৈরি করে। এটি student marks, family expense, shop inventory, freelance client records, survey, এবং class report-এর মতো দৈনন্দিন project-local dataকে সহজে পড়ার জন্য তৈরি।

> এটি local report renderer। এটি cloud upload, email/message sending, payment invoice, HTML/JavaScript execution, spreadsheet macro, browser action, অথবা background report job নয়।

## APIs

| API | Result | Capability |
|---|---|---|
| `report.markdown(title, table)` | In-memory Markdown text return করে | কোনো capability নয় |
| `report.summary(title, table)` | `{title, format, rowCount, columnCount, columns}` map return করে | কোনো capability নয় |
| `report.write_markdown(path, title, table)` | Project-local `.md` file লেখে এবং `true` return করে | project mode-এ `filesystem = ["write"]` |

`table` অবশ্যই `table.read`/`table.select`-এর মতো validated Padma table value হতে হবে। Plain arbitrary map বা wrong schema `P1069` দেয়। `title` empty হতে পারবে না, 160 bytes-এর বেশি হতে পারবে না, control character বা raw `<`/`>` HTML delimiter থাকতে পারবে না। Markdown renderer table cell, header, এবং title content escape করে, তাই a cell containing `<script>` report-এ text হিসেবেই দেখা যায়।

## File boundary

`report.write_markdown` শুধুই manifest project-এর ভিতরে existing parent directoryসহ relative `.md` path-এ লিখতে পারে। Absolute path, `..`, `@downloads`, missing parent, symlinked output component, and non-`.md` suffix rejected। Rendered report সর্বোচ্চ 1 MiB। এটি report পাঠায় না, public publish করে না, payment বা account action তৈরি করে না।

## Termux expense example

```bash
cd ~/padma-lang
cargo build --release
export PATH="$HOME/padma-lang/target/release:$PATH"
cd examples/local-reporting-expense
padma .
cat out/january-expense.md
```

Exampleটি three-row expense CSV থেকে summary দেখায় এবং `out/january-expense.md` তৈরি করে। কোনো extra Termux package, network, browser, account, cloud storage, payment, অথবা shared Android storage প্রয়োজন নেই।

## Errors

Wrong argument count `P1009`, wrong ordinary type `P1010`, absent write capability `P1034`, unsafe project path `P1014`, write failure `P1015`, malformed table `P1069`, এবং unsafe title/output/report policy `P1071` দেয়। Error messages source locale অনুযায়ী Bangla বা English হয়; report error raw external path বা table source echo করে না।

## Non-goals

v1 PDF/DOCX/XLSX generation, charts, formula evaluation, currency/tax calculation, invoice payment, email/WhatsApp delivery, HTML rendering, database sync, remote storage upload, shared Downloads output, report scheduling, and automatic public publishing করে না। এইগুলোর জন্য আলাদা bounded contract, capability, confirmation, tests, and documentation প্রয়োজন।
