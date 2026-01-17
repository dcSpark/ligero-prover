# Ligero WebGPU Runner

Rust helper library and binaries to run the Ligero `webgpu_prover` / `webgpu_verifier` binaries and manage proof artifacts.

This crate is intentionally "just a runner": it shells out to the native WebGPU binaries, writes/reads expected artifacts (e.g., `proof_data.gz` or `proof_data.bin`), and provides light path-discovery with environment-variable overrides.

## Installation

```bash
cargo build --release
```

## Binaries

This crate provides three executable binaries:

### 1. `ligero-http-server`

A synchronous HTTP server that provides RESTful endpoints for zero-knowledge proof generation and verification.

**Usage:**

```bash
ligero-http-server [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `-b, --bind <ADDR>` | Bind address (default: `127.0.0.1:1313`) |
| `--proof-outputs <PATH>` | Base directory for proof outputs |
| `--keep-proof-dir` | Keep proof directories after completion |
| `-t, --threads <N>` | Number of worker threads (default: CPU count) |
| `-h, --help` | Show help message |

**Endpoints:**

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/prove` | POST | Generate a zero-knowledge proof |
| `/verify` | POST | Verify an existing proof |
| `/health` | GET/POST | Health check |

**Request Format:**

```json
{
  "circuit": "note_spend",
  "args": [
    { "str": "value" },
    { "i64": 123 },
    { "hex": "deadbeef" }
  ],
  "proof": null,
  "privateIndices": [1, 2]
}
```

- `circuit`: Circuit name (e.g., `note_spend`, `note_deposit`, `value_validator`)
- `args`: Array of typed arguments (`str`, `i64`, `hex`, or `bytes_b64`)
- `proof`: Base64-encoded proof (null for `/prove`, required for `/verify`)
- `privateIndices`: Optional 1-based indices of private arguments

**Response Format:**

```json
{
  "success": true,
  "exitCode": 0,
  "proof": "base64...",
  "error": null
}
```

**Example:**

```bash
# Start the server
ligero-http-server -b 0.0.0.0:1313

# Generate a proof
curl -X POST http://localhost:1313/prove \
  -H "Content-Type: application/json" \
  -d '{"circuit": "note_spend", "args": [...], "privateIndices": [1,2,3]}'

# Verify a proof
curl -X POST http://localhost:1313/verify \
  -H "Content-Type: application/json" \
  -d '{"circuit": "note_spend", "args": [...], "proof": "H4sIAAAA...", "privateIndices": [1,2,3]}'

# Health check
curl http://localhost:1313/health
```

---

### 2. `ligero-webgpu-daemon-server`

A long-running daemon server that maintains pools of `webgpu_prover` and `webgpu_verifier` child processes for high-throughput proving/verification. Supports both Unix sockets and TCP connections using a length-prefixed binary protocol.

**Usage:**

```bash
ligero-webgpu-daemon-server [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--unix <PATH>` | Unix socket path to listen on |
| `--tcp <ADDR>` | TCP address to listen on |
| `--prover-workers <N>` | Number of prover worker processes (default: CPU count) |
| `--verifier-workers <N>` | Number of verifier worker processes (default: CPU count) |
| `-h, --help` | Show help message |

**Protocol:**

Uses length-prefixed framing (4-byte big-endian length + JSON payload) for communication.

**Request Types:**

```json
// Prove request
{ "kind": "prove", "id": "optional-id", "config": { /* LigeroConfig */ } }

// Verify request
{ "kind": "verify", "id": "optional-id", "config": { /* LigeroConfig */ }, "proof_path": "/path/to/proof" }

// Health check
{ "kind": "health", "id": "optional-id" }
```

**Example:**

```bash
# Start with Unix socket
ligero-webgpu-daemon-server --unix /tmp/ligero.sock --prover-workers 4 --verifier-workers 4

# Start with TCP
ligero-webgpu-daemon-server --tcp 127.0.0.1:7777

# Start with both
ligero-webgpu-daemon-server --unix /tmp/ligero.sock --tcp 127.0.0.1:7777
```

---

### 3. `continuous_transfers_bench`

A throughput benchmark for "Midnight privacy transfers" using the `note_spend` circuit. This benchmark runs without the Sovereign SDK to measure raw prover/verifier performance.

**What it does (per round):**
1. Build a Merkle tree of N input notes (one per wallet)
2. Generate N note-spend proofs in parallel (daemon mode)
3. Verify N note-spend proofs in parallel (daemon mode)
4. Print total and per-proof timings for prove and verify

**Usage:**

```bash
continuous_transfers_bench
```

The benchmark prompts interactively for configuration or uses environment variables as defaults.

**Environment Variables:**

| Variable | Default | Description |
|----------|---------|-------------|
| `TRANSFER_N` | 16 | Number of proofs per round |
| `TRANSFER_ROUNDS` | 1 | Number of rounds |
| `TRANSFER_TREE_DEPTH` | 16 | Merkle tree depth |
| `LIGERO_PACKING` | 8192 | FFT message packing size |
| `LIGERO_GZIP_PROOF` | false | Whether to gzip proof files |
| `LIGERO_PROVER_WORKERS` | 1 | Prover concurrency |
| `LIGERO_VERIFIER_WORKERS` | 1 | Verifier concurrency |

**Example:**

```bash
# Run with defaults
continuous_transfers_bench

# Run with custom settings
TRANSFER_N=32 LIGERO_PROVER_WORKERS=8 continuous_transfers_bench
```

---

## Environment Variables

Common environment variables for path discovery and configuration:

| Variable | Description |
|----------|-------------|
| `LIGERO_ROOT` | Path to the ligero-prover repository root |
| `LIGERO_PROGRAMS_DIR` | Directory containing `.wasm` circuit files |
| `LIGERO_SHADER_PATH` | Path to shader directory |
| `LIGERO_PROVER_BIN` | Full path to `webgpu_prover` binary |
| `LIGERO_VERIFIER_BIN` | Full path to `webgpu_verifier` binary |
| `LIGERO_PACKING` | FFT packing size (default: 8192) |
| `LIGERO_KEEP_PROOF_DIR` | Set to `1` to keep proof directories for debugging |

## Library Usage

The crate also provides a library for programmatic use:

```rust
use ligero_runner::{LigeroRunner, ProverRunOptions, LigeroArg};

// Create a runner for a circuit
let mut runner = LigeroRunner::new("note_spend_guest");

// Add arguments
runner.add_hex_arg("deadbeef".to_string());
runner.add_i64_arg(42);
runner.add_str_arg("hello".to_string());

// Configure private indices (1-based)
runner = runner.with_private_indices(vec![1, 2]);

// Run the prover
let proof_bytes = runner.run_prover()?;

// Verify the proof
use ligero_runner::verifier::{verify_proof, VerifierPaths};

let vpaths = VerifierPaths::discover_with_commitment(None)?;
verify_proof(&vpaths, &proof_bytes, args, private_indices)?;
```

## Running Tests

```bash
# Run all tests
cargo test

# Run HTTP server tests specifically
cargo test --test http_server_test -- --nocapture

# Run with a specific test
cargo test test_http_server_health_endpoint -- --nocapture
```

## License

Apache-2.0
