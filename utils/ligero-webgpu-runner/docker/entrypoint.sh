#!/bin/bash
# entrypoint.sh - Docker entrypoint for Ligero HTTP Server
#
# This script sets up the correct Vulkan ICD path based on architecture
# and starts the HTTP server.

set -e

# Detect architecture and set Vulkan ICD path for lavapipe
ARCH=$(uname -m)
case "$ARCH" in
    x86_64|amd64)
        ICD_FILE="/usr/share/vulkan/icd.d/lvp_icd.x86_64.json"
        ;;
    aarch64|arm64)
        ICD_FILE="/usr/share/vulkan/icd.d/lvp_icd.aarch64.json"
        ;;
    *)
        echo "Warning: Unknown architecture $ARCH, trying to auto-detect Vulkan ICD"
        ICD_FILE=""
        ;;
esac

# Set the Vulkan ICD if the file exists and not already set
if [[ -z "${VK_ICD_FILENAMES:-}" && -n "$ICD_FILE" && -f "$ICD_FILE" ]]; then
    export VK_ICD_FILENAMES="$ICD_FILE"
    echo "Using Vulkan ICD: $VK_ICD_FILENAMES"
fi

# Log configuration
echo "========================================"
echo "Ligero HTTP Server Starting"
echo "========================================"
echo "Architecture:     $ARCH"
echo "Vulkan ICD:       ${VK_ICD_FILENAMES:-auto-detect}"
echo "Device Select:    ${MESA_VK_DEVICE_SELECT:-default}"
echo "Prover Binary:    ${LIGERO_PROVER_BIN:-default}"
echo "Shader Path:      ${LIGERO_SHADER_PATH:-default}"
echo "Programs Dir:     ${LIGERO_PROGRAMS_DIR:-default}"
echo "========================================"
echo ""

# Execute the main command
exec "$@"
