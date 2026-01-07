//! Utilities to run the Ligero WebGPU prover/verifier binaries.
//!
//! This crate is intentionally "just a runner": it shells out to `webgpu_prover` / `webgpu_verifier`,
//! writes/reads expected artifacts (e.g. `proof_data.gz` or `proof_data.bin`) and provides light path-discovery with
//! environment-variable overrides.

mod config;
pub mod daemon;
mod paths;
mod pool;
mod programs;
mod proof_package;
pub mod redaction;
mod runner;

pub mod sovereign_host;
pub mod verifier;

pub use config::{LigeroArg, LigeroConfig};
pub use paths::LigeroPaths;
pub use pool::{default_prover_pool, default_verifier_pool, BinaryWorkerPool};
pub use programs::resolve_program;
pub use proof_package::LigeroProofPackage;
pub use redaction::{redact_arg, redact_private_args, redacted_args};
pub use runner::{LigeroRunner, ProverRunOptions};
