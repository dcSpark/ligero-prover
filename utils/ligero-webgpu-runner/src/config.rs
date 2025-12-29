//! Utilities to run the Ligero WebGPU prover/verifier binaries.
//!
//! This crate is intentionally "just a runner": it shells out to `webgpu_prover` / `webgpu_verifier`,
//! writes/reads expected artifacts (e.g. `proof_data.gz`) and provides light path-discovery with
//! environment-variable overrides.

use serde::{Deserialize, Serialize};

/// Argument type for Ligero prover.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum LigeroArg {
    /// String argument.
    #[serde(rename = "str")]
    String {
        /// String value.
        str: String,
    },
    /// i64 argument.
    #[serde(rename = "i64")]
    I64 {
        /// i64 value.
        i64: i64,
    },
    /// Hex argument.
    #[serde(rename = "hex")]
    Hex {
        /// Hex string value.
        hex: String,
    },
}

/// Configuration for Ligero prover/verifier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LigeroConfig {
    /// Path to the WASM program.
    pub program: String,
    /// Path to shader directory.
    #[serde(rename = "shader-path")]
    pub shader_path: String,
    /// Optional GPU thread count override.
    #[serde(rename = "gpu-threads", skip_serializing_if = "Option::is_none")]
    pub gpu_threads: Option<u32>,
    /// Packing size (FFT message packing size).
    pub packing: u32,
    /// Indices of private arguments (1-based).
    #[serde(rename = "private-indices")]
    pub private_indices: Vec<usize>,
    /// Program arguments.
    pub args: Vec<LigeroArg>,
}
