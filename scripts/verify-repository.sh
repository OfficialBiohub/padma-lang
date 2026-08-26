#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

# A bounded larger test-thread stack keeps the deep, non-recursive-looking
# parser/renderer regression fixtures reproducible across Termux and CI hosts.
export RUST_MIN_STACK="${RUST_MIN_STACK:-16777216}"

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
local_server_smoke_dir="$(mktemp -d)"
local_server_smoke_pid=""
cleanup_local_server_smoke() {
  if [[ -n "$local_server_smoke_pid" ]]; then
    kill -TERM "$local_server_smoke_pid" 2>/dev/null || true
    wait "$local_server_smoke_pid" 2>/dev/null || true
  fi
  rm -rf "$local_server_smoke_dir" "$termux_smoke_dir"
}
trap cleanup_local_server_smoke EXIT
cp -R examples/local-backend-routes/. "$local_server_smoke_dir"
(cd "$local_server_smoke_dir" && "$binary" serve . >server.log 2>&1) &
local_server_smoke_pid=$!
health_body=""
for _ in $(seq 1 80); do
  if health_body="$(curl --noproxy '*' --silent --show-error --fail http://127.0.0.1:8080/health)"; then
    break
  fi
  sleep 0.1
done
if [[ "$health_body" != *'"status":"ok"'* ]]; then
  echo "release local server health smoke test failed" >&2
  exit 1
fi
students_body="$(curl --noproxy '*' --silent --show-error --fail http://127.0.0.1:8080/students)"
if [[ "$students_body" != *'"Rafi"'* ]]; then
  echo "release local server route smoke test failed" >&2
  exit 1
fi
missing_status="$(curl --noproxy '*' --silent --output /dev/null --write-out '%{http_code}' http://127.0.0.1:8080/missing)"
if [[ "$missing_status" != "404" ]]; then
  echo "release local server 404 smoke test failed" >&2
  exit 1
fi
kill -TERM "$local_server_smoke_pid" 2>/dev/null || true
wait "$local_server_smoke_pid" 2>/dev/null || true
local_server_smoke_pid=""
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
