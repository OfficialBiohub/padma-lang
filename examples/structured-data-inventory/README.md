# Structured Data Inventory Example

এই example একটি local inventory CSV থেকে food category filter করে এবং নতুন CSV report বানায়। কোনো extra Termux package, network, browser, database, cloud account, বা Android permission লাগবে না।

```bash
cd ~/padma-lang
cargo build --release
export PATH="$HOME/padma-lang/target/release:$PATH"
cd examples/structured-data-inventory
padma .
cat out/food-inventory.csv
```

Expected terminal output:

```text
Food item count: 2
Created out/food-inventory.csv
```

Generated `out/food-inventory.csv` contains:

```csv
name,price
Tea,40
Coffee,80
```

`table.read` needs `filesystem = ["read"]`; `table.write_csv` needs `filesystem = ["write"]`. Both paths are project-relative and bounded. The program cannot read arbitrary phone files, traverse directories, write Android Downloads, contact a network service, or execute spreadsheet macros.
