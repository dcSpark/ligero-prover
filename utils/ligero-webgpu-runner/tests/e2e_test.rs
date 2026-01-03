//! End-to-end tests for Ligero prover/verifier.
//!
//! These tests are intentionally lightweight and focus on the *host wrapper* surface that this
//! crate provides (`LigeroHostCore`), without pulling in Sovereign-side packaging types.

use std::path::PathBuf;
use std::process::Command;

use ligero_runner::{sovereign_host::LigeroHostCore, LigeroProofPackage};

fn repo_root() -> Option<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.ancestors().nth(2).map(|p| p.to_path_buf())
}

fn note_spend_program_path(repo_root: &PathBuf) -> PathBuf {
    repo_root.join("utils/circuits/bins/note_spend_guest.wasm")
}

fn maybe_build_note_spend_guest(repo_root: &PathBuf) -> bool {
    let wasm = note_spend_program_path(repo_root);
    if wasm.exists() {
        return true;
    }

    let build_sh = repo_root.join("utils/circuits/note-spend/build.sh");
    if !build_sh.exists() {
        return false;
    }

    let status = Command::new("bash")
        .arg(build_sh)
        .current_dir(repo_root)
        .status();

    status.map(|s| s.success()).unwrap_or(false) && wasm.exists()
}

fn get_test_program_path() -> Option<String> {
    let repo = repo_root()?;
    if !maybe_build_note_spend_guest(&repo) {
        return None;
    }
    let p = note_spend_program_path(&repo);
    if p.exists() {
        Some(p.to_string_lossy().to_string())
    } else {
        None
    }
}

// ============================================================================
// Host Creation Tests
// ============================================================================

#[test]
fn test_host_creation() {
    let program_path = match get_test_program_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: no test program found (note_spend_guest.wasm missing)");
            return;
        }
    };

    let host = LigeroHostCore::new(&program_path);

    assert_eq!(host.program_path(), program_path.as_str());
    assert_eq!(host.packing(), 8192); // Default packing
    assert!(host.runner().config().private_indices.is_empty());
    assert!(host.runner().config().args.is_empty());
}

#[test]
fn test_host_with_args() {
    let program_path = match get_test_program_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: no test program found (note_spend_guest.wasm missing)");
            return;
        }
    };

    let mut host = LigeroHostCore::new(&program_path);

    host.add_i64_arg(42);
    host.add_str_arg("test".to_string());
    host.add_hex_arg("abcd1234".to_string());

    assert_eq!(host.runner().config().args.len(), 3);
}

#[test]
fn test_host_with_packing() {
    let program_path = match get_test_program_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: no test program found (note_spend_guest.wasm missing)");
            return;
        }
    };

    let host = LigeroHostCore::new(&program_path).with_packing(4096);

    assert_eq!(host.packing(), 4096);
}

#[test]
fn test_host_with_private_indices() {
    let program_path = match get_test_program_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: no test program found (note_spend_guest.wasm missing)");
            return;
        }
    };

    let host = LigeroHostCore::new(&program_path).with_private_indices(vec![1, 2]);

    assert_eq!(host.runner().config().private_indices, vec![1, 2]);
}

#[test]
fn test_code_commitment() {
    let program_path = match get_test_program_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: no test program found (note_spend_guest.wasm missing)");
            return;
        }
    };

    let host = LigeroHostCore::new(&program_path);
    let commitment = host.code_commitment_raw();

    // Code commitment should be deterministic
    let host2 = LigeroHostCore::new(&program_path);
    let commitment2 = host2.code_commitment_raw();
    assert_eq!(commitment, commitment2);

    // Different packing should give different commitment
    let host3 = LigeroHostCore::new(&program_path).with_packing(4096);
    let commitment3 = host3.code_commitment_raw();
    assert_ne!(commitment, commitment3);
}

#[test]
fn test_public_output_bytes_roundtrip() {
    let program_path = match get_test_program_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: no test program found (note_spend_guest.wasm missing)");
            return;
        }
    };

    let mut host = LigeroHostCore::new(&program_path);
    host.set_public_output_bytes(vec![1, 2, 3, 4]);

    let out = host.require_public_output().unwrap();
    assert_eq!(out, vec![1, 2, 3, 4]);
}

#[test]
fn test_simulation_mode_packaging() {
    let program_path = match get_test_program_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: no test program found (note_spend_guest.wasm missing)");
            return;
        }
    };

    let mut host = LigeroHostCore::new(&program_path);
    host.add_i64_arg(42);
    host.set_public_output_bytes(vec![1, 2, 3, 4]);

    // Simulation mode should always work (no binaries needed)
    let proof_bytes = host.run_simulation().unwrap();

    // Should deserialize to a valid package
    let package = LigeroProofPackage::from_bytes(&proof_bytes).unwrap();
    assert!(package.is_simulation());
    assert_eq!(package.public_output, vec![1, 2, 3, 4]);
}

#[test]
fn test_simulation_with_private_args_redaction() {
    let program_path = match get_test_program_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: no test program found (note_spend_guest.wasm missing)");
            return;
        }
    };

    let mut host = LigeroHostCore::new(&program_path).with_private_indices(vec![2]); // mark hex arg private
    host.add_i64_arg(100); // Public
    host.add_hex_arg("secret_hex_data".to_string()); // Private (not actually hex, but redaction should still be safe)
    host.add_str_arg("public_string".to_string()); // Public
    host.set_public_output_bytes(vec![]);

    let proof_bytes = host.run_simulation().unwrap();
    let package = LigeroProofPackage::from_bytes(&proof_bytes).unwrap();

    // Check that private args are redacted in the package
    let args: Vec<ligero_runner::LigeroArg> = package.args_as().unwrap();
    assert_eq!(args.len(), 3);

    // First arg (public) should be unchanged
    assert_eq!(args[0], ligero_runner::LigeroArg::I64 { i64: 100 });

    // Second arg (private) should be redacted (same length, all zeros)
    match &args[1] {
        ligero_runner::LigeroArg::Hex { hex } => {
            assert_eq!(hex.len(), "secret_hex_data".len());
            assert!(hex.chars().all(|c| c == '0'));
        }
        other => panic!("Expected Hex argument, got {other:?}"),
    }

    // Third arg (public) should be unchanged
    match &args[2] {
        ligero_runner::LigeroArg::String { str } => {
            assert_eq!(str, "public_string");
        }
        other => panic!("Expected String argument, got {other:?}"),
    }
}
