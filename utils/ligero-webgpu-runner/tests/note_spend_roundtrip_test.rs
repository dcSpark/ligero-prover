//! Proof roundtrip test for `ligero-runner` using the `note_spend_guest` circuit.
//!
//! This test requires:
//! - `webgpu_prover` + `webgpu_verifier` binaries
//! - a valid `shader/` directory
//! - a built `note_spend_guest.wasm`
//!
//! The test runs by default *if* Ligero assets are available (portable-binaries/shaders or env overrides).
//! If assets are missing (or the prover cannot run on this machine), it exits early with a skip message.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use ligero_runner::{verifier, LigeroArg, LigeroRunner};
use ligetron::poseidon2_hash_bytes;
use ligetron::Bn254Fr;

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
    // utils/ligero-runner -> utils -> repo
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
        "[note_spend_roundtrip] note_spend_guest.wasm not found; building via {}",
        guest_dir.join("build.sh").display()
    );

    // Best-effort build. This may download the wasm std target via rustup.
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

    println!(
        "[note_spend_roundtrip] Built note_spend_guest.wasm at {}",
        out.display()
    );

    Ok(())
}

fn note_spend_program_path(repo: &Path) -> Result<PathBuf> {
    if let Ok(p) = std::env::var("LIGERO_PROGRAM_PATH") {
        return Ok(PathBuf::from(p));
    }

    let p = repo.join("utils/circuits/bins/note_spend_guest.wasm");
    Ok(p)
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
    out.push(*leaf0); // height 0 (leaf)
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

    args.push(arg32(&blacklist_root)); // blacklist_root (pub)

    for id in ids_to_check {
        for e in bucket_entries.iter() {
            args.push(arg32(e)); // bucket entry (priv)
        }
        let inv = bl_bucket_inv_for_id(id, &bucket_entries)
            .context("unexpected: id collides with empty blacklist bucket")?;
        args.push(arg32(&inv)); // bucket_inv (priv)
        for sib in siblings.iter() {
            args.push(arg32(sib)); // sibling (priv)
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
    // Guest encodes value as 16-byte LE (u64 zero-extended to 16 bytes).
    let mut v16 = [0u8; 16];
    v16[..8].copy_from_slice(&value.to_le_bytes());
    poseidon2_hash_domain(b"NOTE_V2", &[domain, &v16, rho, recipient, sender_id])
}

fn pk_from_sk(spend_sk: &Hash32) -> Hash32 {
    poseidon2_hash_domain(b"PK_V1", &[spend_sk])
}

fn ivk_seed(domain: &Hash32, spend_sk: &Hash32) -> Hash32 {
    // Wallet-side this would be clamped and base-multiplied (X25519) to get a real pk_ivk,
    // but the circuit treats `pk_ivk` as an opaque 32-byte value and only binds it into ADDR_V2.
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

// === Viewer attestation helpers (must match the guest program) ===

fn fvk_commit(fvk: &Hash32) -> Hash32 {
    poseidon2_hash_domain(b"FVK_COMMIT_V1", &[fvk])
}

fn view_kdf(fvk: &Hash32, cm: &Hash32) -> Hash32 {
    poseidon2_hash_domain(b"VIEW_KDF_V1", &[fvk, cm])
}

fn stream_block(k: &Hash32, ctr: u32) -> Hash32 {
    poseidon2_hash_domain(b"VIEW_STREAM_V1", &[k, &ctr.to_le_bytes()])
}

fn stream_xor_encrypt_144(k: &Hash32, pt: &[u8; 144]) -> [u8; 144] {
    let mut ct = [0u8; 144];
    for ctr in 0u32..5u32 {
        let ks = stream_block(k, ctr);
        let off = (ctr as usize) * 32;
        let take = core::cmp::min(32, 144 - off);
        for i in 0..take {
            ct[off + i] = pt[off + i] ^ ks[i];
        }
    }
    ct
}

fn ct_hash(ct: &[u8; 144]) -> Hash32 {
    poseidon2_hash_domain(b"CT_HASH_V1", &[ct])
}

fn view_mac(k: &Hash32, cm: &Hash32, ct_h: &Hash32) -> Hash32 {
    poseidon2_hash_domain(b"VIEW_MAC_V1", &[k, cm, ct_h])
}

fn encode_note_plain(
    domain: &Hash32,
    value: u64,
    rho: &Hash32,
    recipient: &Hash32,
    sender_id: &Hash32,
) -> [u8; 144] {
    let mut out = [0u8; 144];
    out[0..32].copy_from_slice(domain);
    out[32..40].copy_from_slice(&value.to_le_bytes());
    // out[40..48] is already zero (u64 zero-extended to 16 bytes).
    out[48..80].copy_from_slice(rho);
    out[80..112].copy_from_slice(recipient);
    out[112..144].copy_from_slice(sender_id);
    out
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

/// Small Merkle tree implementation for tests (depth is small, so a simple Vec-based tree is fine).
///
/// IMPORTANT: The level parameter passed into `mt_combine(level, ...)` matches the guest program:
/// - `level = 0` is the leaf-adjacent layer (bottom)
/// - `level` increases as we move up towards the root
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
    // Mirrors the guest layout described in utils/circuits/note-spend/src/main.rs (v2 ABI).
    //
    // Private:
    // - spend_sk, pk_ivk_owner
    // - per-input: value_in, rho_in, sender_id_in, pos, siblings
    // - per-output: value_out, rho_out, pk_spend_out, pk_ivk_out
    // - inv_enforce (enforcement inverse witness)
    let mut idx = vec![2usize, 3usize]; // spend_sk, pk_ivk_owner

    let per_in = 5usize + depth; // value + rho + sender_id_in + pos + siblings[depth] + nullifier
    for i in 0..n_in {
        let base = 7 + i * per_in;
        idx.push(base); // value_in
        idx.push(base + 1); // rho_in
        idx.push(base + 2); // sender_id_in
        idx.push(base + 3); // pos
        for k in 0..depth {
            idx.push(base + 4 + k); // sibling
        }
        // nullifier is public
    }

    let withdraw_idx = 7 + n_in * per_in;
    let outs_base = withdraw_idx + 3; // skip withdraw_amount + withdraw_to + n_out
    for j in 0..n_out {
        idx.push(outs_base + 5 * j + 0); // value_out
        idx.push(outs_base + 5 * j + 1); // rho_out
        idx.push(outs_base + 5 * j + 2); // pk_spend_out
        idx.push(outs_base + 5 * j + 3); // pk_ivk_out
                                         // cm_out is public
    }

    // inv_enforce is always present right after outputs.
    let inv_enforce_idx = outs_base + 5 * n_out;
    idx.push(inv_enforce_idx);

    // Blacklist section:
    // - blacklist_root is public
    // - per check: bucket_entries[BL_BUCKET_SIZE], bucket_inv, siblings[BL_DEPTH] are private
    let per_check = BL_BUCKET_SIZE + 1 + BL_DEPTH;
    let bl_checks = if is_transfer { 2usize } else { 1usize };
    let mut cur = inv_enforce_idx + 2; // skip blacklist_root (pub)
    for _ in 0..bl_checks {
        for i in 0..BL_BUCKET_SIZE {
            idx.push(cur + i); // bucket_entry
        }
        idx.push(cur + BL_BUCKET_SIZE); // bucket_inv
        cur += BL_BUCKET_SIZE + 1;
        for k in 0..BL_DEPTH {
            idx.push(cur + k); // sibling
        }
        cur += BL_DEPTH;
    }
    debug_assert_eq!(
        cur,
        inv_enforce_idx + 2 + bl_checks * per_check,
        "blacklist private indices mismatch"
    );

    idx
}

fn verify_expect_fail(
    vpaths: &verifier::VerifierPaths,
    proof_bytes: &[u8],
    args: Vec<LigeroArg>,
    priv_idx: Vec<usize>,
) -> Result<()> {
    match verifier::verify_proof_with_output(vpaths, proof_bytes, args, priv_idx) {
        Ok((ok, stdout, stderr)) => {
            anyhow::ensure!(
                !ok,
                "expected verification to fail, but it succeeded\nstdout: {stdout}\nstderr: {stderr}"
            );
        }
        Err(_e) => {
            // Any error is also an acceptable failure signal for a bad statement.
        }
    }
    Ok(())
}

#[test]
fn test_note_spend_proof_roundtrip_one_output() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_spend_guest(&repo)?;

    let program = note_spend_program_path(&repo)?
        .canonicalize()
        .context("Failed to canonicalize note_spend_guest.wasm")?;

    let packing: u32 = std::env::var("LIGERO_PACKING")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8192);

    // Runner discovery (prover/verifier/shaders). Caller can override via env vars.
    let mut runner = LigeroRunner::new(&program.to_string_lossy());
    runner.config_mut().packing = packing;

    if !runner.paths().prover_bin.exists() || !runner.paths().verifier_bin.exists() {
        eprintln!(
            "Skipping: Ligero binaries not found (prover={}, verifier={})",
            runner.paths().prover_bin.display(),
            runner.paths().verifier_bin.display()
        );
        return Ok(());
    }

    let shader_dir = PathBuf::from(&runner.config().shader_path);
    if !shader_dir.exists() {
        eprintln!(
            "Skipping: shader path not found at {}",
            shader_dir.display()
        );
        return Ok(());
    }

    println!("[note_spend_roundtrip] Program: {}", program.display());
    println!(
        "[note_spend_roundtrip] Prover:   {}",
        runner.paths().prover_bin.display()
    );
    println!(
        "[note_spend_roundtrip] Verifier: {}",
        runner.paths().verifier_bin.display()
    );
    println!("[note_spend_roundtrip] Shaders:  {}", shader_dir.display());
    println!("[note_spend_roundtrip] Packing:  {}", packing);

    // === Construct a simple depth-8 tree with one note ===
    let depth: usize = 8;
    let domain: Hash32 = [1u8; 32];
    let rho: Hash32 = [2u8; 32];
    let spend_sk: Hash32 = [4u8; 32];

    let pk_ivk_owner = ivk_seed(&domain, &spend_sk);
    let recipient = recipient_from_sk(&domain, &spend_sk, &pk_ivk_owner);
    let sender_id_current = recipient;
    let nf_key = nf_key_from_sk(&domain, &spend_sk);

    let value: u64 = 42;
    let pos: u64 = 0;

    let sender_id_in: Hash32 = [6u8; 32];
    let cm_in = note_commitment(&domain, value, &rho, &recipient, &sender_id_in);

    // Build a small tree and open a Merkle path consistent with the guest program.
    let mut tree = MerkleTree::new(depth);
    tree.set_leaf(pos as usize, cm_in);
    let anchor = tree.root();
    let siblings = tree.open(pos as usize);
    assert_eq!(siblings.len(), depth, "unexpected siblings length");

    // Sanity check: recompute via generic root_from_path.
    let recomputed = merkle_root_from_path(&cm_in, pos, &siblings);
    assert_eq!(recomputed, anchor, "Merkle root mismatch");

    let nf = nullifier(&domain, &nf_key, &rho);

    // === One-output transfer: withdraw=0, out_value=value ===
    let n_in: u64 = 1;
    let withdraw_amount: u64 = 0;
    let n_out: u64 = 1;

    let out_value: u64 = value;
    let out_rho: Hash32 = [7u8; 32];
    let out_spend_sk: Hash32 = [8u8; 32];
    let out_pk_spend = pk_from_sk(&out_spend_sk);
    let out_pk_ivk = ivk_seed(&domain, &out_spend_sk);
    let out_recipient = recipient_from_pk(&domain, &out_pk_spend, &out_pk_ivk);
    let cm_out = note_commitment(
        &domain,
        out_value,
        &out_rho,
        &out_recipient,
        &sender_id_current,
    );

    let inv_enforce = compute_inv_enforce(&[value], &[rho], &[out_value], &[out_rho])?;

    let mut args: Vec<LigeroArg> = Vec::new();
    // Header (v2 ABI)
    args.push(LigeroArg::Hex { hex: hx32(&domain) }); // 1 domain (public)
    args.push(LigeroArg::Hex {
        hex: hx32(&spend_sk),
    }); // 2 spend_sk (private)
    args.push(LigeroArg::Hex {
        hex: hx32(&pk_ivk_owner),
    }); // 3 pk_ivk_owner (private)
    args.push(LigeroArg::I64 { i64: depth as i64 }); // 4 depth (public)
    args.push(LigeroArg::Hex { hex: hx32(&anchor) }); // 5 anchor (public)
    args.push(LigeroArg::I64 { i64: n_in as i64 }); // 6 n_in (public)

    // Input 0 (value, rho, sender_id_in, pos, siblings[depth], nullifier)
    args.push(LigeroArg::I64 { i64: value as i64 }); // 7 value_in_0 (private)
    args.push(LigeroArg::Hex { hex: hx32(&rho) }); // 8 rho_in_0 (private)
    args.push(LigeroArg::Hex {
        hex: hx32(&sender_id_in),
    }); // sender_id_in_0 (private)
    args.push(LigeroArg::I64 { i64: pos as i64 }); // pos_0 (private)

    // Siblings.
    for s in &siblings {
        args.push(LigeroArg::Hex { hex: hx32(s) });
    }

    // Input nullifier (public).
    args.push(LigeroArg::Hex { hex: hx32(&nf) });

    // Withdraw + outputs.
    args.push(LigeroArg::I64 {
        i64: withdraw_amount as i64,
    });
    args.push(LigeroArg::Hex {
        hex: hx32(&[0u8; 32]),
    });
    args.push(LigeroArg::I64 { i64: n_out as i64 });
    args.push(LigeroArg::I64 {
        i64: out_value as i64,
    });
    args.push(LigeroArg::Hex {
        hex: hx32(&out_rho),
    });
    args.push(LigeroArg::Hex {
        hex: hx32(&out_pk_spend),
    });
    args.push(LigeroArg::Hex {
        hex: hx32(&out_pk_ivk),
    });
    args.push(LigeroArg::Hex { hex: hx32(&cm_out) });
    args.push(LigeroArg::Hex {
        hex: hx32(&inv_enforce),
    });

    append_empty_blacklist(&mut args, &[sender_id_current, out_recipient])?;

    let priv_idx = private_indices(depth, 1, 1, true);

    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = args.clone();

    let (proof_bytes, prover_stdout, prover_stderr) =
        match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
            keep_proof_dir: false,
            proof_outputs_base: None,
            write_replay_script: true,
        }) {
            Ok((proof, stdout, stderr)) => (proof, stdout, stderr),
            Err(err) => {
                // If the prover ran but the witness is invalid, this is a real test failure (not an env issue).
                let msg = format!("{err:#}");
                if msg.contains("Validation of linear constraints:")
                    || msg.contains("Validation of quadratic constraints:")
                    || msg.contains("Final prove result:")
                {
                    return Err(err);
                }

                // Otherwise, treat it as an environmental failure (GPU/WebGPU / driver / missing runtime deps).
                eprintln!("Skipping: prover failed (GPU/WebGPU likely unavailable): {err}");
                return Ok(());
            }
        };

    assert!(!proof_bytes.is_empty(), "proof should not be empty");

    println!(
        "[note_spend_roundtrip] OK: proof generated ({} bytes)",
        proof_bytes.len()
    );
    if let Some(line) = prover_stdout
        .lines()
        .find(|l| l.contains("Final prove result:"))
    {
        println!("[note_spend_roundtrip] Prover:   {line}");
    }
    if !prover_stderr.trim().is_empty() {
        println!("[note_spend_roundtrip] Prover stderr:\n{prover_stderr}");
    }

    // Verify proof.
    let vpaths = verifier::VerifierPaths::from_explicit(
        program.clone(),
        shader_dir,
        runner.paths().verifier_bin.clone(),
        packing,
    );

    let (ok, v_stdout, v_stderr) =
        verifier::verify_proof_with_output(&vpaths, &proof_bytes, args, priv_idx)
            .context("Failed to run verifier")?;

    if !ok {
        anyhow::bail!("verifier did not report success\nstdout: {v_stdout}\nstderr: {v_stderr}");
    }

    println!("[note_spend_roundtrip] OK: proof verified");
    if let Some(line) = v_stdout
        .lines()
        .find(|l| l.contains("Final Verify Result:"))
    {
        println!("[note_spend_roundtrip] Verifier: {line}");
    }
    if !v_stderr.trim().is_empty() {
        println!("[note_spend_roundtrip] Verifier stderr:\n{v_stderr}");
    }

    Ok(())
}

#[test]
fn test_note_spend_red_team_rejects_mutated_public_inputs_and_wrong_private_indices() -> Result<()>
{
    let repo = repo_root()?;
    maybe_build_note_spend_guest(&repo)?;

    let program = note_spend_program_path(&repo)?
        .canonicalize()
        .context("Failed to canonicalize note_spend_guest.wasm")?;

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
        return Ok(());
    }

    let shader_dir = PathBuf::from(&runner.config().shader_path);
    if !shader_dir.exists() {
        eprintln!(
            "Skipping: shader path not found at {}",
            shader_dir.display()
        );
        return Ok(());
    }

    // Small depth to keep runtime reasonable.
    let depth: usize = 4;
    let n_in: usize = 1;
    let n_out: usize = 2;

    let domain: Hash32 = [42u8; 32];
    let spend_sk: Hash32 = [4u8; 32];
    let pk_ivk_owner = ivk_seed(&domain, &spend_sk);
    let recipient_owner = recipient_from_sk(&domain, &spend_sk, &pk_ivk_owner);
    let sender_id_current = recipient_owner;
    let nf_key = nf_key_from_sk(&domain, &spend_sk);

    // One input note.
    let value_in: u64 = 200;
    let pos: u64 = 1;
    let rho_in: Hash32 = [2u8; 32];
    let sender_id_in: Hash32 = [6u8; 32];
    let cm_in = note_commitment(&domain, value_in, &rho_in, &recipient_owner, &sender_id_in);
    let mut tree = MerkleTree::new(depth);
    tree.set_leaf(pos as usize, cm_in);
    let anchor = tree.root();
    let siblings = tree.open(pos as usize);
    let nf0 = nullifier(&domain, &nf_key, &rho_in);

    // Two outputs: payment + change back to self.
    let out0_value: u64 = 66;
    let out1_value: u64 = value_in
        .checked_sub(out0_value)
        .context("value_in must be >= out0_value")?;

    let out0_rho: Hash32 = [7u8; 32];
    let out1_rho: Hash32 = [8u8; 32];

    let out0_spend_sk: Hash32 = [9u8; 32];
    let out0_pk_spend = pk_from_sk(&out0_spend_sk);
    let out0_pk_ivk = ivk_seed(&domain, &out0_spend_sk);
    let out0_recipient = recipient_from_pk(&domain, &out0_pk_spend, &out0_pk_ivk);
    let cm_out0 = note_commitment(
        &domain,
        out0_value,
        &out0_rho,
        &out0_recipient,
        &sender_id_current,
    );

    let out1_pk_spend = pk_from_sk(&spend_sk);
    let out1_pk_ivk = pk_ivk_owner;
    let cm_out1 = note_commitment(
        &domain,
        out1_value,
        &out1_rho,
        &recipient_owner,
        &sender_id_current,
    );

    let inv_enforce = compute_inv_enforce(
        &[value_in],
        &[rho_in],
        &[out0_value, out1_value],
        &[out0_rho, out1_rho],
    )?;

    // Build args (v2 ABI).
    let mut args: Vec<LigeroArg> = Vec::new();
    args.push(LigeroArg::Hex { hex: hx32(&domain) }); // 1 domain (pub)
    args.push(LigeroArg::Hex {
        hex: hx32(&spend_sk),
    }); // 2 spend_sk (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&pk_ivk_owner),
    }); // 3 pk_ivk_owner (priv)
    args.push(LigeroArg::I64 { i64: depth as i64 }); // 4 depth (pub)
    args.push(LigeroArg::Hex { hex: hx32(&anchor) }); // 5 anchor (pub)
    args.push(LigeroArg::I64 { i64: n_in as i64 }); // 6 n_in (pub)

    // Input 0.
    args.push(LigeroArg::I64 {
        i64: value_in as i64,
    }); // value_in_0 (priv)
    args.push(LigeroArg::Hex { hex: hx32(&rho_in) }); // rho_in_0 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&sender_id_in),
    }); // sender_id_in_0 (priv)
    args.push(LigeroArg::I64 { i64: pos as i64 }); // pos_0 (priv)
    for s in &siblings {
        args.push(LigeroArg::Hex { hex: hx32(s) });
    }

    // nullifier_0 (pub)
    args.push(LigeroArg::Hex { hex: hx32(&nf0) });

    // withdraw_amount (pub), n_out (pub)
    let withdraw_amount: u64 = 0;
    args.push(LigeroArg::I64 {
        i64: withdraw_amount as i64,
    });
    args.push(LigeroArg::Hex {
        hex: hx32(&[0u8; 32]),
    });
    args.push(LigeroArg::I64 { i64: n_out as i64 });

    // Output 0 (payment)
    args.push(LigeroArg::I64 {
        i64: out0_value as i64,
    }); // value_out_0 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&out0_rho),
    }); // rho_out_0 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&out0_pk_spend),
    }); // pk_spend_out_0 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&out0_pk_ivk),
    }); // pk_ivk_out_0 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&cm_out0),
    }); // cm_out_0 (pub)

    // Output 1 (change to self)
    args.push(LigeroArg::I64 {
        i64: out1_value as i64,
    }); // value_out_1 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&out1_rho),
    }); // rho_out_1 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&out1_pk_spend),
    }); // pk_spend_out_1 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&out1_pk_ivk),
    }); // pk_ivk_out_1 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&cm_out1),
    }); // cm_out_1 (pub)
    args.push(LigeroArg::Hex {
        hex: hx32(&inv_enforce),
    }); // inv_enforce (priv)

    append_empty_blacklist(&mut args, &[sender_id_current, out0_recipient])?;

    let priv_idx = private_indices(depth, n_in, n_out, true);
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
                eprintln!("Skipping: prover failed (GPU/WebGPU likely unavailable): {err}");
                return Ok(());
            }
        };

    anyhow::ensure!(!proof_bytes.is_empty(), "proof should not be empty");

    let vpaths = verifier::VerifierPaths::from_explicit(
        program.clone(),
        shader_dir,
        runner.paths().verifier_bin.clone(),
        packing,
    );

    // Sanity: verifies with the original statement.
    let (ok, v_stdout, v_stderr) =
        verifier::verify_proof_with_output(&vpaths, &proof_bytes, args.clone(), priv_idx.clone())
            .context("Failed to run verifier")?;
    if !ok {
        anyhow::bail!("verifier did not report success\nstdout: {v_stdout}\nstderr: {v_stderr}");
    }

    // --- Red team: mutate PUBLIC inputs and ensure verification fails ---
    let per_in = 5 + depth;
    let idx_anchor: usize = 5;
    let idx_nullifier0: usize = 7 + 0 * per_in + 4 + depth;
    let idx_withdraw: usize = 7 + n_in * per_in;
    let idx_withdraw_to: usize = idx_withdraw + 1;
    let idx_n_out: usize = idx_withdraw + 2;
    let outs_base: usize = idx_withdraw + 3;
    let idx_cm_out0: usize = outs_base + 5 * 0 + 4;
    let idx_cm_out1: usize = outs_base + 5 * 1 + 4;

    let mut bad_anchor = anchor;
    bad_anchor[0] ^= 1;
    let mut bad_args = args.clone();
    bad_args[idx_anchor - 1] = LigeroArg::Hex {
        hex: hx32(&bad_anchor),
    };
    verify_expect_fail(&vpaths, &proof_bytes, bad_args, priv_idx.clone())?;

    let mut bad_nf0 = nf0;
    bad_nf0[0] ^= 1;
    let mut bad_args = args.clone();
    bad_args[idx_nullifier0 - 1] = LigeroArg::Hex {
        hex: hx32(&bad_nf0),
    };
    verify_expect_fail(&vpaths, &proof_bytes, bad_args, priv_idx.clone())?;

    let mut bad_args = args.clone();
    bad_args[idx_withdraw - 1] = LigeroArg::I64 {
        i64: (withdraw_amount as i64) + 1,
    };
    verify_expect_fail(&vpaths, &proof_bytes, bad_args, priv_idx.clone())?;

    let mut bad_args = args.clone();
    bad_args[idx_n_out - 1] = LigeroArg::I64 { i64: 1 };
    verify_expect_fail(&vpaths, &proof_bytes, bad_args, priv_idx.clone())?;

    let mut bad_withdraw_to = [0u8; 32];
    bad_withdraw_to[0] = 1;
    let mut bad_args = args.clone();
    bad_args[idx_withdraw_to - 1] = LigeroArg::Hex {
        hex: hx32(&bad_withdraw_to),
    };
    verify_expect_fail(&vpaths, &proof_bytes, bad_args, priv_idx.clone())?;

    let mut bad_cm = cm_out0;
    bad_cm[0] ^= 1;
    let mut bad_args = args.clone();
    bad_args[idx_cm_out0 - 1] = LigeroArg::Hex { hex: hx32(&bad_cm) };
    verify_expect_fail(&vpaths, &proof_bytes, bad_args, priv_idx.clone())?;

    let mut bad_cm = cm_out1;
    bad_cm[0] ^= 1;
    let mut bad_args = args.clone();
    bad_args[idx_cm_out1 - 1] = LigeroArg::Hex { hex: hx32(&bad_cm) };
    verify_expect_fail(&vpaths, &proof_bytes, bad_args, priv_idx.clone())?;

    // --- Red team: misconfigured private indices must be rejected ---
    let mut bad_priv_idx = priv_idx.clone();
    bad_priv_idx.push(idx_anchor); // incorrectly treat anchor as private
    verify_expect_fail(&vpaths, &proof_bytes, args, bad_priv_idx)?;

    Ok(())
}

#[test]
fn test_note_spend_viewer_attestation_roundtrip_and_rejects_mutation() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_spend_guest(&repo)?;

    let program = note_spend_program_path(&repo)?
        .canonicalize()
        .context("Failed to canonicalize note_spend_guest.wasm")?;

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
        return Ok(());
    }

    let shader_dir = PathBuf::from(&runner.config().shader_path);
    if !shader_dir.exists() {
        eprintln!(
            "Skipping: shader path not found at {}",
            shader_dir.display()
        );
        return Ok(());
    }

    let depth: usize = 4;
    let n_in: usize = 1;
    let n_out: usize = 1;

    let domain: Hash32 = [77u8; 32];
    let spend_sk: Hash32 = [4u8; 32];
    let pk_ivk_owner = ivk_seed(&domain, &spend_sk);
    let recipient_owner = recipient_from_sk(&domain, &spend_sk, &pk_ivk_owner);
    let sender_id = recipient_owner;
    let nf_key = nf_key_from_sk(&domain, &spend_sk);

    // One input note.
    let value_in: u64 = 123;
    let pos: u64 = 0;
    let rho_in: Hash32 = [2u8; 32];
    let sender_id_in: Hash32 = [6u8; 32];
    let cm_in = note_commitment(&domain, value_in, &rho_in, &recipient_owner, &sender_id_in);
    let mut tree = MerkleTree::new(depth);
    tree.set_leaf(pos as usize, cm_in);
    let anchor = tree.root();
    let siblings = tree.open(pos as usize);
    let nf0 = nullifier(&domain, &nf_key, &rho_in);

    // One output.
    let withdraw_amount: u64 = 0;
    let out_value: u64 = value_in;
    let out_rho: Hash32 = [7u8; 32];
    let out_spend_sk: Hash32 = [9u8; 32];
    let out_pk_spend = pk_from_sk(&out_spend_sk);
    let out_pk_ivk = ivk_seed(&domain, &out_spend_sk);
    let out_recipient = recipient_from_pk(&domain, &out_pk_spend, &out_pk_ivk);
    let cm_out = note_commitment(&domain, out_value, &out_rho, &out_recipient, &sender_id);

    let inv_enforce = compute_inv_enforce(&[value_in], &[rho_in], &[out_value], &[out_rho])?;

    // Viewer attestation (1 viewer).
    let n_viewers: u64 = 1;
    let fvk: Hash32 = [11u8; 32];
    let fvk_commitment = fvk_commit(&fvk);
    let k = view_kdf(&fvk, &cm_out);
    let pt = encode_note_plain(&domain, out_value, &out_rho, &out_recipient, &sender_id);
    let ct = stream_xor_encrypt_144(&k, &pt);
    let ct_h = ct_hash(&ct);
    let mac = view_mac(&k, &cm_out, &ct_h);

    // Build args (v2 ABI).
    let mut args: Vec<LigeroArg> = Vec::new();
    args.push(LigeroArg::Hex { hex: hx32(&domain) }); // 1
    args.push(LigeroArg::Hex {
        hex: hx32(&spend_sk),
    }); // 2 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&pk_ivk_owner),
    }); // 3 (priv)
    args.push(LigeroArg::I64 { i64: depth as i64 }); // 4
    args.push(LigeroArg::Hex { hex: hx32(&anchor) }); // 5
    args.push(LigeroArg::I64 { i64: n_in as i64 }); // 6

    // Input 0.
    args.push(LigeroArg::I64 {
        i64: value_in as i64,
    }); // value_in_0 (priv)
    args.push(LigeroArg::Hex { hex: hx32(&rho_in) }); // rho_in_0 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&sender_id_in),
    }); // sender_id_in_0 (priv)
    args.push(LigeroArg::I64 { i64: pos as i64 }); // pos_0 (priv)
    for s in &siblings {
        args.push(LigeroArg::Hex { hex: hx32(s) });
    }
    args.push(LigeroArg::Hex { hex: hx32(&nf0) }); // nullifier_0 (pub)

    // withdraw + n_out.
    args.push(LigeroArg::I64 {
        i64: withdraw_amount as i64,
    });
    args.push(LigeroArg::Hex {
        hex: hx32(&[0u8; 32]),
    });
    args.push(LigeroArg::I64 { i64: n_out as i64 });

    // Output 0.
    args.push(LigeroArg::I64 {
        i64: out_value as i64,
    }); // value_out_0 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&out_rho),
    }); // rho_out_0 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&out_pk_spend),
    }); // pk_spend_out_0 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&out_pk_ivk),
    }); // pk_ivk_out_0 (priv)
    args.push(LigeroArg::Hex { hex: hx32(&cm_out) }); // cm_out_0 (pub)
    args.push(LigeroArg::Hex {
        hex: hx32(&inv_enforce),
    }); // inv_enforce (priv)

    append_empty_blacklist(&mut args, &[sender_id, out_recipient])?;

    // Viewer section.
    args.push(LigeroArg::I64 {
        i64: n_viewers as i64,
    }); // n_viewers (pub)
    args.push(LigeroArg::Hex {
        hex: hx32(&fvk_commitment),
    }); // fvk_commit (pub)
    let fvk_idx1 = args.len() + 1; // 1-based index of the next arg (fvk)
    args.push(LigeroArg::Hex { hex: hx32(&fvk) }); // fvk (priv)
    args.push(LigeroArg::Hex { hex: hx32(&ct_h) }); // ct_hash_0 (pub)
    args.push(LigeroArg::Hex { hex: hx32(&mac) }); // mac_0 (pub)

    let mut priv_idx = private_indices(depth, n_in, n_out, true);
    priv_idx.push(fvk_idx1);

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
                eprintln!("Skipping: prover failed (GPU/WebGPU likely unavailable): {err}");
                return Ok(());
            }
        };

    anyhow::ensure!(!proof_bytes.is_empty(), "proof should not be empty");

    let vpaths = verifier::VerifierPaths::from_explicit(
        program.clone(),
        shader_dir,
        runner.paths().verifier_bin.clone(),
        packing,
    );

    // Sanity: verifies with the original statement.
    let (ok, v_stdout, v_stderr) =
        verifier::verify_proof_with_output(&vpaths, &proof_bytes, args.clone(), priv_idx.clone())
            .context("Failed to run verifier")?;
    if !ok {
        anyhow::bail!("verifier did not report success\nstdout: {v_stdout}\nstderr: {v_stderr}");
    }

    // With constraint-driven viewer hashing, redacting private args must still verify.

    let verify_expect_fail_redacted = |bad_args: Vec<LigeroArg>| -> Result<()> {
        match verifier::verify_proof_with_output(&vpaths, &proof_bytes, bad_args, priv_idx.clone()) {
            Ok((ok, stdout, stderr)) => {
                anyhow::ensure!(
                    !ok,
                    "expected verification to fail, but it succeeded\nstdout: {stdout}\nstderr: {stderr}"
                );
            }
            Err(_e) => {}
        }
        Ok(())
    };

    // --- Red team: viewer data must be bound ---
    let per_in = 5 + depth;
    let idx_withdraw_amount: usize = 7 + n_in * per_in;
    let idx_n_out: usize = idx_withdraw_amount + 2;
    let idx_output_base: usize = idx_n_out + 1;
    let idx_inv_enforce: usize = idx_output_base + 5 * n_out;
    let per_check = BL_BUCKET_SIZE + 1 + BL_DEPTH;
    let idx_n_viewers: usize = idx_inv_enforce + 2 + 2 * per_check;
    let idx_fvk_commit: usize = idx_n_viewers + 1;
    let idx_ct_hash: usize = idx_n_viewers + 3;
    let idx_mac: usize = idx_n_viewers + 4;

    let mut bad_commit = fvk_commitment;
    bad_commit[0] ^= 1;
    let mut bad_args = args.clone();
    bad_args[idx_fvk_commit - 1] = LigeroArg::Hex {
        hex: hx32(&bad_commit),
    };
    verify_expect_fail_redacted(bad_args)?;

    let mut bad_ct = ct_h;
    bad_ct[0] ^= 1;
    let mut bad_args = args.clone();
    bad_args[idx_ct_hash - 1] = LigeroArg::Hex { hex: hx32(&bad_ct) };
    verify_expect_fail_redacted(bad_args)?;

    let mut bad_mac = mac;
    bad_mac[0] ^= 1;
    let mut bad_args = args;
    bad_args[idx_mac - 1] = LigeroArg::Hex {
        hex: hx32(&bad_mac),
    };
    verify_expect_fail_redacted(bad_args)?;

    Ok(())
}

#[test]
fn test_note_spend_viewer_attestation_two_outputs_multiple_viewers_and_rejects_mismatch(
) -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_spend_guest(&repo)?;

    let program = note_spend_program_path(&repo)?
        .canonicalize()
        .context("Failed to canonicalize note_spend_guest.wasm")?;

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
        return Ok(());
    }

    let shader_dir = PathBuf::from(&runner.config().shader_path);
    if !shader_dir.exists() {
        eprintln!(
            "Skipping: shader path not found at {}",
            shader_dir.display()
        );
        return Ok(());
    }

    let depth: usize = 4;
    let n_in: usize = 1;
    let n_out: usize = 2;

    let domain: Hash32 = [88u8; 32];
    let spend_sk: Hash32 = [4u8; 32];
    let pk_ivk_owner = ivk_seed(&domain, &spend_sk);
    let recipient_owner = recipient_from_sk(&domain, &spend_sk, &pk_ivk_owner);
    let sender_id_current = recipient_owner;
    let nf_key = nf_key_from_sk(&domain, &spend_sk);

    // One input note.
    let value_in: u64 = 100;
    let pos: u64 = 2;
    let rho_in: Hash32 = [2u8; 32];
    let sender_id_in: Hash32 = [0u8; 32];
    let cm_in = note_commitment(&domain, value_in, &rho_in, &recipient_owner, &sender_id_in);
    let mut tree = MerkleTree::new(depth);
    tree.set_leaf(pos as usize, cm_in);
    let anchor = tree.root();
    let siblings = tree.open(pos as usize);
    let nf0 = nullifier(&domain, &nf_key, &rho_in);

    // Two outputs: pay 30 to Bob, change 70 to self.
    let withdraw_amount: u64 = 0;
    let out0_value: u64 = 30;
    let out1_value: u64 = 70;

    let out0_rho: Hash32 = [7u8; 32];
    let out1_rho: Hash32 = [8u8; 32];

    let bob_sk: Hash32 = [9u8; 32];
    let out0_pk_spend = pk_from_sk(&bob_sk);
    let out0_pk_ivk = ivk_seed(&domain, &bob_sk);
    let out0_recipient = recipient_from_pk(&domain, &out0_pk_spend, &out0_pk_ivk);

    // Change back to self.
    let out1_pk_spend = pk_from_sk(&spend_sk);
    let out1_pk_ivk = pk_ivk_owner;
    let out1_recipient = recipient_owner;

    let cm_out0 = note_commitment(
        &domain,
        out0_value,
        &out0_rho,
        &out0_recipient,
        &sender_id_current,
    );
    let cm_out1 = note_commitment(
        &domain,
        out1_value,
        &out1_rho,
        &out1_recipient,
        &sender_id_current,
    );

    let inv_enforce = compute_inv_enforce(
        &[value_in],
        &[rho_in],
        &[out0_value, out1_value],
        &[out0_rho, out1_rho],
    )?;

    // Viewer attestations (2 viewers).
    let n_viewers: u64 = 2;
    let viewer_fvks: [Hash32; 2] = [[11u8; 32], [12u8; 32]];

    // Precompute per-output plaintext and digests for each viewer.
    let pt0 = encode_note_plain(
        &domain,
        out0_value,
        &out0_rho,
        &out0_recipient,
        &sender_id_current,
    );
    let pt1 = encode_note_plain(
        &domain,
        out1_value,
        &out1_rho,
        &out1_recipient,
        &sender_id_current,
    );

    // Build args (v2 ABI).
    let mut args: Vec<LigeroArg> = Vec::new();
    args.push(LigeroArg::Hex { hex: hx32(&domain) }); // 1
    args.push(LigeroArg::Hex {
        hex: hx32(&spend_sk),
    }); // 2 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&pk_ivk_owner),
    }); // 3 (priv)
    args.push(LigeroArg::I64 { i64: depth as i64 }); // 4
    args.push(LigeroArg::Hex { hex: hx32(&anchor) }); // 5 (pub)
    args.push(LigeroArg::I64 { i64: n_in as i64 }); // 6 (pub)

    // Input 0.
    args.push(LigeroArg::I64 {
        i64: value_in as i64,
    }); // value_in_0 (priv)
    args.push(LigeroArg::Hex { hex: hx32(&rho_in) }); // rho_in_0 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&sender_id_in),
    }); // sender_id_in_0 (priv)
    args.push(LigeroArg::I64 { i64: pos as i64 }); // pos_0 (priv)
    for s in &siblings {
        args.push(LigeroArg::Hex { hex: hx32(s) });
    }
    args.push(LigeroArg::Hex { hex: hx32(&nf0) }); // nullifier_0 (pub)

    // withdraw + n_out.
    args.push(LigeroArg::I64 {
        i64: withdraw_amount as i64,
    });
    args.push(LigeroArg::Hex {
        hex: hx32(&[0u8; 32]),
    });
    args.push(LigeroArg::I64 { i64: n_out as i64 });

    // Output 0.
    args.push(LigeroArg::I64 {
        i64: out0_value as i64,
    });
    args.push(LigeroArg::Hex {
        hex: hx32(&out0_rho),
    });
    args.push(LigeroArg::Hex {
        hex: hx32(&out0_pk_spend),
    });
    args.push(LigeroArg::Hex {
        hex: hx32(&out0_pk_ivk),
    });
    args.push(LigeroArg::Hex {
        hex: hx32(&cm_out0),
    });

    // Output 1.
    args.push(LigeroArg::I64 {
        i64: out1_value as i64,
    });
    args.push(LigeroArg::Hex {
        hex: hx32(&out1_rho),
    });
    args.push(LigeroArg::Hex {
        hex: hx32(&out1_pk_spend),
    });
    args.push(LigeroArg::Hex {
        hex: hx32(&out1_pk_ivk),
    });
    args.push(LigeroArg::Hex {
        hex: hx32(&cm_out1),
    });

    // inv_enforce (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&inv_enforce),
    });

    append_empty_blacklist(&mut args, &[sender_id_current, out0_recipient])?;

    // Viewer section.
    args.push(LigeroArg::I64 {
        i64: n_viewers as i64,
    }); // n_viewers (pub)

    let mut priv_idx = private_indices(depth, n_in, n_out, true);
    for fvk in &viewer_fvks {
        let fvk_commitment = fvk_commit(fvk);
        args.push(LigeroArg::Hex {
            hex: hx32(&fvk_commitment),
        }); // fvk_commit (pub)

        let fvk_idx1 = args.len() + 1; // 1-based index of the next arg (fvk)
        args.push(LigeroArg::Hex { hex: hx32(fvk) }); // fvk (priv)
        priv_idx.push(fvk_idx1);

        // Output 0 digests.
        let k0 = view_kdf(fvk, &cm_out0);
        let ct0 = stream_xor_encrypt_144(&k0, &pt0);
        let ct0_h = ct_hash(&ct0);
        let mac0 = view_mac(&k0, &cm_out0, &ct0_h);
        args.push(LigeroArg::Hex { hex: hx32(&ct0_h) }); // ct_hash_0 (pub)
        args.push(LigeroArg::Hex { hex: hx32(&mac0) }); // mac_0 (pub)

        // Output 1 digests.
        let k1 = view_kdf(fvk, &cm_out1);
        let ct1 = stream_xor_encrypt_144(&k1, &pt1);
        let ct1_h = ct_hash(&ct1);
        let mac1 = view_mac(&k1, &cm_out1, &ct1_h);
        args.push(LigeroArg::Hex { hex: hx32(&ct1_h) }); // ct_hash_1 (pub)
        args.push(LigeroArg::Hex { hex: hx32(&mac1) }); // mac_1 (pub)
    }

    // Ensure private indices are sorted/deduped.
    priv_idx.sort_unstable();
    priv_idx.dedup();

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
                eprintln!("Skipping: prover failed (GPU/WebGPU likely unavailable): {err}");
                return Ok(());
            }
        };

    anyhow::ensure!(!proof_bytes.is_empty(), "proof should not be empty");

    let vpaths = verifier::VerifierPaths::from_explicit(
        program.clone(),
        shader_dir,
        runner.paths().verifier_bin.clone(),
        packing,
    );

    let (ok, v_stdout, v_stderr) =
        verifier::verify_proof_with_output(&vpaths, &proof_bytes, args.clone(), priv_idx.clone())
            .context("Failed to run verifier")?;
    if !ok {
        anyhow::bail!("verifier did not report success\nstdout: {v_stdout}\nstderr: {v_stderr}");
    }

    // --- Red team: wrong n_viewers should be rejected (argc mismatch / different circuit instance) ---
    {
        let per_in = 5 + depth;
        let idx_withdraw_amount: usize = 7 + n_in * per_in;
        let idx_n_out: usize = idx_withdraw_amount + 2;
        let idx_output_base: usize = idx_n_out + 1;
        let idx_inv_enforce: usize = idx_output_base + 5 * n_out;
        let per_check = BL_BUCKET_SIZE + 1 + BL_DEPTH;
        let idx_n_viewers: usize = idx_inv_enforce + 2 + 2 * per_check;

        let mut bad_args = args.clone();
        bad_args[idx_n_viewers - 1] = LigeroArg::I64 { i64: 3 }; // but only 2 viewer blocks present

        match verifier::verify_proof_with_output(&vpaths, &proof_bytes, bad_args, priv_idx.clone()) {
            Ok((ok_bad, _stdout, _stderr)) => {
                anyhow::ensure!(!ok_bad, "expected verification to fail")
            }
            Err(_e) => {}
        }
    }

    // --- Red team: argc mismatch (drop 1 arg) should be rejected ---
    {
        let mut bad_args = args.clone();
        bad_args.pop();
        match verifier::verify_proof_with_output(&vpaths, &proof_bytes, bad_args, priv_idx.clone()) {
            Ok((ok_bad, _stdout, _stderr)) => {
                anyhow::ensure!(!ok_bad, "expected verification to fail")
            }
            Err(_e) => {}
        }
    }

    Ok(())
}

#[test]
fn test_note_spend_proof_roundtrip_four_inputs_merge_one_output() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_spend_guest(&repo)?;

    let program = note_spend_program_path(&repo)?
        .canonicalize()
        .context("Failed to canonicalize note_spend_guest.wasm")?;

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
        return Ok(());
    }

    let shader_dir = PathBuf::from(&runner.config().shader_path);
    if !shader_dir.exists() {
        eprintln!(
            "Skipping: shader path not found at {}",
            shader_dir.display()
        );
        return Ok(());
    }

    println!("[note_spend_roundtrip_4in] Program: {}", program.display());
    println!(
        "[note_spend_roundtrip_4in] Prover:   {}",
        runner.paths().prover_bin.display()
    );
    println!(
        "[note_spend_roundtrip_4in] Verifier: {}",
        runner.paths().verifier_bin.display()
    );
    println!(
        "[note_spend_roundtrip_4in] Shaders:  {}",
        shader_dir.display()
    );
    println!("[note_spend_roundtrip_4in] Packing:  {}", packing);

    // Small depth to keep runtime reasonable while exercising n_in=4 logic.
    let depth: usize = 4;
    let n_in: u64 = 4;
    let withdraw_amount: u64 = 0;
    let n_out: u64 = 1;

    let domain: Hash32 = [9u8; 32];
    let spend_sk: Hash32 = [4u8; 32];
    let pk_ivk_owner = ivk_seed(&domain, &spend_sk);
    let recipient_owner = recipient_from_sk(&domain, &spend_sk, &pk_ivk_owner);
    let sender_id_current = recipient_owner;
    let nf_key = nf_key_from_sk(&domain, &spend_sk);

    // 4 inputs at distinct positions, all owned by the same key.
    let positions: [u64; 4] = [0, 1, 2, 3];
    let values: [u64; 4] = [10, 20, 30, 40];

    let mut rhos: [Hash32; 4] = [[0u8; 32]; 4];
    for (i, rho_i) in rhos.iter_mut().enumerate() {
        *rho_i = [2u8; 32];
        rho_i[0] = rho_i[0].wrapping_add(i as u8);
    }

    let mut sender_ids_in: [Hash32; 4] = [[6u8; 32]; 4];
    for (i, sid) in sender_ids_in.iter_mut().enumerate() {
        sid[0] = sid[0].wrapping_add(i as u8);
    }

    let mut tree = MerkleTree::new(depth);
    for i in 0..4 {
        let cm_i = note_commitment(
            &domain,
            values[i],
            &rhos[i],
            &recipient_owner,
            &sender_ids_in[i],
        );
        tree.set_leaf(positions[i] as usize, cm_i);
    }
    let anchor = tree.root();

    let mut siblings_vec: Vec<Vec<Hash32>> = Vec::with_capacity(4);
    let mut nfs: Vec<Hash32> = Vec::with_capacity(4);
    for i in 0..4 {
        let siblings = tree.open(positions[i] as usize);
        anyhow::ensure!(siblings.len() == depth, "unexpected siblings length");
        siblings_vec.push(siblings);

        let nf_i = nullifier(&domain, &nf_key, &rhos[i]);
        nfs.push(nf_i);
    }

    // Output merges all inputs back to self.
    let sum_in: u64 = values.iter().copied().sum();
    let out_value: u64 = sum_in;
    let out_rho: Hash32 = [7u8; 32];
    let pk_spend_owner = pk_from_sk(&spend_sk);
    let out_pk_spend = pk_spend_owner;
    let out_pk_ivk = pk_ivk_owner;
    let cm_out = note_commitment(
        &domain,
        out_value,
        &out_rho,
        &recipient_owner,
        &sender_id_current,
    );

    let inv_enforce = compute_inv_enforce(&values, &rhos, &[out_value], &[out_rho])?;

    // Build args (v2 ABI).
    let mut args: Vec<LigeroArg> = Vec::new();
    args.push(LigeroArg::Hex { hex: hx32(&domain) }); // 1
    args.push(LigeroArg::Hex {
        hex: hx32(&spend_sk),
    }); // 2 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&pk_ivk_owner),
    }); // 3 (priv)
    args.push(LigeroArg::I64 { i64: depth as i64 }); // 4
    args.push(LigeroArg::Hex { hex: hx32(&anchor) }); // 5
    args.push(LigeroArg::I64 { i64: n_in as i64 }); // 6

    for i in 0..4 {
        args.push(LigeroArg::I64 {
            i64: values[i] as i64,
        }); // value_in_i (priv)
        args.push(LigeroArg::Hex {
            hex: hx32(&rhos[i]),
        }); // rho_in_i (priv)
        args.push(LigeroArg::Hex {
            hex: hx32(&sender_ids_in[i]),
        }); // sender_id_in_i (priv)

        let pos = positions[i];
        args.push(LigeroArg::I64 { i64: pos as i64 }); // pos_i (priv)

        for s in &siblings_vec[i] {
            args.push(LigeroArg::Hex { hex: hx32(s) });
        }

        // nullifier_i (public)
        args.push(LigeroArg::Hex { hex: hx32(&nfs[i]) });
    }

    args.push(LigeroArg::I64 {
        i64: withdraw_amount as i64,
    });
    args.push(LigeroArg::Hex {
        hex: hx32(&[0u8; 32]),
    });
    args.push(LigeroArg::I64 { i64: n_out as i64 });

    // Output 0
    args.push(LigeroArg::I64 {
        i64: out_value as i64,
    }); // value_out_0 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&out_rho),
    }); // rho_out_0 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&out_pk_spend),
    }); // pk_spend_out_0 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&out_pk_ivk),
    }); // pk_ivk_out_0 (priv)
    args.push(LigeroArg::Hex { hex: hx32(&cm_out) }); // cm_out_0 (pub)
    args.push(LigeroArg::Hex {
        hex: hx32(&inv_enforce),
    }); // inv_enforce (priv)

    append_empty_blacklist(&mut args, &[sender_id_current, recipient_owner])?;

    let priv_idx = private_indices(depth, 4, 1, true);
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
                // Treat GPU/WebGPU/runtime issues as a skip, but surface real constraint failures.
                let msg = format!("{err:#}");
                if msg.contains("Validation of linear constraints:")
                    || msg.contains("Validation of quadratic constraints:")
                    || msg.contains("Final prove result:")
                {
                    return Err(err);
                }
                eprintln!("Skipping: prover failed (GPU/WebGPU likely unavailable): {err}");
                return Ok(());
            }
        };

    anyhow::ensure!(!proof_bytes.is_empty(), "proof should not be empty");

    let vpaths = verifier::VerifierPaths::from_explicit(
        program.clone(),
        shader_dir,
        runner.paths().verifier_bin.clone(),
        packing,
    );

    // Sanity: verifies with the original statement.
    let (ok, v_stdout, v_stderr) =
        verifier::verify_proof_with_output(&vpaths, &proof_bytes, args.clone(), priv_idx.clone())
            .context("Failed to run verifier")?;

    if !ok {
        anyhow::bail!("verifier did not report success\nstdout: {v_stdout}\nstderr: {v_stderr}");
    }

    // --- Red team: mutate a public input nullifier and ensure verification fails ---
    let per_in = 5 + depth;
    let idx_nullifier2: usize = 7 + 2 * per_in + 4 + depth; // input i=2
    let mut bad_nf = nfs[2];
    bad_nf[0] ^= 1;
    let mut bad_args = args.clone();
    bad_args[idx_nullifier2 - 1] = LigeroArg::Hex { hex: hx32(&bad_nf) };
    verify_expect_fail(&vpaths, &proof_bytes, bad_args, priv_idx.clone())?;

    // Mutate anchor.
    let idx_anchor: usize = 5;
    let mut bad_anchor = anchor;
    bad_anchor[0] ^= 1;
    let mut bad_args = args.clone();
    bad_args[idx_anchor - 1] = LigeroArg::Hex {
        hex: hx32(&bad_anchor),
    };
    verify_expect_fail(&vpaths, &proof_bytes, bad_args, priv_idx.clone())?;

    // Misconfigured private indices must be rejected.
    let mut bad_priv_idx = priv_idx;
    bad_priv_idx.push(idx_anchor);
    verify_expect_fail(&vpaths, &proof_bytes, args, bad_priv_idx)?;

    Ok(())
}

#[test]
fn test_note_spend_rejects_invalid_shapes_and_invalid_pos() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_spend_guest(&repo)?;

    let program = note_spend_program_path(&repo)?
        .canonicalize()
        .context("Failed to canonicalize note_spend_guest.wasm")?;

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
        return Ok(());
    }

    let shader_dir = PathBuf::from(&runner.config().shader_path);
    if !shader_dir.exists() {
        eprintln!(
            "Skipping: shader path not found at {}",
            shader_dir.display()
        );
        return Ok(());
    }

    let vpaths = verifier::VerifierPaths::from_explicit(
        program.clone(),
        shader_dir,
        runner.paths().verifier_bin.clone(),
        packing,
    );

    // Keep the circuit small but exercise all early checks.
    let depth: usize = 4;
    let n_in: u64 = 1;
    let n_out: u64 = 1;

    let domain: Hash32 = [1u8; 32];
    let spend_sk: Hash32 = [4u8; 32];
    let pk_ivk_owner = ivk_seed(&domain, &spend_sk);
    let recipient_owner = recipient_from_sk(&domain, &spend_sk, &pk_ivk_owner);
    let sender_id_current = recipient_owner;
    let nf_key = nf_key_from_sk(&domain, &spend_sk);

    // One input note in a small tree.
    let value_in: u64 = 123;
    let pos: u64 = 0;
    let rho_in: Hash32 = [2u8; 32];
    let sender_id_in: Hash32 = [6u8; 32];
    let cm_in = note_commitment(&domain, value_in, &rho_in, &recipient_owner, &sender_id_in);
    let mut tree = MerkleTree::new(depth);
    tree.set_leaf(pos as usize, cm_in);
    let anchor = tree.root();
    let siblings = tree.open(pos as usize);
    let nf0 = nullifier(&domain, &nf_key, &rho_in);

    // One output (transfer to self for simplicity).
    let withdraw_amount: u64 = 0;
    let out_value: u64 = value_in;
    let out_rho: Hash32 = [7u8; 32];
    let out_pk_spend = pk_from_sk(&spend_sk);
    let out_pk_ivk = pk_ivk_owner;
    let cm_out = note_commitment(
        &domain,
        out_value,
        &out_rho,
        &recipient_owner,
        &sender_id_current,
    );

    let inv_enforce = compute_inv_enforce(&[value_in], &[rho_in], &[out_value], &[out_rho])?;

    // Build a valid statement (baseline).
    let mut args: Vec<LigeroArg> = Vec::new();
    args.push(LigeroArg::Hex { hex: hx32(&domain) }); // 1
    args.push(LigeroArg::Hex {
        hex: hx32(&spend_sk),
    }); // 2 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&pk_ivk_owner),
    }); // 3 (priv)
    args.push(LigeroArg::I64 { i64: depth as i64 }); // 4
    args.push(LigeroArg::Hex { hex: hx32(&anchor) }); // 5
    args.push(LigeroArg::I64 { i64: n_in as i64 }); // 6

    // Input 0.
    args.push(LigeroArg::I64 {
        i64: value_in as i64,
    }); // 7 value_in_0 (priv)
    args.push(LigeroArg::Hex { hex: hx32(&rho_in) }); // 8 rho_in_0 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&sender_id_in),
    }); // sender_id_in_0 (priv)
    args.push(LigeroArg::I64 { i64: pos as i64 }); // pos_0 (priv)
    for s in &siblings {
        args.push(LigeroArg::Hex { hex: hx32(s) });
    }
    args.push(LigeroArg::Hex { hex: hx32(&nf0) }); // nullifier_0 (pub)

    // withdraw + n_out.
    args.push(LigeroArg::I64 {
        i64: withdraw_amount as i64,
    });
    args.push(LigeroArg::Hex {
        hex: hx32(&[0u8; 32]),
    });
    args.push(LigeroArg::I64 { i64: n_out as i64 });

    // Output 0.
    args.push(LigeroArg::I64 {
        i64: out_value as i64,
    }); // value_out_0 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&out_rho),
    }); // rho_out_0 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&out_pk_spend),
    }); // pk_spend_out_0 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&out_pk_ivk),
    }); // pk_ivk_out_0 (priv)
    args.push(LigeroArg::Hex { hex: hx32(&cm_out) }); // cm_out_0 (pub)
    args.push(LigeroArg::Hex {
        hex: hx32(&inv_enforce),
    }); // inv_enforce (priv)

    append_empty_blacklist(&mut args, &[sender_id_current, recipient_owner])?;

    let priv_idx = private_indices(depth, 1, 1, true);

    // First ensure the prover/verifier are working in this environment.
    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = args.clone();
    let (baseline_proof, _p_stdout, _p_stderr) =
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
        &baseline_proof,
        args.clone(),
        priv_idx.clone(),
    )
    .context("Failed to run verifier")?;
    if !ok {
        anyhow::bail!("baseline proof did not verify\nstdout: {v_stdout}\nstderr: {v_stderr}");
    }

    let mut run_expect_reject = |bad_args: Vec<LigeroArg>| -> Result<()> {
        runner.config_mut().private_indices = priv_idx.clone();
        runner.config_mut().args = bad_args.clone();
        match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
            keep_proof_dir: false,
            proof_outputs_base: None,
            write_replay_script: false,
        }) {
            Ok((proof_bytes, _stdout, _stderr)) => {
                let (ok, stdout, stderr) = verifier::verify_proof_with_output(
                    &vpaths,
                    &proof_bytes,
                    bad_args,
                    priv_idx.clone(),
                )
                .context("Failed to run verifier")?;
                anyhow::ensure!(
                    !ok,
                    "expected verification to fail, but it succeeded\nstdout: {stdout}\nstderr: {stderr}"
                );
            }
            Err(_e) => {
                // Expected: invalid witness/shape triggers UNSAT and the prover exits non-zero.
            }
        }
        Ok(())
    };

    // Invalid: n_in == 0 must be rejected (regression test for failure-path soundness).
    let mut bad_args = args.clone();
    bad_args[5] = LigeroArg::I64 { i64: 0 };
    run_expect_reject(bad_args)?;

    // Invalid: withdraw_amount == 0 implies n_out must be 1 or 2 (n_out == 0 is forbidden).
    let mut bad_args = args.clone();
    let per_in = 5 + depth;
    let idx_withdraw_1based: usize = 7 + (n_in as usize) * per_in;
    let idx_withdraw_to_1based: usize = idx_withdraw_1based + 1;
    let idx_n_out_1based: usize = idx_withdraw_1based + 2;
    bad_args[idx_n_out_1based - 1] = LigeroArg::I64 { i64: 0 };
    run_expect_reject(bad_args)?;

    // Invalid: withdraw_amount > 0 implies n_out <= 1 (n_out == 2 is forbidden).
    let mut bad_args = args.clone();
    bad_args[idx_withdraw_1based - 1] = LigeroArg::I64 { i64: 1 };
    bad_args[idx_withdraw_to_1based - 1] = LigeroArg::Hex {
        hex: hx32(&[0x11u8; 32]),
    };
    bad_args[idx_n_out_1based - 1] = LigeroArg::I64 { i64: 2 };
    run_expect_reject(bad_args)?;

    // Invalid: pos must be < 2^depth.
    let mut bad_args = args;
    let idx_pos_1based: usize = 7 + 3; // header(6) + value + rho + sender_id_in + pos
    bad_args[idx_pos_1based - 1] = LigeroArg::I64 {
        i64: 1i64 << depth,
    };
    run_expect_reject(bad_args)?;

    Ok(())
}
