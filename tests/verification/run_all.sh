#!/bin/bash
# TizenClaw Full Verification Test Runner
# Runs all automated test suites after deployment.
#
# Called by: deploy.sh -T (--full-test)
#
# Usage:
#   ./tests/verification/run_all.sh [-d <device-serial>] [-s <suite1,suite2>] [--list]

set -uo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

PROJECT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
DEVICE_SERIAL=""
SUITE_FILTER=""
LIST_ONLY=0
SUITE_PASS=0
SUITE_FAIL=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    -d|--device) DEVICE_SERIAL="$2"; shift 2 ;;
    -s|--suite) SUITE_FILTER="$2"; shift 2 ;;
    --list) LIST_ONLY=1; shift ;;
    -h|--help)
      echo "Usage: $0 [-d <device-serial>] [-s <suite1,suite2>] [--list]"
      exit 0 ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

if ! command -v sdb &>/dev/null; then
  for _c in "${HOME}/tizen-studio/tools" "/opt/tizen-studio/tools"; do
    [ -x "${_c}/sdb" ] && export PATH="${_c}:${PATH}" && break
  done
fi

sdb_cmd() {
  if [ -n "${DEVICE_SERIAL}" ]; then sdb -s "${DEVICE_SERIAL}" "$@"
  else sdb "$@"; fi
}
sdb_shell() { sdb_cmd shell "$@" 2>/dev/null; }

# List of all suites
declare -A SUITE_MAP
SUITE_ORDER=(
  "device_checks"
  "e2e_smoke"
  "service"
  "embedded_tools"
  "llm_integration"
  "mcp"
  "regression"
)

SUITE_MAP["device_checks"]="Device Binary Checks:device_checks"
SUITE_MAP["e2e_smoke"]="E2E Smoke & Base MCP Tests:tests/e2e"
SUITE_MAP["service"]="Daemon Health & Infrastructure:tests/verification/service"
SUITE_MAP["embedded_tools"]="Embedded Tool Operations:tests/verification/embedded_tools"
SUITE_MAP["llm_integration"]="LLM Agent Tests:tests/verification/llm_integration"
SUITE_MAP["mcp"]="MCP Protocol Compliance:tests/verification/mcp"
SUITE_MAP["regression"]="Regression & Stability:tests/verification/regression"

if [ "$LIST_ONLY" -eq 1 ]; then
  echo "Available Test Suites:"
  for key in "${SUITE_ORDER[@]}"; do
    desc="${SUITE_MAP[$key]%%:*}"
    echo "  - $key : $desc"
  done
  exit 0
fi

# Parse suite filter
run_all=1
declare -A RUN_SUITES
if [ -n "$SUITE_FILTER" ]; then
  run_all=0
  IFS=',' read -ra ADDR <<< "$SUITE_FILTER"
  for s in "${ADDR[@]}"; do
    RUN_SUITES["$s"]=1
  done
fi

should_run() {
  local key="$1"
  if [ "$run_all" -eq 1 ]; then return 0; fi
  if [ "${RUN_SUITES[$key]:-0}" -eq 1 ]; then return 0; fi
  return 1
}

run_device_tests() {
  echo -e "\n${CYAN}── Suite: Device Binary Checks${NC}"
  local pass=0 fail=0

  for bin in tizenclaw tizenclaw-cli tizenclaw-tool-executor; do
    local e; e=$(sdb_shell "test -f /usr/bin/${bin} && echo yes || echo no" | tr -d '\r')
    if [ "${e}" = "yes" ]; then
      echo -e "  ${GREEN}[PASS]${NC} /usr/bin/${bin}"; ((pass++))
    else
      echo -e "  ${RED}[FAIL]${NC} /usr/bin/${bin} missing"; ((fail++))
    fi
  done

  for lib in libtizenclaw.so libtizenclaw_core.so; do
    local e; e=$(sdb_shell "test -f /usr/lib64/${lib} && echo yes || echo no" | tr -d '\r')
    if [ "${e}" = "yes" ]; then
      echo -e "  ${GREEN}[PASS]${NC} /usr/lib64/${lib}"; ((pass++))
    else
      echo -e "  ${RED}[FAIL]${NC} /usr/lib64/${lib} missing"; ((fail++))
    fi
  done

  local st; st=$(sdb_shell systemctl is-active tizenclaw 2>/dev/null | tr -d '[:space:]')
  if [ "${st}" = "active" ]; then
    echo -e "  ${GREEN}[PASS]${NC} tizenclaw.service active"; ((pass++))
  else
    echo -e "  ${RED}[FAIL]${NC} tizenclaw.service ${st}"; ((fail++))
  fi

  local ss; ss=$(sdb_shell systemctl is-active tizenclaw-tool-executor.socket 2>/dev/null | tr -d '[:space:]')
  if [ "${ss}" = "active" ] || [ "${ss}" = "listening" ]; then
    echo -e "  ${GREEN}[PASS]${NC} tool-executor.socket ${ss}"; ((pass++))
  else
    echo -e "  ${RED}[FAIL]${NC} tool-executor.socket ${ss}"; ((fail++))
  fi

  if [ "${fail}" -eq 0 ]; then
    echo -e "  ${GREEN}Device checks: ${pass} passed${NC}"; ((SUITE_PASS++))
  else
    echo -e "  ${RED}Device checks: ${pass} passed, ${fail} failed${NC}"; ((SUITE_FAIL++))
  fi
}

run_script() {
  local script="$1"
  if [ ! -f "${script}" ]; then
    echo -e "  ${YELLOW}[SKIP]${NC} ${script} not found"; return 1
  fi
  local args=()
  [ -n "${DEVICE_SERIAL}" ] && args+=("-d" "${DEVICE_SERIAL}")
  if bash "${script}" "${args[@]+"${args[@]}"}"; then
    return 0
  else
    return 1
  fi
}

run_directory() {
  local key="$1"
  local desc="${SUITE_MAP[$key]%%:*}"
  local path="${SUITE_MAP[$key]#*:}"
  
  echo -e "\n${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo -e "${BOLD}  Running Suite: ${key} (${desc})${NC}"
  echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

  if [ "$key" = "device_checks" ]; then
    run_device_tests
    return
  fi

  local any_failed=0
  local script_count=0
  
  for script in "${PROJECT_DIR}/${path}"/test_*.sh; do
    [ -e "$script" ] || continue
    ((script_count++))
    # echo -e "\n${CYAN}▷ Running: $(basename "$script")${NC}"
    if run_script "$script"; then
      true
    else
      any_failed=1
    fi
  done
  
  if [ "$script_count" -eq 0 ]; then
    echo -e "  ${YELLOW}[WARN]${NC} No test scripts found for ${key}"
  else
    if [ "$any_failed" -eq 0 ]; then
      ((SUITE_PASS++))
    else
      ((SUITE_FAIL++))
    fi
  fi
}

echo -e "\n${BOLD}══════════════════════════════════════════${NC}"
echo -e "${BOLD}  TizenClaw Full Verification Suite${NC}"
echo -e "${BOLD}══════════════════════════════════════════${NC}"

for key in "${SUITE_ORDER[@]}"; do
  if should_run "$key"; then
    run_directory "$key"
  fi
done

echo -e "\n${BOLD}══════════════════════════════════════════${NC}"
TOTAL=$((SUITE_PASS + SUITE_FAIL))
if [ "${SUITE_FAIL}" -eq 0 ]; then
  echo -e "  ${GREEN}${BOLD}ALL SUITES PASSED${NC}: ${SUITE_PASS}/${TOTAL}"
else
  echo -e "  ${RED}${BOLD}FAILED${NC}: ${SUITE_PASS} passed, ${SUITE_FAIL} failed"
fi
echo -e "${BOLD}══════════════════════════════════════════${NC}\n"

[ "${SUITE_FAIL}" -eq 0 ] && exit 0 || exit 1
