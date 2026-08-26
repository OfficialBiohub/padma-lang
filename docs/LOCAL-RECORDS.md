# Local Records v1

Padma Local Records v1 **attendance**, **expense**, এবং **inventory** table validate করে এবং redacted deterministic summary তৈরি করে। এটি student, family, এবং small-business-এর project-local CSV/TSV/JSON dataকে checked workflow-এ আনতে তৈরি। এটি account, cloud sync, remote database, payment, tax calculation, banking, email/message, device control, background job, অথবা process automation নয়।

> Local Records validates data quality. It does not decide a student result, financial eligibility, price, tax, debt, stock purchase, payment, or business outcome.

## APIs

| API | Result | Capability |
|---|---|---|
| `record.validate(kind, table)` | Strict validated table value | None; source table still needs its own `table.read` permission when read from a file. |
| `record.summary(kind, table)` | Redacted count/total metadata and disabled-action markers | None. |

`kind` must be exactly `attendance`, `expense`, or `inventory`. Both APIs accept only an existing Padma validated table value. Use `table.read` for a local file and `report.write_markdown` if you want a reviewed project-local Markdown report.

## Strict schemas

Headers must be in the exact shown order. Each record table must have at least one row; extra, missing, or reordered fields are rejected.

| Kind | Required ordered headers | Validation |
|---|---|---|
| `attendance` | `date`, `student`, `status` | Real `YYYY-MM-DD` date; bounded student text; status is `present`, `absent`, or `late`; each `(date, student)` pair is unique. |
| `expense` | `date`, `category`, `amount`, `currency`, `note` | Real date; bounded category/note; non-negative decimal amount with at most two fractional digits; one three-uppercase-letter currency for the full table. |
| `inventory` | `item`, `category`, `quantity`, `reorderLevel` | Bounded item/category; non-negative whole-number quantity and reorder level; each item is unique. |

All record text rejects control characters and raw `<`/`>` delimiters. Student, item, and category labels are bounded to 160 bytes; an expense note may be empty but is bounded to 512 bytes. Expense amounts are bounded to `1,000,000,000,000`; inventory quantities and reorder levels are bounded to `1,000,000,000`.

## Summaries

`record.summary` never returns personal labels, note text, or the original rows. It returns the following calculated metadata.

| Kind | Summary fields |
|---|---|
| `attendance` | `recordCount`, `presentCount`, `absentCount`, `lateCount` |
| `expense` | `recordCount`, `totalAmount`, `currency`, `categoryCount` |
| `inventory` | `recordCount`, `itemCount`, `categoryCount`, `lowStockCount` |

`lowStockCount` counts items whose `quantity` is less than or equal to `reorderLevel`. Every summary includes the fixed values `account: "disabled"`, `cloudSync: "disabled"`, `network: "disabled"`, `payment: "disabled"`, and `childProcess: "disabled"`.

## Bangla-English example

```padma
ধরি expenses = table.read("data/expenses.csv", "csv")
ধরি summary = record.summary("expense", expenses)
দেখাও text.format("Total: {totalAmount} {currency}", summary)
দেখাও summary["payment"]
```

Run the standalone example on Termux:

```bash
cd ~/padma-lang
cargo build --release
export PATH="$HOME/padma-lang/target/release:$PATH"
cd examples/local-records-household
padma .
cat out/family-expense.md
```

The example reads only `data/` files and writes one reviewed Markdown report below `out/`; therefore its manifest grants `filesystem = ["read", "write"]`. `record.validate` and `record.summary` themselves do not require a capability and do not create a file.

## Errors and boundaries

Malformed table shape still uses `P1069`. Invalid record kind, exact headers, required text, date, status, currency, amount, quantity, duplicate identity, raw markup, or record-specific bounds use **P1074**. File permission and project path errors stay with the underlying table/report API (`P1034`, `P1014`, `P1071`, and related codes). Record values and external paths are not echoed in record diagnostic details.

No Local Records API contacts a school, client, shop, bank, provider, cloud service, or device. It cannot collect account/login data, send a record, create an invoice/payment, schedule a report, start a background process, inspect other applications, or take stock/account action. Review data and any resulting report yourself before manually sharing or acting on it.
