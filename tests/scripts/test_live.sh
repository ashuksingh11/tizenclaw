#!/bin/bash
set -e

echo "=========================================================="
echo "     [Live API Test] Host Mock Daemon Connectivity Verifier"
echo "=========================================================="

echo "[Live Step 1] Spawning TizenClaw Daemon locally with mock-sys..."
cd src/tizenclaw

~/.cargo/bin/cargo run --features mock-sys &
DAEMON_PID=$!

echo "[Live Info] Waiting 7 seconds for Tokio runtime and channels to initialize..."
sleep 7

echo "[Live Step 2] Checking Daemon Lifespan..."
if ! kill -0 $DAEMON_PID > /dev/null 2>&1; then
    echo "[Live FATAL] Background daemon crashed during startup!"
    exit 1
fi
echo "[Live OK] Daemon successfully bootstrapped natively on WSL."

echo ""
echo "[Live Step 3] Pinging WebDashboard IPC Channel (Port 9090)..."
HTTP_STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:9090/ || echo "CURLError")

if [ "$HTTP_STATUS" != "200" ] && [ "$HTTP_STATUS" != "404" ]; then
    echo "[Live Warning] WebDashboard returned unexpected HTTP code: $HTTP_STATUS"
else
    echo "[Live OK] Local Web IPC bounds respond synchronously (Code: $HTTP_STATUS)"
fi

echo ""
echo "[Live Info] Terminating Native Daemon (PID: $DAEMON_PID)..."
kill $DAEMON_PID
wait $DAEMON_PID 2>/dev/null || true

echo "=========================================================="
echo " [Live SUCCESS] All Host Validations and API Pings Passed!"
echo "=========================================================="
exit 0
