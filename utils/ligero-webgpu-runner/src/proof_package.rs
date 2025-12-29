//! Proof packaging utilities.
//!
//! This crate deliberately avoids Sovereign-specific types, but it is still useful to have a
//! lightweight “proof package” format for tests and debugging.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::{redacted_args, LigeroArg};

/// A compact proof package suitable for logging, tests, and simple integrations.
///
/// IMPORTANT: `args` MUST already be redacted for the indices in `private_indices`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LigeroProofPackage {
    /// Compressed Ligero proof bytes (`proof_data.gz`).
    ///
    /// If empty, this is considered “simulation mode”.
    pub proof: Vec<u8>,
    /// Public output bytes (caller-defined).
    pub public_output: Vec<u8>,
    /// Redacted arguments passed to the guest program, serialized as JSON.
    ///
    /// We store JSON bytes instead of `Vec<LigeroArg>` because `LigeroArg` is `#[serde(untagged)]`,
    /// and bincode v1 cannot deserialize untagged enums.
    pub args_json: Vec<u8>,
    /// Indices of private arguments (1-based).
    pub private_indices: Vec<usize>,
}

impl LigeroProofPackage {
    pub fn new(
        proof: Vec<u8>,
        public_output: Vec<u8>,
        args: Vec<LigeroArg>,
        private_indices: Vec<usize>,
    ) -> Self {
        let args = redacted_args(&args, &private_indices);
        let args_json = serde_json::to_vec(&args).unwrap_or_default();
        Self {
            proof,
            public_output,
            private_indices,
            args_json,
        }
    }

    pub fn is_simulation(&self) -> bool {
        self.proof.is_empty()
    }

    pub fn args(&self) -> Result<Vec<LigeroArg>> {
        serde_json::from_slice(&self.args_json).context("Failed to parse LigeroProofPackage args_json")
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).context("Failed to bincode-serialize LigeroProofPackage")
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).context("Failed to bincode-deserialize LigeroProofPackage")
    }

    pub fn is_valid_gzip(&self) -> bool {
        // gzip magic: 1f 8b
        self.proof.len() >= 2 && self.proof[0] == 0x1f && self.proof[1] == 0x8b
    }
}


