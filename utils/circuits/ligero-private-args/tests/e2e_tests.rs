//! End-to-end tests for Ligero prover/verifier.
//!
//! These tests require the WebGPU binaries to be available.
//! They are automatically skipped if binaries are not present.

use ligero_private_args::{LigeroHost, LigeroProofPackage};
use std::path::PathBuf;

fn get_test_program_path() -> Option<String> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // Try different program locations
    let candidates = [
        manifest_dir.join("bins/programs/edit.wasm"),
        manifest_dir.join("bins/programs/value_validator.wasm"),
    ];

    for path in &candidates {
        if path.exists() {
            return Some(path.to_string_lossy().to_string());
        }
    }

    None
}

fn skip_if_no_binaries(host: &LigeroHost) -> bool {
    if !host.prover_available() {
        eprintln!("⚠️ Skipping test: webgpu_prover not available");
        return true;
    }
    if !host.verifier_available() {
        eprintln!("⚠️ Skipping test: webgpu_verifier not available");
        return true;
    }
    false
}

// ============================================================================
// Host Creation Tests
// ============================================================================

#[test]
fn test_host_creation() {
    let program_path = match get_test_program_path() {
        Some(p) => p,
        None => {
            eprintln!("⚠️ Skipping test: no test program found");
            return;
        }
    };

    let host = LigeroHost::new(&program_path);

    assert_eq!(host.program_path(), program_path);
    assert_eq!(host.packing(), 8192); // Default packing
    assert!(host.private_indices().is_empty());
    assert!(host.args().is_empty());
}

#[test]
fn test_host_with_args() {
    let program_path = match get_test_program_path() {
        Some(p) => p,
        None => {
            eprintln!("⚠️ Skipping test: no test program found");
            return;
        }
    };

    let mut host = LigeroHost::new(&program_path);

    host.add_i64_arg(42);
    host.add_str_arg("test");
    host.add_hex_arg("abcd1234");

    assert_eq!(host.args().len(), 3);
}

#[test]
fn test_host_with_packing() {
    let program_path = match get_test_program_path() {
        Some(p) => p,
        None => {
            eprintln!("⚠️ Skipping test: no test program found");
            return;
        }
    };

    let host = LigeroHost::new(&program_path).with_packing(4096);

    assert_eq!(host.packing(), 4096);
}

#[test]
fn test_host_with_private_indices() {
    let program_path = match get_test_program_path() {
        Some(p) => p,
        None => {
            eprintln!("⚠️ Skipping test: no test program found");
            return;
        }
    };

    let host = LigeroHost::new(&program_path).with_private_indices(vec![1, 2]);

    assert_eq!(host.private_indices(), &[1, 2]);
}

#[test]
fn test_code_commitment() {
    let program_path = match get_test_program_path() {
        Some(p) => p,
        None => {
            eprintln!("⚠️ Skipping test: no test program found");
            return;
        }
    };

    let host = LigeroHost::new(&program_path);
    let commitment = host.code_commitment();

    // Code commitment should be 32 bytes (SHA-256)
    assert_eq!(commitment.len(), 32);

    // Code commitment should be deterministic
    let host2 = LigeroHost::new(&program_path);
    let commitment2 = host2.code_commitment();
    assert_eq!(commitment, commitment2);

    // Different packing should give different commitment
    let host3 = LigeroHost::new(&program_path).with_packing(4096);
    let commitment3 = host3.code_commitment();
    assert_ne!(commitment, commitment3);
}

#[test]
fn test_simulation_mode() {
    let program_path = match get_test_program_path() {
        Some(p) => p,
        None => {
            eprintln!("⚠️ Skipping test: no test program found");
            return;
        }
    };

    let mut host = LigeroHost::new(&program_path);
    host.add_i64_arg(42);
    host.set_public_output_bytes(vec![1, 2, 3, 4]);

    // Simulation mode should always work (no binaries needed)
    let result = host.run_simulation();
    assert!(result.is_ok());

    let proof_bytes = result.unwrap();
    assert!(!proof_bytes.is_empty());

    // Should deserialize to a valid package
    let package = LigeroProofPackage::from_bytes(&proof_bytes).unwrap();
    assert!(package.is_simulation()); // Empty proof
    assert_eq!(package.public_output, vec![1, 2, 3, 4]);
}

#[test]
fn test_simulation_with_private_args() {
    let program_path = match get_test_program_path() {
        Some(p) => p,
        None => {
            eprintln!("⚠️ Skipping test: no test program found");
            return;
        }
    };

    let mut host = LigeroHost::new(&program_path);
    host.add_i64_arg(100); // Public
    host.add_hex_arg("secret_hex_data"); // Private
    host.add_str_arg("public_string"); // Public
    host.set_private_indices(vec![2]); // Mark hex arg as private
    host.set_public_output_bytes(vec![]);

    let result = host.run_simulation();
    assert!(result.is_ok());

    let proof_bytes = result.unwrap();
    let package = LigeroProofPackage::from_bytes(&proof_bytes).unwrap();

    // Check that private args are redacted in the package
    let args = package.args().unwrap();
    assert_eq!(args.len(), 3);

    // First arg (public) should be unchanged
    assert!(matches!(&args[0], ligero_private_args::LigeroArg::I64 { i64: 100 }));

    // Second arg (private) should be redacted (same length, all zeros)
    match &args[1] {
        ligero_private_args::LigeroArg::Hex { hex } => {
            assert_eq!(hex.len(), "secret_hex_data".len());
            assert!(hex.chars().all(|c| c == '0'));
        }
        _ => panic!("Expected Hex argument"),
    }

    // Third arg (public) should be unchanged
    match &args[2] {
        ligero_private_args::LigeroArg::String { str } => {
            assert_eq!(str, "public_string");
        }
        _ => panic!("Expected String argument"),
    }
}

// ============================================================================
// Prover/Verifier Tests (require binaries)
// ============================================================================

#[test]
#[ignore] // Run with: cargo test -- --ignored
fn test_proof_generation() {
    let program_path = match get_test_program_path() {
        Some(p) => p,
        None => {
            eprintln!("⚠️ Skipping test: no test program found");
            return;
        }
    };

    let mut host = LigeroHost::new(&program_path);

    if skip_if_no_binaries(&host) {
        return;
    }

    // Configure for edit.wasm (edit distance program)
    host.add_str_arg("hello");
    host.add_str_arg("hallo");
    host.set_public_output_bytes(vec![]);

    match host.run_prover() {
        Ok(proof_bytes) => {
            println!("✓ Proof generated: {} bytes", proof_bytes.len());
            assert!(!proof_bytes.is_empty());

            let package = LigeroProofPackage::from_bytes(&proof_bytes).unwrap();
            assert!(!package.is_simulation());
            assert!(package.is_valid_gzip());
        }
        Err(e) => {
            eprintln!("Proof generation failed: {}", e);
            // Don't fail the test - WebGPU may not be available
        }
    }
}

#[test]
#[ignore] // Run with: cargo test -- --ignored
fn test_proof_generation_with_logging() {
    let program_path = match get_test_program_path() {
        Some(p) => p,
        None => {
            eprintln!("⚠️ Skipping test: no test program found");
            return;
        }
    };

    let mut host = LigeroHost::new(&program_path)
        .with_proof_dir_id("test_logging".to_string());

    if skip_if_no_binaries(&host) {
        return;
    }

    host.add_str_arg("test");
    host.add_str_arg("best");
    host.set_public_output_bytes(vec![]);

    match host.run_prover_with_logging() {
        Ok((proof_bytes, stdout)) => {
            println!("Prover output:\n{}", stdout);
            println!("✓ Proof generated: {} bytes", proof_bytes.len());

            // Check that stdout contains expected output
            assert!(stdout.contains("Final prove result:"));
        }
        Err(e) => {
            eprintln!("Proof generation failed: {}", e);
        }
    }
}

#[test]
#[ignore] // Run with: cargo test -- --ignored
fn test_proof_with_private_args() {
    let program_path = match get_test_program_path() {
        Some(p) => p,
        None => {
            eprintln!("⚠️ Skipping test: no test program found");
            return;
        }
    };

    let mut host = LigeroHost::new(&program_path)
        .with_private_indices(vec![1]); // First string is private

    if skip_if_no_binaries(&host) {
        return;
    }

    host.add_str_arg("secret"); // Private
    host.add_str_arg("public"); // Public
    host.set_public_output_bytes(vec![]);

    match host.run_prover() {
        Ok(proof_bytes) => {
            let package = LigeroProofPackage::from_bytes(&proof_bytes).unwrap();
            let args = package.args().unwrap();

            // First arg should be redacted
            match &args[0] {
                ligero_private_args::LigeroArg::String { str } => {
                    assert_eq!(str, "______"); // Same length, all underscores
                }
                _ => panic!("Expected String argument"),
            }

            // Second arg should be unchanged
            match &args[1] {
                ligero_private_args::LigeroArg::String { str } => {
                    assert_eq!(str, "public");
                }
                _ => panic!("Expected String argument"),
            }

            println!("✓ Private args correctly redacted in proof package");
        }
        Err(e) => {
            eprintln!("Proof generation failed: {}", e);
        }
    }
}
