//! Multi-proof timing test for prover + verifier using always-on worker pools.
//!
//! This generates multiple distinct proofs (different witnesses) and verifies each.
//! It helps distinguish "process warmup" from "same-input caching" effects.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use anyhow::{Context, Result};
use ligero_runner::{verifier, BinaryWorkerPool, LigeroArg, LigeroRunner};
use ligetron::poseidon2_hash_bytes;

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

fn note_commitment(domain: &Hash32, value: u64, rho: &Hash32, recipient: &Hash32) -> Hash32 {
    // Guest encodes value as 16-byte LE (u64 zero-extended to 16 bytes).
    let mut v16 = [0u8; 16];
    v16[..8].copy_from_slice(&value.to_le_bytes());
    poseidon2_hash_domain(b"NOTE_V1", &[domain, &v16, rho, recipient])
}

fn pk_from_sk(spend_sk: &Hash32) -> Hash32 {
    poseidon2_hash_domain(b"PK_V1", &[spend_sk])
}

fn recipient_from_pk(domain: &Hash32, pk: &Hash32) -> Hash32 {
    poseidon2_hash_domain(b"ADDR_V1", &[domain, pk])
}

fn recipient_from_sk(domain: &Hash32, spend_sk: &Hash32) -> Hash32 {
    recipient_from_pk(domain, &pk_from_sk(spend_sk))
}

fn nf_key_from_sk(domain: &Hash32, spend_sk: &Hash32) -> Hash32 {
    poseidon2_hash_domain(b"NFKEY_V1", &[domain, spend_sk])
}

fn nullifier(domain: &Hash32, nf_key: &Hash32, rho: &Hash32) -> Hash32 {
    poseidon2_hash_domain(b"PRF_NF_V1", &[domain, nf_key, rho])
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

fn private_indices(depth: usize, n_out: usize) -> Vec<usize> {
    // NOTE: `private-indices` are 1-based indices into the args list (excluding argv[0]).
    //
    // This matches the guest's documented layout in:
    // `utils/circuits/note-spend-guest/src/main.rs` (see top-of-file comment).
    //
    // Private (witness):
    // - recipient (4), spend_sk (5)
    // - pos_bits (7..7+depth-1), siblings (7+depth..7+2*depth-1)
    // - for each output: value_out, rho_out, pk_out
    let mut idx = Vec::new();
    idx.push(4); // recipient
    idx.push(5); // spend_sk

    // pos_bits
    for i in 0..depth {
        idx.push(7 + i);
    }
    // siblings
    for i in 0..depth {
        idx.push(7 + depth + i);
    }

    // outputs
    let outs_base = 11 + 2 * depth; // index of value_out_0
    for j in 0..n_out {
        idx.push(outs_base + 4 * j + 0); // value_out_j
        idx.push(outs_base + 4 * j + 1); // rho_out_j
        idx.push(outs_base + 4 * j + 2); // pk_out_j
        // cm_out_j is public (outs_base + 4*j + 3)
    }

    idx
}

fn maybe_build_note_spend_guest(repo: &Path) -> Result<()> {
    let out = repo.join("utils/circuits/bins/programs/note_spend_guest.wasm");
    if out.exists() {
        return Ok(());
    }

    let guest_dir = repo.join("utils/circuits/note-spend-guest");
    if !guest_dir.exists() {
        anyhow::bail!("note-spend-guest sources not found at {}", guest_dir.display());
    }

    println!(
        "[two_pass] note_spend_guest.wasm not found; building via {}",
        guest_dir.join("build.sh").display()
    );

    let status = Command::new("bash")
        .arg("build.sh")
        .current_dir(&guest_dir)
        .status()
        .context("Failed to run note-spend-guest/build.sh")?;

    if !status.success() {
        anyhow::bail!("note-spend-guest/build.sh failed with status {status}");
    }

    Ok(())
}

#[test]
fn test_two_pass_prover_and_verifier_timings() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_spend_guest(&repo)?;

    let program = repo
        .join("utils/circuits/bins/programs/note_spend_guest.wasm")
        .canonicalize()
        .context("Failed to canonicalize note_spend_guest.wasm")?;

    let packing: u32 = std::env::var("LIGERO_PACKING")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8192);

    let mut runner = LigeroRunner::new(&program.to_string_lossy());
    runner.config_mut().packing = packing;

    // Force single-worker pools so all runs execute on the same dedicated thread.
    let prover_pool = BinaryWorkerPool::new(1);
    let verifier_pool = BinaryWorkerPool::new(1);

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

    println!("[two_pass] Prover:   {}", runner.paths().prover_bin.display());
    println!("[two_pass] Verifier: {}", runner.paths().verifier_bin.display());
    println!("[two_pass] Shaders:  {}", shader_dir.display());
    println!("[two_pass] Program:  {}", program.display());
    println!("[two_pass] Packing:  {}", packing);

    let depth: usize = 8;
    let domain: Hash32 = [1u8; 32];
    let value: u64 = 42;
    let pos: u64 = 0;
    let n_out: u64 = 1;

    // --- Prover: generate 3 distinct proofs ---
    let mut proofs: Vec<Vec<u8>> = Vec::new();
    let mut configs: Vec<Vec<LigeroArg>> = Vec::new();
    let mut all_priv_idx: Vec<Vec<usize>> = Vec::new();

    // Build 3 distinct spend statements (with PUBLIC differences) so that cross-verification
    // (proof_i verified with params_j for i!=j) must fail.
    for run in 0..3u8 {
        // Vary public rho (arg[3]) so the statement differs even if you ignore private witness.
        let mut rho: Hash32 = [2u8; 32];
        rho[0] = rho[0].wrapping_add(run);

        // Vary private spend_sk (arg[5]) so witness differs too.
        let mut spend_sk: Hash32 = [4u8; 32];
        spend_sk[0] = spend_sk[0].wrapping_add(run);

        // Vary withdraw (PUBLIC) and out_value (PRIVATE but affects PUBLIC cm_out via constraint).
        let withdraw_amount: u64 = run as u64; // 0, 1, 2
        let out_value: u64 = value
            .checked_sub(withdraw_amount)
            .context("value must be >= withdraw_amount")?;

        let recipient = recipient_from_sk(&domain, &spend_sk);
        let nf_key = nf_key_from_sk(&domain, &spend_sk);

        let cm_in = note_commitment(&domain, value, &rho, &recipient);
        let mut tree = MerkleTree::new(depth);
        tree.set_leaf(pos as usize, cm_in);
        let anchor = tree.root();
        let siblings = tree.open(pos as usize);
        let nf = nullifier(&domain, &nf_key, &rho);

        let mut out_rho: Hash32 = [7u8; 32];
        out_rho[0] = out_rho[0].wrapping_add(run);
        let mut out_spend_sk: Hash32 = [8u8; 32];
        out_spend_sk[0] = out_spend_sk[0].wrapping_add(run);
        let out_pk = pk_from_sk(&out_spend_sk);
        let out_recipient = recipient_from_pk(&domain, &out_pk);
        let cm_out = note_commitment(&domain, out_value, &out_rho, &out_recipient);

        let mut args: Vec<LigeroArg> = Vec::new();
        args.push(LigeroArg::Hex { hex: hx32(&domain) });
        args.push(LigeroArg::I64 { i64: value as i64 });
        args.push(LigeroArg::Hex { hex: hx32(&rho) });
        args.push(LigeroArg::Hex { hex: hx32(&recipient) });
        args.push(LigeroArg::Hex { hex: hx32(&spend_sk) });
        args.push(LigeroArg::I64 { i64: depth as i64 });

        for lvl in 0..depth {
            let bit = ((pos >> lvl) & 1) as u8;
            let mut bit_bytes = [0u8; 32];
            bit_bytes[31] = bit;
            args.push(LigeroArg::Hex { hex: hx32(&bit_bytes) });
        }

        for s in &siblings {
            args.push(LigeroArg::Hex { hex: hx32(s) });
        }

        args.push(LigeroArg::String { str: format!("0x{}", hx32(&anchor)) });
        args.push(LigeroArg::String { str: format!("0x{}", hx32(&nf)) });

        args.push(LigeroArg::I64 { i64: withdraw_amount as i64 });
        args.push(LigeroArg::I64 { i64: n_out as i64 });
        args.push(LigeroArg::I64 { i64: out_value as i64 });
        args.push(LigeroArg::Hex { hex: hx32(&out_rho) });
        args.push(LigeroArg::Hex { hex: hx32(&out_pk) });
        args.push(LigeroArg::Hex { hex: hx32(&cm_out) });

        let priv_idx = private_indices(depth, 1);
        runner.config_mut().private_indices = priv_idx.clone();
        runner.config_mut().args = args.clone();

        let t = Instant::now();
        let (proof, _s, _e) = runner.run_prover_with_output_in_pool(
            &prover_pool,
            ligero_runner::ProverRunOptions {
                keep_proof_dir: false,
                proof_outputs_base: None,
                write_replay_script: true,
            },
        )?;
        let d = t.elapsed();
        println!(
            "[two_pass] Prover run #{}: {:?} ({} bytes) withdraw={} out_value={}",
            run + 1,
            d,
            proof.len(),
            withdraw_amount,
            out_value
        );
        proofs.push(proof);
        configs.push(args);
        all_priv_idx.push(priv_idx);
    }

    // --- Verifier: verify the 3 different proofs ---
    let vpaths = verifier::VerifierPaths::from_explicit(
        program.clone(),
        shader_dir,
        runner.paths().verifier_bin.clone(),
        packing,
    );

    for (i, ((proof, args), priv_idx)) in proofs
        .iter()
        .zip(configs.iter())
        .zip(all_priv_idx.iter())
        .enumerate()
    {
        let tv = Instant::now();
        let (ok, _vs, _ve) = verifier::verify_proof_with_output_in_pool(
            &verifier_pool,
            &vpaths,
            proof,
            args.clone(),
            priv_idx.clone(),
        )?;
        let vd = tv.elapsed();
        assert!(ok, "verifier should report success for run #{}", i + 1);
        println!("[two_pass] Verifier run #{}: {:?}", i + 1, vd);
    }

    // --- Negative: cross-verify must FAIL (public statements differ) ---
    for i in 0..proofs.len() {
        for j in 0..configs.len() {
            if i == j {
                continue;
            }
            match verifier::verify_proof_with_output_in_pool(
                &verifier_pool,
                &vpaths,
                &proofs[i],
                configs[j].clone(),
                all_priv_idx[j].clone(),
            ) {
                Ok((ok, _vs, _ve)) => {
                    anyhow::ensure!(
                        !ok,
                        "expected cross-verify to fail, but proof#{} verified with cfg#{} (this implies verifier is not binding to provided public inputs)",
                        i + 1,
                        j + 1
                    );
                }
                Err(_e) => {
                    // Any error is also an acceptable failure signal for cross-verification.
                }
            }
        }
    }

    Ok(())
}
