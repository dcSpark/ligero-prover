//! Utilities to run the Ligero WebGPU prover/verifier binaries.
//!
//! This crate is intentionally "just a runner": it shells out to `webgpu_prover` / `webgpu_verifier`,
//! writes/reads expected artifacts (e.g. `proof_data.gz`) and provides light path-discovery with
//! environment-variable overrides.


mod config;
mod paths;
mod runner;

pub mod sovereign_host;
pub mod verifier;

pub use config::{LigeroArg, LigeroConfig};
pub use paths::LigeroPaths;
pub use runner::{LigeroRunner, ProverRunOptions};
