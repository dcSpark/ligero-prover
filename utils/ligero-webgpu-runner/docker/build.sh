#!/bin/bash
# build.sh - Build the Ligero HTTP Server Docker image
#
# Usage:
#   ./build.sh [--arch linux-amd64|linux-arm64] [--no-cache]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Auto-detect architecture based on host
HOST_ARCH=$(uname -m)
case "$HOST_ARCH" in
    x86_64|amd64)
        ARCH="linux-amd64"
        ;;
    aarch64|arm64)
        ARCH="linux-arm64"
        ;;
    *)
        echo "Warning: Unknown host architecture $HOST_ARCH, defaulting to linux-amd64"
        ARCH="linux-amd64"
        ;;
esac

NO_CACHE=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --arch)
            shift
            ARCH="$1"
            ;;
        --no-cache)
            NO_CACHE="--no-cache"
            ;;
        -h|--help)
            echo "Usage: $0 [--arch linux-amd64|linux-arm64] [--no-cache]"
            echo ""
            echo "Options:"
            echo "  --arch      Architecture (default: linux-amd64)"
            echo "  --no-cache  Build without Docker cache"
            echo "  -h, --help  Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
    shift
done

# Validate architecture
case "$ARCH" in
    linux-amd64|linux-arm64)
        ;;
    *)
        echo "Error: Unsupported architecture '$ARCH'"
        echo "Supported: linux-amd64, linux-arm64"
        exit 1
        ;;
esac

# Check if portable binaries exist
if [[ ! -f "$REPO_ROOT/utils/portable-binaries/$ARCH/bin/webgpu_prover" ]]; then
    echo "Error: Portable binaries not found for $ARCH"
    echo "Please run the following first:"
    echo "  bash utils/portable-binaries/build-portable-binaries.sh --arch $ARCH"
    exit 1
fi

echo "========================================"
echo "Building Ligero HTTP Server Docker Image"
echo "========================================"
echo "Architecture: $ARCH"
echo "Repository:   $REPO_ROOT"
echo ""

# Build the Docker image
cd "$REPO_ROOT"

docker build \
    $NO_CACHE \
    --build-arg ARCH="$ARCH" \
    -t ligero-http-server:latest \
    -t ligero-http-server:$ARCH \
    -f utils/ligero-webgpu-runner/docker/Dockerfile \
    .

echo ""
echo "========================================"
echo "Build complete!"
echo "========================================"
echo ""
echo "To run the server:"
echo "  docker run -p 1313:1313 ligero-http-server:latest"
echo ""
echo "Or with docker-compose:"
echo "  cd utils/ligero-webgpu-runner/docker && docker-compose up"
echo ""
echo "To test:"
echo "  ./utils/ligero-webgpu-runner/scripts/test_prove_verify.sh http://localhost:1313"
