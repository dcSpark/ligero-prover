//! Two-pass timing test for prover + verifier using *daemon-mode* binaries.
//!
//! This measures cold vs warm runs on the **same long-lived process** by sending two requests
//! to a single-worker daemon pool.

use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use anyhow::{Context, Result};
use ligero_runner::{daemon::DaemonPool, LigeroArg, LigeroRunner};
use ligetron::poseidon2_hash_bytes;
use ligetron::Bn254Fr;

type Hash32 = [u8; 32];

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

fn private_indices(depth: usize, n_in: usize, n_out: usize) -> Vec<usize> {
    // Mirrors the guest layout described in utils/circuits/note-spend/src/main.rs (v2 ABI).
    //
    // Private:
    // - spend_sk, pk_ivk_owner
    // - per-input: value_in, rho_in, sender_id_in, pos_bits, siblings
    // - per-output: value_out, rho_out, pk_spend_out, pk_ivk_out
    // - inv_enforce witness
    let mut idx = vec![2usize, 3usize]; // spend_sk, pk_ivk_owner

    let per_in = 4usize + 2usize * depth; // value + rho + sender_id_in + pos_bits[depth] + siblings[depth] + nullifier
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
    let outs_base = withdraw_idx + 2; // skip withdraw_amount + n_out
    for j in 0..n_out {
        idx.push(outs_base + 5 * j + 0); // value_out
        idx.push(outs_base + 5 * j + 1); // rho_out
        idx.push(outs_base + 5 * j + 2); // pk_spend_out
        idx.push(outs_base + 5 * j + 3); // pk_ivk_out
                                         // cm_out is public
    }

    idx.push(outs_base + 5 * n_out); // inv_enforce

    idx
}

fn maybe_build_note_spend_guest(repo: &PathBuf) -> Result<()> {
    let wasm = repo.join("utils/circuits/bins/note_spend_guest.wasm");
    if wasm.exists() {
        return Ok(());
    }

    let build_sh = repo.join("utils/circuits/note-spend/build.sh");
    if !build_sh.exists() {
        anyhow::bail!(
            "note_spend_guest.wasm not found at {}, and build script missing at {}",
            wasm.display(),
            build_sh.display()
        );
    }

    let status = Command::new("bash")
        .arg(build_sh)
        .current_dir(repo)
        .status()
        .context("Failed to run note-spend/build.sh")?;

    if !status.success() {
        anyhow::bail!("note-spend/build.sh failed with status {status}");
    }

    Ok(())
}

#[test]
fn test_two_pass_daemon_prover_and_verifier_timings() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_spend_guest(&repo)?;

    let program = repo
        .join("utils/circuits/bins/note_spend_guest.wasm")
        .canonicalize()
        .context("Failed to canonicalize note_spend_guest.wasm")?;

    let packing: u32 = std::env::var("LIGERO_PACKING")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8192);

    let mut runner = LigeroRunner::new(&program.to_string_lossy());
    runner.config_mut().packing = packing;
    runner.config_mut().gzip_proof = false;

    if !runner.paths().prover_bin.exists() || !runner.paths().verifier_bin.exists() {
        eprintln!(
            "Skipping: Ligero binaries not found (prover={}, verifier={})",
            runner.paths().prover_bin.display(),
            runner.paths().verifier_bin.display()
        );
        return Ok(());
    }

    // Single-worker daemon pools so pass1 and pass2 hit the same long-lived process.
    // If the precompiled portable binaries don't yet include `--daemon`, skip with a clear message.
    let prover_pool = match DaemonPool::new_prover(runner.paths(), 1) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Skipping: prover binary does not appear to support --daemon yet: {e:#}");
            return Ok(());
        }
    };
    let verifier_pool = match DaemonPool::new_verifier(runner.paths(), 1) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Skipping: verifier binary does not appear to support --daemon yet: {e:#}");
            return Ok(());
        }
    };

    let shader_dir = PathBuf::from(&runner.config().shader_path);
    if !shader_dir.exists() {
        eprintln!(
            "Skipping: shader path not found at {}",
            shader_dir.display()
        );
        return Ok(());
    }

    println!(
        "[daemon_two_pass] Prover:   {}",
        runner.paths().prover_bin.display()
    );
    println!(
        "[daemon_two_pass] Verifier: {}",
        runner.paths().verifier_bin.display()
    );
    println!("[daemon_two_pass] Shaders:  {}", shader_dir.display());
    println!("[daemon_two_pass] Program:  {}", program.display());
    println!("[daemon_two_pass] Packing:  {}", packing);

    // Build one-output spend witness.
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
    let mut tree = MerkleTree::new(depth);
    tree.set_leaf(pos as usize, cm_in);
    let anchor = tree.root();
    let siblings = tree.open(pos as usize);
    let nf = nullifier(&domain, &nf_key, &rho);

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

    // Input 0 (value, rho, sender_id_in, pos_bits[depth], siblings[depth], nullifier)
    args.push(LigeroArg::I64 { i64: value as i64 }); // 7 value_in_0 (private)
    args.push(LigeroArg::Hex { hex: hx32(&rho) }); // 8 rho_in_0 (private)
    args.push(LigeroArg::Hex {
        hex: hx32(&sender_id_in),
    }); // sender_id_in_0 (private)

    for lvl in 0..depth {
        let bit = ((pos >> lvl) & 1) as u8;
        let mut bit_bytes = [0u8; 32];
        bit_bytes[31] = bit;
        args.push(LigeroArg::Hex {
            hex: hx32(&bit_bytes),
        });
    }

    for s in &siblings {
        args.push(LigeroArg::Hex { hex: hx32(s) });
    }

    // Input nullifier (public).
    args.push(LigeroArg::Hex { hex: hx32(&nf) });

    args.push(LigeroArg::I64 {
        i64: withdraw_amount as i64,
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

    let priv_idx = private_indices(depth, 1, 1);
    runner.config_mut().private_indices = priv_idx;
    runner.config_mut().args = args;

    let config_val = serde_json::to_value(runner.config()).context("Failed to to_value config")?;

    // --- Prover two-pass timing ---
    let t0 = Instant::now();
    let r1 = prover_pool.prove(config_val.clone())?;
    let d1 = t0.elapsed();
    assert!(r1.ok, "prover pass1 failed: {:?}", r1.error);
    let proof_path_1 = r1
        .proof_path
        .clone()
        .context("prover pass1 did not return proof_path")?;

    let t1 = Instant::now();
    let r2 = prover_pool.prove(config_val.clone())?;
    let d2 = t1.elapsed();
    assert!(r2.ok, "prover pass2 failed: {:?}", r2.error);
    let proof_path_2 = r2
        .proof_path
        .clone()
        .context("prover pass2 did not return proof_path")?;

    let size1 = std::fs::metadata(&proof_path_1)
        .map(|m| m.len())
        .unwrap_or(0);
    let size2 = std::fs::metadata(&proof_path_2)
        .map(|m| m.len())
        .unwrap_or(0);

    println!(
        "[daemon_two_pass] Prover pass1: {:?} ({} bytes) -> {}",
        d1, size1, proof_path_1
    );
    println!(
        "[daemon_two_pass] Prover pass2: {:?} ({} bytes) -> {}",
        d2, size2, proof_path_2
    );

    // --- Verifier two-pass timing (verify proof1 twice) ---
    let tv0 = Instant::now();
    let v1 = verifier_pool.verify(config_val.clone(), &proof_path_1)?;
    let vd1 = tv0.elapsed();

    let tv1 = Instant::now();
    let v2 = verifier_pool.verify(config_val, &proof_path_1)?;
    let vd2 = tv1.elapsed();

    assert!(v1.ok && v2.ok, "verifier should report success");

    println!("[daemon_two_pass] Verifier pass1: {:?}", vd1);
    println!("[daemon_two_pass] Verifier pass2: {:?}", vd2);

    Ok(())
}
