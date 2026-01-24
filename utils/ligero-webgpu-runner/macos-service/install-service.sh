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
SERVICE_UID=$(id -u)

# Paths
BINARY_PATH="$RUNNER_DIR/target/release/ligero-http-server"
PLIST_NAME="com.dcspark.ligero-http-server.plist"
LAUNCH_AGENTS_DIR="$HOME/Library/LaunchAgents"
PLIST_PATH="$LAUNCH_AGENTS_DIR/$PLIST_NAME"
LOG_DIR="$RUNNER_DIR/logs"
PORTABLE_BINARIES="$LIGERO_ROOT/utils/portable-binaries"

echo -e "${YELLOW}Step 1: Building release binary...${NC}"
cd "$RUNNER_DIR"
cargo build --release --bin ligero-http-server
echo -e "${GREEN}✓ Build complete${NC}"
echo ""

# Verify binary exists and is executable
if [ ! -x "$BINARY_PATH" ]; then
    echo -e "${RED}ERROR: Binary not found or not executable: $BINARY_PATH${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Binary verified: $BINARY_PATH${NC}"
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

echo -e "${YELLOW}Step 3: Creating log directory and files...${NC}"
mkdir -p "$LOG_DIR"
# Create empty log files (launchd needs them to exist)
touch "$LOG_DIR/ligero-http-server.log"
touch "$LOG_DIR/ligero-http-server.error.log"
echo -e "${GREEN}✓ Log directory created: $LOG_DIR${NC}"
echo ""

echo -e "${YELLOW}Step 4: Generating customized plist...${NC}"
mkdir -p "$LAUNCH_AGENTS_DIR"

cat > "$PLIST_PATH" << EOF
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
    <true/>

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

echo -e "${GREEN}✓ Plist generated: $PLIST_PATH${NC}"
echo ""

# Validate plist syntax
echo -e "${YELLOW}Validating plist syntax...${NC}"
if plutil -lint "$PLIST_PATH" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Plist syntax is valid${NC}"
else
    echo -e "${RED}ERROR: Plist syntax is invalid:${NC}"
    plutil -lint "$PLIST_PATH"
    exit 1
fi
echo ""

echo -e "${YELLOW}Step 5: Loading service...${NC}"

# Use modern launchctl commands (bootout/bootstrap)
# First, try to stop any existing service
echo "Stopping existing service (if any)..."
launchctl bootout "gui/${SERVICE_UID}/com.dcspark.ligero-http-server" 2>/dev/null || true
# Also try legacy unload
launchctl unload "$PLIST_PATH" 2>/dev/null || true

sleep 1

# Load the service using bootstrap (modern approach)
echo "Starting service..."
if launchctl bootstrap "gui/${SERVICE_UID}" "$PLIST_PATH" 2>&1; then
    echo -e "${GREEN}✓ Service loaded via bootstrap${NC}"
else
    # Fallback to legacy load command
    echo "Bootstrap failed, trying legacy load..."
    if launchctl load -w "$PLIST_PATH" 2>&1; then
        echo -e "${GREEN}✓ Service loaded via legacy load${NC}"
    else
        echo -e "${RED}Failed to load service. Trying manual start...${NC}"
    fi
fi
echo ""

# Wait a moment for service to start
sleep 3

echo -e "${YELLOW}Checking service status...${NC}"
SERVICE_STATUS=$(launchctl list 2>/dev/null | grep "com.dcspark.ligero-http-server" || true)
if [ -n "$SERVICE_STATUS" ]; then
    echo -e "${GREEN}✓ Service is registered:${NC}"
    echo "  $SERVICE_STATUS"
    
    # Check if process is actually running
    PID=$(echo "$SERVICE_STATUS" | awk '{print $1}')
    if [ "$PID" != "-" ] && [ -n "$PID" ]; then
        echo -e "${GREEN}✓ Process is running (PID: $PID)${NC}"
    else
        echo -e "${YELLOW}⚠ Service registered but process not running yet${NC}"
        echo "  Check logs for startup errors:"
        echo "    cat $LOG_DIR/ligero-http-server.error.log"
    fi
else
    echo -e "${RED}✗ Service not found in launchctl list${NC}"
    echo ""
    echo "Debugging info:"
    echo "  Plist location: $PLIST_PATH"
    echo "  Binary location: $BINARY_PATH"
    echo ""
    echo "Try manual start to see errors:"
    echo "  $BINARY_PATH --bind $BIND_ADDRESS"
fi
echo ""

# Check if server is responding
echo -e "${YELLOW}Testing server endpoint...${NC}"
sleep 2
if curl -s --connect-timeout 2 "http://127.0.0.1:${BIND_ADDRESS##*:}/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Server is responding to health checks!${NC}"
else
    echo -e "${YELLOW}⚠ Server not responding yet (may still be starting)${NC}"
    echo "  Check: curl http://127.0.0.1:${BIND_ADDRESS##*:}/health"
fi
echo ""

echo "============================================="
echo -e "${GREEN}Installation complete!${NC}"
echo ""
echo "Service management commands:"
echo "  Stop:    launchctl bootout gui/${SERVICE_UID}/com.dcspark.ligero-http-server"
echo "  Start:   launchctl bootstrap gui/${SERVICE_UID} $PLIST_PATH"
echo "  Status:  launchctl list | grep ligero"
echo "  Logs:    tail -f $LOG_DIR/ligero-http-server.log"
echo "  Errors:  tail -f $LOG_DIR/ligero-http-server.error.log"
echo ""
echo "Server endpoint: http://127.0.0.1:${BIND_ADDRESS##*:}"
echo "  Health check:  curl http://127.0.0.1:${BIND_ADDRESS##*:}/health"
echo ""
echo "To run manually for debugging:"
echo "  LIGERO_ROOT=$LIGERO_ROOT $BINARY_PATH --bind $BIND_ADDRESS"
echo ""
