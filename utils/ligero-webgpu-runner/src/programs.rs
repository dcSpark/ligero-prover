use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

fn looks_like_path(s: &str) -> bool {
    s.contains('/') || s.contains('\\') || s.starts_with('.') || s.starts_with('~')
}

fn normalize_program_name(name: &str) -> String {
    let raw = name.strip_prefix("circuits:").unwrap_or(name);
    if raw.ends_with(".wasm") {
        raw.to_string()
    } else {
        format!("{raw}.wasm")
    }
}

/// Resolve a Ligero guest program to an on-disk `.wasm` path.
///
/// Accepts either:
/// - a **path** to a `.wasm` file, or
/// - a **program name** (e.g. `"note_spend_guest"`), resolved under the Ligero repo's
///   `utils/circuits/bins/` directory.
///
/// Resolution order for names:
/// 1) `LIGERO_PROGRAMS_DIR` (dir containing `.wasm` files)
/// 2) `LIGERO_ROOT` (repo root; programs at `<root>/utils/circuits/bins/`)
/// 3) heuristic: walk up from this crate (`CARGO_MANIFEST_DIR`) to find a repo root that contains
///    `utils/circuits/bins/<name>.wasm`
pub fn resolve_program(program: &str) -> Result<PathBuf> {
    // If it's already a valid path, canonicalize it.
    if looks_like_path(program) {
        let p = PathBuf::from(program);
        if p.exists() {
            return std::fs::canonicalize(&p)
                .with_context(|| format!("Failed to canonicalize program path: {}", p.display()));
        }
        // fall through: it might be a relative path that doesn't exist from current cwd, or a name.
    }

    let filename = normalize_program_name(program);

    // 1) LIGERO_PROGRAMS_DIR override
    if let Ok(dir) = std::env::var("LIGERO_PROGRAMS_DIR") {
        let p = PathBuf::from(dir).join(&filename);
        if p.exists() {
            return std::fs::canonicalize(&p)
                .with_context(|| format!("Failed to canonicalize program path: {}", p.display()));
        }
    }

    // 2) LIGERO_ROOT override
    if let Ok(root) = std::env::var("LIGERO_ROOT") {
        let p = PathBuf::from(root)
            .join("utils")
            .join("circuits")
            .join("bins")
            .join(&filename);
        if p.exists() {
            return std::fs::canonicalize(&p)
                .with_context(|| format!("Failed to canonicalize program path: {}", p.display()));
        }
    }

    // 3) heuristic: find repo root by walking up from this crate
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if let Some(p) = find_program_upwards(&manifest_dir, &filename) {
        return std::fs::canonicalize(&p)
            .with_context(|| format!("Failed to canonicalize program path: {}", p.display()));
    }

    anyhow::bail!(
        "Could not locate Ligero guest program '{program}'.\n\
         Provide one of:\n\
         - LIGERO_PROGRAM_PATH=<full path to a .wasm>\n\
         - LIGERO_PROGRAMS_DIR=<dir containing .wasm files>\n\
         - LIGERO_ROOT=<ligero-prover repo root>\n\
         Expected name-based programs under <root>/utils/circuits/bins/{filename}"
    )
}

fn find_program_upwards(start: &Path, filename: &str) -> Option<PathBuf> {
    for anc in start.ancestors().take(20) {
        let cand = anc
            .join("utils")
            .join("circuits")
            .join("bins")
            .join(filename);
        if cand.exists() {
            return Some(cand);
        }
    }
    None
}
