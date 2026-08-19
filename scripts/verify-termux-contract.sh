#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

installer="install-termux.sh"
recipe="packaging/termux/packages/padma/build.sh"

for file in "$installer" "$recipe"; do
  if [[ ! -f "$file" ]]; then
    echo "Termux contract error: missing $file" >&2
    exit 1
  fi
done

grep -Fq 'cargo build --release' "$installer"
grep -Fq 'target/release/padma' "$installer"
grep -Fq '"$PREFIX/bin/padma"' "$installer"
grep -Fq 'cargo build' "$recipe"
grep -Fq 'target/release/padma' "$recipe"

echo "Termux installer contract: ok"
