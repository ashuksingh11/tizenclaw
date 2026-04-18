#!/bin/bash
# Install TizenClaw git hooks
# Usage: ./scripts/setup-hooks.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

if ! git -C "${REPO_DIR}" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "Error: Not a git repository."
  exit 1
fi

_git_common_dir="$(git -C "${REPO_DIR}" rev-parse --git-common-dir)"
if [[ "${_git_common_dir}" != /* ]]; then
  _git_common_dir="${REPO_DIR}/${_git_common_dir}"
fi
HOOKS_DIR="${_git_common_dir}/hooks"
mkdir -p "${HOOKS_DIR}"

ln -sf "${SCRIPT_DIR}/pre-commit" \
  "${HOOKS_DIR}/pre-commit"
chmod +x "${SCRIPT_DIR}/pre-commit"

echo "Git hooks installed:"
echo "  pre-commit -> scripts/pre-commit"
echo ""
echo "To uninstall: rm .git/hooks/pre-commit"
