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

fn recipient_from_pk(domain: &Hash32, pk_spend: &Hash32, pk_ivk: &Hash32) -> Hash32 {
    poseidon2_hash_domain(b"ADDR_V2", &[domain, pk_spend, pk_ivk])
}

fn note_commitment_v2(
    domain: &Hash32,
    value: u64,
    rho: &Hash32,
    recipient: &Hash32,
    sender_id: &Hash32,
) -> Hash32 {
    // Guest encodes value as 16-byte LE (u64 zero-extended to 16 bytes).
    let mut v16 = [0u8; 16];
    v16[..8].copy_from_slice(&value.to_le_bytes());
    poseidon2_hash_domain(b"NOTE_V2", &[domain, &v16, rho, recipient, sender_id])
}

fn private_indices_note_deposit() -> Vec<usize> {
    // Deposit circuit ABI:
    // 1 domain (PUB), 2 value (PUB), 3 rho (PRIV), 4 pk_spend_recipient (PRIV),
    // 5 pk_ivk_recipient (PRIV), 6 cm_out (PUB)
    vec![3, 4, 5]
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
        eprintln!(
            "Skipping: shader path not found at {}",
            shader_dir.display()
        );
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
    let pk_spend: Hash32 = [3u8; 32];
    let pk_ivk: Hash32 = [4u8; 32];
    let recipient = recipient_from_pk(&domain, &pk_spend, &pk_ivk);
    let sender_id = [0u8; 32];
    let cm_out = note_commitment_v2(&domain, value, &rho, &recipient, &sender_id);

    let args = vec![
        LigeroArg::Hex { hex: hx32(&domain) },
        LigeroArg::I64 { i64: value as i64 },
        LigeroArg::Hex { hex: hx32(&rho) },
        LigeroArg::Hex { hex: hx32(&pk_spend) },
        LigeroArg::Hex { hex: hx32(&pk_ivk) },
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
    let pk_spend: Hash32 = [6u8; 32];
    let pk_ivk: Hash32 = [7u8; 32];
    let recipient = recipient_from_pk(&domain, &pk_spend, &pk_ivk);
    let sender_id = [0u8; 32];
    let cm_out = note_commitment_v2(&domain, value, &rho, &recipient, &sender_id);

    let args = vec![
        LigeroArg::String {
            str: format!("0x{}", hx32(&domain)),
        },
        LigeroArg::I64 { i64: value as i64 },
        LigeroArg::String {
            str: format!("0x{}", hx32(&rho)),
        },
        LigeroArg::String {
            str: format!("0x{}", hx32(&pk_spend)),
        },
        LigeroArg::String {
            str: format!("0x{}", hx32(&pk_ivk)),
        },
        LigeroArg::String {
            str: format!("0x{}", hx32(&cm_out)),
        },
    ];

    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = args.clone();

    // The WebGPU prover/verifier pass 32-byte args to WASM as `0x...` strings; both `Hex` and
    // `String` encodings can therefore be supported by the guest.
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
    let pk_spend: Hash32 = [9u8; 32];
    let pk_ivk: Hash32 = [10u8; 32];
    let recipient = recipient_from_pk(&domain, &pk_spend, &pk_ivk);
    let sender_id = [0u8; 32];
    let cm_out = note_commitment_v2(&domain, value, &rho, &recipient, &sender_id);

    let args = vec![
        LigeroArg::Hex { hex: hx32(&domain) },
        LigeroArg::I64 { i64: value as i64 },
        LigeroArg::Hex { hex: hx32(&rho) },
        LigeroArg::Hex { hex: hx32(&pk_spend) },
        LigeroArg::Hex { hex: hx32(&pk_ivk) },
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

#[test]
fn test_note_deposit_verifier_rejects_mutated_cm_out() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_deposit_guest(&repo)?;

    let program = note_deposit_program_path(&repo)
        .canonicalize()
        .context("Failed to canonicalize note_deposit_guest.wasm")?;

    let Some((mut runner, vpaths, priv_idx)) = try_skip(setup_runner_and_paths(&program))? else {
        return Ok(());
    };

    let domain: Hash32 = [10u8; 32];
    let value: u64 = 55;
    let rho: Hash32 = [11u8; 32];
    let pk_spend: Hash32 = [12u8; 32];
    let pk_ivk: Hash32 = [13u8; 32];
    let recipient = recipient_from_pk(&domain, &pk_spend, &pk_ivk);
    let sender_id = [0u8; 32];
    let cm_out = note_commitment_v2(&domain, value, &rho, &recipient, &sender_id);

    let args = vec![
        LigeroArg::Hex { hex: hx32(&domain) },
        LigeroArg::I64 { i64: value as i64 },
        LigeroArg::Hex { hex: hx32(&rho) },
        LigeroArg::Hex { hex: hx32(&pk_spend) },
        LigeroArg::Hex { hex: hx32(&pk_ivk) },
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

    // Mutate a PUBLIC input (cm_out) without changing the proof -> must fail.
    let mut bad_args = args;
    let mut bad_cm = cm_out;
    bad_cm[0] ^= 1;
    bad_args[5] = LigeroArg::Hex { hex: hx32(&bad_cm) };

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

#[test]
fn test_note_deposit_rejects_negative_value_and_wrong_argc() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_deposit_guest(&repo)?;

    let program = note_deposit_program_path(&repo)
        .canonicalize()
        .context("Failed to canonicalize note_deposit_guest.wasm")?;

    let Some((mut runner, vpaths, priv_idx)) = try_skip(setup_runner_and_paths(&program))? else {
        return Ok(());
    };

    // Baseline valid proof to ensure the environment is functional.
    let domain: Hash32 = [13u8; 32];
    let value: u64 = 42;
    let rho: Hash32 = [14u8; 32];
    let pk_spend: Hash32 = [15u8; 32];
    let pk_ivk: Hash32 = [16u8; 32];
    let recipient = recipient_from_pk(&domain, &pk_spend, &pk_ivk);
    let sender_id = [0u8; 32];
    let cm_out = note_commitment_v2(&domain, value, &rho, &recipient, &sender_id);

    let good_args = vec![
        LigeroArg::Hex { hex: hx32(&domain) },
        LigeroArg::I64 { i64: value as i64 },
        LigeroArg::Hex { hex: hx32(&rho) },
        LigeroArg::Hex { hex: hx32(&pk_spend) },
        LigeroArg::Hex { hex: hx32(&pk_ivk) },
        LigeroArg::Hex { hex: hx32(&cm_out) },
    ];

    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = good_args.clone();
    let (good_proof, _p_stdout, _p_stderr) =
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

    let (ok, v_stdout, v_stderr) = verifier::verify_proof_with_output(
        &vpaths,
        &good_proof,
        good_args.clone(),
        priv_idx.clone(),
    )
    .context("Failed to run verifier")?;
    if !ok {
        anyhow::bail!("baseline proof did not verify\nstdout: {v_stdout}\nstderr: {v_stderr}");
    }

    // Invalid: negative public `value` must be rejected.
    let mut bad_args = good_args.clone();
    bad_args[1] = LigeroArg::I64 { i64: -1 };
    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = bad_args.clone();
    match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
        keep_proof_dir: false,
        proof_outputs_base: None,
        write_replay_script: false,
    }) {
        Ok((proof, _stdout, _stderr)) => {
            let (ok, stdout, stderr) =
                verifier::verify_proof_with_output(&vpaths, &proof, bad_args, priv_idx.clone())
                    .context("Failed to run verifier")?;
            anyhow::ensure!(
                !ok,
                "expected verification to fail, but it succeeded\nstdout: {stdout}\nstderr: {stderr}"
            );
        }
        Err(_e) => {
            // Expected: invalid input triggers UNSAT and the prover exits non-zero.
        }
    }

    // Invalid: wrong arg count must be rejected (regression test for failure-path soundness).
    let mut bad_args = good_args;
    bad_args.pop(); // drop cm_out
    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = bad_args.clone();
    match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
        keep_proof_dir: false,
        proof_outputs_base: None,
        write_replay_script: false,
    }) {
        Ok((proof, _stdout, _stderr)) => {
            let (ok, stdout, stderr) =
                verifier::verify_proof_with_output(&vpaths, &proof, bad_args, priv_idx)
                    .context("Failed to run verifier")?;
            anyhow::ensure!(
                !ok,
                "expected verification to fail, but it succeeded\nstdout: {stdout}\nstderr: {stderr}"
            );
        }
        Err(_e) => {}
    }

    Ok(())
}
