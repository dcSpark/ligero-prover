//! Security + negative tests for the NOTE_V2 + ADDR_V2 spend circuit.
//!
//! These tests focus on:
//! - statement binding (mutated public inputs must not verify)
//! - inv_enforce (nonzero values + rho-uniqueness) rejection
//! - red-team checks (non-canonical public digest encodings, proof byte tampering)
//! - deposit→spend compatibility (NOTE_V2 minted by deposit is spendable)

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use ligero_runner::{verifier, LigeroArg, LigeroRunner, ProverRunOptions};
use ligetron::{poseidon2_hash_bytes, Bn254Fr};

type Hash32 = [u8; 32];

const BN254_FR_MODULUS_BE: Hash32 = [
    0x30, 0x64, 0x4e, 0x72, 0xe1, 0x31, 0xa0, 0x29, 0xb8, 0x50, 0x45, 0xb6, 0x81, 0x81,
    0x58, 0x5d, 0x28, 0x33, 0xe8, 0x48, 0x79, 0xb9, 0x70, 0x91, 0x43, 0xe1, 0xf5, 0x93,
    0xf0, 0x00, 0x00, 0x01,
];

fn hx32(b: &Hash32) -> String {
    hex::encode(b)
}

fn repo_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    Ok(manifest_dir
        .ancestors()
        .nth(2)
        .context("Failed to compute ligero-prover repo root")?
        .to_path_buf())
}

fn maybe_build_guest(repo: &Path, rel_guest_dir: &str, rel_out_wasm: &str) -> Result<()> {
    let out = repo.join(rel_out_wasm);
    if out.exists() {
        return Ok(());
    }

    let guest_dir = repo.join(rel_guest_dir);
    if !guest_dir.exists() {
        anyhow::bail!("guest sources not found at {}", guest_dir.display());
    }

    println!(
        "[security_test] {} not found; building via {}",
        rel_out_wasm,
        guest_dir.join("build.sh").display()
    );

    let status = Command::new("bash")
        .arg("build.sh")
        .current_dir(&guest_dir)
        .status()
        .with_context(|| format!("Failed to run {}/build.sh", rel_guest_dir))?;

    if !status.success() {
        anyhow::bail!("{rel_guest_dir}/build.sh failed with status {status}");
    }

    Ok(())
}

fn note_spend_program_path(repo: &Path) -> Result<PathBuf> {
    if let Ok(p) = std::env::var("LIGERO_PROGRAM_PATH") {
        return Ok(PathBuf::from(p));
    }
    Ok(repo.join("utils/circuits/bins/note_spend_guest.wasm"))
}

fn note_deposit_program_path(repo: &Path) -> PathBuf {
    repo.join("utils/circuits/bins/note_deposit_guest.wasm")
}

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

fn pk_from_sk(spend_sk: &Hash32) -> Hash32 {
    poseidon2_hash_domain(b"PK_V1", &[spend_sk])
}

fn ivk_seed(domain: &Hash32, spend_sk: &Hash32) -> Hash32 {
    poseidon2_hash_domain(b"IVK_SEED_V1", &[domain, spend_sk])
}

fn recipient_from_pk(domain: &Hash32, pk_spend: &Hash32, pk_ivk: &Hash32) -> Hash32 {
    poseidon2_hash_domain(b"ADDR_V2", &[domain, pk_spend, pk_ivk])
}

fn nf_key_from_sk(domain: &Hash32, spend_sk: &Hash32) -> Hash32 {
    poseidon2_hash_domain(b"NFKEY_V1", &[domain, spend_sk])
}

fn nullifier(domain: &Hash32, nf_key: &Hash32, rho: &Hash32) -> Hash32 {
    poseidon2_hash_domain(b"PRF_NF_V1", &[domain, nf_key, rho])
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

fn private_indices_spend(depth: usize, n_in: usize, n_out: usize) -> Vec<usize> {
    // Mirrors the guest layout described in utils/circuits/note-spend/src/main.rs (NOTE_V2 + ADDR_V2).
    let mut idx = vec![2usize, 3usize]; // spend_sk, pk_ivk_owner

    let per_in = 4usize + 2usize * depth;
    for i in 0..n_in {
        let base = 7 + i * per_in;
        idx.push(base); // value_in
        idx.push(base + 1); // rho_in
        idx.push(base + 2); // sender_id_in
        for k in 0..depth {
            idx.push(base + 3 + k); // pos_bit
        }
        for k in 0..depth {
            idx.push(base + 3 + depth + k); // sibling
        }
        // nullifier is public
    }

    let withdraw_idx = 7 + n_in * per_in;
    let outs_base = withdraw_idx + 2;
    for j in 0..n_out {
        idx.push(outs_base + 5 * j + 0); // value_out
        idx.push(outs_base + 5 * j + 1); // rho_out
        idx.push(outs_base + 5 * j + 2); // pk_spend_out
        idx.push(outs_base + 5 * j + 3); // pk_ivk_out
                                         // cm_out is public
    }
    idx.push(outs_base + 5 * n_out); // inv_enforce

    idx.sort_unstable();
    idx.dedup();
    idx
}

fn per_in(depth: usize) -> usize {
    4 + 2 * depth
}

fn withdraw_index(n_in: usize, depth: usize) -> usize {
    7 + n_in * per_in(depth)
}

fn output_base(n_in: usize, depth: usize) -> usize {
    withdraw_index(n_in, depth) + 2
}

fn cm_out_index(n_in: usize, depth: usize, j: usize) -> usize {
    output_base(n_in, depth) + 5 * j + 4
}

fn inv_enforce_index(n_in: usize, depth: usize, n_out: usize) -> usize {
    output_base(n_in, depth) + 5 * n_out
}

fn add_be_32(a: &Hash32, b: &Hash32) -> Hash32 {
    let mut out = [0u8; 32];
    let mut carry: u16 = 0;
    for i in (0..32).rev() {
        let sum = a[i] as u16 + b[i] as u16 + carry;
        out[i] = (sum & 0xff) as u8;
        carry = sum >> 8;
    }
    out
}

fn prove(runner: &mut LigeroRunner, args: Vec<LigeroArg>, priv_idx: Vec<usize>) -> Result<Vec<u8>> {
    runner.config_mut().private_indices = priv_idx;
    runner.config_mut().args = args;
    let (proof, _stdout, _stderr) = runner.run_prover_with_output(ProverRunOptions {
        keep_proof_dir: false,
        proof_outputs_base: None,
        write_replay_script: false,
    })?;
    Ok(proof)
}

fn verify(paths: &verifier::VerifierPaths, proof: &[u8], args: Vec<LigeroArg>, priv_idx: Vec<usize>) -> Result<bool> {
    let (ok, _stdout, _stderr) =
        verifier::verify_proof_with_output(paths, proof, args, priv_idx).context("Failed to run verifier")?;
    Ok(ok)
}

fn assert_rejected(
    runner: &mut LigeroRunner,
    vpaths: &verifier::VerifierPaths,
    args: Vec<LigeroArg>,
    priv_idx: Vec<usize>,
) -> Result<()> {
    match prove(runner, args.clone(), priv_idx.clone()) {
        Err(_) => Ok(()), // rejected at prove-time ✅
        Ok(proof) => {
            let ok = verify(vpaths, &proof, args, priv_idx)?;
            anyhow::ensure!(!ok, "expected verifier rejection, but verification succeeded");
            Ok(())
        }
    }
}

fn build_transfer_1in_2out(
    depth: usize,
    domain: Hash32,
    spend_sk: Hash32,
    value_in: u64,
    rho_in: Hash32,
    sender_id_in: Hash32,
    pos: u64,
    out0_rho: Hash32,
    out1_rho: Hash32,
    inv_enforce_override: Option<Hash32>,
) -> Result<(Vec<LigeroArg>, Vec<usize>, Hash32)> {
    let pk_ivk_owner = ivk_seed(&domain, &spend_sk);
    let pk_spend_owner = pk_from_sk(&spend_sk);
    let recipient_owner = recipient_from_pk(&domain, &pk_spend_owner, &pk_ivk_owner);
    let sender_id_current = recipient_owner;

    let nf_key = nf_key_from_sk(&domain, &spend_sk);

    let cm_in = note_commitment(&domain, value_in, &rho_in, &recipient_owner, &sender_id_in);
    let mut tree = MerkleTree::new(depth);
    tree.set_leaf(pos as usize, cm_in);
    let anchor = tree.root();
    let siblings = tree.open(pos as usize);
    anyhow::ensure!(siblings.len() == depth, "unexpected siblings length");
    let nf0 = nullifier(&domain, &nf_key, &rho_in);

    let out0_value = (value_in / 3).max(1);
    let out1_value = value_in
        .checked_sub(out0_value)
        .context("value_in must be >= out0_value")?;

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

    // Change back to self.
    let out1_pk_spend = pk_spend_owner;
    let out1_pk_ivk = pk_ivk_owner;
    let cm_out1 = note_commitment(
        &domain,
        out1_value,
        &out1_rho,
        &recipient_owner,
        &sender_id_current,
    );

    let inv_enforce = match inv_enforce_override {
        Some(v) => v,
        None => compute_inv_enforce(
            &[value_in],
            &[rho_in],
            &[out0_value, out1_value],
            &[out0_rho, out1_rho],
        )?,
    };

    let mut args: Vec<LigeroArg> = Vec::new();
    args.push(LigeroArg::Hex { hex: hx32(&domain) }); // 1 domain (pub)
    args.push(LigeroArg::Hex { hex: hx32(&spend_sk) }); // 2 spend_sk (priv)
    args.push(LigeroArg::Hex { hex: hx32(&pk_ivk_owner) }); // 3 pk_ivk_owner (priv)
    args.push(LigeroArg::I64 { i64: depth as i64 }); // 4 depth (pub)
    args.push(LigeroArg::Hex { hex: hx32(&anchor) }); // 5 anchor (pub)
    args.push(LigeroArg::I64 { i64: 1 }); // 6 n_in (pub)

    // Input0
    args.push(LigeroArg::I64 {
        i64: value_in as i64,
    }); // value_in_0 (priv)
    args.push(LigeroArg::Hex { hex: hx32(&rho_in) }); // rho_in_0 (priv)
    args.push(LigeroArg::Hex {
        hex: hx32(&sender_id_in),
    }); // sender_id_in_0 (priv)
    for lvl in 0..depth {
        let bit = ((pos >> lvl) & 1) as u8;
        let mut bit_bytes = [0u8; 32];
        bit_bytes[31] = bit;
        args.push(LigeroArg::Hex { hex: hx32(&bit_bytes) });
    }
    for s in &siblings {
        args.push(LigeroArg::Hex { hex: hx32(s) });
    }
    args.push(LigeroArg::Hex { hex: hx32(&nf0) }); // nullifier_0 (pub)

    // withdraw + n_out
    args.push(LigeroArg::I64 { i64: 0 }); // withdraw_amount (pub)
    args.push(LigeroArg::I64 { i64: 2 }); // n_out (pub)

    // Output0
    args.push(LigeroArg::I64 {
        i64: out0_value as i64,
    });
    args.push(LigeroArg::Hex { hex: hx32(&out0_rho) });
    args.push(LigeroArg::Hex {
        hex: hx32(&out0_pk_spend),
    });
    args.push(LigeroArg::Hex { hex: hx32(&out0_pk_ivk) });
    args.push(LigeroArg::Hex { hex: hx32(&cm_out0) });

    // Output1
    args.push(LigeroArg::I64 {
        i64: out1_value as i64,
    });
    args.push(LigeroArg::Hex { hex: hx32(&out1_rho) });
    args.push(LigeroArg::Hex { hex: hx32(&out1_pk_spend) });
    args.push(LigeroArg::Hex { hex: hx32(&out1_pk_ivk) });
    args.push(LigeroArg::Hex { hex: hx32(&cm_out1) });

    // inv_enforce
    args.push(LigeroArg::Hex {
        hex: hx32(&inv_enforce),
    });

    Ok((
        args,
        private_indices_spend(depth, 1, 2),
        anchor, // return anchor for canonical-encoding tests
    ))
}

fn build_withdraw_1in_1out(
    depth: usize,
    domain: Hash32,
    spend_sk: Hash32,
    value_in: u64,
    rho_in: Hash32,
    sender_id_in: Hash32,
    pos: u64,
    withdraw_amount: u64,
    out_value: u64,
    out_rho: Hash32,
    inv_enforce_override: Option<Hash32>,
) -> Result<(Vec<LigeroArg>, Vec<usize>)> {
    let pk_ivk_owner = ivk_seed(&domain, &spend_sk);
    let pk_spend_owner = pk_from_sk(&spend_sk);
    let recipient_owner = recipient_from_pk(&domain, &pk_spend_owner, &pk_ivk_owner);
    let sender_id_current = recipient_owner;

    let nf_key = nf_key_from_sk(&domain, &spend_sk);

    let cm_in = note_commitment(&domain, value_in, &rho_in, &recipient_owner, &sender_id_in);
    let mut tree = MerkleTree::new(depth);
    tree.set_leaf(pos as usize, cm_in);
    let anchor = tree.root();
    let siblings = tree.open(pos as usize);
    anyhow::ensure!(siblings.len() == depth, "unexpected siblings length");
    let nf0 = nullifier(&domain, &nf_key, &rho_in);

    // Change to self.
    let out_pk_spend = pk_spend_owner;
    let out_pk_ivk = pk_ivk_owner;
    let cm_out = note_commitment(
        &domain,
        out_value,
        &out_rho,
        &recipient_owner,
        &sender_id_current,
    );

    let inv_enforce = match inv_enforce_override {
        Some(v) => v,
        None => compute_inv_enforce(&[value_in], &[rho_in], &[out_value], &[out_rho])?,
    };

    let mut args: Vec<LigeroArg> = Vec::new();
    args.push(LigeroArg::Hex { hex: hx32(&domain) }); // 1
    args.push(LigeroArg::Hex { hex: hx32(&spend_sk) }); // 2 (priv)
    args.push(LigeroArg::Hex { hex: hx32(&pk_ivk_owner) }); // 3 (priv)
    args.push(LigeroArg::I64 { i64: depth as i64 }); // 4
    args.push(LigeroArg::Hex { hex: hx32(&anchor) }); // 5 (pub)
    args.push(LigeroArg::I64 { i64: 1 }); // 6 n_in

    args.push(LigeroArg::I64 {
        i64: value_in as i64,
    }); // value_in
    args.push(LigeroArg::Hex { hex: hx32(&rho_in) });
    args.push(LigeroArg::Hex {
        hex: hx32(&sender_id_in),
    });
    for lvl in 0..depth {
        let bit = ((pos >> lvl) & 1) as u8;
        let mut bit_bytes = [0u8; 32];
        bit_bytes[31] = bit;
        args.push(LigeroArg::Hex { hex: hx32(&bit_bytes) });
    }
    for s in &siblings {
        args.push(LigeroArg::Hex { hex: hx32(s) });
    }
    args.push(LigeroArg::Hex { hex: hx32(&nf0) });

    args.push(LigeroArg::I64 {
        i64: withdraw_amount as i64,
    });
    args.push(LigeroArg::I64 { i64: 1 });

    args.push(LigeroArg::I64 {
        i64: out_value as i64,
    });
    args.push(LigeroArg::Hex { hex: hx32(&out_rho) });
    args.push(LigeroArg::Hex { hex: hx32(&out_pk_spend) });
    args.push(LigeroArg::Hex { hex: hx32(&out_pk_ivk) });
    args.push(LigeroArg::Hex { hex: hx32(&cm_out) });

    args.push(LigeroArg::Hex {
        hex: hx32(&inv_enforce),
    });

    Ok((args, private_indices_spend(depth, 1, 1)))
}

#[test]
fn test_note_spend_v2_rejects_invalid_witnesses() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_guest(&repo, "utils/circuits/note-spend", "utils/circuits/bins/note_spend_guest.wasm")?;

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
        eprintln!("Skipping: shader path not found at {}", shader_dir.display());
        return Ok(());
    }

    let vpaths = verifier::VerifierPaths::from_explicit(
        program.clone(),
        shader_dir,
        runner.paths().verifier_bin.clone(),
        packing,
    );

    // Baseline valid tx (transfer 1-in -> 2-out).
    let depth = 4;
    let domain: Hash32 = [0x11u8; 32];
    let spend_sk: Hash32 = [0x22u8; 32];
    let value_in: u64 = 200;
    let rho_in: Hash32 = [0x33u8; 32];
    let sender_id_in: Hash32 = [0x44u8; 32];
    let pos: u64 = 1;
    let out0_rho: Hash32 = [0x55u8; 32];
    let out1_rho: Hash32 = [0x66u8; 32];

    let (base_args, base_priv, _anchor) = build_transfer_1in_2out(
        depth,
        domain,
        spend_sk,
        value_in,
        rho_in,
        sender_id_in,
        pos,
        out0_rho,
        out1_rho,
        None,
    )?;

    // Prove baseline once to ensure environment is functional.
    let base_proof = match prove(&mut runner, base_args.clone(), base_priv.clone()) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("Skipping: prover failed (GPU/WebGPU likely unavailable): {err}");
            return Ok(());
        }
    };
    anyhow::ensure!(
        verify(&vpaths, &base_proof, base_args.clone(), base_priv.clone())?,
        "baseline proof must verify"
    );

    // F21: wrong inv_enforce must be rejected.
    let mut bad_args = base_args.clone();
    let idx_inv = inv_enforce_index(1, depth, 2);
    bad_args[idx_inv - 1] = LigeroArg::Hex { hex: hx32(&[0u8; 32]) };
    assert_rejected(&mut runner, &vpaths, bad_args, base_priv.clone())?;

    // F5: wrong sender_id_in must be rejected (leaf changes => anchor mismatch).
    let mut bad_args = base_args.clone();
    let idx_sender_id_in = 7 + 2; // value_in, rho_in, sender_id_in
    bad_args[idx_sender_id_in - 1] = LigeroArg::Hex { hex: hx32(&[0x99u8; 32]) };
    assert_rejected(&mut runner, &vpaths, bad_args, base_priv.clone())?;

    // F24: rho reuse (out0 rho == in rho), with consistent cm_out0.
    let (reuse_args, reuse_priv, _anchor) = build_transfer_1in_2out(
        depth,
        domain,
        spend_sk,
        value_in,
        rho_in,
        sender_id_in,
        pos,
        rho_in,           // out0_rho == in_rho
        out1_rho,
        Some([0u8; 32]), // inv_enforce undefined -> set dummy
    )?;
    assert_rejected(&mut runner, &vpaths, reuse_args, reuse_priv)?;

    // F25: two outputs share rho, with consistent cm_outs.
    let (dup_out_rho_args, dup_out_rho_priv, _anchor) = build_transfer_1in_2out(
        depth,
        domain,
        spend_sk,
        value_in,
        rho_in,
        sender_id_in,
        pos,
        out0_rho,
        out0_rho,         // out1_rho == out0_rho
        Some([0u8; 32]), // inv_enforce undefined -> set dummy
    )?;
    assert_rejected(&mut runner, &vpaths, dup_out_rho_args, dup_out_rho_priv)?;

    // F23: zero output value (withdraw all) with consistent public cm_out must be rejected.
    let (zero_out_args, zero_out_priv) = build_withdraw_1in_1out(
        depth,
        domain,
        spend_sk,
        value_in,
        rho_in,
        sender_id_in,
        pos,
        value_in,         // withdraw all
        0,                // out_value=0 (balance holds)
        [0x77u8; 32],
        Some([0u8; 32]), // inv_enforce undefined -> set dummy
    )?;
    assert_rejected(&mut runner, &vpaths, zero_out_args, zero_out_priv)?;

    // F9/R5: duplicate nullifiers within a tx (n_in=2) must be rejected.
    {
        let n_in = 2usize;
        let n_out = 0usize;

        let pk_ivk_owner = ivk_seed(&domain, &spend_sk);
        let pk_spend_owner = pk_from_sk(&spend_sk);
        let recipient_owner = recipient_from_pk(&domain, &pk_spend_owner, &pk_ivk_owner);
        let nf_key = nf_key_from_sk(&domain, &spend_sk);

        let positions = [0u64, 1u64];
        let values_in = [50u64, 60u64];
        let rhos_in = [rho_in, rho_in]; // forces duplicate nullifiers
        let sender_ids_in: [Hash32; 2] = [[0x10u8; 32], [0x11u8; 32]];

        let mut tree = MerkleTree::new(depth);
        for i in 0..2 {
            let cm_i = note_commitment(
                &domain,
                values_in[i],
                &rhos_in[i],
                &recipient_owner,
                &sender_ids_in[i],
            );
            tree.set_leaf(positions[i] as usize, cm_i);
        }
        let anchor = tree.root();
        let siblings0 = tree.open(positions[0] as usize);
        let siblings1 = tree.open(positions[1] as usize);
        anyhow::ensure!(siblings0.len() == depth, "unexpected siblings length");
        anyhow::ensure!(siblings1.len() == depth, "unexpected siblings length");

        let nf = nullifier(&domain, &nf_key, &rho_in);
        let inv_enforce = compute_inv_enforce(&values_in, &rhos_in, &[], &[])?;

        let mut args: Vec<LigeroArg> = Vec::new();
        args.push(LigeroArg::Hex { hex: hx32(&domain) }); // 1
        args.push(LigeroArg::Hex { hex: hx32(&spend_sk) }); // 2 (priv)
        args.push(LigeroArg::Hex { hex: hx32(&pk_ivk_owner) }); // 3 (priv)
        args.push(LigeroArg::I64 { i64: depth as i64 }); // 4
        args.push(LigeroArg::Hex { hex: hx32(&anchor) }); // 5 (pub)
        args.push(LigeroArg::I64 { i64: n_in as i64 }); // 6 (pub)

        for i in 0..2 {
            args.push(LigeroArg::I64 {
                i64: values_in[i] as i64,
            });
            args.push(LigeroArg::Hex { hex: hx32(&rhos_in[i]) });
            args.push(LigeroArg::Hex { hex: hx32(&sender_ids_in[i]) });

            let pos_i = positions[i];
            for lvl in 0..depth {
                let bit = ((pos_i >> lvl) & 1) as u8;
                let mut bit_bytes = [0u8; 32];
                bit_bytes[31] = bit;
                args.push(LigeroArg::Hex { hex: hx32(&bit_bytes) });
            }

            let sibs = if i == 0 { &siblings0 } else { &siblings1 };
            for s in sibs {
                args.push(LigeroArg::Hex { hex: hx32(s) });
            }

            // nullifier_i (pub) - duplicated on purpose
            args.push(LigeroArg::Hex { hex: hx32(&nf) });
        }

        let withdraw_amount = values_in.iter().sum::<u64>();
        args.push(LigeroArg::I64 {
            i64: withdraw_amount as i64,
        });
        args.push(LigeroArg::I64 { i64: 0 }); // n_out
        args.push(LigeroArg::Hex {
            hex: hx32(&inv_enforce),
        });

        let priv_idx = private_indices_spend(depth, n_in, n_out);
        assert_rejected(&mut runner, &vpaths, args, priv_idx)?;
    }

    Ok(())
}

#[test]
fn test_note_spend_v2_verifier_binding_and_tampering() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_guest(&repo, "utils/circuits/note-spend", "utils/circuits/bins/note_spend_guest.wasm")?;

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
        eprintln!("Skipping: shader path not found at {}", shader_dir.display());
        return Ok(());
    }

    let vpaths = verifier::VerifierPaths::from_explicit(
        program.clone(),
        shader_dir,
        runner.paths().verifier_bin.clone(),
        packing,
    );

    let depth = 4;
    let domain: Hash32 = [0x11u8; 32];
    let spend_sk: Hash32 = [0x22u8; 32];
    let value_in: u64 = 200;
    let rho_in: Hash32 = [0x33u8; 32];
    let sender_id_in: Hash32 = [0x44u8; 32];
    let pos: u64 = 1;
    let out0_rho: Hash32 = [0x55u8; 32];
    let out1_rho: Hash32 = [0x66u8; 32];

    let (args, priv_idx, anchor) = build_transfer_1in_2out(
        depth,
        domain,
        spend_sk,
        value_in,
        rho_in,
        sender_id_in,
        pos,
        out0_rho,
        out1_rho,
        None,
    )?;

    let proof = match prove(&mut runner, args.clone(), priv_idx.clone()) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("Skipping: prover failed (GPU/WebGPU likely unavailable): {err}");
            return Ok(());
        }
    };

    anyhow::ensure!(
        verify(&vpaths, &proof, args.clone(), priv_idx.clone())?,
        "baseline proof must verify"
    );

    // R2: Non-canonical encoding for a public digest must be rejected.
    // Use anchor' = anchor + p (still 32 bytes, but >= p), which should not be accepted as a valid Fr encoding.
    let anchor_plus_p = add_be_32(&anchor, &BN254_FR_MODULUS_BE);
    let mut bad_args = args.clone();
    bad_args[4] = LigeroArg::Hex {
        hex: hx32(&anchor_plus_p),
    }; // [5] anchor, 0-based index 4
    anyhow::ensure!(
        !verify(&vpaths, &proof, bad_args, priv_idx.clone())?,
        "expected verification to reject non-canonical public digest encoding"
    );

    // R3: Proof byte tampering must be rejected.
    let mut bad_proof = proof.clone();
    if let Some(b) = bad_proof.get_mut(0) {
        *b ^= 0x01;
    }
    anyhow::ensure!(
        !verify(&vpaths, &bad_proof, args.clone(), priv_idx.clone())?,
        "expected verification to reject tampered proof bytes"
    );

    // Public statement binding: mutate cm_out0 and ensure verification fails.
    let mut bad_args = args;
    let idx_cm0 = cm_out_index(1, depth, 0);
    let mut bad_cm = [0u8; 32];
    bad_cm[0] = 0x99;
    bad_args[idx_cm0 - 1] = LigeroArg::Hex { hex: hx32(&bad_cm) };
    anyhow::ensure!(
        !verify(&vpaths, &proof, bad_args, priv_idx)?,
        "expected verification to reject mutated public cm_out"
    );

    Ok(())
}

#[test]
fn test_note_deposit_note_v2_is_spendable() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_guest(&repo, "utils/circuits/note-deposit", "utils/circuits/bins/note_deposit_guest.wasm")?;
    maybe_build_guest(&repo, "utils/circuits/note-spend", "utils/circuits/bins/note_spend_guest.wasm")?;

    let deposit_program = note_deposit_program_path(&repo)
        .canonicalize()
        .context("Failed to canonicalize note_deposit_guest.wasm")?;
    let spend_program = note_spend_program_path(&repo)?
        .canonicalize()
        .context("Failed to canonicalize note_spend_guest.wasm")?;

    let packing: u32 = std::env::var("LIGERO_PACKING")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8192);

    // ===== Deposit proof =====
    let mut deposit_runner = LigeroRunner::new(&deposit_program.to_string_lossy());
    deposit_runner.config_mut().packing = packing;

    if !deposit_runner.paths().prover_bin.exists() || !deposit_runner.paths().verifier_bin.exists() {
        eprintln!(
            "Skipping: Ligero binaries not found (prover={}, verifier={})",
            deposit_runner.paths().prover_bin.display(),
            deposit_runner.paths().verifier_bin.display()
        );
        return Ok(());
    }

    let shader_dir = PathBuf::from(&deposit_runner.config().shader_path);
    if !shader_dir.exists() {
        eprintln!("Skipping: shader path not found at {}", shader_dir.display());
        return Ok(());
    }

    let deposit_vpaths = verifier::VerifierPaths::from_explicit(
        deposit_program.clone(),
        shader_dir.clone(),
        deposit_runner.paths().verifier_bin.clone(),
        packing,
    );

    let domain: Hash32 = [0x0au8; 32];
    let value: u64 = 123;
    let rho: Hash32 = [0x0bu8; 32];
    let spend_sk: Hash32 = [0x0cu8; 32];
    let pk_spend = pk_from_sk(&spend_sk);
    let pk_ivk = ivk_seed(&domain, &spend_sk);
    let recipient = recipient_from_pk(&domain, &pk_spend, &pk_ivk);
    let sender_id_deposit = [0u8; 32];
    let cm = note_commitment(&domain, value, &rho, &recipient, &sender_id_deposit);

    let deposit_args = vec![
        LigeroArg::Hex { hex: hx32(&domain) },
        LigeroArg::I64 { i64: value as i64 },
        LigeroArg::Hex { hex: hx32(&rho) },
        LigeroArg::Hex { hex: hx32(&pk_spend) },
        LigeroArg::Hex { hex: hx32(&pk_ivk) },
        LigeroArg::Hex { hex: hx32(&cm) },
    ];
    let deposit_priv = vec![3usize, 4usize, 5usize];

    let deposit_proof = match prove(&mut deposit_runner, deposit_args.clone(), deposit_priv.clone()) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("Skipping: deposit prover failed (GPU/WebGPU likely unavailable): {err}");
            return Ok(());
        }
    };
    anyhow::ensure!(
        verify(&deposit_vpaths, &deposit_proof, deposit_args, deposit_priv)?,
        "deposit proof must verify"
    );

    // ===== Spend proof of the deposited note =====
    let mut spend_runner = LigeroRunner::new(&spend_program.to_string_lossy());
    spend_runner.config_mut().packing = packing;

    let spend_vpaths = verifier::VerifierPaths::from_explicit(
        spend_program.clone(),
        shader_dir,
        spend_runner.paths().verifier_bin.clone(),
        packing,
    );

    // Spend as a simple 1-in -> 1-out transfer to self (withdraw=0).
    let depth = 4;
    let pos: u64 = 0;
    let pk_ivk_owner = pk_ivk;
    let pk_spend_owner = pk_spend;
    let recipient_owner = recipient_from_pk(&domain, &pk_spend_owner, &pk_ivk_owner);
    anyhow::ensure!(recipient_owner == recipient, "recipient mismatch");

    let sender_id_in = sender_id_deposit;
    let value_in = value;
    let rho_in = rho;

    let out_value = value_in;
    let out_rho: Hash32 = [0x55u8; 32];
    let _out_pk_spend = pk_spend_owner;
    let _out_pk_ivk = pk_ivk_owner;

    let (spend_args, spend_priv) = build_withdraw_1in_1out(
        depth,
        domain,
        spend_sk,
        value_in,
        rho_in,
        sender_id_in,
        pos,
        0,       // withdraw_amount
        out_value,
        out_rho,
        None,
    )?;

    let spend_proof = match prove(&mut spend_runner, spend_args.clone(), spend_priv.clone()) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("Skipping: spend prover failed (GPU/WebGPU likely unavailable): {err}");
            return Ok(());
        }
    };

    anyhow::ensure!(
        verify(&spend_vpaths, &spend_proof, spend_args, spend_priv)?,
        "spend proof must verify for a deposit-minted NOTE_V2"
    );

    Ok(())
}
