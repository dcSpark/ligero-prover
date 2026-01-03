//! Utilities to run the Ligero WebGPU prover/verifier binaries.
//!
//! This crate is intentionally "just a runner": it shells out to `webgpu_prover` / `webgpu_verifier`,
//! writes/reads expected artifacts (e.g. `proof_data.gz` or `proof_data.bin`) and provides light path-discovery with
//! environment-variable overrides.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

use crate::config::LigeroConfig;
use crate::paths::{discover_paths, fallback_paths, LigeroPaths};
use crate::pool::default_prover_pool;
use crate::pool::default_verifier_pool;
use crate::pool::BinaryWorkerPool;
use crate::LigeroArg;

fn sh_single_quote(value: &str) -> String {
    // POSIX shell-safe single-quote escaping: abc'def -> 'abc'\''def'
    format!("'{}'", value.replace("'", "'\\''"))
}

fn canonicalize_config_for_run(config: &LigeroConfig, caller_cwd: &Path) -> Result<LigeroConfig> {
    fn resolve(caller_cwd: &Path, raw: &str) -> Result<PathBuf> {
        let p = Path::new(raw);
        let resolved = if p.is_absolute() {
            p.to_path_buf()
        } else {
            caller_cwd.join(p)
        };
        std::fs::canonicalize(&resolved)
            .with_context(|| format!("Failed to canonicalize path: {}", resolved.display()))
    }

    let mut cfg = config.clone();
    cfg.program = match resolve(caller_cwd, &cfg.program) {
        Ok(p) => p.to_string_lossy().into_owned(),
        Err(_) => crate::resolve_program(&cfg.program)
            .with_context(|| format!("Failed to resolve Ligero program (path or name): {}", cfg.program))?
            .to_string_lossy()
            .into_owned(),
    };
    cfg.shader_path = resolve(caller_cwd, &cfg.shader_path)
        .with_context(|| format!("Failed to resolve Ligero shader path: {}", cfg.shader_path))?
        .to_string_lossy()
        .into_owned();
    Ok(cfg)
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
                gzip_proof: true,
                proof_path: None,
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
                gzip_proof: true,
                proof_path: None,
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

    /// Run the prover and return the proof bytes written by `webgpu_prover`.
    ///
    /// By default this is the compressed `proof_data.gz`. If `gzip-proof=false` is set in the
    /// config, the prover writes an uncompressed proof file instead.
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
    ///
    /// The prover execution is dispatched onto an always-on worker pool sized to `available_parallelism()`.
    pub fn run_prover_with_output(
        &self,
        options: ProverRunOptions,
    ) -> Result<(Vec<u8>, String, String)> {
        self.run_prover_with_output_in_pool(default_prover_pool(), options)
    }

    /// Run the prover on a specific worker pool (useful for tests/benchmarks).
    pub fn run_prover_with_output_in_pool(
        &self,
        pool: &BinaryWorkerPool,
        options: ProverRunOptions,
    ) -> Result<(Vec<u8>, String, String)> {
        let runner = self.clone();
        pool.execute(move || runner.run_prover_with_output_direct(options))
    }

    fn run_prover_with_output_direct(
        &self,
        options: ProverRunOptions,
    ) -> Result<(Vec<u8>, String, String)> {
        let caller_cwd = std::env::current_dir().context("Failed to get current directory")?;
        let config = canonicalize_config_for_run(&self.config, &caller_cwd)?;
        let config_json =
            serde_json::to_string(&config).context("Failed to serialize Ligero config")?;

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
            caller_cwd.join("proof_outputs")
        };

        let unique_proof_dir = proof_outputs_base.join(dir_name);
        std::fs::create_dir_all(&unique_proof_dir)
            .context("Failed to create unique proof directory")?;

        let keep_proof_dir = options.keep_proof_dir
            || std::env::var("LIGERO_KEEP_PROOF_DIR")
                .ok()
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);

        if options.write_replay_script {
            let replay_path = proof_outputs_base.join("last_prover_command.sh");
            let prover_bin = sh_single_quote(&self.paths.prover_bin.to_string_lossy());
            let proof_dir = sh_single_quote(&unique_proof_dir.to_string_lossy());
            let script = format!(
                r#"#!/usr/bin/env bash
set -euo pipefail

PROVER_BIN={prover_bin}
PROOF_DIR={proof_dir}

CONFIG_JSON=$(cat <<'JSON'
{config_json}
JSON
)

mkdir -p \"$PROOF_DIR\"
cd \"$PROOF_DIR\"
exec \"$PROVER_BIN\" \"$CONFIG_JSON\"
"#,
                prover_bin = prover_bin,
                proof_dir = proof_dir,
                config_json = config_json
            );
            if let Err(err) = std::fs::write(&replay_path, script) {
                tracing::warn!(
                    "Failed to write Ligero replay script {}: {}",
                    replay_path.display(),
                    err
                );
            } else {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(meta) = std::fs::metadata(&replay_path) {
                        let mut perms = meta.permissions();
                        perms.set_mode(0o755);
                        let _ = std::fs::set_permissions(&replay_path, perms);
                    }
                }
            }
        }

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

        // Read the proof file (compressed by default).
        let proof_filename = if config.gzip_proof {
            "proof_data.gz"
        } else {
            "proof_data.bin"
        };
        let proof_path = unique_proof_dir.join(proof_filename);
        if !proof_path.exists() {
            anyhow::bail!(
                "Ligero prover did not produce {}\nproof dir: {}\nNote: check prover.stdout.log / prover.stderr.log in that directory.",
                proof_filename,
                unique_proof_dir.display()
            );
        }
        let proof = std::fs::read(&proof_path)
            .with_context(|| format!("Failed to read {}", proof_path.display()))?;
        if proof.is_empty() {
            anyhow::bail!(
                "Ligero prover produced an empty proof file ({})\nproof dir: {}",
                proof_filename,
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
    ///
    /// The verifier execution is dispatched onto an always-on worker pool sized to `available_parallelism()`.
    pub fn verify_proof_smoke(&self) -> Result<bool> {
        self.verify_proof_smoke_in_pool(default_verifier_pool())
    }

    /// Run the verifier smoke check on a specific worker pool (useful for tests/benchmarks).
    pub fn verify_proof_smoke_in_pool(&self, pool: &BinaryWorkerPool) -> Result<bool> {
        let runner = self.clone();
        pool.execute(move || runner.verify_proof_smoke_direct())
    }

    fn verify_proof_smoke_direct(&self) -> Result<bool> {
        let caller_cwd = std::env::current_dir().context("Failed to get current directory")?;
        let config = canonicalize_config_for_run(&self.config, &caller_cwd)?;
        let config_json =
            serde_json::to_string(&config).context("Failed to serialize Ligero config")?;

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
