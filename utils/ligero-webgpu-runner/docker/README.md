# Ligero HTTP Server Docker

This directory contains Docker configuration for running the Ligero HTTP proving/verification server.

## Quick Start

### 1. Build the Docker Image

```bash
# From the ligero-prover repository root:
cd utils/ligero-webgpu-runner/docker
./build.sh
```

Or manually:

```bash
docker build -t ligero-http-server -f utils/ligero-webgpu-runner/docker/Dockerfile .
```

### 2. Run the Server

```bash
# Interactive mode (see logs):
./run.sh

# Background mode:
./run.sh --detach

# Custom port:
./run.sh --port 8080
```

Or with docker-compose:

```bash
docker-compose up -d
```

### 3. Test the Server

From your host machine:

```bash
# First, build the test request generator if not already built:
cargo build --manifest-path=utils/ligero-webgpu-runner/Cargo.toml --bin generate-test-request --release

# Run the test script:
./utils/ligero-webgpu-runner/scripts/test_prove_verify.sh http://localhost:1313
```

## Architecture

The Docker container includes:

- **Portable Binaries**: Pre-built `webgpu_prover` and `webgpu_verifier` with bundled libraries
- **Software Vulkan**: Mesa's lavapipe for CPU-based Vulkan rendering (no GPU required)
- **Circuit WASM Files**: Pre-compiled circuits (`note_spend_guest.wasm`, etc.)
- **HTTP Server**: Rust-based `ligero-http-server` exposing `/prove` and `/verify` endpoints

### Software Rendering (Default)

By default, the container uses Mesa's **lavapipe** for software Vulkan rendering. This allows WebGPU to work without physical GPU access, but proving will be slower than with GPU acceleration.

### GPU Acceleration (Optional)

For better performance, you can use GPU acceleration if you have:

1. **NVIDIA GPU** with nvidia-container-toolkit installed:

```bash
./run.sh --gpu
```

Or uncomment the GPU service in `docker-compose.yml`.

## API Endpoints

### POST /prove

Generate a proof for a circuit.

```bash
curl -X POST http://localhost:1313/prove \
  -H "Content-Type: application/json" \
  -d '{
    "circuit": "note_spend",
    "args": [...],
    "privateIndices": [1, 2, 3]
  }'
```

Response:
```json
{
  "success": true,
  "exitCode": 0,
  "proof": "base64-encoded-proof..."
}
```

### POST /verify

Verify an existing proof.

```bash
curl -X POST http://localhost:1313/verify \
  -H "Content-Type: application/json" \
  -d '{
    "circuit": "note_spend",
    "args": [...],
    "privateIndices": [1, 2, 3],
    "proof": "base64-encoded-proof..."
  }'
```

Response:
```json
{
  "success": true,
  "exitCode": 0
}
```

### GET /health (or POST /health)

Health check endpoint.

```bash
curl http://localhost:1313/health
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Log level (trace, debug, info, warn, error) | `info` |
| `VK_ICD_FILENAMES` | Vulkan ICD file for device selection | (lavapipe) |
| `MESA_VK_DEVICE_SELECT` | Mesa device selection | `llvmpipe` |
| `WGPU_TIMEOUT_MS` | WebGPU operation timeout in ms | `60000` |

### Command Line Arguments

The HTTP server accepts these arguments:

| Argument | Description | Default |
|----------|-------------|---------|
| `-b, --bind` | Bind address | `127.0.0.1:1313` |
| `--proof-outputs` | Base directory for proof outputs | `./proof_outputs` |
| `--keep-proof-dir` | Keep proof directories after completion | false |
| `-t, --threads` | Number of HTTP worker threads | CPU count |
| `-p, --prover-workers` | Number of prover daemon workers | CPU count |
| `-v, --verifier-workers` | Number of verifier daemon workers | CPU count |

## Building for Different Architectures

### AMD64 (x86_64)

```bash
./build.sh --arch linux-amd64
```

### ARM64 (aarch64)

```bash
./build.sh --arch linux-arm64
```

## Troubleshooting

### "GPU/WebGPU likely unavailable"

This indicates that Vulkan is not working properly. Check:

1. The container has access to Mesa's lavapipe (`/usr/share/vulkan/icd.d/lvp_icd.x86_64.json`)
2. The environment variables are set correctly
3. Try running `vulkaninfo` inside the container to diagnose

### Slow Proving

Software rendering (lavapipe) is significantly slower than GPU acceleration. For production use, consider:

1. Using GPU passthrough with nvidia-container-toolkit
2. Running the server natively on a machine with GPU access
3. Increasing the `WGPU_TIMEOUT_MS` value for complex proofs

### Container Crashes

Check the logs:

```bash
docker logs ligero-http-server
```

Common issues:
- Missing shared libraries (ensure the image was built correctly)
- Insufficient memory (increase Docker memory limit)
- Timeout during WebGPU initialization (increase startup time)

## Development

### Rebuilding After Changes

```bash
./build.sh --no-cache
```

### Running with Custom Circuits

Mount your circuit directory:

```bash
docker run -p 1313:1313 \
  -v /path/to/circuits:/app/circuits \
  -e LIGERO_PROGRAMS_DIR=/app/circuits \
  ligero-http-server:latest
```
