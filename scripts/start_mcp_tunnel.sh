#!/bin/bash
# Helper script to start TizenClaw MCP Server via SDB tunnel
# Run this on your host machine (PC)

DEVICE_SERIAL=$(sdb devices | grep -v "List" | head -n 1 | awk '{print $1}')

if [ -z "$DEVICE_SERIAL" ]; then
    echo "Error: No Tizen device/emulator found via sdb."
    exit 1
fi

echo "Connecting to TizenClaw MCP Server on $DEVICE_SERIAL..."
echo "Press Ctrl+C to stop."

# We run the mcp server script directly via sdb shell.
# stdio will be piped through sdb.
sdb -s "$DEVICE_SERIAL" shell "python3 /opt/usr/share/tizenclaw/skills/mcp_server/server.py"
