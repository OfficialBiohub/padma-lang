#!/data/data/com.termux/files/usr/bin/bash
set -euo pipefail

readonly PADMA_REPOSITORY_URL="https://github.com/OfficialBiohub/padma-lang.git"
readonly TERMUX_PREFIX_EXPECTED="/data/data/com.termux/files/usr"
readonly repo_dir="${PADMA_REPO_DIR:-$HOME/padma-lang}"

fail() {
  echo "Padma installer error: $*" >&2
  exit 1
}

usage() {
  cat <<'EOF'
Padma Termux source installer

Usage:
  bash install-termux.sh             Build/update and install Padma to $PREFIX/bin/padma.
  bash install-termux.sh --check     Validate the supported Termux install boundary only.
  bash install-termux.sh uninstall   Remove only $PREFIX/bin/padma.
  bash install-termux.sh --help      Show this help.

The default install action explicitly runs pkg update/install, clones or fast-forwards
the official repository, builds cargo --release --locked, and atomically replaces one
Padma binary. It does not install yt-dlp, edit shell profiles, read credentials, start
services, request Android permissions, or perform browser/device/provider actions.
EOF
}

require_termux() {
  command -v pkg >/dev/null 2>&1 || fail "এই installer শুধু Termux-এর ভিতরে চালান। / Run this installer inside Termux."
  [ "${PREFIX:-}" = "$TERMUX_PREFIX_EXPECTED" ] || fail "Supported prefix is $TERMUX_PREFIX_EXPECTED; refusing a non-Termux prefix."
}

validate_repo_dir() {
  case "$repo_dir" in
    "$HOME"/*) ;;
    *) fail "PADMA_REPO_DIR must be inside HOME; refusing an unsafe repository path." ;;
  esac
  case "/$repo_dir/" in
    *"/../"*) fail "PADMA_REPO_DIR must not contain traversal segments." ;;
  esac
}

verify_official_checkout() {
  [ -d "$repo_dir/.git" ] || return 1
  local remote
  remote="$(git -C "$repo_dir" remote get-url origin 2>/dev/null || true)"
  [ "$remote" = "$PADMA_REPOSITORY_URL" ] || fail "Existing repository origin is not the supported Padma repository."
  [ -z "$(git -C "$repo_dir" status --porcelain)" ] || fail "Existing Padma repository has local changes; commit/stash them before an upgrade."
}

install_release_binary() {
  local target="$PREFIX/bin/padma"
  local stage="$PREFIX/bin/.padma.new.$$"
  local backup=""
  local had_previous=0

  mkdir -p "$PREFIX/bin"
  if [ -e "$target" ] || [ -L "$target" ]; then
    [ ! -L "$target" ] && [ -f "$target" ] || fail "Refusing to replace a non-regular or symlinked $target."
    backup="$PREFIX/bin/.padma.backup.$$"
    cp -p "$target" "$backup"
    had_previous=1
  fi

  cleanup_install() {
    rm -f "$stage"
    if [ -n "$backup" ] && [ -f "$backup" ]; then
      rm -f "$backup"
    fi
  }
  trap cleanup_install EXIT

  install -m755 target/release/padma "$stage"
  "$stage" --version >/dev/null || fail "New release binary did not pass --version verification."
  mv -f "$stage" "$target"
  if ! "$target" --version >/dev/null; then
    if [ "$had_previous" -eq 1 ]; then
      mv -f "$backup" "$target"
      backup=""
      fail "New binary verification failed; the previous Padma binary was restored."
    fi
    rm -f "$target"
    fail "New binary verification failed; no previous Padma binary existed to restore."
  fi
  rm -f "$backup"
  backup=""
  trap - EXIT
  hash -r 2>/dev/null || true
}

run_check() {
  require_termux
  validate_repo_dir
  echo "Termux prefix: $PREFIX"
  echo "Repository path: $repo_dir"
  echo "Install target: $PREFIX/bin/padma"
  echo "Check complete: no package install, network, clone, build, binary replacement, or uninstall was run."
}

run_install() {
  require_termux
  validate_repo_dir
  pkg update -y
  pkg install -y git rust python

  if verify_official_checkout; then
    git -C "$repo_dir" pull --ff-only origin main
  else
    [ ! -e "$repo_dir" ] || fail "PADMA_REPO_DIR exists but is not a supported clean Git checkout."
    git clone "$PADMA_REPOSITORY_URL" "$repo_dir"
  fi

  cd "$repo_dir"
  cargo build --release --locked
  install_release_binary

  echo "Padma installed: $PREFIX/bin/padma"
  "$PREFIX/bin/padma" --version
  if [ "$(command -v padma 2>/dev/null || true)" != "$PREFIX/bin/padma" ]; then
    echo "PATH warning: open a new Termux shell, or run: export PATH=\"$PREFIX/bin:\$PATH\""
  fi
  echo "Try REPL: padma"
  echo "Try script: padma hello.pd"
  echo "Optional media workflows need a separate user-installed yt-dlp dependency."
}

run_uninstall() {
  require_termux
  local target="$PREFIX/bin/padma"
  if [ ! -e "$target" ] && [ ! -L "$target" ]; then
    echo "Padma is not installed at $target."
    return 0
  fi
  [ ! -L "$target" ] && [ -f "$target" ] || fail "Refusing to remove a non-regular or symlinked $target."
  rm -f "$target"
  hash -r 2>/dev/null || true
  echo "Removed only $target. The source checkout at $repo_dir was kept."
}

case "${1:-install}" in
  install) run_install ;;
  --check) run_check ;;
  uninstall) run_uninstall ;;
  --help|-h) usage ;;
  *) usage >&2; fail "Unknown installer action: $1" ;;
esac
