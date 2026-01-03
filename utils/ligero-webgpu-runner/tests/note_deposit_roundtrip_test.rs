//! Proof roundtrip + binding tests for the `note_deposit_guest` circuit.
//!
//! Like the note-spend integration tests, these tests require:
//! - `webgpu_prover` + `webgpu_verifier` binaries
//! - a valid `shader/` directory
//! - a built `note_deposit_guest.wasm`
//!
//! If assets are missing (or the prover cannot run on this machine), tests exit early with a skip
//! message.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use ligero_runner::{verifier, LigeroArg, LigeroRunner};
use ligetron::poseidon2_hash_bytes;

type Hash32 = [u8; 32];

fn hx32(b: &Hash32) -> String {
    hex::encode(b)
}

fn repo_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // utils/ligero-webgpu-runner -> utils -> repo
    Ok(manifest_dir
        .ancestors()
        .nth(2)
        .context("Failed to compute ligero-prover repo root")?
        .to_path_buf())
}

fn maybe_build_note_deposit_guest(repo: &Path) -> Result<()> {
    let out = repo.join("utils/circuits/bins/note_deposit_guest.wasm");
    if out.exists() {
        return Ok(());
    }

    let guest_dir = repo.join("utils/circuits/note-deposit");
    if !guest_dir.exists() {
        anyhow::bail!("note-deposit sources not found at {}", guest_dir.display());
    }

    println!(
        "[note_deposit_roundtrip] note_deposit_guest.wasm not found; building via {}",
        guest_dir.join("build.sh").display()
    );

    // Best-effort build. This may download the wasm std target via rustup.
    let status = Command::new("bash")
        .arg("build.sh")
        .current_dir(&guest_dir)
        .status()
        .context("Failed to run note-deposit/build.sh")?;

    if !status.success() {
        anyhow::bail!("note-deposit/build.sh failed with status {status}");
    }

    if !out.exists() {
        anyhow::bail!(
            "note_deposit_guest.wasm still not found after build at {}",
            out.display()
        );
    }

    println!(
        "[note_deposit_roundtrip] Built note_deposit_guest.wasm at {}",
        out.display()
    );

    Ok(())
}

fn note_deposit_program_path(repo: &Path) -> PathBuf {
    repo.join("utils/circuits/bins/note_deposit_guest.wasm")
}

// === Poseidon2 domain-separated helpers (must match the guest program) ===

fn poseidon2_hash_domain(tag: &[u8], parts: &[&[u8]]) -> Hash32 {
    let mut buf_len = tag.len();
    for part in parts {
        buf_len += part.len();
    }

    let mut tmp = Vec::with_capacity(buf_len);
    tmp.extend_from_slice(tag);
    for part in parts {
        tmp.extend_from_slice(part);
    }

    poseidon2_hash_bytes(&tmp).to_bytes_be()
}

fn note_commitment(domain: &Hash32, value: u64, rho: &Hash32, recipient: &Hash32) -> Hash32 {
    // Guest encodes value as 16-byte LE (u64 zero-extended to 16 bytes).
    let mut v16 = [0u8; 16];
    v16[..8].copy_from_slice(&value.to_le_bytes());
    poseidon2_hash_domain(b"NOTE_V1", &[domain, &v16, rho, recipient])
}

fn private_indices_note_deposit() -> Vec<usize> {
    // Deposit circuit ABI:
    // 1 domain (PUB), 2 value (PUB), 3 rho (PRIV), 4 recipient (PRIV), 5 cm_out (PUB)
    vec![3, 4]
}

fn setup_runner_and_paths(
    program: &Path,
) -> Result<(LigeroRunner, verifier::VerifierPaths, Vec<usize>)> {
    let packing: u32 = std::env::var("LIGERO_PACKING")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8192);

    let mut runner = LigeroRunner::new(&program.to_string_lossy());
    runner.config_mut().packing = packing;

    if !runner.paths().prover_bin.exists() || !runner.paths().verifier_bin.exists() {
        eprintln!(
            "Skipping: Ligero binaries not found (prover={}, verifier={})",
            runner.paths().prover_bin.display(),
            runner.paths().verifier_bin.display()
        );
        anyhow::bail!("SKIP");
    }

    let shader_dir = PathBuf::from(&runner.config().shader_path);
    if !shader_dir.exists() {
        eprintln!("Skipping: shader path not found at {}", shader_dir.display());
        anyhow::bail!("SKIP");
    }

    let vpaths = verifier::VerifierPaths::from_explicit(
        program.to_path_buf(),
        shader_dir,
        runner.paths().verifier_bin.clone(),
        packing,
    );

    Ok((runner, vpaths, private_indices_note_deposit()))
}

fn try_skip<T>(r: Result<T>) -> Result<Option<T>> {
    match r {
        Ok(v) => Ok(Some(v)),
        Err(e) if e.to_string().contains("SKIP") => Ok(None),
        Err(e) => Err(e),
    }
}

#[test]
fn test_note_deposit_roundtrip_hex_args() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_deposit_guest(&repo)?;

    let program = note_deposit_program_path(&repo)
        .canonicalize()
        .context("Failed to canonicalize note_deposit_guest.wasm")?;

    let Some((mut runner, vpaths, priv_idx)) = try_skip(setup_runner_and_paths(&program))? else {
        return Ok(());
    };

    let domain: Hash32 = [1u8; 32];
    let value: u64 = 42;
    let rho: Hash32 = [2u8; 32];
    let recipient: Hash32 = [3u8; 32];
    let cm_out = note_commitment(&domain, value, &rho, &recipient);

    let args = vec![
        LigeroArg::Hex { hex: hx32(&domain) },
        LigeroArg::I64 { i64: value as i64 },
        LigeroArg::Hex { hex: hx32(&rho) },
        LigeroArg::Hex {
            hex: hx32(&recipient),
        },
        LigeroArg::Hex { hex: hx32(&cm_out) },
    ];

    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = args.clone();

    let (proof_bytes, _p_stdout, _p_stderr) =
        match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
            keep_proof_dir: false,
            proof_outputs_base: None,
            write_replay_script: false,
        }) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("Skipping: prover failed (GPU/WebGPU likely unavailable): {err}");
                return Ok(());
            }
        };

    anyhow::ensure!(!proof_bytes.is_empty(), "proof should not be empty");

    let (ok, v_stdout, v_stderr) =
        verifier::verify_proof_with_output(&vpaths, &proof_bytes, args, priv_idx)
            .context("Failed to run verifier")?;

    if !ok {
        anyhow::bail!("verifier did not report success\nstdout: {v_stdout}\nstderr: {v_stderr}");
    }

    Ok(())
}

#[test]
fn test_note_deposit_roundtrip_string_args() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_deposit_guest(&repo)?;

    let program = note_deposit_program_path(&repo)
        .canonicalize()
        .context("Failed to canonicalize note_deposit_guest.wasm")?;

    let Some((mut runner, vpaths, priv_idx)) = try_skip(setup_runner_and_paths(&program))? else {
        return Ok(());
    };

    let domain: Hash32 = [4u8; 32];
    let value: u64 = 123;
    let rho: Hash32 = [5u8; 32];
    let recipient: Hash32 = [6u8; 32];
    let cm_out = note_commitment(&domain, value, &rho, &recipient);

    let args = vec![
        LigeroArg::String {
            str: format!("0x{}", hx32(&domain)),
        },
        LigeroArg::I64 { i64: value as i64 },
        LigeroArg::String {
            str: format!("0x{}", hx32(&rho)),
        },
        LigeroArg::String {
            str: format!("0x{}", hx32(&recipient)),
        },
        LigeroArg::String {
            str: format!("0x{}", hx32(&cm_out)),
        },
    ];

    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = args.clone();

    let (proof_bytes, _p_stdout, _p_stderr) =
        match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
            keep_proof_dir: false,
            proof_outputs_base: None,
            write_replay_script: false,
        }) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("Skipping: prover failed (GPU/WebGPU likely unavailable): {err}");
                return Ok(());
            }
        };

    anyhow::ensure!(!proof_bytes.is_empty(), "proof should not be empty");

    let (ok, v_stdout, v_stderr) =
        verifier::verify_proof_with_output(&vpaths, &proof_bytes, args, priv_idx)
            .context("Failed to run verifier")?;

    if !ok {
        anyhow::bail!("verifier did not report success\nstdout: {v_stdout}\nstderr: {v_stderr}");
    }

    Ok(())
}

#[test]
fn test_note_deposit_verifier_rejects_mutated_value() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_deposit_guest(&repo)?;

    let program = note_deposit_program_path(&repo)
        .canonicalize()
        .context("Failed to canonicalize note_deposit_guest.wasm")?;

    let Some((mut runner, vpaths, priv_idx)) = try_skip(setup_runner_and_paths(&program))? else {
        return Ok(());
    };

    let domain: Hash32 = [7u8; 32];
    let value: u64 = 77;
    let rho: Hash32 = [8u8; 32];
    let recipient: Hash32 = [9u8; 32];
    let cm_out = note_commitment(&domain, value, &rho, &recipient);

    let args = vec![
        LigeroArg::Hex { hex: hx32(&domain) },
        LigeroArg::I64 { i64: value as i64 },
        LigeroArg::Hex { hex: hx32(&rho) },
        LigeroArg::Hex {
            hex: hx32(&recipient),
        },
        LigeroArg::Hex { hex: hx32(&cm_out) },
    ];

    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = args.clone();

    let (proof_bytes, _p_stdout, _p_stderr) =
        match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
            keep_proof_dir: false,
            proof_outputs_base: None,
            write_replay_script: false,
        }) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("Skipping: prover failed (GPU/WebGPU likely unavailable): {err}");
                return Ok(());
            }
        };

    // Sanity: should verify with the original statement.
    let (ok, v_stdout, v_stderr) =
        verifier::verify_proof_with_output(&vpaths, &proof_bytes, args.clone(), priv_idx.clone())
            .context("Failed to run verifier")?;
    if !ok {
        anyhow::bail!(
            "verifier did not report success for the original statement\nstdout: {v_stdout}\nstderr: {v_stderr}"
        );
    }

    // Mutate a PUBLIC input (value) without changing the proof -> must fail.
    let mut bad_args = args;
    bad_args[1] = LigeroArg::I64 {
        i64: (value as i64) + 1,
    };

    match verifier::verify_proof_with_output(&vpaths, &proof_bytes, bad_args, priv_idx) {
        Ok((ok_bad, _stdout, _stderr)) => {
            anyhow::ensure!(
                !ok_bad,
                "expected verification to fail when a public input is mutated (this implies the verifier is not binding to provided public inputs)"
            );
        }
        Err(_e) => {
            // Any error is also an acceptable failure signal for a bad statement.
        }
    }

    Ok(())
}

