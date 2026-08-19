#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

required_files=(
  README.md
  CONTRIBUTING.md
  SECURITY.md
  CODE_OF_CONDUCT.md
  SUPPORT.md
  CHANGELOG.md
  LICENSE
  .editorconfig
  .gitattributes
  docs/README.md
  docs/REPOSITORY-ARCHITECTURE.md
  examples/README.md
  tooling/README.md
  tests/README.md
  scripts/README.md
  src/README.md
  packaging/README.md
  .github/workflows/ci.yml
)

for file in "${required_files[@]}"; do
  if [[ ! -f "$file" ]]; then
    echo "repository hygiene error: missing required file: $file" >&2
    exit 1
  fi
done

forbidden_pattern='(^|/)(target|node_modules|dist)(/|$)|(^|/)\.DS_Store$|\.vsix$'
if git ls-files | grep -E "$forbidden_pattern" >/dev/null; then
  echo "repository hygiene error: generated artifact tracked by Git:" >&2
  git ls-files | grep -E "$forbidden_pattern" >&2
  exit 1
fi

if git ls-files | grep -E '(^|/)(\.env|\.env\..+|.*\.pem|.*\.key)$' >/dev/null; then
  echo "repository hygiene error: potential secret file tracked by Git:" >&2
  git ls-files | grep -E '(^|/)(\.env|\.env\..+|.*\.pem|.*\.key)$' >&2
  exit 1
fi

echo "repository hygiene: ok"
