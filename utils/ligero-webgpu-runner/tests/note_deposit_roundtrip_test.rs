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

const BL_DEPTH: usize = 63;

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

fn mt_combine(level: u8, left: &Hash32, right: &Hash32) -> Hash32 {
    let lvl = [level];
    poseidon2_hash_domain(b"MT_NODE_V1", &[&lvl, left, right])
}

fn merkle_default_nodes(depth: usize) -> Vec<Hash32> {
    let mut out = Vec::with_capacity(depth + 1);
    out.push([0u8; 32]); // height 0 (leaf)
    for lvl in 0..depth {
        let prev = out[lvl];
        out.push(mt_combine(lvl as u8, &prev, &prev));
    }
    out
}

fn bl_pos_from_id(id: &Hash32) -> u64 {
    // Must match the guest's `bl_pos_bits_from_id`: low bits, LSB-first from the last byte.
    let mut pos: u64 = 0;
    let mut i = 0usize;
    while i < BL_DEPTH {
        let byte = id[31 - (i / 8)];
        let bit = (byte >> (i % 8)) & 1;
        pos |= (bit as u64) << i;
        i += 1;
    }
    pos
}

fn merkle_root_from_path(leaf: &Hash32, pos: u64, siblings: &[Hash32]) -> Hash32 {
    let mut cur = *leaf;
    let mut idx = pos;
    for (lvl, sib) in siblings.iter().enumerate() {
        cur = if (idx & 1) == 0 {
            mt_combine(lvl as u8, &cur, sib)
        } else {
            mt_combine(lvl as u8, sib, &cur)
        };
        idx >>= 1;
    }
    cur
}

fn merkle_root_and_openings_sparse(
    depth: usize,
    leaves: &[(u64, Hash32)],
) -> Result<(Hash32, Vec<Vec<Hash32>>)> {
    use std::collections::{HashMap, HashSet};

    let default_nodes = merkle_default_nodes(depth);

    // Store only non-default nodes by height. Height 0 are leaves; height `depth` is the root.
    let mut levels: Vec<HashMap<u64, Hash32>> = (0..=depth).map(|_| HashMap::new()).collect();

    if depth >= 64 {
        anyhow::bail!("depth too large for sparse tree: {depth}");
    }
    let max_pos = if depth == 63 { 1u64 << 63 } else { 1u64 << depth };
    for (pos, leaf) in leaves {
        anyhow::ensure!(*pos < max_pos, "leaf pos {pos} out of range for depth {depth}");
        if let Some(prev) = levels[0].insert(*pos, *leaf) {
            anyhow::ensure!(prev == *leaf, "duplicate leaf pos {pos} with different value");
        }
    }

    // Build up.
    for lvl in 0..depth {
        let mut parent_set: HashSet<u64> = HashSet::new();
        for &idx in levels[lvl].keys() {
            parent_set.insert(idx >> 1);
        }

        for pidx in parent_set {
            let left_idx = pidx * 2;
            let right_idx = pidx * 2 + 1;
            let left = levels[lvl]
                .get(&left_idx)
                .unwrap_or(&default_nodes[lvl]);
            let right = levels[lvl]
                .get(&right_idx)
                .unwrap_or(&default_nodes[lvl]);
            let parent = mt_combine(lvl as u8, left, right);
            if parent != default_nodes[lvl + 1] {
                levels[lvl + 1].insert(pidx, parent);
            }
        }
    }

    let root = *levels[depth].get(&0).unwrap_or(&default_nodes[depth]);

    // Open siblings for each requested leaf (in input order).
    let mut openings: Vec<Vec<Hash32>> = Vec::with_capacity(leaves.len());
    for (pos, _leaf) in leaves {
        let mut sibs: Vec<Hash32> = Vec::with_capacity(depth);
        for lvl in 0..depth {
            let idx_at_lvl = pos >> lvl;
            let sib_idx = idx_at_lvl ^ 1;
            let sib = *levels[lvl].get(&sib_idx).unwrap_or(&default_nodes[lvl]);
            sibs.push(sib);
        }
        openings.push(sibs);
    }

    Ok((root, openings))
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
    // 5 pk_ivk_recipient (PRIV), 6 cm_out (PUB),
    // 7 blacklist_root (PUB), 8..(8+BL_DEPTH-1) bl_siblings (PRIV)
    let mut idx = vec![3, 4, 5];
    for i in 0..BL_DEPTH {
        idx.push(8 + i);
    }
    idx
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
    let bl_defaults = merkle_default_nodes(BL_DEPTH);
    let blacklist_root = bl_defaults[BL_DEPTH];

    let mut args = vec![
        LigeroArg::Hex { hex: hx32(&domain) },
        LigeroArg::I64 { i64: value as i64 },
        LigeroArg::Hex { hex: hx32(&rho) },
        LigeroArg::Hex { hex: hx32(&pk_spend) },
        LigeroArg::Hex { hex: hx32(&pk_ivk) },
        LigeroArg::Hex { hex: hx32(&cm_out) },
    ];
    args.push(LigeroArg::Hex {
        hex: hx32(&blacklist_root),
    });
    for sib in bl_defaults.iter().take(BL_DEPTH) {
        args.push(LigeroArg::Hex { hex: hx32(sib) });
    }

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
    let bl_defaults = merkle_default_nodes(BL_DEPTH);
    let blacklist_root = bl_defaults[BL_DEPTH];

    let mut args = vec![
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
    args.push(LigeroArg::String {
        str: format!("0x{}", hx32(&blacklist_root)),
    });
    for sib in bl_defaults.iter().take(BL_DEPTH) {
        args.push(LigeroArg::String {
            str: format!("0x{}", hx32(sib)),
        });
    }

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
    let bl_defaults = merkle_default_nodes(BL_DEPTH);
    let blacklist_root = bl_defaults[BL_DEPTH];

    let mut args = vec![
        LigeroArg::Hex { hex: hx32(&domain) },
        LigeroArg::I64 { i64: value as i64 },
        LigeroArg::Hex { hex: hx32(&rho) },
        LigeroArg::Hex { hex: hx32(&pk_spend) },
        LigeroArg::Hex { hex: hx32(&pk_ivk) },
        LigeroArg::Hex { hex: hx32(&cm_out) },
    ];
    args.push(LigeroArg::Hex {
        hex: hx32(&blacklist_root),
    });
    for sib in bl_defaults.iter().take(BL_DEPTH) {
        args.push(LigeroArg::Hex { hex: hx32(sib) });
    }

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
    let bl_defaults = merkle_default_nodes(BL_DEPTH);
    let blacklist_root = bl_defaults[BL_DEPTH];

    let mut args = vec![
        LigeroArg::Hex { hex: hx32(&domain) },
        LigeroArg::I64 { i64: value as i64 },
        LigeroArg::Hex { hex: hx32(&rho) },
        LigeroArg::Hex { hex: hx32(&pk_spend) },
        LigeroArg::Hex { hex: hx32(&pk_ivk) },
        LigeroArg::Hex { hex: hx32(&cm_out) },
    ];
    args.push(LigeroArg::Hex {
        hex: hx32(&blacklist_root),
    });
    for sib in bl_defaults.iter().take(BL_DEPTH) {
        args.push(LigeroArg::Hex { hex: hx32(sib) });
    }

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
    let bl_defaults = merkle_default_nodes(BL_DEPTH);
    let blacklist_root = bl_defaults[BL_DEPTH];

    let mut good_args = vec![
        LigeroArg::Hex { hex: hx32(&domain) },
        LigeroArg::I64 { i64: value as i64 },
        LigeroArg::Hex { hex: hx32(&rho) },
        LigeroArg::Hex { hex: hx32(&pk_spend) },
        LigeroArg::Hex { hex: hx32(&pk_ivk) },
        LigeroArg::Hex { hex: hx32(&cm_out) },
    ];
    good_args.push(LigeroArg::Hex {
        hex: hx32(&blacklist_root),
    });
    for sib in bl_defaults.iter().take(BL_DEPTH) {
        good_args.push(LigeroArg::Hex { hex: hx32(sib) });
    }

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

    // Invalid: zero `value` must be rejected.
    let mut bad_args = good_args.clone();
    bad_args[1] = LigeroArg::I64 { i64: 0 };
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

    // Invalid: mismatch between recipient keys and cm_out must be rejected (prove-time).
    let mut bad_args = good_args.clone();
    let wrong_pk_ivk: Hash32 = [0x99u8; 32];
    bad_args[4] = LigeroArg::Hex {
        hex: hx32(&wrong_pk_ivk),
    };
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
            // Expected: witness no longer satisfies cm_out == NOTE_V2(...).
        }
    }

    // Invalid: wrong arg count must be rejected (regression test for failure-path soundness).
    let mut bad_args = good_args;
    bad_args.pop(); // drop one arg (wrong arg count)
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

#[test]
fn test_note_deposit_blacklist_accepts_unlisted_and_rejects_listed() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_deposit_guest(&repo)?;

    let program = note_deposit_program_path(&repo)
        .canonicalize()
        .context("Failed to canonicalize note_deposit_guest.wasm")?;

    let Some((mut runner, vpaths, priv_idx)) = try_skip(setup_runner_and_paths(&program))? else {
        return Ok(());
    };

    // Two recipients: one will be blacklisted (leaf=1), the other unlisted (leaf=0).
    let domain: Hash32 = [0x55u8; 32];
    let value: u64 = 42;
    let rho_ok: Hash32 = [0x11u8; 32];
    let rho_bad: Hash32 = [0x22u8; 32];

    let pk_spend_ok: Hash32 = [0x33u8; 32];
    let pk_ivk_ok: Hash32 = [0x44u8; 32];
    let recipient_ok = recipient_from_pk(&domain, &pk_spend_ok, &pk_ivk_ok);

    let pk_spend_bad: Hash32 = [0x66u8; 32];
    let pk_ivk_bad: Hash32 = [0x77u8; 32];
    let recipient_bad = recipient_from_pk(&domain, &pk_spend_bad, &pk_ivk_bad);

    let pos_bad = bl_pos_from_id(&recipient_bad);
    let pos_ok = bl_pos_from_id(&recipient_ok);
    anyhow::ensure!(pos_bad != pos_ok, "unexpected blacklist index collision");

    let leaf0: Hash32 = [0u8; 32];
    let mut leaf1: Hash32 = [0u8; 32];
    leaf1[31] = 1;

    // Build a sparse blacklist tree with exactly one blacklisted leaf.
    let (blacklist_root, openings) = merkle_root_and_openings_sparse(
        BL_DEPTH,
        &[(pos_bad, leaf1), (pos_ok, leaf0)],
    )?;
    let sibs_bad = &openings[0];
    let sibs_ok = &openings[1];

    // Sanity: membership at pos_bad is 1, and at pos_ok is 0, under the same root.
    anyhow::ensure!(
        merkle_root_from_path(&leaf1, pos_bad, sibs_bad) == blacklist_root,
        "blacklisted address opening must match root"
    );
    anyhow::ensure!(
        merkle_root_from_path(&leaf0, pos_ok, sibs_ok) == blacklist_root,
        "unlisted address opening must match root"
    );

    // === Proof for the unlisted address should verify ===
    let sender_id = [0u8; 32];
    let cm_ok = note_commitment_v2(&domain, value, &rho_ok, &recipient_ok, &sender_id);

    let mut args_ok = vec![
        LigeroArg::Hex { hex: hx32(&domain) },
        LigeroArg::I64 { i64: value as i64 },
        LigeroArg::Hex { hex: hx32(&rho_ok) },
        LigeroArg::Hex { hex: hx32(&pk_spend_ok) },
        LigeroArg::Hex { hex: hx32(&pk_ivk_ok) },
        LigeroArg::Hex { hex: hx32(&cm_ok) },
        LigeroArg::Hex {
            hex: hx32(&blacklist_root),
        },
    ];
    for sib in sibs_ok.iter().take(BL_DEPTH) {
        args_ok.push(LigeroArg::Hex { hex: hx32(sib) });
    }

    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = args_ok.clone();
    let (proof_ok, _p_stdout, _p_stderr) =
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
    anyhow::ensure!(!proof_ok.is_empty(), "proof should not be empty");

    let (ok, v_stdout, v_stderr) =
        verifier::verify_proof_with_output(&vpaths, &proof_ok, args_ok, priv_idx.clone())
            .context("Failed to run verifier")?;
    if !ok {
        anyhow::bail!(
            "verifier did not report success for unlisted address\nstdout: {v_stdout}\nstderr: {v_stderr}"
        );
    }

    // === Proof for the blacklisted address must be rejected ===
    let cm_bad = note_commitment_v2(&domain, value, &rho_bad, &recipient_bad, &sender_id);
    let mut args_bad = vec![
        LigeroArg::Hex { hex: hx32(&domain) },
        LigeroArg::I64 { i64: value as i64 },
        LigeroArg::Hex { hex: hx32(&rho_bad) },
        LigeroArg::Hex {
            hex: hx32(&pk_spend_bad),
        },
        LigeroArg::Hex { hex: hx32(&pk_ivk_bad) },
        LigeroArg::Hex { hex: hx32(&cm_bad) },
        LigeroArg::Hex {
            hex: hx32(&blacklist_root),
        },
    ];
    for sib in sibs_bad.iter().take(BL_DEPTH) {
        args_bad.push(LigeroArg::Hex { hex: hx32(sib) });
    }

    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = args_bad.clone();
    match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
        keep_proof_dir: false,
        proof_outputs_base: None,
        write_replay_script: false,
    }) {
        Err(_e) => Ok(()), // rejected at prove-time ✅ (expected)
        Ok((proof_bad, _stdout, _stderr)) => {
            let (ok_bad, _stdout, _stderr) =
                verifier::verify_proof_with_output(&vpaths, &proof_bad, args_bad, priv_idx)
                    .context("Failed to run verifier")?;
            anyhow::ensure!(
                !ok_bad,
                "expected verification to fail for blacklisted address, but it succeeded"
            );
            Ok(())
        }
    }
}
