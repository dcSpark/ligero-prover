# ligero-private-args

A standalone crate for working with Ligero proofs, including private argument handling, redaction, and prover/verifier orchestration.

## Features

- **Argument handling**: Types for prover/verifier arguments with privacy support
- **Redaction**: Logic to redact private arguments while preserving type/length
- **Host orchestration**: Run the WebGPU prover/verifier binaries
- **Proof packaging**: Serialize/deserialize proof packages with bincode

## Usage

### Redacting Private Arguments

```rust
use ligero_private_args::{LigeroArg, redact_private_args};

let mut args = vec![
    LigeroArg::string("public_data"),
    LigeroArg::hex("deadbeef"),  // This will be private
    LigeroArg::i64(42),
];

// Mark argument at index 2 as private (1-based indexing)
redact_private_args(&mut args, &[2]);

// The hex argument is now redacted with same-length placeholder
assert!(matches!(&args[1], LigeroArg::Hex { hex } if hex == "00000000"));
```

### Running the Prover

```rust,no_run
use ligero_private_args::LigeroHost;

let mut host = LigeroHost::new("path/to/program.wasm");
host.add_i64_arg(42);
host.add_hex_arg("deadbeef");
host.set_private_indices(vec![2]); // Mark hex arg as private
host.set_public_output(&"output")?;

// Check if binaries are available
if host.prover_available() {
    // Generate proof (requires webgpu_prover binary)
    let proof_bytes = host.run_prover()?;
}
```

### Simulation Mode

```rust,no_run
use ligero_private_args::LigeroHost;

let mut host = LigeroHost::new("path/to/program.wasm");
host.add_i64_arg(42);
host.set_public_output_bytes(vec![]);

// Simulation mode works without binaries
let proof_bytes = host.run_simulation()?;
```

## Redaction Rules

Private arguments are replaced with same-length placeholders to preserve type consistency:

| Type | Redacted Value |
|------|----------------|
| `String` | `"_"` repeated to match length |
| `I64` | `0` |
| `Hex` | `"0"` repeated to match length |

## Security

The proof package is serialized and may be transmitted or stored. Without redaction, anyone who receives the proof bytes can decode the package and read the private values, breaking privacy even if the verifier never sees them.

## API

### Core Types

- `LigeroArg` — Enum representing argument types (`String`, `I64`, `Hex`)
- `LigeroConfig` — Configuration for prover/verifier
- `LigeroHost` — Orchestrates prover/verifier execution
- `LigeroProofPackage` — Serializable proof package

### Functions

- `redact_private_args(&mut args, &indices)` — Redact arguments in place
- `redacted_args(&args, &indices)` — Return a redacted copy

## Binaries

The `bins/` directory contains pre-built WebGPU prover/verifier binaries for:

- macOS (Apple Silicon): `bins/macos/bin/`
- Linux (AMD64): `bins/linux-amd64/bin/`

Also includes:
- WASM programs: `bins/programs/`
- Shaders: `bins/*/shader/`

## Running Tests

```bash
# Run unit tests (no binaries needed)
cargo test

# Run E2E tests (requires binaries and WebGPU)
cargo test -- --ignored
```

## License

Apache-2.0
