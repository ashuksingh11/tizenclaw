#!/bin/bash
set -e

echo "=========================================================="
echo "          [E2E Test] Tizen TPK/RPM Deployment Verifier"
echo "=========================================================="

echo "[E2E Step 1] Triggering deploy.sh to compile and push natively..."
./deploy.sh

if ! command -v sdb &>/dev/null; then
  for _sdb_candidate in "${HOME}/tizen-studio/tools" "${HOME}/tizen-studio/tools/emulator/bin" "/opt/tizen-studio/tools" "/usr/local/tizen-studio/tools"; do
    if [ -x "${_sdb_candidate}/sdb" ]; then
      export PATH="${_sdb_candidate}:${PATH}"
      break
    fi
  done
fi

echo ""
echo "[E2E Step 2] Querying sdb shell for systemctl service status..."
# Strip carriage returns from the sdb stdout
STATUS=$(sdb shell systemctl is-active tizenclaw | tr -d '\r')

if [ "$STATUS" != "active" ]; then
    echo "[E2E FATAL] Service failed to start or crashed on the emulator!"
    echo "[E2E FATAL] Current systemctl status: $STATUS"
    
    echo ""
    echo "--- [Recent Journald Logs] ---"
    sdb shell journalctl -u tizenclaw --no-pager | tail -n 20
    echo "------------------------------"
    exit 1
fi

echo "[E2E OK] Service 'tizenclaw' is actively running on the target device."

echo ""
echo "[E2E Step 3] Extracting recent daemon stdout..."
sdb shell journalctl -u tizenclaw --no-pager | tail -n 15

echo ""
echo "=========================================================="
echo " [E2E SUCCESS] All Pipelines and Deployment Hooks Passed!"
echo "=========================================================="
exit 0
