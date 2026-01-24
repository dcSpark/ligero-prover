#!/bin/bash
# Simple process supervisor - keeps ligero-http-server running forever
# Use this if launchd doesn't work (e.g., on headless EC2 Macs)
#
# Usage:
#   nohup ./run-forever.sh > /dev/null 2>&1 &
#   # Or in a tmux/screen session:
#   ./run-forever.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUNNER_DIR="$(dirname "$SCRIPT_DIR")"
LIGERO_ROOT="$(cd "$RUNNER_DIR/../.." && pwd)"

# Configuration
BIND_ADDRESS="${LIGERO_BIND_ADDRESS:-0.0.0.0:1313}"
PROVER_WORKERS="${LIGERO_PROVER_WORKERS:-4}"
VERIFIER_WORKERS="${LIGERO_VERIFIER_WORKERS:-4}"
RESTART_DELAY="${LIGERO_RESTART_DELAY:-5}"

BINARY_PATH="$RUNNER_DIR/target/release/ligero-http-server"
LOG_DIR="$RUNNER_DIR/logs"

# Ensure log directory exists
mkdir -p "$LOG_DIR"

# Remove quarantine from binaries (Gatekeeper)
PORTABLE_BINARIES="$LIGERO_ROOT/utils/portable-binaries"
if [ -d "$PORTABLE_BINARIES/macos-arm64" ]; then
    xattr -dr com.apple.quarantine "$PORTABLE_BINARIES/macos-arm64/" 2>/dev/null || true
fi

# Build if binary doesn't exist
if [ ! -x "$BINARY_PATH" ]; then
    echo "Building ligero-http-server..."
    cd "$RUNNER_DIR"
    cargo build --release --bin ligero-http-server
fi

export LIGERO_ROOT
export RUST_LOG="${RUST_LOG:-info}"

echo "Starting ligero-http-server supervisor"
echo "  Binary: $BINARY_PATH"
echo "  Bind: $BIND_ADDRESS"
echo "  Workers: $PROVER_WORKERS provers, $VERIFIER_WORKERS verifiers"
echo "  Logs: $LOG_DIR"
echo ""
echo "Press Ctrl+C to stop"
echo ""

# Trap to handle clean shutdown
cleanup() {
    echo ""
    echo "Shutting down..."
    if [ -n "$SERVER_PID" ]; then
        kill "$SERVER_PID" 2>/dev/null || true
        wait "$SERVER_PID" 2>/dev/null || true
    fi
    exit 0
}
trap cleanup SIGINT SIGTERM

# Main supervisor loop
while true; do
    echo "[$(date)] Starting ligero-http-server..."
    
    "$BINARY_PATH" \
        --bind "$BIND_ADDRESS" \
        --prover-workers "$PROVER_WORKERS" \
        --verifier-workers "$VERIFIER_WORKERS" \
        >> "$LOG_DIR/ligero-http-server.log" 2>> "$LOG_DIR/ligero-http-server.error.log" &
    
    SERVER_PID=$!
    echo "[$(date)] Server started (PID: $SERVER_PID)"
    
    # Wait for the server to exit
    wait "$SERVER_PID" || true
    EXIT_CODE=$?
    
    echo "[$(date)] Server exited with code $EXIT_CODE"
    echo "[$(date)] Restarting in $RESTART_DELAY seconds..."
    sleep "$RESTART_DELAY"
done
