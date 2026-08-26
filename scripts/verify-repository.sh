#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

bash scripts/verify-repository-hygiene.sh
bash scripts/verify-doc-links.sh
bash scripts/verify-termux-contract.sh

cargo fmt --check
cargo test --locked
cargo test --manifest-path tooling/padma-lsp/Cargo.toml --locked
cargo build --release --locked

binary="$root/target/release/padma"
"$binary" examples/hello-bn.pd
"$binary" examples/hello-en.pd
termux_smoke_dir="$(mktemp -d)"
cleanup_termux_smoke() {
  rm -rf "$termux_smoke_dir"
}
trap cleanup_termux_smoke EXIT
cp -R examples/termux-cli-smoke/. "$termux_smoke_dir"
termux_smoke_output="$(cd "$termux_smoke_dir" && "$binary" .)"
if ! grep -Fq 'Padma Termux CLI ready' <<<"$termux_smoke_output" || ! grep -Fq '5' <<<"$termux_smoke_output"; then
  echo "release Termux project smoke test failed" >&2
  exit 1
fi
repl_output="$(printf '1+1\n২ + ৩\nexit()\n' | "$binary")"
if ! grep -Fq 'padma> 2' <<<"$repl_output" || ! grep -Fq 'padma> 5' <<<"$repl_output"; then
  echo "release REPL bare-expression smoke test failed" >&2
  exit 1
fi
"$binary" gui plan examples/gui-static
"$binary" android plan examples/gui-static
"$binary" render plan examples/render-git-linked
"$binary" render api-plan examples/render-git-linked

echo "repository verification: ok"
