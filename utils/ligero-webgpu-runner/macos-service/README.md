# Ligero HTTP Server - macOS Service

This directory contains scripts to run `ligero-http-server` as a persistent macOS launchd service.

## Quick Start

```bash
# Install and start the service
chmod +x install-service.sh
./install-service.sh
```

The service will:
- Build the release binary
- Remove Gatekeeper quarantine from portable binaries
- Install a launchd plist to `~/Library/LaunchAgents/`
- Start the service immediately
- Auto-restart on crash or unexpected exit

## Configuration

Set environment variables before running the installer:

```bash
# Change bind address (default: 0.0.0.0:1313)
export LIGERO_BIND_ADDRESS="127.0.0.1:8080"

# Change worker counts (default: 4 each)
export LIGERO_PROVER_WORKERS=8
export LIGERO_VERIFIER_WORKERS=8

./install-service.sh
```

## Service Management

### Check Status
```bash
launchctl list | grep ligero
# PID column shows process ID if running, "-" if stopped
```

### Stop Service
```bash
launchctl unload ~/Library/LaunchAgents/com.dcspark.ligero-http-server.plist
```

### Start Service
```bash
launchctl load ~/Library/LaunchAgents/com.dcspark.ligero-http-server.plist
```

### Restart Service
```bash
launchctl unload ~/Library/LaunchAgents/com.dcspark.ligero-http-server.plist
launchctl load ~/Library/LaunchAgents/com.dcspark.ligero-http-server.plist
```

### View Logs
```bash
# Standard output
tail -f logs/ligero-http-server.log

# Error output
tail -f logs/ligero-http-server.error.log
```

### Uninstall
```bash
./uninstall-service.sh
```

## Troubleshooting

### SIGKILL (Signal 9) on Startup

This is usually macOS Gatekeeper blocking unsigned binaries. Fix:

```bash
# Remove quarantine from portable binaries
xattr -dr com.apple.quarantine ../portable-binaries/macos-arm64/
```

### Service Exits Immediately

Check the error log:
```bash
cat logs/ligero-http-server.error.log
```

Common issues:
- **Binary not found**: Ensure the binary is built (`cargo build --release`)
- **Port in use**: Another process is using port 1313
- **Missing dependencies**: Portable binaries not present

### Verify Binary Works Manually

Before installing as a service, test manually:
```bash
LIGERO_ROOT=/path/to/ligero-prover ./target/release/ligero-http-server
```

### GPU Access Issues

The service uses `ProcessType=Interactive` to ensure GPU access. If you still have issues:

1. Ensure you're logged in to the Mac (launchd agents need an active session)
2. For headless servers, you may need a system-level daemon instead (requires admin)

## Files

- `install-service.sh` - Builds binary, removes quarantine, installs and starts service
- `uninstall-service.sh` - Stops and removes the service
- `com.dcspark.ligero-http-server.plist` - Template plist (install script generates a customized version)

## Architecture Notes

This uses a **LaunchAgent** (user-level service) rather than a **LaunchDaemon** (system-level):

- **LaunchAgent**: Runs as your user, has access to user resources and GPU
- **LaunchDaemon**: Runs as root at boot, no GUI/GPU access by default

For GPU-based proving with Metal, LaunchAgent is the correct choice.
