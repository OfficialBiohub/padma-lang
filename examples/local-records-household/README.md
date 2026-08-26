# Local Household Records Example

এই standalone project attendance, family expense, এবং small inventory CSV validate করে। এটি `out/family-expense.md`-এ একটি local review report লেখে; network, account, cloud sync, payment, browser, device, shared Android storage, child process, বা background task ব্যবহার করে না।

```bash
cd ~/padma-lang
cargo build --release
export PATH="$HOME/padma-lang/target/release:$PATH"
cd examples/local-records-household
padma .
cat out/family-expense.md
```

Expected terminal output:

```text
Present: 1
Expense: 160.5 BDT
Low stock: 1
Report saved: true
disabled
```

The manifest grants only `filesystem = ["read", "write"]`: read is needed for the three project-local CSV files and write is needed for the Markdown report. `record.summary` returns redacted counts/totals rather than student/item/note rows. Review every input and output yourself; this example does not calculate tax, create a payment, send a report, decide stock purchase, or guarantee a financial/business result.
