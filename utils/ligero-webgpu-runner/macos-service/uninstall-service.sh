#!/bin/bash
# Uninstall Ligero HTTP Server macOS launchd service

set -e

PLIST_NAME="com.dcspark.ligero-http-server.plist"
LAUNCH_AGENTS_DIR="$HOME/Library/LaunchAgents"
PLIST_PATH="$LAUNCH_AGENTS_DIR/$PLIST_NAME"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}Uninstalling Ligero HTTP Server service...${NC}"

if [ -f "$PLIST_PATH" ]; then
    echo "Stopping service..."
    launchctl unload "$PLIST_PATH" 2>/dev/null || true
    
    echo "Removing plist..."
    rm -f "$PLIST_PATH"
    
    echo -e "${GREEN}✓ Service uninstalled${NC}"
else
    echo -e "${YELLOW}Service plist not found at: $PLIST_PATH${NC}"
    echo "Service may not be installed."
fi

echo ""
echo "Note: Log files are preserved in the logs/ directory."
echo "To remove them: rm -rf <runner-dir>/logs/"
