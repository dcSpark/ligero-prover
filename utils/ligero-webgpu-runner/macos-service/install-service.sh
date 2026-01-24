#!/bin/bash
# Install Ligero HTTP Server as a macOS launchd service
#
# This script:
# 1. Builds the release binary
# 2. Removes quarantine from binaries (Gatekeeper)
# 3. Creates log directory
# 4. Generates a customized plist
# 5. Installs and starts the service

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUNNER_DIR="$(dirname "$SCRIPT_DIR")"
LIGERO_ROOT="$(cd "$RUNNER_DIR/../.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Ligero HTTP Server - macOS Service Installer${NC}"
echo "============================================="
echo ""
echo "Ligero root: $LIGERO_ROOT"
echo "Runner dir:  $RUNNER_DIR"
echo ""

# Configuration - can be overridden via environment
BIND_ADDRESS="${LIGERO_BIND_ADDRESS:-0.0.0.0:1313}"
PROVER_WORKERS="${LIGERO_PROVER_WORKERS:-4}"
VERIFIER_WORKERS="${LIGERO_VERIFIER_WORKERS:-4}"
SERVICE_USER="${USER}"
SERVICE_HOME="${HOME}"

# Paths
BINARY_PATH="$RUNNER_DIR/target/release/ligero-http-server"
PLIST_NAME="com.dcspark.ligero-http-server.plist"
LAUNCH_AGENTS_DIR="$HOME/Library/LaunchAgents"
LOG_DIR="$RUNNER_DIR/logs"
PORTABLE_BINARIES="$LIGERO_ROOT/utils/portable-binaries"

echo -e "${YELLOW}Step 1: Building release binary...${NC}"
cd "$RUNNER_DIR"
cargo build --release --bin ligero-http-server
echo -e "${GREEN}✓ Build complete${NC}"
echo ""

echo -e "${YELLOW}Step 2: Removing macOS quarantine from binaries...${NC}"
# Remove quarantine from portable binaries (Gatekeeper protection)
if [ -d "$PORTABLE_BINARIES/macos-arm64" ]; then
    xattr -dr com.apple.quarantine "$PORTABLE_BINARIES/macos-arm64/bin/" 2>/dev/null || true
    xattr -dr com.apple.quarantine "$PORTABLE_BINARIES/macos-arm64/lib/" 2>/dev/null || true
    echo -e "${GREEN}✓ Quarantine removed from macos-arm64 binaries${NC}"
fi
if [ -d "$PORTABLE_BINARIES/macos-x86_64" ]; then
    xattr -dr com.apple.quarantine "$PORTABLE_BINARIES/macos-x86_64/bin/" 2>/dev/null || true
    xattr -dr com.apple.quarantine "$PORTABLE_BINARIES/macos-x86_64/lib/" 2>/dev/null || true
    echo -e "${GREEN}✓ Quarantine removed from macos-x86_64 binaries${NC}"
fi
echo ""

echo -e "${YELLOW}Step 3: Creating log directory...${NC}"
mkdir -p "$LOG_DIR"
echo -e "${GREEN}✓ Log directory created: $LOG_DIR${NC}"
echo ""

echo -e "${YELLOW}Step 4: Generating customized plist...${NC}"
mkdir -p "$LAUNCH_AGENTS_DIR"

cat > "$LAUNCH_AGENTS_DIR/$PLIST_NAME" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.dcspark.ligero-http-server</string>

    <key>ProgramArguments</key>
    <array>
        <string>${BINARY_PATH}</string>
        <string>--bind</string>
        <string>${BIND_ADDRESS}</string>
        <string>--prover-workers</string>
        <string>${PROVER_WORKERS}</string>
        <string>--verifier-workers</string>
        <string>${VERIFIER_WORKERS}</string>
    </array>

    <key>WorkingDirectory</key>
    <string>${RUNNER_DIR}</string>

    <key>EnvironmentVariables</key>
    <dict>
        <key>LIGERO_ROOT</key>
        <string>${LIGERO_ROOT}</string>
        <key>RUST_LOG</key>
        <string>info</string>
        <key>HOME</key>
        <string>${SERVICE_HOME}</string>
    </dict>

    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
        <key>Crashed</key>
        <true/>
    </dict>

    <key>ThrottleInterval</key>
    <integer>5</integer>

    <key>RunAtLoad</key>
    <true/>

    <key>StandardOutPath</key>
    <string>${LOG_DIR}/ligero-http-server.log</string>

    <key>StandardErrorPath</key>
    <string>${LOG_DIR}/ligero-http-server.error.log</string>

    <key>ProcessType</key>
    <string>Interactive</string>

    <key>Nice</key>
    <integer>0</integer>
</dict>
</plist>
EOF

echo -e "${GREEN}✓ Plist generated: $LAUNCH_AGENTS_DIR/$PLIST_NAME${NC}"
echo ""

echo -e "${YELLOW}Step 5: Loading service...${NC}"
# Unload if already loaded (ignore errors)
launchctl unload "$LAUNCH_AGENTS_DIR/$PLIST_NAME" 2>/dev/null || true

# Load the service
launchctl load "$LAUNCH_AGENTS_DIR/$PLIST_NAME"
echo -e "${GREEN}✓ Service loaded${NC}"
echo ""

# Wait a moment for service to start
sleep 2

echo -e "${YELLOW}Checking service status...${NC}"
if launchctl list | grep -q "com.dcspark.ligero-http-server"; then
    echo -e "${GREEN}✓ Service is running!${NC}"
else
    echo -e "${RED}✗ Service may not have started. Check logs:${NC}"
    echo "  tail -f $LOG_DIR/ligero-http-server.log"
    echo "  tail -f $LOG_DIR/ligero-http-server.error.log"
fi
echo ""

echo "============================================="
echo -e "${GREEN}Installation complete!${NC}"
echo ""
echo "Service management commands:"
echo "  Stop:    launchctl unload ~/Library/LaunchAgents/$PLIST_NAME"
echo "  Start:   launchctl load ~/Library/LaunchAgents/$PLIST_NAME"
echo "  Status:  launchctl list | grep ligero"
echo "  Logs:    tail -f $LOG_DIR/ligero-http-server.log"
echo ""
echo "Server endpoint: http://${BIND_ADDRESS}"
echo "  Health check:  curl http://${BIND_ADDRESS}/health"
echo ""
