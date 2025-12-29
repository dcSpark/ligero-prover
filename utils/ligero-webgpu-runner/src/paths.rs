//! Utilities to run the Ligero WebGPU prover/verifier binaries.
//!
//! This crate is intentionally "just a runner": it shells out to `webgpu_prover` / `webgpu_verifier`,
//! writes/reads expected artifacts (e.g. `proof_data.gz`) and provides light path-discovery with
//! environment-variable overrides.

use std::path::{Path, PathBuf};

use anyhow::Result;

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

impl LigeroPaths {
    /// Discover `webgpu_prover` / `webgpu_verifier` / `shader` paths.
    ///
    /// This uses the same environment-variable overrides and repo-root heuristics as `LigeroRunner`.
    pub fn discover() -> Result<Self> {
        discover_paths()
    }

    /// Return minimal fallback paths (assumes binaries are in PATH and shaders are in `./shader`).
    pub fn fallback() -> Self {
        fallback_paths()
    }
}

pub(crate) fn discover_paths() -> Result<LigeroPaths> {
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

pub(crate) fn fallback_paths() -> LigeroPaths {
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
