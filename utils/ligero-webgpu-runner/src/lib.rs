//! Utilities to run the Ligero WebGPU prover/verifier binaries.
//!
//! This crate is intentionally "just a runner": it shells out to `webgpu_prover` / `webgpu_verifier`,
//! writes/reads expected artifacts (e.g. `proof_data.gz`) and provides light path-discovery with
//! environment-variable overrides.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Argument type for Ligero prover.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Paths to the Ligero binaries and shader directory.
#[derive(Debug, Clone)]
pub struct LigeroPaths {
    /// Path to `webgpu_prover`.
    pub prover_bin: PathBuf,
    /// Path to `webgpu_verifier`.
    pub verifier_bin: PathBuf,
    /// Path to `shader/` directory (or equivalent).
    pub shader_dir: PathBuf,
    /// Directory that contains the binaries (used as a default working directory).
    pub bins_dir: PathBuf,
}

/// Options controlling how the prover is executed and how artifacts are managed.
#[derive(Debug, Clone)]
pub struct ProverRunOptions {
    /// If true, keep the per-run proof directory (for debugging).
    pub keep_proof_dir: bool,
    /// Optional base directory under which proof directories are created.
    /// If None, uses `<cwd>/proof_outputs`.
    pub proof_outputs_base: Option<PathBuf>,
    /// If true, write a `last_prover_command.sh` replay script to the base directory.
    pub write_replay_script: bool,
}

impl Default for ProverRunOptions {
    fn default() -> Self {
        Self {
            keep_proof_dir: false,
            proof_outputs_base: None,
            write_replay_script: true,
        }
    }
}

/// A lightweight runner for the Ligero prover/verifier binaries.
///
/// This is suitable for embedding in other systems (e.g. Sovereign SDK adapters) where the actual
/// proving/verifying is performed by external binaries.
#[derive(Clone, Debug)]
pub struct LigeroRunner {
    config: LigeroConfig,
    paths: LigeroPaths,
    proof_dir_id: Option<String>,
}

impl LigeroRunner {
    /// Create a new runner with a WASM program path and default path discovery.
    ///
    /// Path discovery order:
    /// - `LIGERO_PROVER_BIN` or `LIGERO_PROVER_BINARY_PATH` (full path to `webgpu_prover`)
    /// - `LIGERO_ROOT` (repo/install root); checks `<root>/build`, `<root>/build-web`, `<root>/bins`
    /// - best-effort relative to this crate (`CARGO_MANIFEST_DIR` -> repo root heuristics)
    pub fn new(program_path: &str) -> Self {
        let paths = discover_paths().unwrap_or_else(|_| fallback_paths());

        // shader-path: allow override, else derive from discovered paths.
        let shader_path = std::env::var("LIGERO_SHADER_PATH")
            .ok()
            .and_then(|p| std::fs::canonicalize(&p).ok())
            .unwrap_or_else(|| paths.shader_dir.clone())
            .to_string_lossy()
            .to_string();

        Self {
            config: LigeroConfig {
                program: program_path.to_string(),
                shader_path,
                gpu_threads: None,
                packing: 8192,
                private_indices: vec![],
                args: vec![],
            },
            paths,
            proof_dir_id: None,
        }
    }

    /// Create a new runner using explicit binary/shader paths (no environment-variable discovery).
    pub fn new_with_paths(program_path: &str, paths: LigeroPaths) -> Self {
        let shader_path = std::env::var("LIGERO_SHADER_PATH")
            .ok()
            .and_then(|p| std::fs::canonicalize(&p).ok())
            .unwrap_or_else(|| paths.shader_dir.clone())
            .to_string_lossy()
            .to_string();

        Self {
            config: LigeroConfig {
                program: program_path.to_string(),
                shader_path,
                gpu_threads: None,
                packing: 8192,
                private_indices: vec![],
                args: vec![],
            },
            paths,
            proof_dir_id: None,
        }
    }


    /// Access the current config (immutable).
    pub fn config(&self) -> &LigeroConfig {
        &self.config
    }

    /// Access the current config (mutable).
    pub fn config_mut(&mut self) -> &mut LigeroConfig {
        &mut self.config
    }

    /// Access discovered paths.
    pub fn paths(&self) -> &LigeroPaths {
        &self.paths
    }

    /// Set the packing size.
    pub fn with_packing(mut self, packing: u32) -> Self {
        self.config.packing = packing;
        self
    }
    /// Set an explicit GPU thread count (serialized as `gpu-threads`).
    pub fn with_gpu_threads(mut self, gpu_threads: Option<u32>) -> Self {
        self.config.gpu_threads = gpu_threads;
        self
    }


    /// Set private argument indices (1-based).
    pub fn with_private_indices(mut self, indices: Vec<usize>) -> Self {
        self.config.private_indices = indices;
        self
    }

    /// Set a custom identifier for the proof directory (for deterministic paths).
    pub fn with_proof_dir_id(mut self, id: String) -> Self {
        self.proof_dir_id = Some(id);
        self
    }

    /// Set a custom identifier for the proof directory (mutable version).
    pub fn set_proof_dir_id(&mut self, id: String) {
        self.proof_dir_id = Some(id);
    }

    /// Add a string argument.
    pub fn add_str_arg(&mut self, value: String) {
        self.config.args.push(LigeroArg::String { str: value });
    }

    /// Add an i64 argument.
    pub fn add_i64_arg(&mut self, value: i64) {
        self.config.args.push(LigeroArg::I64 { i64: value });
    }

    /// Add a hex argument.
    pub fn add_hex_arg(&mut self, value: String) {
        self.config.args.push(LigeroArg::Hex { hex: value });
    }

    /// Run the prover and return the compressed proof bytes (the contents of `proof_data.gz`).
    ///
    /// The prover is executed in a per-run directory under `<cwd>/proof_outputs/` (unless overridden).
    pub fn run_prover(&self) -> Result<Vec<u8>> {
        self.run_prover_with_options(ProverRunOptions::default())
    }

    /// Run the prover with explicit options controlling artifact directories and cleanup.
    pub fn run_prover_with_options(&self, options: ProverRunOptions) -> Result<Vec<u8>> {
        let (proof, _stdout, _stderr) = self.run_prover_with_output(options)?;
        Ok(proof)
    }

    /// Run the prover and return `(proof_bytes, stdout, stderr)`.
    ///
    /// This is useful for tests/benchmarks that want to print prover output without re-implementing
    /// process management outside this crate.
    pub fn run_prover_with_output(
        &self,
        options: ProverRunOptions,
    ) -> Result<(Vec<u8>, String, String)> {
        let config_json =
            serde_json::to_string(&self.config).context("Failed to serialize Ligero config")?;

        tracing::debug!("Running Ligero prover with config: {}", config_json);

        // Create a deterministic directory for this proof in the project's proof_outputs folder.
        let dir_name = if let Some(ref id) = self.proof_dir_id {
            format!("ligero_proof_{}", id)
        } else {
            format!("ligero_proof_{:?}", std::thread::current().id())
        };

        let proof_outputs_base = if let Some(base) = options.proof_outputs_base.clone() {
            base
        } else {
            std::env::current_dir()
                .context("Failed to get current directory")?
                .join("proof_outputs")
        };

        let unique_proof_dir = proof_outputs_base.join(dir_name);
        std::fs::create_dir_all(&unique_proof_dir)
            .context("Failed to create unique proof directory")?;

        let keep_proof_dir = options.keep_proof_dir
            || std::env::var("LIGERO_KEEP_PROOF_DIR")
                .ok()
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);

        let output = Command::new(&self.paths.prover_bin)
            .arg(&config_json)
            .current_dir(&unique_proof_dir)
            .output()
            .with_context(|| format!("Failed to execute {}", self.paths.prover_bin.display()))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // Always persist prover logs for debugging (even on success).
        let _ = std::fs::write(unique_proof_dir.join("prover.stdout.log"), &stdout);
        let _ = std::fs::write(unique_proof_dir.join("prover.stderr.log"), &stderr);

        if !output.status.success() {
            anyhow::bail!(
                "Ligero prover failed with status {:?}\nproof dir: {}\nstdout: {}\nstderr: {}",
                output.status.code(),
                unique_proof_dir.display(),
                stdout,
                stderr
            );
        }

        // Read the proof from proof_data.gz (compressed - this goes into the transaction).
        let proof_path = unique_proof_dir.join("proof_data.gz");
        if !proof_path.exists() {
            anyhow::bail!(
                "Ligero prover did not produce proof_data.gz\nproof dir: {}\nNote: check prover.stdout.log / prover.stderr.log in that directory.",
                unique_proof_dir.display()
            );
        }
        let proof = std::fs::read(&proof_path).context("Failed to read proof_data.gz")?;
        if proof.is_empty() {
            anyhow::bail!(
                "Ligero prover produced an empty proof_data.gz\nproof dir: {}",
                unique_proof_dir.display()
            );
        }

        // Clean up after reading the proof unless user asked to keep it.
        if !keep_proof_dir {
            if let Err(e) = std::fs::remove_dir_all(&unique_proof_dir) {
                tracing::warn!("Failed to clean up proof directory: {}", e);
            }
        } else {
            tracing::info!(
                "Keeping Ligero proof directory for debugging (LIGERO_KEEP_PROOF_DIR=1): {}",
                unique_proof_dir.display()
            );
        }

    
        Ok((proof, stdout, stderr))
    }

    /// Run the verifier binary for the current config and return true if it prints a successful result.
    pub fn verify_proof_smoke(&self) -> Result<bool> {
        let config_json =
            serde_json::to_string(&self.config).context("Failed to serialize Ligero config")?;

        let output = Command::new(&self.paths.verifier_bin)
            .arg(&config_json)
            .current_dir(&self.paths.bins_dir)
            .output()
            .with_context(|| format!("Failed to execute {}", self.paths.verifier_bin.display()))?;

        if !output.status.success() {
            return Ok(false);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.contains("Final Verify Result:                 true"))
    }
}

fn discover_paths() -> Result<LigeroPaths> {
    // 1) explicit prover binary override
    if let Some(prover) = env_path("LIGERO_PROVER_BIN")
        .or_else(|| env_path("LIGERO_PROVER_BINARY_PATH"))
    {
        return Ok(paths_from_prover_bin(&prover));
    }

    // 2) explicit repo root
    if let Some(root) = env_path("LIGERO_ROOT") {
        if let Some(p) = find_bins_in_root(&root) {
            return Ok(p);
        }
    }

    // 3) heuristic: walk up from this crate towards the repo root (sdk/rust/ligero-webgpu-runner -> root)
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for ancestor in manifest_dir.ancestors().take(6) {
        if let Some(p) = find_bins_in_root(ancestor) {
            return Ok(p);
        }
    }

    anyhow::bail!("Could not discover Ligero WebGPU binaries; set LIGERO_PROVER_BIN or LIGERO_ROOT")
}

fn fallback_paths() -> LigeroPaths {
    // Minimal fallback: assume `webgpu_prover` and `webgpu_verifier` are in PATH and shader is `./shader`.
    LigeroPaths {
        prover_bin: PathBuf::from("webgpu_prover"),
        verifier_bin: PathBuf::from("webgpu_verifier"),
        shader_dir: PathBuf::from("shader"),
        bins_dir: PathBuf::from("."),
    }
}

fn env_path(key: &str) -> Option<PathBuf> {
    std::env::var(key)
        .ok()
        .and_then(|p| std::fs::canonicalize(&p).ok())
}

fn paths_from_prover_bin(prover_bin: &Path) -> LigeroPaths {
    let prover_bin = prover_bin.to_path_buf();
    let bins_dir = prover_bin
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let verifier_bin = bins_dir.join("webgpu_verifier");
    let shader_dir = find_shader_dir_near(&bins_dir).unwrap_or_else(|| PathBuf::from("shader"));

    LigeroPaths {
        prover_bin,
        verifier_bin,
        shader_dir,
        bins_dir,
    }
}

fn find_bins_in_root(root: &Path) -> Option<LigeroPaths> {
    // Common build outputs:
    // - <root>/build/webgpu_prover
    // - <root>/build-web/webgpu_prover
    // - <root>/bins/webgpu_prover  (some repos package portable binaries like this)
    // - <root>/utils/portable-binaries/<platform>/bin/webgpu_prover (dcSpark packaging)
    fn portable_platform_dir() -> Option<&'static str> {
        if cfg!(target_os = "macos") {
            if cfg!(target_arch = "aarch64") {
                Some("macos-arm64")
            } else {
                None
            }
        } else if cfg!(target_os = "linux") {
            if cfg!(target_arch = "aarch64") {
                Some("linux-arm64")
            } else if cfg!(target_arch = "x86_64") {
                Some("linux-amd64")
            } else {
                None
            }
        } else {
            None
        }
    }

    let portable_dir = portable_platform_dir().map(|p| {
        root.join("utils")
            .join("portable-binaries")
            .join(p)
            .join("bin")
    });

    let mut candidates: Vec<PathBuf> = Vec::with_capacity(6);
    if let Some(p) = portable_dir {
        candidates.push(p);
    }
    candidates.extend([
        root.join("build"),
        root.join("build-web"),
        root.join("bins"),
        root.join("bin"),
    ]);

    for dir in candidates {
        let prover = dir.join("webgpu_prover");
        let verifier = dir.join("webgpu_verifier");
        if prover.exists() && verifier.exists() {
            let shader_dir = find_shader_dir_near(&dir).unwrap_or_else(|| root.join("shader"));
            return Some(LigeroPaths {
                prover_bin: prover,
                verifier_bin: verifier,
                shader_dir,
                bins_dir: dir,
            });
        }
    }

    None
}

fn find_shader_dir_near(start: &Path) -> Option<PathBuf> {
    // look for `shader/` either in this dir or in ancestors (sibling of build dir).
    for ancestor in start.ancestors().take(6) {
        let candidate = ancestor.join("shader");
        if candidate.exists() {
            return Some(candidate);
        }
        // Sovereign-style packaging: bins/<platform>/bin + bins/shader
        if ancestor.ends_with("bin") {
            if let Some(bins) = ancestor.parent().and_then(|p| p.parent()) {
                let candidate2 = bins.join("shader");
                if candidate2.exists() {
                    return Some(candidate2);
                }
            }
        }
    }
    None
}




/// Utilities for running `webgpu_verifier` against an existing proof.
pub mod verifier {
    use super::{LigeroArg, LigeroConfig};

    use anyhow::{Context, Result};
    use sha2::{Digest, Sha256};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use tempfile::tempdir;

    /// Resolved verifier inputs.
    #[derive(Debug, Clone)]
    pub struct VerifierPaths {
        pub program: PathBuf,
        pub shader_path: PathBuf,
        pub verifier_bin: PathBuf,
        pub packing: u32,
    }

    impl VerifierPaths {
        /// Construct from explicit paths.
        pub fn from_explicit(
            program: PathBuf,
            shader_path: PathBuf,
            verifier_bin: PathBuf,
            packing: u32,
        ) -> Self {
            Self {
                program,
                shader_path,
                verifier_bin,
                packing,
            }
        }

        /// Discover verifier configuration using env vars or heuristics.
        ///
        /// Order:
        /// - `LIGERO_CONFIG_PATH` (full JSON config)
        /// - If `expected_commitment` is provided, try to find a matching program in known locations
        /// - `LIGERO_PROGRAM_PATH` + `LIGERO_SHADER_PATH` (+ optional `LIGERO_PACKING`)
        pub fn discover_with_commitment(expected_commitment: Option<&[u8; 32]>) -> Result<Self> {
            let config = if let Ok(config_path) = std::env::var("LIGERO_CONFIG_PATH") {
                let config_contents = fs::read_to_string(&config_path)
                    .with_context(|| format!("Failed to read Ligero config at {config_path}"))?;
                serde_json::from_str::<LigeroConfig>(&config_contents)
                    .with_context(|| format!("Failed to parse Ligero config JSON from {config_path}"))?
            } else {
                if let Some(commitment) = expected_commitment {
                    if let Some(found) = Self::find_program_for_commitment(commitment)? {
                        return Ok(found);
                    }
                }

                let program = std::env::var("LIGERO_PROGRAM_PATH").context(
                    "LIGERO_PROGRAM_PATH environment variable is required for Ligero verification",
                )?;
                // `LIGERO_SHADER_PATH` is optional: we can auto-discover the Ligero repo's `shader/`
                // directory in most setups (including when this crate is pulled via a git dependency).
                let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                let shader_path = std::env::var("LIGERO_SHADER_PATH")
                    .ok()
                    .and_then(|p| canonicalize(&p).ok())
                    .or_else(|| Self::find_shader_path(&cwd))
                    .context("Failed to find shader path")?
                    .to_string_lossy()
                    .to_string();
                let packing = std::env::var("LIGERO_PACKING")
                    .ok()
                    .and_then(|value| value.parse::<u32>().ok())
                    .unwrap_or(8192);

                LigeroConfig {
                    program,
                    shader_path,
                    gpu_threads: None,
                    packing,
                    private_indices: Vec::new(),
                    args: Vec::new(),
                }
            };

            let program = canonicalize(&config.program)
                .with_context(|| format!("Failed to resolve Ligero program path: {}", config.program))?;
            let shader_path = canonicalize(&config.shader_path)
                .with_context(|| format!("Failed to resolve Ligero shader path: {}", config.shader_path))?;

            let verifier_bin = locate_verifier_binary(program.parent())
                .context("Failed to locate webgpu_verifier binary")?;

            Ok(Self {
                program,
                shader_path,
                verifier_bin,
                packing: config.packing,
            })
        }

        fn find_program_for_commitment(commitment: &[u8; 32]) -> Result<Option<Self>> {
            let current_dir = std::env::current_dir()?;
            let packing: u32 = std::env::var("LIGERO_PACKING")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8192);

            // Common program names.
            let program_candidates = ["note_spend_guest.wasm", "value_validator.wasm"];

            // Common base directories in Sovereign checkouts.
            let base_paths = [
                current_dir.join("crates/adapters/ligero/guest/bins/programs"),
                current_dir.join("../crates/adapters/ligero/guest/bins/programs"),
                current_dir.join("../../crates/adapters/ligero/guest/bins/programs"),
            ];

            for base_path in &base_paths {
                for program_name in &program_candidates {
                    let program_path = base_path.join(program_name);
                    if !program_path.exists() {
                        continue;
                    }

                    if let Ok(wasm_bytes) = fs::read(&program_path) {
                        let mut hasher = Sha256::new();
                        hasher.update(&wasm_bytes);
                        hasher.update(packing.to_le_bytes());
                        let computed: [u8; 32] = hasher.finalize().into();

                        if &computed == commitment {
                            tracing::debug!(
                                "Found matching program for commitment {}: {}",
                                hex::encode(commitment),
                                program_path.display()
                            );

                            let shader_path = std::env::var("LIGERO_SHADER_PATH")
                                .ok()
                                .and_then(|p| canonicalize(&p).ok())
                                .or_else(|| Self::find_shader_path(&current_dir))
                                .context("Failed to find shader path")?;

                            let verifier_bin = locate_verifier_binary(program_path.parent())
                                .context("Failed to locate webgpu_verifier binary")?;

                            return Ok(Some(Self {
                                program: canonicalize(
                                    program_path.to_str().context("Invalid program path")?,
                                )?,
                                shader_path,
                                verifier_bin,
                                packing,
                            }));
                        }
                    }
                }
            }

            Ok(None)
        }

        fn find_shader_path(current_dir: &Path) -> Option<PathBuf> {
            // 1) Sovereign historical layout (kept for backward compatibility)
            let legacy = [
                current_dir.join("crates/adapters/ligero/bins/shader"),
                current_dir.join("../crates/adapters/ligero/bins/shader"),
            ];
            if let Some(p) = legacy.into_iter().find(|p| p.exists()) {
                if let Some(s) = p.to_str() {
                    if let Ok(c) = canonicalize(s) {
                        return Some(c);
                    }
                }
            }

            // 2) dcSpark Ligero repo layout: `<ligero-prover>/shader`
            // Try to locate `shader/` by walking up from:
            // - the current working directory
            // - this crate's source directory
            fn find_shader_upwards(start: &Path) -> Option<PathBuf> {
                for ancestor in start.ancestors().take(10) {
                    let cand = ancestor.join("shader");
                    if cand.exists() {
                        return cand.to_str().and_then(|s| canonicalize(s).ok());
                    }
                }
                None
            }

            if let Some(p) = find_shader_upwards(current_dir) {
                return Some(p);
            }

            let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            if let Some(p) = find_shader_upwards(&manifest_dir) {
                return Some(p);
            }

            // 3) If user provided `LIGERO_ROOT`, prefer `<root>/shader`
            if let Ok(root) = std::env::var("LIGERO_ROOT") {
                let cand = PathBuf::from(root).join("shader");
                if cand.exists() {
                    return cand.to_str().and_then(|s| canonicalize(s).ok());
                }
            }

            None
        }

        pub fn to_config(&self, args: Vec<LigeroArg>, private_indices: Vec<usize>) -> LigeroConfig {
            LigeroConfig {
                program: self.program.to_string_lossy().into_owned(),
                shader_path: self.shader_path.to_string_lossy().into_owned(),
                gpu_threads: None,
                packing: self.packing,
                private_indices,
                args,
            }
        }
    }

    pub fn ensure_code_commitment(paths: &VerifierPaths, expected: &[u8; 32]) -> Result<()> {
        let wasm_bytes = fs::read(&paths.program).with_context(|| {
            format!(
                "Failed to read Ligero WASM program at {}",
                paths.program.display()
            )
        })?;

        let mut hasher = Sha256::new();
        hasher.update(&wasm_bytes);
        hasher.update(paths.packing.to_le_bytes());
        let computed: [u8; 32] = hasher.finalize().into();

        if &computed != expected {
            anyhow::bail!(
                "Ligero code commitment mismatch: expected {}, computed {}",
                hex::encode(expected),
                hex::encode(computed)
            );
        }

        Ok(())
    }

    /// Verify a proof by writing `proof_data.gz` into a temp dir and invoking `webgpu_verifier`.
    ///
    /// Private arguments are redacted according to `private_indices` (1-based).
    /// Verify a proof and return `(success, stdout, stderr)` for debugging.
    ///
    /// This runs `webgpu_verifier` and captures its output even when verification fails.
    pub fn verify_proof_with_output(
        paths: &VerifierPaths,
        proof_bytes: &[u8],
        mut args: Vec<LigeroArg>,
        private_indices: Vec<usize>,
    ) -> Result<(bool, String, String)> {
        let temp_dir = tempdir().context("Failed to create temporary directory for Ligero verification")?;

        // Write proof as proof_data.gz (the format verifier expects)
        let proof_path = temp_dir.path().join("proof_data.gz");
        fs::write(&proof_path, proof_bytes)
            .context("Failed to write proof_data.gz for Ligero verification")?;

        // Redact private arguments (replace with dummy values) while preserving basic parseability.
        for &idx in &private_indices {
            if idx > 0 && idx <= args.len() {
                let arg_idx = idx - 1;
                args[arg_idx] = redact(&args[arg_idx]);
            }
        }

        let config = paths.to_config(args, private_indices);
        let config_json = serde_json::to_string(&config)
            .context("Failed to serialize Ligero verifier config")?;

        let output = Command::new(&paths.verifier_bin)
            .arg(&config_json)
            .current_dir(temp_dir.path())
            .output()
            .with_context(|| {
                format!(
                    "Failed to execute Ligero verifier at {}",
                    paths.verifier_bin.display()
                )
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let success = output.status.success()
            && stdout.contains("Final Verify Result:")
            && stdout.contains("true");

        Ok((success, stdout, stderr))
    }

    pub fn verify_proof(
        paths: &VerifierPaths,
        proof_bytes: &[u8],
        args: Vec<LigeroArg>,
        private_indices: Vec<usize>,
    ) -> Result<()> {
        let (success, stdout, stderr) = verify_proof_with_output(paths, proof_bytes, args, private_indices)?;

        if !success {
            tracing::error!("Ligero verifier did not report success: stdout={stdout}, stderr={stderr}");
            anyhow::bail!("Ligero verifier did not confirm proof validity");
        }

        Ok(())
    }

    fn redact(arg: &LigeroArg) -> LigeroArg {
        match arg {
            LigeroArg::String { str: s } => {
                let trimmed = s.trim();
                if trimmed.starts_with("0x") && trimmed.len() >= 2 {
                    let body = &trimmed[2..];
                    return LigeroArg::String {
                        str: format!("0x{}", "0".repeat(body.len())),
                    };
                }

                let is_hex = !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_hexdigit());
                if is_hex {
                    return LigeroArg::String {
                        str: "0".repeat(trimmed.len()),
                    };
                }

                let is_dec = !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_digit());
                if is_dec {
                    return LigeroArg::String {
                        str: "0".repeat(trimmed.len().max(1)),
                    };
                }

                LigeroArg::String {
                    str: "x".repeat(trimmed.len().max(1)),
                }
            }
            LigeroArg::I64 { .. } => LigeroArg::I64 { i64: 0 },
            LigeroArg::Hex { hex: h } => LigeroArg::Hex {
                hex: "0".repeat(h.len().max(1)),
            },
        }
    }

    fn canonicalize(path: &str) -> Result<PathBuf> {
        let path = Path::new(path);
        fs::canonicalize(path)
            .or_else(|_| {
                if path.is_absolute() {
                    Err(anyhow::anyhow!("Path does not exist: {}", path.display()))
                } else {
                    let current_dir = std::env::current_dir().context("Failed to get current directory")?;
                    let joined = current_dir.join(path);
                    Ok(fs::canonicalize(&joined)?)
                }
            })
            .with_context(|| format!("Failed to canonicalize {}", path.display()))
    }

    fn locate_verifier_binary(program_parent: Option<&Path>) -> Result<PathBuf> {
        if let Ok(path_str) = std::env::var("LIGERO_VERIFIER_BIN") {
            let path = Path::new(&path_str);
            if path.exists() {
                return fs::canonicalize(path)
                    .with_context(|| format!("Failed to resolve verifier binary path {path_str}"));
            }
        }

        if let Some(dir) = program_parent {
            let candidate = dir.join("webgpu_verifier");
            if candidate.exists() {
                return fs::canonicalize(candidate)
                    .context("Failed to resolve verifier binary located next to program");
            }
        }

        if let Ok(root) = std::env::var("LIGERO_ROOT") {
            let root = PathBuf::from(root);
            let candidates = [
                root.join("build/webgpu_verifier"),
                root.join("build-web/webgpu_verifier"),
                root.join("bins/webgpu_verifier"),
                root.join("bin/webgpu_verifier"),
            ];
            for c in candidates {
                if c.exists() {
                    return fs::canonicalize(c).context("Failed to resolve verifier binary under LIGERO_ROOT");
                }
            }
        }

        anyhow::bail!(
            "Unable to locate webgpu_verifier binary. Set LIGERO_VERIFIER_BIN or LIGERO_ROOT"
        );
    }
}
