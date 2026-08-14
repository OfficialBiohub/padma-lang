#!/data/data/com.termux/files/usr/bin/bash
set -euo pipefail

repo_dir="${PADMA_REPO_DIR:-$HOME/padma-lang}"

if ! command -v pkg >/dev/null 2>&1; then
  echo "এই installer শুধু Termux-এর ভিতরে চালান। / Run this installer inside Termux."
  exit 1
fi

pkg update -y
pkg install -y git rust

if [ ! -d "$repo_dir/.git" ]; then
  git clone https://github.com/OfficialBiohub/padma-lang.git "$repo_dir"
else
  git -C "$repo_dir" pull --ff-only origin main
fi

cd "$repo_dir"
cargo build --release
install -Dm755 target/release/padma "$PREFIX/bin/padma"

echo "Padma installed. Try: padma run examples/hello-bn.pd"
