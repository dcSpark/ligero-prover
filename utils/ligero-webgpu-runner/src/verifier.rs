//! Utilities to run the Ligero WebGPU prover/verifier binaries.
//!
//! This crate is intentionally "just a runner": it shells out to `webgpu_prover` / `webgpu_verifier`,
//! writes/reads expected artifacts (e.g. `proof_data.gz`) and provides light path-discovery with
//! environment-variable overrides.

use crate::config::{LigeroArg, LigeroConfig};

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
        let mut candidates: Vec<PathBuf> = vec![
            root.join("build/webgpu_verifier"),
            root.join("build-web/webgpu_verifier"),
            root.join("bins/webgpu_verifier"),
            root.join("bin/webgpu_verifier"),
        ];
        if let Some(p) = portable_platform_dir() {
            candidates.push(
                root.join("utils")
                    .join("portable-binaries")
                    .join(p)
                    .join("bin")
                    .join("webgpu_verifier"),
            );
        }
        for c in candidates {
            if c.exists() {
                return fs::canonicalize(c).context("Failed to resolve verifier binary under LIGERO_ROOT");
            }
        }
    }



    // Final fallback: try to locate the verifier binary inside the Ligero repo checkout that
    // this crate is built from (works for git/path dependencies).
    //
    // We walk upwards from CARGO_MANIFEST_DIR and look for:
    // - `utils/portable-binaries/<platform>/bin/webgpu_verifier`
    // - `build*/webgpu_verifier`, `bins/webgpu_verifier`, `bin/webgpu_verifier`
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for ancestor in manifest_dir.ancestors().take(10) {
        let root = ancestor;
        let mut candidates: Vec<PathBuf> = vec![
            root.join("build/webgpu_verifier"),
            root.join("build-web/webgpu_verifier"),
            root.join("bins/webgpu_verifier"),
            root.join("bin/webgpu_verifier"),
        ];
        if let Some(p) = portable_platform_dir() {
            candidates.push(
                root.join("utils")
                    .join("portable-binaries")
                    .join(p)
                    .join("bin")
                    .join("webgpu_verifier"),
            );
        }
        for c in candidates {
            if c.exists() {
                return fs::canonicalize(c)
                    .context("Failed to resolve verifier binary relative to crate checkout");
            }
        }
    }

    anyhow::bail!(
        "Unable to locate webgpu_verifier binary. Set LIGERO_VERIFIER_BIN or LIGERO_ROOT"
    );
}
