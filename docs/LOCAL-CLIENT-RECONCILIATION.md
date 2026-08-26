# Local Client-Data Reconciliation v1

`client.reconcile_summary(left, right, key)` compares two existing bounded table values by one unique local match key. It returns counts and SHA-256 digests, never the matched identifiers. `client.reconcile_markdown(title, left, right, key)` renders the same redacted review artifact. `client.write_reconciliation(path, title, left, right, key)` writes one project-local non-symlink `.md` file and requires `filesystem = ["write"]`.

Both tables must contain the same chosen key header; every key value must be non-empty, unique within its table, and safe local text. The result reports left/right row counts, matched count, left-only count, right-only count, and deterministic checksums. It does not reconcile payments, contact a client, upload anything, submit delivery, open a browser, use an account, call a network, or run a process. Invalid inputs use `P1079`.

Run [`examples/freelancer-client-reconciliation`](../examples/freelancer-client-reconciliation/README.md) in Termux. Review every mismatch and checksum yourself before any manual external action.
