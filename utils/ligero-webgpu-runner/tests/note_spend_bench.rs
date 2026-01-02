//! Note-spend proving/verifying benchmark tests.
//!
//! This file intentionally contains both:
//! - a **daemon** benchmark using `webgpu_{prover,verifier} --daemon` via `DaemonPool`
//! - a **direct** benchmark using `BinaryWorkerPool` (spawns processes per request)
//!
//! The two tests share the same witness/statement construction code so we can do apples-to-apples
//! comparisons across strategies.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use anyhow::{Context, Result};
use ligetron::poseidon2_hash_bytes;
use sha2::{Digest, Sha256};

use ligero_runner::{
    daemon::DaemonPool, verifier, BinaryWorkerPool, LigeroArg, LigeroRunner, ProverRunOptions,
};

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

fn read_packing() -> u32 {
    std::env::var("LIGERO_PACKING")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8192)
}

fn read_gzip_proof(default: bool) -> bool {
    std::env::var("LIGERO_GZIP_PROOF")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(default)
}

fn is_verbose() -> bool {
    std::env::var("LIGERO_VERBOSE")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn read_num_runs() -> u8 {
    std::env::var("LIGERO_RUNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3)
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
    // `utils/circuits/note-spend/src/main.rs` (see top-of-file comment).
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
    let out = repo.join("utils/circuits/bins/note_spend_guest.wasm");
    if out.exists() {
        return Ok(());
    }

    let guest_dir = repo.join("utils/circuits/note-spend");
    if !guest_dir.exists() {
        anyhow::bail!("note-spend sources not found at {}", guest_dir.display());
    }

    println!(
        "[note_spend] note_spend_guest.wasm not found; building via {}",
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

    Ok(())
}

/// Build one statement (args) plus a small public summary for sanity checks.
fn build_statement(
    run: u8,
    depth: usize,
    domain: Hash32,
    value: u64,
    pos: u64,
) -> Result<(Vec<LigeroArg>, (Hash32, Hash32, Vec<Hash32>, u64, usize))> {
    // Vary public rho (arg[3]) so the statement differs even if you ignore private witness.
    let mut rho: Hash32 = [2u8; 32];
    rho[0] = rho[0].wrapping_add(run);

    // Vary private spend_sk (arg[5]) so witness differs too.
    let mut spend_sk: Hash32 = [4u8; 32];
    spend_sk[0] = spend_sk[0].wrapping_add(run);

    // Keep `n_out` constant across runs so `private-indices` layout is identical.
    let n_out: usize = 1;
    let withdraw_amount: u64 = run as u64; // 0,1,2
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

    // Outputs: value/rho/pk are private; cm_out is public.
    let mut cm_outs: Vec<Hash32> = Vec::with_capacity(1);
    let mut out_triples: Vec<(u64, Hash32, Hash32, Hash32)> = Vec::with_capacity(1);
    {
        let mut out_rho: Hash32 = [7u8; 32];
        out_rho[0] = out_rho[0].wrapping_add(run);
        let mut out_spend_sk: Hash32 = [8u8; 32];
        out_spend_sk[0] = out_spend_sk[0].wrapping_add(run);
        let out_pk = pk_from_sk(&out_spend_sk);
        let out_recipient = recipient_from_pk(&domain, &out_pk);
        let cm_out = note_commitment(&domain, out_value, &out_rho, &out_recipient);
        cm_outs.push(cm_out);
        out_triples.push((out_value, out_rho, out_pk, cm_out));
    }

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

    args.push(LigeroArg::String {
        str: format!("0x{}", hx32(&anchor)),
    });
    args.push(LigeroArg::String {
        str: format!("0x{}", hx32(&nf)),
    });

    args.push(LigeroArg::I64 {
        i64: withdraw_amount as i64,
    });
    args.push(LigeroArg::I64 { i64: n_out as i64 });

    for (out_value, out_rho, out_pk, cm_out) in out_triples {
        args.push(LigeroArg::I64 {
            i64: out_value as i64,
        });
        args.push(LigeroArg::Hex { hex: hx32(&out_rho) });
        args.push(LigeroArg::Hex { hex: hx32(&out_pk) });
        args.push(LigeroArg::Hex { hex: hx32(&cm_out) });
    }

    Ok((args, (anchor, nf, cm_outs, withdraw_amount, n_out)))
}

#[test]
fn test_note_spend_daemon_bench() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_spend_guest(&repo)?;

    let program = repo
        .join("utils/circuits/bins/note_spend_guest.wasm")
        .canonicalize()
        .context("Failed to canonicalize note_spend_guest.wasm")?;

    let packing = read_packing();
    let gzip_proof = read_gzip_proof(true);

    let mut runner = LigeroRunner::new(&program.to_string_lossy());
    runner.config_mut().packing = packing;
    runner.config_mut().gzip_proof = gzip_proof;

    if !runner.paths().prover_bin.exists() || !runner.paths().verifier_bin.exists() {
        eprintln!(
            "Skipping: Ligero binaries not found (prover={}, verifier={})",
            runner.paths().prover_bin.display(),
            runner.paths().verifier_bin.display()
        );
        return Ok(());
    }

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
        eprintln!("Skipping: shader path not found at {}", shader_dir.display());
        return Ok(());
    }

    println!("[daemon] Prover:   {}", runner.paths().prover_bin.display());
    println!("[daemon] Verifier: {}", runner.paths().verifier_bin.display());
    println!("[daemon] Shaders:  {}", shader_dir.display());
    println!("[daemon] Program:  {}", program.display());
    println!("[daemon] Packing:  {}", packing);
    println!("[daemon] Gzip:     {}", gzip_proof);

    let depth: usize = 8;
    let domain: Hash32 = [1u8; 32];
    let value: u64 = 42;
    let pos: u64 = 0;

    let mut configs: Vec<serde_json::Value> = Vec::new();
    let mut public_summaries: Vec<(Hash32, Hash32, Vec<Hash32>, u64, usize)> = Vec::new();

    for run in 0..3u8 {
        let (args, summary) = build_statement(run, depth, domain, value, pos)?;
        runner.config_mut().private_indices = private_indices(depth, 1);
        runner.config_mut().args = args;
        let cfg_val = serde_json::to_value(runner.config()).context("Failed to to_value config")?;
        configs.push(cfg_val);
        public_summaries.push(summary);
    }

    // Sanity: all public statements should differ.
    for i in 0..public_summaries.len() {
        for j in (i + 1)..public_summaries.len() {
            anyhow::ensure!(
                public_summaries[i] != public_summaries[j],
                "expected run #{} and run #{} to differ in public statement, but they were equal",
                i + 1,
                j + 1
            );
        }
    }

    // Warm-up (unmeasured).
    {
        let r = prover_pool
            .prove(configs[0].clone())
            .context("warm-up prove request failed")?;
        assert!(r.ok, "warm-up prove failed: {:?}", r.error);
        if let Some(p) = r.proof_path {
            let _ = std::fs::remove_file(&p);
            let _ = std::fs::remove_dir_all(
                std::path::Path::new(&p)
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new(".")),
            );
        }
    }

    // Prove 3 distinct proofs.
    let mut proof_paths: Vec<String> = Vec::new();
    for (i, cfg) in configs.iter().cloned().enumerate() {
        let t0 = Instant::now();
        let r = prover_pool.prove(cfg)?;
        let d = t0.elapsed();
        assert!(r.ok, "prover run #{i} failed: {:?}", r.error);
        let proof_path = r
            .proof_path
            .clone()
            .with_context(|| format!("prover run #{i} did not return proof_path"))?;
        let size = std::fs::metadata(&proof_path).map(|m| m.len()).unwrap_or(0);
        println!(
            "[daemon] Prover run #{}: {:?} ({} bytes) -> {}",
            i + 1,
            d,
            size,
            proof_path
        );
        proof_paths.push(proof_path);
    }

    // Warm-up verify (unmeasured).
    {
        let v = verifier_pool
            .verify(configs[0].clone(), &proof_paths[0])
            .context("warm-up verify request failed")?;
        assert!(v.ok, "warm-up verify failed: {:?}", v.error);
        assert_eq!(
            v.verify_ok,
            Some(true),
            "warm-up verify did not confirm validity (verify_ok={:?}, error={:?})",
            v.verify_ok,
            v.error
        );
    }

    // Assert configs differ.
    let mut cfg_hashes: Vec<[u8; 32]> = Vec::new();
    for (i, cfg) in configs.iter().enumerate() {
        let bytes = serde_json::to_vec(cfg)
            .with_context(|| format!("Failed to serialize config JSON for run #{}", i + 1))?;
        let mut h = Sha256::new();
        h.update(&bytes);
        let out: [u8; 32] = h.finalize().into();
        cfg_hashes.push(out);
    }
    for i in 0..cfg_hashes.len() {
        for j in (i + 1)..cfg_hashes.len() {
            anyhow::ensure!(
                cfg_hashes[i] != cfg_hashes[j],
                "expected configs to differ, but run #{} and run #{} had the same sha256={}",
                i + 1,
                j + 1,
                hex::encode(cfg_hashes[i])
            );
        }
    }

    // Assert proof bytes differ.
    let mut proof_hashes: Vec<[u8; 32]> = Vec::new();
    for (i, p) in proof_paths.iter().enumerate() {
        let bytes = std::fs::read(p)
            .with_context(|| format!("Failed to read proof bytes for run #{} at {}", i + 1, p))?;
        let mut h = Sha256::new();
        h.update(&bytes);
        let out: [u8; 32] = h.finalize().into();
        proof_hashes.push(out);
    }
    for i in 0..proof_hashes.len() {
        for j in (i + 1)..proof_hashes.len() {
            anyhow::ensure!(
                proof_hashes[i] != proof_hashes[j],
                "expected proof outputs to differ, but run #{} and run #{} had the same sha256={}",
                i + 1,
                j + 1,
                hex::encode(proof_hashes[i])
            );
        }
    }

    // Cross-verify must fail.
    for (pi, ci) in [(0usize, 1usize), (1, 2), (2, 0)] {
        let v = verifier_pool
            .verify(configs[ci].clone(), &proof_paths[pi])
            .with_context(|| format!("cross-verify proof#{} with cfg#{}", pi + 1, ci + 1))?;
        anyhow::ensure!(
            !(v.ok && v.verify_ok == Some(true)),
            "expected cross-verify to fail, but proof#{} verified with cfg#{} (this implies verifier is not binding to provided public inputs)",
            pi + 1,
            ci + 1
        );
    }

    // Verify matching pairs (timed).
    for (i, (cfg, proof_path)) in configs.iter().cloned().zip(proof_paths.iter()).enumerate() {
        let tv = Instant::now();
        let v = verifier_pool.verify(cfg, proof_path)?;
        let vd = tv.elapsed();
        assert!(v.ok, "verifier run #{i} failed: {:?}", v.error);
        assert_eq!(
            v.verify_ok,
            Some(true),
            "verifier run #{i} did not confirm validity (verify_ok={:?}, error={:?})",
            v.verify_ok,
            v.error
        );
        println!("[daemon] Verifier run #{}: {:?}", i + 1, vd);
    }

    Ok(())
}

#[test]
fn test_note_spend_direct_bench() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_spend_guest(&repo)?;

    let program = repo
        .join("utils/circuits/bins/note_spend_guest.wasm")
        .canonicalize()
        .context("Failed to canonicalize note_spend_guest.wasm")?;

    let packing = read_packing();
    let gzip_proof = read_gzip_proof(true);

    let mut runner = LigeroRunner::new(&program.to_string_lossy());
    runner.config_mut().packing = packing;
    runner.config_mut().gzip_proof = gzip_proof;

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

    println!("[direct] Prover:   {}", runner.paths().prover_bin.display());
    println!("[direct] Verifier: {}", runner.paths().verifier_bin.display());
    println!("[direct] Shaders:  {}", shader_dir.display());
    println!("[direct] Program:  {}", program.display());
    println!("[direct] Packing:  {}", packing);
    println!("[direct] Gzip:     {}", gzip_proof);

    let depth: usize = 8;
    let domain: Hash32 = [1u8; 32];
    let value: u64 = 42;
    let pos: u64 = 0;

    // Prove N distinct proofs (controlled by LIGERO_RUNS env var, default 3).
    let num_runs = read_num_runs();
    println!("[direct] Running {} proof(s)", num_runs);
    let mut proofs: Vec<Vec<u8>> = Vec::new();
    let mut configs: Vec<Vec<LigeroArg>> = Vec::new();
    let mut all_priv_idx: Vec<Vec<usize>> = Vec::new();

    for run in 0..num_runs {
        let (args, _summary) = build_statement(run, depth, domain, value, pos)?;
        let priv_idx = private_indices(depth, 1);
        runner.config_mut().private_indices = priv_idx.clone();
        runner.config_mut().args = args.clone();

        let t = Instant::now();
        let (proof, stdout, stderr) = runner.run_prover_with_output_in_pool(
            &prover_pool,
            ProverRunOptions {
                keep_proof_dir: false,
                proof_outputs_base: None,
                write_replay_script: true,
            },
        )?;
        let d = t.elapsed();
        println!(
            "[direct] Prover run #{}: {:?} ({} bytes)",
            run + 1,
            d,
            proof.len()
        );
        if is_verbose() {
            println!("--- Prover stdout (run #{}) ---", run + 1);
            print!("{}", stdout);
            if !stderr.is_empty() {
                println!("--- Prover stderr (run #{}) ---", run + 1);
                print!("{}", stderr);
            }
            println!("--- End prover output ---");
        }
        proofs.push(proof);
        configs.push(args);
        all_priv_idx.push(priv_idx);
    }

    // Verify the 3 proofs (timed).
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
        let (ok, vs, ve) = verifier::verify_proof_with_output_in_pool(
            &verifier_pool,
            &vpaths,
            proof,
            args.clone(),
            priv_idx.clone(),
        )?;
        let vd = tv.elapsed();
        assert!(ok, "verifier should report success for run #{}", i + 1);
        println!("[direct] Verifier run #{}: {:?}", i + 1, vd);
        if is_verbose() {
            println!("--- Verifier stdout (run #{}) ---", i + 1);
            print!("{}", vs);
            if !ve.is_empty() {
                println!("--- Verifier stderr (run #{}) ---", i + 1);
                print!("{}", ve);
            }
            println!("--- End verifier output ---");
        }
    }

    // Cross-verify must fail (only when we have 2+ proofs).
    if proofs.len() >= 2 {
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
    }

    Ok(())
}


