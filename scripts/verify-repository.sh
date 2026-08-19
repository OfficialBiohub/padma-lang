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
"$binary" gui plan examples/gui-static
"$binary" android plan examples/gui-static
"$binary" render plan examples/render-git-linked
"$binary" render api-plan examples/render-git-linked

echo "repository verification: ok"
