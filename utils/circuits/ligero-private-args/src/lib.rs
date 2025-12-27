//! # Ligero Private Args
//!
//! A standalone crate for working with Ligero proofs, including:
//!
//! - **Argument handling**: Types for prover/verifier arguments with privacy support
//! - **Redaction**: Logic to redact private arguments while preserving type/length
//! - **Host orchestration**: Run the WebGPU prover/verifier binaries
//! - **Proof packaging**: Serialize/deserialize proof packages with bincode
//!
//! ## Quick Start
//!
//! ### Redacting Private Arguments
//!
//! ```rust
//! use ligero_private_args::{LigeroArg, redact_private_args};
//!
//! let mut args = vec![
//!     LigeroArg::string("public_data"),
//!     LigeroArg::hex("deadbeef"),  // This will be private
//!     LigeroArg::i64(42),
//! ];
//!
//! // Mark argument at index 2 as private (1-based indexing)
//! redact_private_args(&mut args, &[2]);
//!
//! // The hex argument is now redacted
//! assert!(matches!(&args[1], LigeroArg::Hex { hex } if hex == "00000000"));
//! ```
//!
//! ### Running the Prover (requires binaries)
//!
//! ```rust,no_run
//! use ligero_private_args::LigeroHost;
//!
//! let mut host = LigeroHost::new("path/to/program.wasm");
//! host.add_i64_arg(42);
//! host.add_hex_arg("deadbeef");
//! host.set_private_indices(vec![2]); // Mark hex arg as private
//! host.set_public_output(&"output")?;
//!
//! // Generate proof (requires webgpu_prover binary)
//! let proof_bytes = host.run_prover()?;
//! # Ok::<(), ligero_private_args::host::Error>(())
//! ```
//!
//! ## Security
//!
//! The proof package is serialized and may be transmitted or stored. Without
//! redaction, anyone who receives the proof bytes can decode the package and
//! read the private values, breaking privacy even if the verifier never sees them.
//!
//! ## Features
//!
//! - `host` (default): Include the host module for prover/verifier orchestration

mod args;
mod config;
pub mod host;
mod proof;

pub use args::{redact_private_args, redacted_args, LigeroArg};
pub use config::LigeroConfig;
pub use host::LigeroHost;
pub use proof::LigeroProofPackage;
