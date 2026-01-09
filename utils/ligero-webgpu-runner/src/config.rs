//! Utilities to run the Ligero WebGPU prover/verifier binaries.
//!
//! This crate is intentionally "just a runner": it shells out to `webgpu_prover` / `webgpu_verifier`,
//! writes/reads expected artifacts (e.g. `proof_data.gz` or `proof_data.bin`) and provides light path-discovery with
//! environment-variable overrides.

use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

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
    /// Backwards/forwards compatible byte argument: carries both `hex` and `bytes_b64`.
    ///
    /// - New binaries will prefer `bytes_b64` and pass raw bytes to the guest.
    /// - Older binaries will ignore `bytes_b64` and fall back to `hex`.
    ///
    /// This is the recommended encoding for 32-byte values used by the circuits.
    #[serde(rename = "hex")]
    HexBytesB64 {
        /// Hex string value (no `0x` prefix required).
        hex: String,
        /// Base64-encoded raw bytes.
        bytes_b64: String,
    },
    /// Base64-encoded raw bytes argument.
    #[serde(rename = "bytes_b64")]
    BytesB64 {
        /// Base64-encoded raw bytes.
        bytes_b64: String,
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
    /// Whether to gzip-compress the proof file output.
    ///
    /// When true (default), the prover writes `proof_data.gz` and the verifier expects gzip.
    /// When false, the prover writes an uncompressed proof file and the verifier will read it
    /// without gzip decompression.
    #[serde(rename = "gzip-proof", default = "default_true")]
    pub gzip_proof: bool,
    /// Optional override for where the prover/verifier read/write proof bytes.
    ///
    /// If set, the prover will write proof bytes to this path and the verifier will read from it.
    #[serde(rename = "proof-path", skip_serializing_if = "Option::is_none")]
    pub proof_path: Option<String>,
    /// Indices of private arguments (1-based).
    #[serde(rename = "private-indices")]
    pub private_indices: Vec<usize>,
    /// Program arguments.
    pub args: Vec<LigeroArg>,
}
