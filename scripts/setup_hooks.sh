#!/bin/bash
# TizenClaw Git Hooks Setup Script

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

if ! git -C "${PROJECT_DIR}" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "Error: Not a git repository (${PROJECT_DIR})."
    exit 1
fi

_git_common_dir="$(git -C "${PROJECT_DIR}" rev-parse --git-common-dir)"
if [[ "${_git_common_dir}" != /* ]]; then
    _git_common_dir="${PROJECT_DIR}/${_git_common_dir}"
fi
HOOKS_DIR="${_git_common_dir}/hooks"
PRE_COMMIT_HOOK="${HOOKS_DIR}/pre-commit"

mkdir -p "$HOOKS_DIR"

cat << 'EOF' > "$PRE_COMMIT_HOOK"
#!/bin/bash
# Ensure standard paths are available (especially cargo)
export PATH=$PATH:/usr/local/cargo/bin:${HOME}/.cargo/bin

echo "🔄 Running TizenClaw Workflow checks (pre-commit hook)..."
python3 .agents/skills/workflow_manager/workflow_manager.py --action verify_status
if [ $? -ne 0 ]; then
    echo ""
    echo "❌ COMMIT BLOCKED: Pre-commit workflow verification failed."
    echo "   You MUST adhere to the Dev Workflow policies."
    exit 1
fi
exit 0
EOF

chmod +x "$PRE_COMMIT_HOOK"
echo "✅ Installed pre-commit git hook successfully at ${PRE_COMMIT_HOOK}"
