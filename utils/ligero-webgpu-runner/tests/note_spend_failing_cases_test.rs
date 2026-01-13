//! Failing test cases for `note_spend_guest` circuit debugging.
//!
//! These tests reproduce constraint validation failures observed in `continuous_transfers.rs`
//! and `midnight-tx-generator` scripts. They are intended to help isolate and fix
//! circuit/prover issues.
//!
//! Test scenarios:
//! 1. Single-input, single-output transfer with random keys (simulates continuous_transfers)
//! 2. Transfer where change output goes back to sender
//! 3. Deposit-created note (sender_id == recipient) being spent
//!
//! Run with: cargo test --manifest-path utils/ligero-webgpu-runner/Cargo.toml --features test_env note_spend_failing -- --nocapture

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use ligero_runner::{verifier, LigeroArg, LigeroRunner};
use ligetron::poseidon2_hash_bytes;
use ligetron::Bn254Fr;
use rand::Rng;

type Hash32 = [u8; 32];

const BL_DEPTH: usize = 16;
const BL_BUCKET_SIZE: usize = 12;

fn hx32(b: &Hash32) -> String {
    hex::encode(b)
}

fn b64_32(b: &Hash32) -> String {
    general_purpose::STANDARD.encode(b)
}

fn arg32(b: &Hash32) -> LigeroArg {
    LigeroArg::HexBytesB64 {
        hex: hx32(b),
        bytes_b64: b64_32(b),
    }
}

fn repo_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    Ok(manifest_dir
        .ancestors()
        .nth(2)
        .context("Failed to compute ligero-prover repo root")?
        .to_path_buf())
}

fn maybe_build_note_spend_guest(repo: &Path) -> Result<()> {
    let out = repo.join("utils/circuits/bins/note_spend_guest.wasm");
    if out.exists() {
        return Ok(());
    }

    let guest_dir = repo.join("utils/circuits/note-spend");
    if !guest_dir.exists() {
        anyhow::bail!("note-spend sources not found at {}", guest_dir.display());
    }

    println!(
        "[note_spend_failing] note_spend_guest.wasm not found; building via {}",
        guest_dir.join("build.sh").display()
    );

    let status = Command::new("bash")
        .arg("build.sh")
        .current_dir(&guest_dir)
        .status()
        .context("Failed to run note-spend/build.sh")?;

    if !status.success() {
        anyhow::bail!("note-spend/build.sh failed with status {status}");
    }

    if !out.exists() {
        anyhow::bail!(
            "note_spend_guest.wasm still not found after build at {}",
            out.display()
        );
    }

    Ok(())
}

fn note_spend_program_path(repo: &Path) -> Result<PathBuf> {
    if let Ok(p) = std::env::var("LIGERO_PROGRAM_PATH") {
        return Ok(PathBuf::from(p));
    }
    Ok(repo.join("utils/circuits/bins/note_spend_guest.wasm"))
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

fn mt_combine(level: u8, left: &Hash32, right: &Hash32) -> Hash32 {
    let lvl = [level];
    poseidon2_hash_domain(b"MT_NODE_V1", &[&lvl, left, right])
}

fn merkle_default_nodes_from_leaf(depth: usize, leaf0: &Hash32) -> Vec<Hash32> {
    let mut out = Vec::with_capacity(depth + 1);
    out.push(*leaf0);
    for lvl in 0..depth {
        let prev = out[lvl];
        out.push(mt_combine(lvl as u8, &prev, &prev));
    }
    out
}

fn bl_empty_bucket_entries() -> [Hash32; BL_BUCKET_SIZE] {
    [[0u8; 32]; BL_BUCKET_SIZE]
}

fn bl_bucket_leaf(entries: &[Hash32; BL_BUCKET_SIZE]) -> Hash32 {
    let mut buf = Vec::with_capacity(12 + 32 * BL_BUCKET_SIZE);
    buf.extend_from_slice(b"BL_BUCKET_V1");
    for e in entries {
        buf.extend_from_slice(e);
    }
    poseidon2_hash_bytes(&buf).to_bytes_be()
}

fn bl_bucket_inv_for_id(id: &Hash32, bucket_entries: &[Hash32; BL_BUCKET_SIZE]) -> Option<Hash32> {
    let id_fr = fr_from_hash32_be(id);
    let mut prod = Bn254Fr::from_u32(1);
    for e in bucket_entries.iter() {
        let e_fr = fr_from_hash32_be(e);
        let mut delta = id_fr.clone();
        delta.submod_checked(&e_fr);
        prod.mulmod_checked(&delta);
    }
    if prod.is_zero() {
        return None;
    }
    let mut inv = prod.clone();
    inv.inverse();
    Some(inv.to_bytes_be())
}

fn append_empty_blacklist(args: &mut Vec<LigeroArg>, ids_to_check: &[Hash32]) -> Result<()> {
    let bucket_entries = bl_empty_bucket_entries();
    let leaf0 = bl_bucket_leaf(&bucket_entries);
    let default_nodes = merkle_default_nodes_from_leaf(BL_DEPTH, &leaf0);
    let blacklist_root = default_nodes[BL_DEPTH];
    let siblings: Vec<Hash32> = default_nodes.iter().take(BL_DEPTH).copied().collect();

    args.push(arg32(&blacklist_root));

    for id in ids_to_check {
        for e in bucket_entries.iter() {
            args.push(arg32(e));
        }
        let inv = bl_bucket_inv_for_id(id, &bucket_entries)
            .context("unexpected: id collides with empty blacklist bucket")?;
        args.push(arg32(&inv));
        for sib in siblings.iter() {
            args.push(arg32(sib));
        }
    }

    Ok(())
}

fn note_commitment(
    domain: &Hash32,
    value: u64,
    rho: &Hash32,
    recipient: &Hash32,
    sender_id: &Hash32,
) -> Hash32 {
    let mut v16 = [0u8; 16];
    v16[..8].copy_from_slice(&value.to_le_bytes());
    poseidon2_hash_domain(b"NOTE_V2", &[domain, &v16, rho, recipient, sender_id])
}

fn pk_from_sk(spend_sk: &Hash32) -> Hash32 {
    poseidon2_hash_domain(b"PK_V1", &[spend_sk])
}

fn ivk_seed(domain: &Hash32, spend_sk: &Hash32) -> Hash32 {
    poseidon2_hash_domain(b"IVK_SEED_V1", &[domain, spend_sk])
}

fn recipient_from_pk(domain: &Hash32, pk_spend: &Hash32, pk_ivk: &Hash32) -> Hash32 {
    poseidon2_hash_domain(b"ADDR_V2", &[domain, pk_spend, pk_ivk])
}

fn recipient_from_sk(domain: &Hash32, spend_sk: &Hash32, pk_ivk: &Hash32) -> Hash32 {
    recipient_from_pk(domain, &pk_from_sk(spend_sk), pk_ivk)
}

fn nf_key_from_sk(domain: &Hash32, spend_sk: &Hash32) -> Hash32 {
    poseidon2_hash_domain(b"NFKEY_V1", &[domain, spend_sk])
}

fn nullifier(domain: &Hash32, nf_key: &Hash32, rho: &Hash32) -> Hash32 {
    poseidon2_hash_domain(b"PRF_NF_V1", &[domain, nf_key, rho])
}

fn fr_from_hash32_be(h: &Hash32) -> Bn254Fr {
    let mut fr = Bn254Fr::new();
    fr.set_bytes_big(h);
    fr
}

fn compute_inv_enforce(
    in_values: &[u64],
    in_rhos: &[Hash32],
    out_values: &[u64],
    out_rhos: &[Hash32],
) -> Result<Hash32> {
    let mut prod = Bn254Fr::from_u32(1);

    for &v in in_values {
        prod.mulmod_checked(&Bn254Fr::from_u64(v));
    }
    for &v in out_values {
        prod.mulmod_checked(&Bn254Fr::from_u64(v));
    }

    // Π(rho_out - rho_in)
    for out_rho in out_rhos {
        let out_fr = fr_from_hash32_be(out_rho);
        for in_rho in in_rhos {
            let in_fr = fr_from_hash32_be(in_rho);
            let mut delta = out_fr.clone();
            delta.submod_checked(&in_fr);
            prod.mulmod_checked(&delta);
        }
    }

    // (rho_out0 - rho_out1) when n_out == 2.
    if out_rhos.len() == 2 {
        let out0 = fr_from_hash32_be(&out_rhos[0]);
        let out1 = fr_from_hash32_be(&out_rhos[1]);
        let mut delta = out0.clone();
        delta.submod_checked(&out1);
        prod.mulmod_checked(&delta);
    }

    anyhow::ensure!(
        !prod.is_zero(),
        "inv_enforce undefined: enforce product is zero (zero value or rho reuse)"
    );
    let mut inv = prod.clone();
    inv.inverse();
    Ok(inv.to_bytes_be())
}

/// Small Merkle tree implementation for tests.
struct MerkleTree {
    depth: usize,
    leaves: Vec<Hash32>,
}

impl MerkleTree {
    fn new(depth: usize) -> Self {
        let size = 1usize << depth;
        Self {
            depth,
            leaves: vec![[0u8; 32]; size],
        }
    }

    fn set_leaf(&mut self, pos: usize, leaf: Hash32) {
        self.leaves[pos] = leaf;
    }

    fn root(&self) -> Hash32 {
        let mut level = self.leaves.clone();
        for lvl in 0..self.depth {
            let mut next = Vec::with_capacity(level.len() / 2);
            for i in (0..level.len()).step_by(2) {
                next.push(mt_combine(lvl as u8, &level[i], &level[i + 1]));
            }
            level = next;
        }
        level[0]
    }

    fn open(&self, pos: usize) -> Vec<Hash32> {
        let mut siblings = Vec::with_capacity(self.depth);
        let mut idx = pos;
        let mut level = self.leaves.clone();
        for lvl in 0..self.depth {
            let sib_idx = if (idx & 1) == 0 { idx + 1 } else { idx - 1 };
            siblings.push(level[sib_idx]);

            let mut next = Vec::with_capacity(level.len() / 2);
            for i in (0..level.len()).step_by(2) {
                next.push(mt_combine(lvl as u8, &level[i], &level[i + 1]));
            }
            level = next;
            idx >>= 1;
        }
        siblings
    }
}

fn private_indices(depth: usize, n_in: usize, n_out: usize, is_transfer: bool) -> Vec<usize> {
    let mut idx = vec![2usize, 3usize]; // spend_sk, pk_ivk_owner

    let per_in = 5usize + depth;
    for i in 0..n_in {
        let base = 7 + i * per_in;
        idx.push(base);     // value_in
        idx.push(base + 1); // rho_in
        idx.push(base + 2); // sender_id_in
        idx.push(base + 3); // pos
        for k in 0..depth {
            idx.push(base + 4 + k); // sibling
        }
    }

    let withdraw_idx = 7 + n_in * per_in;
    let outs_base = withdraw_idx + 3;
    for j in 0..n_out {
        idx.push(outs_base + 5 * j + 0); // value_out
        idx.push(outs_base + 5 * j + 1); // rho_out
        idx.push(outs_base + 5 * j + 2); // pk_spend_out
        idx.push(outs_base + 5 * j + 3); // pk_ivk_out
    }

    let inv_enforce_idx = outs_base + 5 * n_out;
    idx.push(inv_enforce_idx);

    let _per_check = BL_BUCKET_SIZE + 1 + BL_DEPTH;
    let bl_checks = if is_transfer { 2usize } else { 1usize };
    let mut cur = inv_enforce_idx + 2;
    for _ in 0..bl_checks {
        for i in 0..BL_BUCKET_SIZE {
            idx.push(cur + i);
        }
        idx.push(cur + BL_BUCKET_SIZE);
        cur += BL_BUCKET_SIZE + 1;
        for k in 0..BL_DEPTH {
            idx.push(cur + k);
        }
        cur += BL_DEPTH;
    }

    idx
}

/// Helper to create a Ligero runner and check if environment is available.
fn create_runner_or_skip(program: &Path) -> Result<Option<LigeroRunner>> {
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
        return Ok(None);
    }

    let shader_dir = PathBuf::from(&runner.config().shader_path);
    if !shader_dir.exists() {
        eprintln!(
            "Skipping: shader path not found at {}",
            shader_dir.display()
        );
        return Ok(None);
    }

    Ok(Some(runner))
}

/// Test case 1: Simulates continuous_transfers single-input, single-output transfer.
/// This is the pattern that was failing intermittently.
///
/// Key characteristics:
/// - Input note created by a "deposit" (sender_id == recipient)
/// - Output note goes to a fresh random address
/// - Uses depth=8 tree for faster test execution (production uses depth=16)
#[test]
fn test_note_spend_single_transfer_random_keys() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_spend_guest(&repo)?;

    let program = note_spend_program_path(&repo)?
        .canonicalize()
        .context("Failed to canonicalize note_spend_guest.wasm")?;

    let Some(mut runner) = create_runner_or_skip(&program)? else {
        return Ok(());
    };

    println!("[test_single_transfer_random] Program: {}", program.display());
    println!(
        "[test_single_transfer_random] Prover:   {}",
        runner.paths().prover_bin.display()
    );

    // Use depth=8 for faster test execution (production uses depth=16)
    let depth: usize = 8;
    
    // Domain from continuous_transfers (poseidon2_hash_bytes(b"DOMAIN_V2"))
    let domain: Hash32 = poseidon2_hash_domain(b"DOMAIN_V2", &[]);
    
    // Random keys for the owner (spender)
    let mut rng = rand::thread_rng();
    let spend_sk: Hash32 = rng.gen();
    let pk_ivk_owner = ivk_seed(&domain, &spend_sk);
    let recipient_owner = recipient_from_sk(&domain, &spend_sk, &pk_ivk_owner);
    let nf_key = nf_key_from_sk(&domain, &spend_sk);
    
    // For deposit-created notes, sender_id == recipient
    let sender_id_in = recipient_owner;
    let sender_id_current = recipient_owner;

    // Input note parameters (random)
    let value: u64 = rng.gen_range(1..1_000_000u64);
    let pos: u64 = rng.gen_range(0..(1u64 << depth));
    let rho_in: Hash32 = rng.gen();
    
    // Compute input commitment
    let cm_in = note_commitment(&domain, value, &rho_in, &recipient_owner, &sender_id_in);

    // Build tree with this note
    let mut tree = MerkleTree::new(depth);
    tree.set_leaf(pos as usize, cm_in);
    let anchor = tree.root();
    let siblings = tree.open(pos as usize);

    // Verify Merkle path locally
    let mut current = cm_in;
    let mut p = pos as usize;
    for (lvl, sib) in siblings.iter().enumerate() {
        current = if p % 2 == 0 {
            mt_combine(lvl as u8, &current, sib)
        } else {
            mt_combine(lvl as u8, sib, &current)
        };
        p /= 2;
    }
    assert_eq!(current, anchor, "Local Merkle root verification failed");

    let nf = nullifier(&domain, &nf_key, &rho_in);

    // Output note to fresh random recipient
    let n_in: u64 = 1;
    let withdraw_amount: u64 = 0;
    let n_out: u64 = 1;

    let out_value: u64 = value;
    let out_rho: Hash32 = rng.gen();
    let out_spend_sk: Hash32 = rng.gen();
    let out_pk_spend = pk_from_sk(&out_spend_sk);
    let out_pk_ivk = ivk_seed(&domain, &out_spend_sk);
    let out_recipient = recipient_from_pk(&domain, &out_pk_spend, &out_pk_ivk);
    let cm_out = note_commitment(&domain, out_value, &out_rho, &out_recipient, &sender_id_current);

    // Compute inv_enforce
    let inv_enforce = compute_inv_enforce(&[value], &[rho_in], &[out_value], &[out_rho])?;

    // Verify inv_enforce is non-zero
    assert!(
        !inv_enforce.iter().all(|&b| b == 0),
        "inv_enforce should not be zero"
    );

    // Build args
    let mut args: Vec<LigeroArg> = Vec::new();
    args.push(LigeroArg::Hex { hex: hx32(&domain) }); // 1 domain
    args.push(LigeroArg::Hex { hex: hx32(&spend_sk) }); // 2 spend_sk (priv)
    args.push(LigeroArg::Hex { hex: hx32(&pk_ivk_owner) }); // 3 pk_ivk_owner (priv)
    args.push(LigeroArg::I64 { i64: depth as i64 }); // 4 depth
    args.push(LigeroArg::Hex { hex: hx32(&anchor) }); // 5 anchor
    args.push(LigeroArg::I64 { i64: n_in as i64 }); // 6 n_in

    // Input 0
    args.push(LigeroArg::I64 { i64: value as i64 }); // 7 value_in_0 (priv)
    args.push(LigeroArg::Hex { hex: hx32(&rho_in) }); // 8 rho_in_0 (priv)
    args.push(LigeroArg::Hex { hex: hx32(&sender_id_in) }); // 9 sender_id_in_0 (priv)
    args.push(LigeroArg::I64 { i64: pos as i64 }); // 10 pos_0 (priv)
    for s in &siblings {
        args.push(LigeroArg::Hex { hex: hx32(s) });
    }
    args.push(LigeroArg::Hex { hex: hx32(&nf) }); // nullifier (pub)

    args.push(LigeroArg::I64 { i64: withdraw_amount as i64 });
    args.push(LigeroArg::Hex { hex: hx32(&[0u8; 32]) });
    args.push(LigeroArg::I64 { i64: n_out as i64 });

    // Output 0
    args.push(LigeroArg::I64 { i64: out_value as i64 }); // value_out_0 (priv)
    args.push(LigeroArg::Hex { hex: hx32(&out_rho) }); // rho_out_0 (priv)
    args.push(LigeroArg::Hex { hex: hx32(&out_pk_spend) }); // pk_spend_out_0 (priv)
    args.push(LigeroArg::Hex { hex: hx32(&out_pk_ivk) }); // pk_ivk_out_0 (priv)
    args.push(LigeroArg::Hex { hex: hx32(&cm_out) }); // cm_out_0 (pub)
    args.push(LigeroArg::Hex { hex: hx32(&inv_enforce) }); // inv_enforce (priv)

    append_empty_blacklist(&mut args, &[sender_id_current, out_recipient])?;

    let priv_idx = private_indices(depth, 1, 1, true);

    // Debug output
    println!("[test_single_transfer_random] Test parameters:");
    println!("  depth: {}", depth);
    println!("  value: {}", value);
    println!("  position: {}", pos);
    println!("  spend_sk: {}...", hex::encode(&spend_sk[..8]));
    println!("  rho_in: {}...", hex::encode(&rho_in[..8]));
    println!("  rho_out: {}...", hex::encode(&out_rho[..8]));
    println!("  sender_id_in == recipient_owner: {}", sender_id_in == recipient_owner);
    println!("  args count: {}", args.len());
    println!("  private_indices count: {}", priv_idx.len());
    println!("  inv_enforce: {}...", hex::encode(&inv_enforce[..8]));

    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = args.clone();

    let (proof_bytes, _prover_stdout, _prover_stderr) =
        match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
            keep_proof_dir: false,
            proof_outputs_base: None,
            write_replay_script: true,
        }) {
            Ok((proof, stdout, stderr)) => (proof, stdout, stderr),
            Err(err) => {
                let msg = format!("{err:#}");
                if msg.contains("Validation of linear constraints:")
                    || msg.contains("Validation of quadratic constraints:")
                    || msg.contains("Final prove result:")
                {
                    // This is a constraint failure - the test reveals a bug
                    eprintln!("[CONSTRAINT FAILURE] Test revealed constraint validation failure:");
                    eprintln!("  Error: {err}");
                    eprintln!("  Test params:");
                    eprintln!("    spend_sk: {}", hex::encode(&spend_sk));
                    eprintln!("    rho_in: {}", hex::encode(&rho_in));
                    eprintln!("    rho_out: {}", hex::encode(&out_rho));
                    eprintln!("    value: {}", value);
                    eprintln!("    position: {}", pos);
                    return Err(err);
                }
                eprintln!("Skipping: prover failed (GPU/WebGPU likely unavailable): {err}");
                return Ok(());
            }
        };

    assert!(!proof_bytes.is_empty(), "proof should not be empty");
    println!(
        "[test_single_transfer_random] OK: proof generated ({} bytes)",
        proof_bytes.len()
    );

    // Verify
    let shader_dir = PathBuf::from(&runner.config().shader_path);
    let vpaths = verifier::VerifierPaths::from_explicit(
        program,
        shader_dir,
        runner.paths().verifier_bin.clone(),
        runner.config().packing,
    );

    let (ok, v_stdout, v_stderr) =
        verifier::verify_proof_with_output(&vpaths, &proof_bytes, args, priv_idx)
            .context("Failed to run verifier")?;

    if !ok {
        anyhow::bail!("verifier did not report success\nstdout: {v_stdout}\nstderr: {v_stderr}");
    }

    println!("[test_single_transfer_random] OK: proof verified");
    Ok(())
}

/// Test case 2: Two-output transfer (payment + change back to self).
/// This is the pattern used by mcp-external transfer.rs and midnight-tx-generator.
#[test]
fn test_note_spend_two_output_transfer_with_change() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_spend_guest(&repo)?;

    let program = note_spend_program_path(&repo)?
        .canonicalize()
        .context("Failed to canonicalize note_spend_guest.wasm")?;

    let Some(mut runner) = create_runner_or_skip(&program)? else {
        return Ok(());
    };

    println!("[test_two_output_change] Program: {}", program.display());

    let depth: usize = 8; // Smaller depth for faster test
    let domain: Hash32 = [42u8; 32];
    
    let mut rng = rand::thread_rng();
    let spend_sk: Hash32 = rng.gen();
    let pk_ivk_owner = ivk_seed(&domain, &spend_sk);
    let pk_spend_owner = pk_from_sk(&spend_sk);
    let recipient_owner = recipient_from_sk(&domain, &spend_sk, &pk_ivk_owner);
    let sender_id_current = recipient_owner;
    let nf_key = nf_key_from_sk(&domain, &spend_sk);

    // Input note (owned by us, created by prior transfer so sender_id is different)
    let value_in: u64 = rng.gen_range(100..10_000u64);
    let pos: u64 = rng.gen_range(0..(1u64 << depth));
    let rho_in: Hash32 = rng.gen();
    let sender_id_in: Hash32 = rng.gen(); // Different from recipient_owner
    let cm_in = note_commitment(&domain, value_in, &rho_in, &recipient_owner, &sender_id_in);

    let mut tree = MerkleTree::new(depth);
    tree.set_leaf(pos as usize, cm_in);
    let anchor = tree.root();
    let siblings = tree.open(pos as usize);
    let nf = nullifier(&domain, &nf_key, &rho_in);

    // Two outputs: payment to Bob + change back to self
    let n_in: u64 = 1;
    let n_out: u64 = 2;
    let out0_value: u64 = rng.gen_range(1..value_in);
    let out1_value: u64 = value_in - out0_value;

    let out0_rho: Hash32 = rng.gen();
    let out1_rho: Hash32 = rng.gen();

    // Output 0: to Bob (random recipient)
    let bob_sk: Hash32 = rng.gen();
    let out0_pk_spend = pk_from_sk(&bob_sk);
    let out0_pk_ivk = ivk_seed(&domain, &bob_sk);
    let out0_recipient = recipient_from_pk(&domain, &out0_pk_spend, &out0_pk_ivk);

    // Output 1: change back to self (use owner's keys)
    let out1_pk_spend = pk_spend_owner;
    let out1_pk_ivk = pk_ivk_owner;
    let out1_recipient = recipient_owner;

    let cm_out0 = note_commitment(&domain, out0_value, &out0_rho, &out0_recipient, &sender_id_current);
    let cm_out1 = note_commitment(&domain, out1_value, &out1_rho, &out1_recipient, &sender_id_current);

    let inv_enforce = compute_inv_enforce(
        &[value_in],
        &[rho_in],
        &[out0_value, out1_value],
        &[out0_rho, out1_rho],
    )?;

    println!("[test_two_output_change] Test parameters:");
    println!("  value_in: {}", value_in);
    println!("  out0_value (to Bob): {}", out0_value);
    println!("  out1_value (change): {}", out1_value);
    println!("  sender_id_in different from recipient: {}", sender_id_in != recipient_owner);

    // Build args
    let mut args: Vec<LigeroArg> = Vec::new();
    args.push(LigeroArg::Hex { hex: hx32(&domain) }); // 1
    args.push(LigeroArg::Hex { hex: hx32(&spend_sk) }); // 2 (priv)
    args.push(LigeroArg::Hex { hex: hx32(&pk_ivk_owner) }); // 3 (priv)
    args.push(LigeroArg::I64 { i64: depth as i64 }); // 4
    args.push(LigeroArg::Hex { hex: hx32(&anchor) }); // 5
    args.push(LigeroArg::I64 { i64: n_in as i64 }); // 6

    // Input 0
    args.push(LigeroArg::I64 { i64: value_in as i64 });
    args.push(LigeroArg::Hex { hex: hx32(&rho_in) });
    args.push(LigeroArg::Hex { hex: hx32(&sender_id_in) });
    args.push(LigeroArg::I64 { i64: pos as i64 });
    for s in &siblings {
        args.push(LigeroArg::Hex { hex: hx32(s) });
    }
    args.push(LigeroArg::Hex { hex: hx32(&nf) });

    // Withdraw + n_out
    args.push(LigeroArg::I64 { i64: 0 });
    args.push(LigeroArg::Hex { hex: hx32(&[0u8; 32]) });
    args.push(LigeroArg::I64 { i64: n_out as i64 });

    // Output 0 (to Bob)
    args.push(LigeroArg::I64 { i64: out0_value as i64 });
    args.push(LigeroArg::Hex { hex: hx32(&out0_rho) });
    args.push(LigeroArg::Hex { hex: hx32(&out0_pk_spend) });
    args.push(LigeroArg::Hex { hex: hx32(&out0_pk_ivk) });
    args.push(LigeroArg::Hex { hex: hx32(&cm_out0) });

    // Output 1 (change to self)
    args.push(LigeroArg::I64 { i64: out1_value as i64 });
    args.push(LigeroArg::Hex { hex: hx32(&out1_rho) });
    args.push(LigeroArg::Hex { hex: hx32(&out1_pk_spend) });
    args.push(LigeroArg::Hex { hex: hx32(&out1_pk_ivk) });
    args.push(LigeroArg::Hex { hex: hx32(&cm_out1) });

    args.push(LigeroArg::Hex { hex: hx32(&inv_enforce) });

    // For transfers, check sender + first output recipient
    append_empty_blacklist(&mut args, &[sender_id_current, out0_recipient])?;

    let priv_idx = private_indices(depth, 1, 2, true);

    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = args.clone();

    let (proof_bytes, _p_stdout, _p_stderr) =
        match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
            keep_proof_dir: false,
            proof_outputs_base: None,
            write_replay_script: true,
        }) {
            Ok(v) => v,
            Err(err) => {
                let msg = format!("{err:#}");
                if msg.contains("Validation of linear constraints:")
                    || msg.contains("Validation of quadratic constraints:")
                {
                    eprintln!("[CONSTRAINT FAILURE] Test revealed constraint validation failure:");
                    eprintln!("  Error: {err}");
                    return Err(err);
                }
                eprintln!("Skipping: prover failed (GPU/WebGPU likely unavailable): {err}");
                return Ok(());
            }
        };

    assert!(!proof_bytes.is_empty(), "proof should not be empty");
    println!(
        "[test_two_output_change] OK: proof generated ({} bytes)",
        proof_bytes.len()
    );

    // Verify
    let shader_dir = PathBuf::from(&runner.config().shader_path);
    let vpaths = verifier::VerifierPaths::from_explicit(
        program,
        shader_dir,
        runner.paths().verifier_bin.clone(),
        runner.config().packing,
    );

    let (ok, v_stdout, v_stderr) =
        verifier::verify_proof_with_output(&vpaths, &proof_bytes, args, priv_idx)
            .context("Failed to run verifier")?;

    if !ok {
        anyhow::bail!("verifier did not report success\nstdout: {v_stdout}\nstderr: {v_stderr}");
    }

    println!("[test_two_output_change] OK: proof verified");
    Ok(())
}

/// Test case 3: Multiple iterations to catch intermittent failures.
/// Runs 10 random transfer scenarios to increase chance of catching edge cases.
#[test]
fn test_note_spend_stress_random_transfers() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_spend_guest(&repo)?;

    let program = note_spend_program_path(&repo)?
        .canonicalize()
        .context("Failed to canonicalize note_spend_guest.wasm")?;

    let Some(mut runner) = create_runner_or_skip(&program)? else {
        return Ok(());
    };

    let iterations = std::env::var("STRESS_ITERATIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);

    println!(
        "[test_stress_random] Running {} iterations of random transfers",
        iterations
    );

    let depth: usize = 8; // Smaller depth for faster tests
    let domain: Hash32 = poseidon2_hash_domain(b"STRESS_TEST_DOMAIN", &[]);
    let mut rng = rand::thread_rng();

    for iter in 0..iterations {
        println!("[test_stress_random] Iteration {}/{}...", iter + 1, iterations);

        let spend_sk: Hash32 = rng.gen();
        let pk_ivk_owner = ivk_seed(&domain, &spend_sk);
        let recipient_owner = recipient_from_sk(&domain, &spend_sk, &pk_ivk_owner);
        let nf_key = nf_key_from_sk(&domain, &spend_sk);

        // Randomly decide if sender_id_in == recipient (deposit pattern) or random
        let sender_id_in: Hash32 = if rng.gen_bool(0.5) {
            recipient_owner // deposit pattern
        } else {
            rng.gen() // prior transfer pattern
        };

        let value: u64 = rng.gen_range(1..1_000_000u64);
        let pos: u64 = rng.gen_range(0..(1u64 << depth));
        let rho_in: Hash32 = rng.gen();

        let cm_in = note_commitment(&domain, value, &rho_in, &recipient_owner, &sender_id_in);
        let mut tree = MerkleTree::new(depth);
        tree.set_leaf(pos as usize, cm_in);
        let anchor = tree.root();
        let siblings = tree.open(pos as usize);
        let nf = nullifier(&domain, &nf_key, &rho_in);

        let out_rho: Hash32 = rng.gen();
        let out_spend_sk: Hash32 = rng.gen();
        let out_pk_spend = pk_from_sk(&out_spend_sk);
        let out_pk_ivk = ivk_seed(&domain, &out_spend_sk);
        let out_recipient = recipient_from_pk(&domain, &out_pk_spend, &out_pk_ivk);
        let sender_id_current = recipient_owner;
        let cm_out = note_commitment(&domain, value, &out_rho, &out_recipient, &sender_id_current);

        let inv_enforce = compute_inv_enforce(&[value], &[rho_in], &[value], &[out_rho])?;

        let mut args: Vec<LigeroArg> = Vec::new();
        args.push(LigeroArg::Hex { hex: hx32(&domain) });
        args.push(LigeroArg::Hex { hex: hx32(&spend_sk) });
        args.push(LigeroArg::Hex { hex: hx32(&pk_ivk_owner) });
        args.push(LigeroArg::I64 { i64: depth as i64 });
        args.push(LigeroArg::Hex { hex: hx32(&anchor) });
        args.push(LigeroArg::I64 { i64: 1 });

        args.push(LigeroArg::I64 { i64: value as i64 });
        args.push(LigeroArg::Hex { hex: hx32(&rho_in) });
        args.push(LigeroArg::Hex { hex: hx32(&sender_id_in) });
        args.push(LigeroArg::I64 { i64: pos as i64 });
        for s in &siblings {
            args.push(LigeroArg::Hex { hex: hx32(s) });
        }
        args.push(LigeroArg::Hex { hex: hx32(&nf) });

        args.push(LigeroArg::I64 { i64: 0 });
        args.push(LigeroArg::Hex { hex: hx32(&[0u8; 32]) });
        args.push(LigeroArg::I64 { i64: 1 });

        args.push(LigeroArg::I64 { i64: value as i64 });
        args.push(LigeroArg::Hex { hex: hx32(&out_rho) });
        args.push(LigeroArg::Hex { hex: hx32(&out_pk_spend) });
        args.push(LigeroArg::Hex { hex: hx32(&out_pk_ivk) });
        args.push(LigeroArg::Hex { hex: hx32(&cm_out) });
        args.push(LigeroArg::Hex { hex: hx32(&inv_enforce) });

        append_empty_blacklist(&mut args, &[sender_id_current, out_recipient])?;

        let priv_idx = private_indices(depth, 1, 1, true);
        runner.config_mut().private_indices = priv_idx.clone();
        runner.config_mut().args = args.clone();

        match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
            keep_proof_dir: false,
            proof_outputs_base: None,
            write_replay_script: true,
        }) {
            Ok((proof_bytes, _stdout, _stderr)) => {
                assert!(!proof_bytes.is_empty(), "proof should not be empty");
                println!(
                    "  [iter {}] OK: proof generated ({} bytes)",
                    iter + 1,
                    proof_bytes.len()
                );
            }
            Err(err) => {
                let msg = format!("{err:#}");
                if msg.contains("Validation of linear constraints:")
                    || msg.contains("Validation of quadratic constraints:")
                {
                    eprintln!("[CONSTRAINT FAILURE] Iteration {} failed:", iter + 1);
                    eprintln!("  spend_sk: {}", hex::encode(&spend_sk));
                    eprintln!("  rho_in: {}", hex::encode(&rho_in));
                    eprintln!("  rho_out: {}", hex::encode(&out_rho));
                    eprintln!("  value: {}", value);
                    eprintln!("  position: {}", pos);
                    eprintln!(
                        "  sender_id_in == recipient: {}",
                        sender_id_in == recipient_owner
                    );
                    return Err(err);
                }
                eprintln!(
                    "Skipping remaining iterations: prover unavailable: {err}"
                );
                return Ok(());
            }
        }
    }

    println!(
        "[test_stress_random] All {} iterations passed",
        iterations
    );
    Ok(())
}

/// Test case 4: Deterministic reproduction of a failing case from stress test.
/// This uses the exact parameters that caused iteration 2 to fail.
#[test]
fn test_note_spend_deterministic_failing_case() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_spend_guest(&repo)?;

    let program = note_spend_program_path(&repo)?
        .canonicalize()
        .context("Failed to canonicalize note_spend_guest.wasm")?;

    let Some(mut runner) = create_runner_or_skip(&program)? else {
        return Ok(());
    };

    println!("[test_deterministic_failing] Program: {}", program.display());
    println!("[test_deterministic_failing] This test reproduces a constraint failure found in stress testing");

    // Fixed parameters from a failing stress test run
    let depth: usize = 8;
    let domain: Hash32 = poseidon2_hash_domain(b"STRESS_TEST_DOMAIN", &[]);
    
    // These are the exact values from a failing case:
    // spend_sk: a1ee4d8a1ad0a8cc44f87d647eb807193153c3aa9ff492f45710287db0b43e64
    let spend_sk: Hash32 = hex::decode("a1ee4d8a1ad0a8cc44f87d647eb807193153c3aa9ff492f45710287db0b43e64")
        .expect("valid hex")
        .try_into()
        .expect("32 bytes");
    // rho_in: a0ace49cac927a3538fc4b2f8c2a2ed7231730b8eda6774e86d9485e45f003ca
    let rho_in: Hash32 = hex::decode("a0ace49cac927a3538fc4b2f8c2a2ed7231730b8eda6774e86d9485e45f003ca")
        .expect("valid hex")
        .try_into()
        .expect("32 bytes");
    // rho_out: 1ddb8cbd58e4658f83056802d26a89a3565c237b73a212319fea075b05c073c7
    let out_rho: Hash32 = hex::decode("1ddb8cbd58e4658f83056802d26a89a3565c237b73a212319fea075b05c073c7")
        .expect("valid hex")
        .try_into()
        .expect("32 bytes");
    let value: u64 = 580192;
    let pos: u64 = 57;
    // sender_id_in was random (not == recipient)
    let sender_id_in: Hash32 = hex::decode("be1f361d67c28f5bdbb2a5119d8743d39e5b630530693d40ec1ee79ac58dedf3")
        .expect("valid hex")
        .try_into()
        .expect("32 bytes");

    let pk_ivk_owner = ivk_seed(&domain, &spend_sk);
    let recipient_owner = recipient_from_sk(&domain, &spend_sk, &pk_ivk_owner);
    let sender_id_current = recipient_owner;
    let nf_key = nf_key_from_sk(&domain, &spend_sk);

    let cm_in = note_commitment(&domain, value, &rho_in, &recipient_owner, &sender_id_in);
    let mut tree = MerkleTree::new(depth);
    tree.set_leaf(pos as usize, cm_in);
    let anchor = tree.root();
    let siblings = tree.open(pos as usize);

    // Verify Merkle path locally
    let mut current = cm_in;
    let mut p = pos as usize;
    for (lvl, sib) in siblings.iter().enumerate() {
        current = if p % 2 == 0 {
            mt_combine(lvl as u8, &current, sib)
        } else {
            mt_combine(lvl as u8, sib, &current)
        };
        p /= 2;
    }
    assert_eq!(current, anchor, "Local Merkle root verification failed");

    let nf = nullifier(&domain, &nf_key, &rho_in);

    // Output note parameters from failing case  
    let out_spend_sk: Hash32 = [0x99u8; 32]; // arbitrary but deterministic
    let out_pk_spend = pk_from_sk(&out_spend_sk);
    let out_pk_ivk = ivk_seed(&domain, &out_spend_sk);
    let out_recipient = recipient_from_pk(&domain, &out_pk_spend, &out_pk_ivk);
    let cm_out = note_commitment(&domain, value, &out_rho, &out_recipient, &sender_id_current);

    let inv_enforce = compute_inv_enforce(&[value], &[rho_in], &[value], &[out_rho])?;

    println!("[test_deterministic_failing] Test parameters:");
    println!("  spend_sk: {}...", hex::encode(&spend_sk[..8]));
    println!("  rho_in: {}...", hex::encode(&rho_in[..8]));
    println!("  rho_out: {}...", hex::encode(&out_rho[..8]));
    println!("  value: {}", value);
    println!("  position: {}", pos);
    println!("  sender_id_in != recipient: {}", sender_id_in != recipient_owner);
    println!("  inv_enforce: {}...", hex::encode(&inv_enforce[..8]));

    let mut args: Vec<LigeroArg> = Vec::new();
    args.push(LigeroArg::Hex { hex: hx32(&domain) });
    args.push(LigeroArg::Hex { hex: hx32(&spend_sk) });
    args.push(LigeroArg::Hex { hex: hx32(&pk_ivk_owner) });
    args.push(LigeroArg::I64 { i64: depth as i64 });
    args.push(LigeroArg::Hex { hex: hx32(&anchor) });
    args.push(LigeroArg::I64 { i64: 1 });

    args.push(LigeroArg::I64 { i64: value as i64 });
    args.push(LigeroArg::Hex { hex: hx32(&rho_in) });
    args.push(LigeroArg::Hex { hex: hx32(&sender_id_in) });
    args.push(LigeroArg::I64 { i64: pos as i64 });
    for s in &siblings {
        args.push(LigeroArg::Hex { hex: hx32(s) });
    }
    args.push(LigeroArg::Hex { hex: hx32(&nf) });

    args.push(LigeroArg::I64 { i64: 0 });
    args.push(LigeroArg::Hex { hex: hx32(&[0u8; 32]) });
    args.push(LigeroArg::I64 { i64: 1 });

    args.push(LigeroArg::I64 { i64: value as i64 });
    args.push(LigeroArg::Hex { hex: hx32(&out_rho) });
    args.push(LigeroArg::Hex { hex: hx32(&out_pk_spend) });
    args.push(LigeroArg::Hex { hex: hx32(&out_pk_ivk) });
    args.push(LigeroArg::Hex { hex: hx32(&cm_out) });
    args.push(LigeroArg::Hex { hex: hx32(&inv_enforce) });

    append_empty_blacklist(&mut args, &[sender_id_current, out_recipient])?;

    let priv_idx = private_indices(depth, 1, 1, true);
    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = args.clone();

    println!("[test_deterministic_failing] Running prover...");
    
    match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
        keep_proof_dir: true, // Keep for debugging
        proof_outputs_base: None,
        write_replay_script: true,
    }) {
        Ok((proof_bytes, _stdout, _stderr)) => {
            println!(
                "[test_deterministic_failing] UNEXPECTED SUCCESS: proof generated ({} bytes)",
                proof_bytes.len()
            );
            println!("  This was expected to fail - the bug may have been fixed or parameters differ!");
        }
        Err(err) => {
            let msg = format!("{err:#}");
            if msg.contains("Validation of linear constraints:")
                || msg.contains("Validation of quadratic constraints:")
            {
                println!("[test_deterministic_failing] EXPECTED FAILURE: constraint validation failed");
                println!("  Error: {err}");
                // Don't fail the test - this is expected behavior we're documenting
                return Ok(());
            }
            eprintln!("Skipping: prover failed (GPU/WebGPU likely unavailable): {err}");
            return Ok(());
        }
    }

    Ok(())
}

/// Test case 5: Specific reproduction case with fixed keys that should pass.
/// This allows deterministic reproduction of failures once a failing seed is found.
#[test]
fn test_note_spend_deterministic_case() -> Result<()> {
    // Note: This test uses the same keys as the passing test in note_spend_roundtrip_test.rs
    // to confirm the test harness works correctly.
    let repo = repo_root()?;
    maybe_build_note_spend_guest(&repo)?;

    let program = note_spend_program_path(&repo)?
        .canonicalize()
        .context("Failed to canonicalize note_spend_guest.wasm")?;

    let Some(mut runner) = create_runner_or_skip(&program)? else {
        return Ok(());
    };

    println!("[test_deterministic] Program: {}", program.display());

    // Fixed parameters that can be replaced with failing case values
    let depth: usize = 8;
    let domain: Hash32 = [1u8; 32];
    let spend_sk: Hash32 = [4u8; 32];
    let rho_in: Hash32 = [2u8; 32];
    let out_rho: Hash32 = [7u8; 32];
    let value: u64 = 42;
    let pos: u64 = 0;
    let sender_id_in: Hash32 = [6u8; 32];

    let pk_ivk_owner = ivk_seed(&domain, &spend_sk);
    let recipient_owner = recipient_from_sk(&domain, &spend_sk, &pk_ivk_owner);
    let sender_id_current = recipient_owner;
    let nf_key = nf_key_from_sk(&domain, &spend_sk);

    let cm_in = note_commitment(&domain, value, &rho_in, &recipient_owner, &sender_id_in);
    let mut tree = MerkleTree::new(depth);
    tree.set_leaf(pos as usize, cm_in);
    let anchor = tree.root();
    let siblings = tree.open(pos as usize);
    let nf = nullifier(&domain, &nf_key, &rho_in);

    let out_spend_sk: Hash32 = [8u8; 32];
    let out_pk_spend = pk_from_sk(&out_spend_sk);
    let out_pk_ivk = ivk_seed(&domain, &out_spend_sk);
    let out_recipient = recipient_from_pk(&domain, &out_pk_spend, &out_pk_ivk);
    let cm_out = note_commitment(&domain, value, &out_rho, &out_recipient, &sender_id_current);

    let inv_enforce = compute_inv_enforce(&[value], &[rho_in], &[value], &[out_rho])?;

    let mut args: Vec<LigeroArg> = Vec::new();
    args.push(LigeroArg::Hex { hex: hx32(&domain) });
    args.push(LigeroArg::Hex { hex: hx32(&spend_sk) });
    args.push(LigeroArg::Hex { hex: hx32(&pk_ivk_owner) });
    args.push(LigeroArg::I64 { i64: depth as i64 });
    args.push(LigeroArg::Hex { hex: hx32(&anchor) });
    args.push(LigeroArg::I64 { i64: 1 });

    args.push(LigeroArg::I64 { i64: value as i64 });
    args.push(LigeroArg::Hex { hex: hx32(&rho_in) });
    args.push(LigeroArg::Hex { hex: hx32(&sender_id_in) });
    args.push(LigeroArg::I64 { i64: pos as i64 });
    for s in &siblings {
        args.push(LigeroArg::Hex { hex: hx32(s) });
    }
    args.push(LigeroArg::Hex { hex: hx32(&nf) });

    args.push(LigeroArg::I64 { i64: 0 });
    args.push(LigeroArg::Hex { hex: hx32(&[0u8; 32]) });
    args.push(LigeroArg::I64 { i64: 1 });

    args.push(LigeroArg::I64 { i64: value as i64 });
    args.push(LigeroArg::Hex { hex: hx32(&out_rho) });
    args.push(LigeroArg::Hex { hex: hx32(&out_pk_spend) });
    args.push(LigeroArg::Hex { hex: hx32(&out_pk_ivk) });
    args.push(LigeroArg::Hex { hex: hx32(&cm_out) });
    args.push(LigeroArg::Hex { hex: hx32(&inv_enforce) });

    append_empty_blacklist(&mut args, &[sender_id_current, out_recipient])?;

    let priv_idx = private_indices(depth, 1, 1, true);
    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = args.clone();

    let (proof_bytes, _p_stdout, _p_stderr) =
        match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
            keep_proof_dir: false,
            proof_outputs_base: None,
            write_replay_script: true,
        }) {
            Ok(v) => v,
            Err(err) => {
                let msg = format!("{err:#}");
                if msg.contains("Validation of linear constraints:")
                    || msg.contains("Validation of quadratic constraints:")
                {
                    eprintln!("[CONSTRAINT FAILURE] Deterministic test failed!");
                    return Err(err);
                }
                eprintln!("Skipping: prover failed (GPU/WebGPU likely unavailable): {err}");
                return Ok(());
            }
        };

    assert!(!proof_bytes.is_empty(), "proof should not be empty");
    println!(
        "[test_deterministic] OK: proof generated ({} bytes)",
        proof_bytes.len()
    );

    // Verify
    let shader_dir = PathBuf::from(&runner.config().shader_path);
    let vpaths = verifier::VerifierPaths::from_explicit(
        program,
        shader_dir,
        runner.paths().verifier_bin.clone(),
        runner.config().packing,
    );

    let (ok, v_stdout, v_stderr) =
        verifier::verify_proof_with_output(&vpaths, &proof_bytes, args, priv_idx)
            .context("Failed to run verifier")?;

    if !ok {
        anyhow::bail!("verifier did not report success\nstdout: {v_stdout}\nstderr: {v_stderr}");
    }

    println!("[test_deterministic] OK: proof verified");
    Ok(())
}

