# Local Expense Reporting Example

এই example project-local CSV expense data থেকে একটি Markdown report বানায়। কোনো extra Termux package, network, browser, cloud account, payment gateway, অথবা Android shared-storage permission লাগে না।

```bash
cd ~/padma-lang
cargo build --release
export PATH="$HOME/padma-lang/target/release:$PATH"
cd examples/local-reporting-expense
padma .
cat out/january-expense.md
```

Expected terminal output:

```text
Expense rows: 3
Report created: true
```

Generated `out/january-expense.md` begins with:

```markdown
# January Expense Report

Rows: 3

| date | category | amount |
```

`table.read` requires `filesystem = ["read"]`; `report.write_markdown` requires `filesystem = ["write"]`. Both paths stay inside the project root. The example creates a local readable report only: it does not calculate tax, send an invoice, charge a payment, email/share/upload the report, create an online account, render HTML/JavaScript, or write Android Downloads.
