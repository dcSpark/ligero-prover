#!/bin/bash
# run.sh - Run the Ligero HTTP Server Docker container
#
# Usage:
#   ./run.sh [--detach] [--port PORT] [--workers N] [--gpu]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

DETACH=""
PORT="1313"
WORKERS=""
GPU_MODE=""
EXTRA_ARGS=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -d|--detach)
            DETACH="-d"
            ;;
        -p|--port)
            shift
            PORT="$1"
            ;;
        -w|--workers)
            shift
            WORKERS="$1"
            ;;
        --gpu)
            GPU_MODE="1"
            ;;
        -h|--help)
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  -d, --detach      Run container in background"
            echo "  -p, --port PORT   Host port to bind (default: 1313)"
            echo "  -w, --workers N   Number of HTTP/prover/verifier workers"
            echo "  --gpu             Enable GPU mode (requires nvidia-container-toolkit)"
            echo "  -h, --help        Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
    shift
done

# Build command args
CMD_ARGS=""
if [[ -n "$WORKERS" ]]; then
    CMD_ARGS="--threads $WORKERS --prover-workers $WORKERS --verifier-workers $WORKERS"
fi

# Build docker run command
DOCKER_ARGS=(
    --rm
    --name ligero-http-server
    -p "$PORT:1313"
    -e RUST_LOG=info
)

if [[ -n "$DETACH" ]]; then
    DOCKER_ARGS+=($DETACH)
fi

if [[ -n "$GPU_MODE" ]]; then
    # GPU mode: use nvidia runtime
    DOCKER_ARGS+=(
        --gpus all
    )
    echo "Running with GPU acceleration..."
else
    # CPU mode: use Mesa lavapipe software Vulkan
    # VK_ICD_FILENAMES is auto-detected by entrypoint.sh based on architecture
    DOCKER_ARGS+=(
        -e MESA_VK_DEVICE_SELECT=llvmpipe
        -e WGPU_TIMEOUT_MS=120000
    )
    echo "Running with software Vulkan (lavapipe)..."
fi

echo "========================================"
echo "Starting Ligero HTTP Server"
echo "========================================"
echo "Port: $PORT"
echo "Container: ligero-http-server"
if [[ -n "$WORKERS" ]]; then
    echo "Workers: $WORKERS"
fi
echo ""

# Run the container
if [[ -n "$CMD_ARGS" ]]; then
    docker run "${DOCKER_ARGS[@]}" ligero-http-server:latest \
        /app/bin/ligero-http-server --bind 0.0.0.0:1313 --proof-outputs /app/proof_outputs $CMD_ARGS
else
    docker run "${DOCKER_ARGS[@]}" ligero-http-server:latest
fi
