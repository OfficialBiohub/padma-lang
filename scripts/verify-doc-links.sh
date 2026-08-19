#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

failed=0

while IFS= read -r -d '' markdown_file; do
  while IFS= read -r match; do
    target="${match#](}"
    target="${target%)}"
    target="${target%%#*}"
    target="${target%% *}"

    case "$target" in
      ""|http://*|https://*|mailto:*|tel:*) continue ;;
    esac

    if [[ "$target" = /* ]]; then
      candidate="$root/${target#/}"
    else
      candidate="$(dirname "$markdown_file")/$target"
    fi

    if [[ ! -e "$candidate" ]]; then
      echo "documentation link error: $markdown_file -> $target" >&2
      failed=1
    fi
  done < <(grep -oE '\]\([^)]*\)' "$markdown_file" || true)
done < <(
  find . \
    -type d \( -name .git -o -name node_modules -o -name target -o -name dist \) -prune -o \
    -type f -name '*.md' -print0
)

if [[ "$failed" -ne 0 ]]; then
  exit 1
fi

echo "documentation links: ok"
