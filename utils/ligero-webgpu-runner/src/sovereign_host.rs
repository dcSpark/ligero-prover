//! Sovereign-friendly Ligero host utilities.
//!
//! This module intentionally does **not** depend on Sovereign traits. Sovereign-side crates can
//! wrap/compose this type to implement `ZkvmHost` and package results as needed.

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::LigeroRunner;
use crate::LigeroProofPackage;

/// Core host implementation for Ligero zkVM, suitable for wrapping by Sovereign adapters.
#[derive(Clone, Debug)]
pub struct LigeroHostCore {
    runner: LigeroRunner,
    public_output: Option<Vec<u8>>,
}

impl LigeroHostCore {
    /// Create a new host with the given WASM program path.
    pub fn new(program_path: &str) -> Self {
        Self {
            runner: LigeroRunner::new(program_path),
            public_output: None,
        }
    }

    /// Access the underlying runner (immutable).
    pub fn runner(&self) -> &LigeroRunner {
        &self.runner
    }

    /// Access the underlying runner (mutable).
    pub fn runner_mut(&mut self) -> &mut LigeroRunner {
        &mut self.runner
    }

    /// Set the packing size.
    pub fn with_packing(mut self, packing: u32) -> Self {
        self.runner = self.runner.clone().with_packing(packing);
        self
    }

    /// Set private argument indices (1-based).
    pub fn with_private_indices(mut self, indices: Vec<usize>) -> Self {
        self.runner = self.runner.clone().with_private_indices(indices);
        self
    }

    /// Set a custom identifier for the proof directory (for deterministic paths).
    pub fn with_proof_dir_id(mut self, id: String) -> Self {
        self.runner = self.runner.clone().with_proof_dir_id(id);
        self
    }

    /// Set a custom identifier for the proof directory (mutable version).
    pub fn set_proof_dir_id(&mut self, id: String) {
        self.runner.set_proof_dir_id(id);
    }

    /// Record the public output that will be embedded in the proof package.
    ///
    /// The value is serialized using `bincode` so the verifier can recover it.
    pub fn set_public_output<T: Serialize>(&mut self, value: &T) -> Result<()> {
        let bytes =
            bincode::serialize(value).context("Failed to serialize Ligero public output with bincode")?;
        self.public_output = Some(bytes);
        Ok(())
    }

    /// Record the public output using raw bytes.
    pub fn set_public_output_bytes(&mut self, bytes: Vec<u8>) {
        self.public_output = Some(bytes);
    }

    /// Get the recorded public output bytes (if set).
    pub fn public_output_bytes(&self) -> Option<&[u8]> {
        self.public_output.as_deref()
    }

    /// Add a string argument.
    pub fn add_str_arg(&mut self, value: String) {
        self.runner.add_str_arg(value);
    }

    /// Add an i64 argument.
    pub fn add_i64_arg(&mut self, value: i64) {
        self.runner.add_i64_arg(value);
    }

    /// Add a u64 argument (encoded as i64; guest checks non-negative).
    pub fn add_u64_arg(&mut self, value: u64) {
        assert!(
            value <= i64::MAX as u64,
            "u64 value too large for i64 encoding"
        );
        self.runner.add_i64_arg(value as i64);
    }

    /// Add a hex argument.
    ///
    /// Accepts either raw hex or `0x`-prefixed hex; stores without the prefix.
    pub fn add_hex_arg(&mut self, value: String) {
        let hex = value
            .strip_prefix("0x")
            .or_else(|| value.strip_prefix("0X"))
            .map(|s| s.to_string())
            .unwrap_or(value);
        self.runner.add_hex_arg(hex);
    }

    /// Get the program path used by this host.
    pub fn program_path(&self) -> &str {
        &self.runner.config().program
    }

    /// Get the shader path used by this host.
    pub fn shader_path(&self) -> &str {
        &self.runner.config().shader_path
    }

    /// Get the packing parameter used by this host.
    pub fn packing(&self) -> u32 {
        self.runner.config().packing
    }

    /// Get the verifier binary path (best-effort discovered by the runner).
    pub fn verifier_bin(&self) -> &std::path::PathBuf {
        &self.runner.paths().verifier_bin
    }

    /// Return true if the discovered prover binary exists.
    pub fn prover_available(&self) -> bool {
        self.runner.paths().prover_bin.exists()
    }

    /// Return true if the discovered verifier binary exists.
    pub fn verifier_available(&self) -> bool {
        self.runner.paths().verifier_bin.exists()
    }

    /// Verify a proof for the current config (smoke check).
    ///
    /// Returns `true` if the verifier prints a successful result.
    pub fn verify_proof_smoke(&self) -> Result<bool> {
        self.runner.verify_proof_smoke()
    }

    /// Verify a proof for the current config (smoke check).
    ///
    /// Alias of [`Self::verify_proof_smoke`] kept for compatibility with downstream wrappers.
    pub fn verify_proof(&self) -> Result<bool> {
        self.verify_proof_smoke()
    }


    /// Compute the Ligero code commitment as `SHA-256(wasm_bytes || packing_u32_le)`.
    pub fn code_commitment_raw(&self) -> [u8; 32] {
        let program_bytes = std::fs::read(&self.runner.config().program).unwrap_or_default();

        let mut hasher = Sha256::new();
        hasher.update(&program_bytes);
        hasher.update(&self.runner.config().packing.to_le_bytes());

        let hash = hasher.finalize();
        let mut commitment = [0u8; 32];
        commitment.copy_from_slice(&hash);
        commitment
    }

    /// Run the prover and return the compressed proof bytes.
    pub fn run_prover(&self) -> Result<Vec<u8>> {
        self.runner.run_prover()
    }

    /// Run the prover and return `(proof_bytes, stdout)`.
    pub fn run_prover_with_output(&self) -> Result<(Vec<u8>, String)> {
        let (proof, stdout, _stderr) = self
            .runner
            .run_prover_with_output(crate::ProverRunOptions::default())?;
        Ok((proof, stdout))
    }

    /// Produce a bincode-serialized [`LigeroProofPackage`] without running the prover.
    ///
    /// This is useful for tests that validate packaging/redaction behavior in environments where
    /// WebGPU is unavailable.
    pub fn run_simulation(&self) -> Result<Vec<u8>> {
        let public_output = self.public_output.clone().unwrap_or_default();
        let cfg = self.runner.config();
        let pkg = LigeroProofPackage::new(
            Vec::new(),
            public_output,
            cfg.args.clone(),
            cfg.private_indices.clone(),
        );
        pkg.to_bytes()
    }

    /// Run the prover and return a bincode-serialized [`LigeroProofPackage`].
    ///
    /// NOTE: private arguments are redacted in the packaged args.
    pub fn run_prover_as_package(&self) -> Result<Vec<u8>> {
        let proof = self.run_prover()?;
        let public_output = self.public_output.clone().unwrap_or_default();
        let cfg = self.runner.config();
        let pkg = LigeroProofPackage::new(
            proof,
            public_output,
            cfg.args.clone(),
            cfg.private_indices.clone(),
        );
        pkg.to_bytes()
    }

    /// Ensure public output has been set and clone it (for packaging outside this crate).
    pub fn require_public_output(&self) -> Result<Vec<u8>> {
        self.public_output.clone().ok_or_else(|| {
            anyhow!("Ligero public output not set; call set_public_output before generating a proof")
        })
    }
}


