//! Proof package types for Ligero.

use serde::{Deserialize, Serialize};

use crate::LigeroArg;

/// A Ligero proof package containing both the proof and metadata.
///
/// This struct is serialized with bincode and can be stored or transmitted.
/// Private arguments are automatically redacted before being included.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LigeroProofPackage {
    /// The compressed Ligero proof bytes (from proof_data.gz - boost serialized + gzipped).
    pub proof: Vec<u8>,

    /// Serialized public output committed by the guest program.
    pub public_output: Vec<u8>,

    /// Arguments passed to the guest program (JSON-serialized).
    ///
    /// Private arguments are redacted with placeholder values.
    pub args_json: Vec<u8>,

    /// Indices of private arguments (1-based).
    ///
    /// These indices indicate which arguments in `args_json` are redacted.
    pub private_indices: Vec<usize>,
}

impl LigeroProofPackage {
    /// Create a new proof package.
    ///
    /// # Arguments
    ///
    /// * `proof` - The raw proof bytes (gzipped)
    /// * `public_output` - Serialized public output
    /// * `args` - The arguments (will be stored as JSON)
    /// * `private_indices` - Indices of private arguments (1-based)
    pub fn new(
        proof: Vec<u8>,
        public_output: Vec<u8>,
        args: &[LigeroArg],
        private_indices: Vec<usize>,
    ) -> Result<Self, serde_json::Error> {
        // Redact private args before storing
        let redacted = crate::redacted_args(args, &private_indices);
        let args_json = serde_json::to_vec(&redacted)?;

        Ok(Self {
            proof,
            public_output,
            args_json,
            private_indices,
        })
    }

    /// Deserialize the proof package from bincode bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }

    /// Serialize the proof package to bincode bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Get the redacted arguments.
    pub fn args(&self) -> Result<Vec<LigeroArg>, serde_json::Error> {
        serde_json::from_slice(&self.args_json)
    }

    /// Check if this is an empty/simulation proof (no actual proof data).
    pub fn is_simulation(&self) -> bool {
        self.proof.is_empty()
    }

    /// Check if the proof data appears to be valid gzip format.
    pub fn is_valid_gzip(&self) -> bool {
        self.proof.len() >= 2 && self.proof[0] == 0x1f && self.proof[1] == 0x8b
    }
}
