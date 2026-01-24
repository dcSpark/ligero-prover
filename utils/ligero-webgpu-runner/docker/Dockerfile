# Dockerfile for Ligero HTTP Server
# 
# This container provides the ligero-http-server for proving and verification.
# It includes software Vulkan rendering (lavapipe) for environments without GPU access.
#
# Build (auto-detects architecture):
#   ./utils/ligero-webgpu-runner/docker/build.sh
#
# Build manually:
#   # For Apple Silicon (ARM64):
#   docker build --build-arg ARCH=linux-arm64 -t ligero-http-server -f utils/ligero-webgpu-runner/docker/Dockerfile .
#   # For Intel/AMD (x86_64):
#   docker build --build-arg ARCH=linux-amd64 -t ligero-http-server -f utils/ligero-webgpu-runner/docker/Dockerfile .
#
# Run:
#   docker run -p 1313:1313 ligero-http-server

# ==============================================================================
# Stage 1: Build the Rust HTTP server binary
# ==============================================================================
FROM rust:1.83-bookworm AS rust-builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy only the necessary Rust source files for building
COPY sdk/rust /build/sdk/rust
COPY utils/ligero-webgpu-runner /build/utils/ligero-webgpu-runner

# Build the HTTP server and generate-test-request binaries
WORKDIR /build/utils/ligero-webgpu-runner
RUN cargo build --release --bin ligero-http-server --bin generate-test-request

# ==============================================================================
# Stage 2: Runtime container with Vulkan support
# ==============================================================================
# IMPORTANT: Must use Ubuntu 24.04 to match GLIBC 2.38 used to build portable binaries
FROM ubuntu:24.04 AS runtime

# Install runtime dependencies and Mesa for software Vulkan (lavapipe)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    # Vulkan runtime and ICD loader
    libvulkan1 \
    vulkan-tools \
    # Mesa software renderer (lavapipe for Vulkan)
    mesa-vulkan-drivers \
    # Additional libraries that might be needed
    libssl3 \
    libgmp10 \
    liblzma5 \
    libzstd1 \
    libbz2-1.0 \
    zlib1g \
    libx11-6 \
    libxcb1 \
    libwayland-client0 \
    # Python for test scripts
    python3 \
    && rm -rf /var/lib/apt/lists/*

# Create app directory structure
WORKDIR /app

# Copy portable Linux binaries (choose architecture based on build platform)
# Use --build-arg ARCH=linux-amd64 for Intel/AMD or ARCH=linux-arm64 for ARM
ARG ARCH=linux-arm64
COPY utils/portable-binaries/${ARCH}/bin/webgpu_prover /app/bin/webgpu_prover
COPY utils/portable-binaries/${ARCH}/bin/webgpu_verifier /app/bin/webgpu_verifier
COPY utils/portable-binaries/${ARCH}/lib/ /app/lib/

# Copy shader files (from portable-binaries which has a copy)
COPY utils/portable-binaries/shader/ /app/shader/

# Copy circuit WASM files
COPY utils/circuits/bins/*.wasm /app/circuits/

# Copy the Rust HTTP server binary
COPY --from=rust-builder /build/utils/ligero-webgpu-runner/target/release/ligero-http-server /app/bin/ligero-http-server
COPY --from=rust-builder /build/utils/ligero-webgpu-runner/target/release/generate-test-request /app/bin/generate-test-request

# Copy entrypoint script
COPY utils/ligero-webgpu-runner/docker/entrypoint.sh /app/entrypoint.sh

# Make binaries and scripts executable
RUN chmod +x /app/bin/* /app/entrypoint.sh

# Set up library paths for the bundled libraries
ENV LD_LIBRARY_PATH="/app/lib:${LD_LIBRARY_PATH}"

# Configure Ligero paths
ENV LIGERO_PROVER_BIN=/app/bin/webgpu_prover
ENV LIGERO_VERIFIER_BIN=/app/bin/webgpu_verifier
ENV LIGERO_SHADER_PATH=/app/shader
ENV LIGERO_PROGRAMS_DIR=/app/circuits

# Configure Vulkan to use lavapipe (Mesa software renderer)
# This allows WebGPU to work without physical GPU access
# Note: The ICD file path varies by architecture:
#   - x86_64: /usr/share/vulkan/icd.d/lvp_icd.x86_64.json
#   - aarch64: /usr/share/vulkan/icd.d/lvp_icd.aarch64.json
# We use a wildcard match via the startup script, or let Mesa auto-detect
ENV MESA_VK_DEVICE_SELECT=llvmpipe
# Increase timeout for software rendering (can be slower)
ENV WGPU_TIMEOUT_MS=120000

# Create directories for proof outputs
RUN mkdir -p /app/proof_outputs

# Expose the HTTP server port
EXPOSE 1313

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -f -X POST http://localhost:1313/health -H "Content-Type: application/json" -d '{}' || exit 1

# Use entrypoint script for architecture detection and setup
ENTRYPOINT ["/app/entrypoint.sh"]

# Default command: run the HTTP server
# Bind to 0.0.0.0 to accept connections from outside the container
CMD ["/app/bin/ligero-http-server", "--bind", "0.0.0.0:1313", "--proof-outputs", "/app/proof_outputs"]
