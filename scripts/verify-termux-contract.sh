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
grep -Fq 'cargo build --release --locked' "$installer"
grep -Fq 'target/release/padma' "$installer"
grep -Fq '"$PREFIX/bin/padma"' "$installer"
grep -Fq -- '--check' "$installer"
grep -Fq 'uninstall' "$installer"
grep -Fq 'TERMUX_PREFIX_EXPECTED' "$installer"
grep -Fq 'cargo build' "$recipe"
grep -Fq 'target/release/padma' "$recipe"

if grep -Fq 'pip install' "$installer" || grep -Fq '.bashrc' "$installer" || grep -Fq '.zshrc' "$installer"; then
  echo "Termux contract error: installer must not install optional tools or edit shell profiles" >&2
  exit 1
fi

bash -n "$installer"
bash "$installer" --help >/dev/null

fixture="$(mktemp -d)"
cleanup() {
  rm -rf "$fixture"
}
trap cleanup EXIT
mkdir -p "$fixture/bin" "$fixture/home"
cat > "$fixture/bin/pkg" <<'EOF'
#!/usr/bin/env sh
exit 0
EOF
chmod +x "$fixture/bin/pkg"
PATH="$fixture/bin:$PATH" \
  HOME="$fixture/home" \
  PREFIX="/data/data/com.termux/files/usr" \
  bash "$installer" --check > "$fixture/check-output"
grep -Fq 'Check complete: no package install, network, clone, build, binary replacement, or uninstall was run.' "$fixture/check-output"

if PATH="$fixture/bin:$PATH" HOME="$fixture/home" PREFIX="/tmp/not-termux" bash "$installer" --check > "$fixture/prefix-error" 2>&1; then
  echo "Termux contract error: installer accepted an unsafe prefix" >&2
  exit 1
fi
grep -Fq 'refusing a non-Termux prefix' "$fixture/prefix-error"

if PATH="$fixture/bin:$PATH" HOME="$fixture/home" PREFIX="/data/data/com.termux/files/usr" PADMA_REPO_DIR="/tmp/padma" bash "$installer" --check > "$fixture/repo-error" 2>&1; then
  echo "Termux contract error: installer accepted an unsafe repository path" >&2
  exit 1
fi
grep -Fq 'must be inside HOME' "$fixture/repo-error"

PATH="$fixture/bin:$PATH" \
  HOME="$fixture/home" \
  PREFIX="/data/data/com.termux/files/usr" \
  bash "$installer" uninstall > "$fixture/uninstall-output"
grep -Fq 'Padma is not installed at /data/data/com.termux/files/usr/bin/padma.' "$fixture/uninstall-output"

echo "Termux installer contract: ok"
