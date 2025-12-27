//! Host implementation for running Ligero prover/verifier.
//!
//! This module provides the [`LigeroHost`] struct for orchestrating
//! proof generation and verification using the WebGPU-based prover/verifier binaries.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{LigeroArg, LigeroConfig, LigeroProofPackage, redacted_args};

/// Result type for host operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for host operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Bincode serialization error
    #[error("Bincode error: {0}")]
    Bincode(#[from] bincode::Error),

    /// Prover execution failed
    #[error("Prover failed: {0}")]
    ProverFailed(String),

    /// Verifier execution failed
    #[error("Verifier failed: {0}")]
    VerifierFailed(String),

    /// WASM program exited with non-zero code
    #[error("WASM program exited with code {0}: {1}")]
    WasmExitCode(i32, String),

    /// Missing configuration
    #[error("Missing configuration: {0}")]
    MissingConfig(String),

    /// Binary not found
    #[error("Binary not found: {0}")]
    BinaryNotFound(String),
}

/// Host for Ligero prover/verifier orchestration.
///
/// This struct manages the configuration and execution of the WebGPU-based
/// prover and verifier binaries.
///
/// # Example
///
/// ```rust,no_run
/// use ligero_private_args::LigeroHost;
///
/// let mut host = LigeroHost::new("path/to/program.wasm");
/// host.add_i64_arg(42);
/// host.add_hex_arg("deadbeef");
/// host.set_private_indices(vec![2]); // Mark hex arg as private
///
/// // Generate proof
/// let proof_package = host.run_prover()?;
/// # Ok::<(), ligero_private_args::host::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct LigeroHost {
    config: LigeroConfig,
    prover_bin: PathBuf,
    verifier_bin: PathBuf,
    bins_dir: PathBuf,
    public_output: Option<Vec<u8>>,
    proof_dir_id: Option<String>,
}

impl LigeroHost {
    /// Create a new LigeroHost with the given WASM program path.
    ///
    /// Automatically discovers the bins directory and shader path.
    pub fn new(program_path: impl Into<String>) -> Self {
        let bins_dir = Self::find_bins_dir();

        // Use absolute path for shader_path to work from any working directory
        let shader_path = if bins_dir.ends_with("bin") {
            // If using platform-specific bin directory, shader is at parent level
            bins_dir
                .parent()
                .unwrap_or(&bins_dir)
                .canonicalize()
                .unwrap_or_else(|_| bins_dir.parent().unwrap_or(&bins_dir).to_path_buf())
                .join("shader")
                .to_string_lossy()
                .to_string()
        } else {
            // If using generic bins directory, shader is at same level
            bins_dir
                .canonicalize()
                .unwrap_or_else(|_| bins_dir.clone())
                .join("shader")
                .to_string_lossy()
                .to_string()
        };

        Self {
            config: LigeroConfig::new(program_path).with_shader_path(shader_path),
            prover_bin: bins_dir.join("webgpu_prover"),
            verifier_bin: bins_dir.join("webgpu_verifier"),
            bins_dir,
            public_output: None,
            proof_dir_id: None,
        }
    }

    /// Create a new LigeroHost with explicit paths.
    pub fn with_paths(
        program_path: impl Into<String>,
        bins_dir: impl Into<PathBuf>,
        shader_path: impl Into<String>,
    ) -> Self {
        let bins_dir = bins_dir.into();
        Self {
            config: LigeroConfig::new(program_path).with_shader_path(shader_path),
            prover_bin: bins_dir.join("webgpu_prover"),
            verifier_bin: bins_dir.join("webgpu_verifier"),
            bins_dir,
            public_output: None,
            proof_dir_id: None,
        }
    }

    /// Find the bins directory.
    fn find_bins_dir() -> PathBuf {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");

        // Check for platform-specific binaries first (they take priority)
        #[cfg(target_os = "macos")]
        {
            let macos_bins = PathBuf::from(manifest_dir).join("bins/macos/bin");
            if macos_bins.join("webgpu_prover").exists()
                && macos_bins.join("webgpu_verifier").exists()
            {
                return macos_bins;
            }
        }

        #[cfg(target_os = "linux")]
        {
            let linux_bins = PathBuf::from(manifest_dir).join("bins/linux-amd64/bin");
            if linux_bins.join("webgpu_prover").exists()
                && linux_bins.join("webgpu_verifier").exists()
            {
                return linux_bins;
            }
        }

        // Try to find relative to the crate root (generic bins directory)
        let bins_dir = PathBuf::from(manifest_dir).join("bins");
        if bins_dir.join("webgpu_prover").exists() && bins_dir.join("webgpu_verifier").exists() {
            return bins_dir;
        }

        // Fallback to current directory
        PathBuf::from("bins")
    }

    /// Check if the prover binary is available.
    pub fn prover_available(&self) -> bool {
        self.prover_bin.exists()
    }

    /// Check if the verifier binary is available.
    pub fn verifier_available(&self) -> bool {
        self.verifier_bin.exists()
    }

    /// Set the packing size.
    pub fn with_packing(mut self, packing: u32) -> Self {
        self.config.packing = packing;
        self
    }

    /// Set private argument indices (1-based).
    pub fn with_private_indices(mut self, indices: Vec<usize>) -> Self {
        self.config.private_indices = indices;
        self
    }

    /// Set private argument indices (1-based).
    pub fn set_private_indices(&mut self, indices: Vec<usize>) {
        self.config.private_indices = indices;
    }

    /// Set a custom identifier for the proof directory.
    ///
    /// This is useful for debugging and ensures proof directories have meaningful names.
    pub fn with_proof_dir_id(mut self, id: impl Into<String>) -> Self {
        self.proof_dir_id = Some(id.into());
        self
    }

    /// Set a custom identifier for the proof directory (mutable version).
    pub fn set_proof_dir_id(&mut self, id: impl Into<String>) {
        self.proof_dir_id = Some(id.into());
    }

    /// Set the public output that will be embedded in the proof package.
    ///
    /// The value is serialized using `bincode`.
    pub fn set_public_output<T: serde::Serialize>(&mut self, value: &T) -> Result<()> {
        let bytes = bincode::serialize(value)?;
        self.public_output = Some(bytes);
        Ok(())
    }

    /// Set the public output using raw bytes.
    pub fn set_public_output_bytes(&mut self, bytes: Vec<u8>) {
        self.public_output = Some(bytes);
    }

    /// Add a string argument.
    pub fn add_str_arg(&mut self, value: impl Into<String>) {
        self.config.add_str_arg(value);
    }

    /// Add an i64 argument.
    pub fn add_i64_arg(&mut self, value: i64) {
        self.config.add_i64_arg(value);
    }

    /// Add a u64 argument (stored as i64).
    ///
    /// # Panics
    ///
    /// Panics if value > i64::MAX
    pub fn add_u64_arg(&mut self, value: u64) {
        self.config.add_u64_arg(value);
    }

    /// Add a hex argument.
    ///
    /// Strips `0x` or `0X` prefix if present.
    pub fn add_hex_arg(&mut self, value: impl Into<String>) {
        self.config.add_hex_arg(value);
    }

    /// Get the program path.
    pub fn program_path(&self) -> &str {
        &self.config.program
    }

    /// Get the shader path.
    pub fn shader_path(&self) -> &str {
        &self.config.shader_path
    }

    /// Get the packing value.
    pub fn packing(&self) -> u32 {
        self.config.packing
    }

    /// Get the private indices.
    pub fn private_indices(&self) -> &[usize] {
        &self.config.private_indices
    }

    /// Get the arguments.
    pub fn args(&self) -> &[LigeroArg] {
        &self.config.args
    }

    /// Get the verifier binary path.
    pub fn verifier_bin(&self) -> &Path {
        &self.verifier_bin
    }

    /// Get the prover binary path.
    pub fn prover_bin(&self) -> &Path {
        &self.prover_bin
    }

    /// Get the bins directory.
    pub fn bins_dir(&self) -> &Path {
        &self.bins_dir
    }

    /// Get the configuration.
    pub fn config(&self) -> &LigeroConfig {
        &self.config
    }

    /// Compute the code commitment (SHA-256 of WASM + packing).
    pub fn code_commitment(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};

        let program_bytes = std::fs::read(&self.config.program).unwrap_or_default();

        let mut hasher = Sha256::new();
        hasher.update(&program_bytes);
        hasher.update(&self.config.packing.to_le_bytes());

        let hash = hasher.finalize();
        let mut commitment = [0u8; 32];
        commitment.copy_from_slice(&hash);
        commitment
    }

    /// Run the prover and generate a proof package.
    ///
    /// Returns the serialized proof package (bincode).
    pub fn run_prover(&mut self) -> Result<Vec<u8>> {
        let public_output = self.public_output.clone().ok_or_else(|| {
            Error::MissingConfig(
                "Public output not set; call set_public_output before generating a proof"
                    .to_string(),
            )
        })?;

        let (proof, _, _) = self.run_prover_internal()?;

        let package = LigeroProofPackage::new(
            proof,
            public_output,
            &self.config.args,
            self.config.private_indices.clone(),
        )?;

        Ok(package.to_bytes()?)
    }

    /// Run the prover with detailed logging output.
    ///
    /// Returns the serialized proof package and the prover stdout.
    pub fn run_prover_with_logging(&mut self) -> Result<(Vec<u8>, String)> {
        let public_output = self.public_output.clone().ok_or_else(|| {
            Error::MissingConfig(
                "Public output not set; call set_public_output before generating a proof"
                    .to_string(),
            )
        })?;

        let (proof, stdout, _) = self.run_prover_internal()?;

        let package = LigeroProofPackage::new(
            proof,
            public_output,
            &self.config.args,
            self.config.private_indices.clone(),
        )?;

        Ok((package.to_bytes()?, stdout))
    }

    /// Internal prover implementation.
    fn run_prover_internal(&self) -> Result<(Vec<u8>, String, String)> {
        if !self.prover_bin.exists() {
            return Err(Error::BinaryNotFound(
                self.prover_bin.display().to_string(),
            ));
        }

        let config_json = serde_json::to_string(&self.config)?;

        // Create a deterministic directory for this proof
        let dir_name = if let Some(ref id) = self.proof_dir_id {
            format!("ligero_proof_{}", id)
        } else {
            format!("ligero_proof_{:?}", std::thread::current().id())
        };

        let proof_outputs_base = std::env::current_dir()?.join("proof_outputs");
        let unique_proof_dir = proof_outputs_base.join(dir_name);
        std::fs::create_dir_all(&unique_proof_dir)?;

        let output = Command::new(&self.prover_bin)
            .arg(&config_json)
            .current_dir(&unique_proof_dir)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let _ = std::fs::remove_dir_all(&unique_proof_dir);
            return Err(Error::ProverFailed(format!(
                "Prover failed with status {:?}\nstdout: {}\nstderr: {}",
                output.status.code(),
                stdout,
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();

        // Check WASM exit code
        for line in stdout.lines() {
            if line.contains("Exit with code") {
                if let Some(code_str) = line.strip_prefix("Exit with code ") {
                    if let Ok(code) = code_str.trim().parse::<i32>() {
                        if code != 0 {
                            let _ = std::fs::remove_dir_all(&unique_proof_dir);
                            return Err(Error::WasmExitCode(
                                code,
                                "WASM program exited with non-zero code".to_string(),
                            ));
                        }
                    }
                }
            }
        }

        if !stdout.contains("Final prove result:                  true") {
            let _ = std::fs::remove_dir_all(&unique_proof_dir);
            return Err(Error::ProverFailed(
                "Prover did not produce a valid proof".to_string(),
            ));
        }

        // Read the proof from proof_data.gz
        let proof_path = unique_proof_dir.join("proof_data.gz");
        let proof = std::fs::read(&proof_path)?;

        // Clean up
        let _ = std::fs::remove_dir_all(&unique_proof_dir);

        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Ok((proof, stdout, stderr))
    }

    /// Run the verifier on a proof.
    ///
    /// The `proof_dir` should contain `proof_data.gz`.
    pub fn run_verifier(&self, proof_dir: &Path) -> Result<bool> {
        if !self.verifier_bin.exists() {
            return Err(Error::BinaryNotFound(
                self.verifier_bin.display().to_string(),
            ));
        }

        // Create verifier config with redacted private args
        let mut verifier_config = self.config.clone();
        verifier_config.args = redacted_args(&self.config.args, &self.config.private_indices);

        let config_json = serde_json::to_string(&verifier_config)?;

        let output = Command::new(&self.verifier_bin)
            .arg(&config_json)
            .current_dir(proof_dir)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let success =
            output.status.success() && stdout.contains("Final Verify Result:                 true");

        Ok(success)
    }

    /// Run the verifier with detailed logging output.
    ///
    /// Returns (success, stdout).
    pub fn run_verifier_with_logging(&self, proof_dir: &Path) -> Result<(bool, String)> {
        if !self.verifier_bin.exists() {
            return Err(Error::BinaryNotFound(
                self.verifier_bin.display().to_string(),
            ));
        }

        // Create verifier config with redacted private args
        let mut verifier_config = self.config.clone();
        verifier_config.args = redacted_args(&self.config.args, &self.config.private_indices);

        let config_json = serde_json::to_string(&verifier_config)?;

        let output = Command::new(&self.verifier_bin)
            .arg(&config_json)
            .current_dir(proof_dir)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let success =
            output.status.success() && stdout.contains("Final Verify Result:                 true");

        Ok((success, stdout))
    }

    /// Run in simulation mode (no actual proof generation).
    ///
    /// Returns a proof package with empty proof data.
    pub fn run_simulation(&self) -> Result<Vec<u8>> {
        let public_output = self.public_output.clone().unwrap_or_default();

        let package = LigeroProofPackage::new(
            vec![], // Empty proof for simulation
            public_output,
            &self.config.args,
            self.config.private_indices.clone(),
        )?;

        Ok(package.to_bytes()?)
    }
}
