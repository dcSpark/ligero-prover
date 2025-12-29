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
use sha2::{Digest, Sha256};

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
    // Public (must be bound by the proof & verifier):
    // - domain (1), value (2), rho (3), depth (6)
    // - anchor (7+2*depth), nullifier (8+2*depth)
    // - withdraw_amount (9+2*depth), n_out (10+2*depth)
    // - cm_out_j for each output
    //
    // Private (witness; verifier must NOT rely on config-provided values):
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

fn maybe_build_note_spend_guest(repo: &PathBuf) -> Result<()> {
    let wasm = repo.join("utils/circuits/bins/programs/note_spend_guest.wasm");
    if wasm.exists() {
        return Ok(());
    }

    let build_sh = repo.join("utils/circuits/note-spend-guest/build.sh");
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
        .context("Failed to run note-spend-guest/build.sh")?;

    if !status.success() {
        anyhow::bail!("note-spend-guest/build.sh failed with status {status}");
    }

    Ok(())
}

#[test]
fn test_two_pass_daemon_prover_and_verifier_timings() -> Result<()> {
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
            eprintln!(
                "Skipping: prover binary does not appear to support --daemon yet: {e:#}"
            );
            return Ok(());
        }
    };
    let verifier_pool = match DaemonPool::new_verifier(runner.paths(), 1) {
        Ok(p) => p,
        Err(e) => {
            eprintln!(
                "Skipping: verifier binary does not appear to support --daemon yet: {e:#}"
            );
            return Ok(());
        }
    };

    let shader_dir = PathBuf::from(&runner.config().shader_path);
    if !shader_dir.exists() {
        eprintln!("Skipping: shader path not found at {}", shader_dir.display());
        return Ok(());
    }

    println!("[daemon_two_pass] Prover:   {}", runner.paths().prover_bin.display());
    println!(
        "[daemon_two_pass] Verifier: {}",
        runner.paths().verifier_bin.display()
    );
    println!("[daemon_two_pass] Shaders:  {}", shader_dir.display());
    println!("[daemon_two_pass] Program:  {}", program.display());
    println!("[daemon_two_pass] Packing:  {}", packing);

    // Build 3 distinct spend statements (with PUBLIC differences) so that cross-verification
    // (proof_i verified with params_j for i!=j) must fail.
    // We keep daemon pools at 1 worker so every request hits the same long-lived process.
    let depth: usize = 8;
    let domain: Hash32 = [1u8; 32];
    let value: u64 = 42;
    let pos: u64 = 0;

    let mut configs: Vec<serde_json::Value> = Vec::new();
    let mut public_summaries: Vec<(Hash32, Hash32, Vec<Hash32>, u64, usize)> = Vec::new();

    for run in 0..3u8 {
        // Vary public rho (arg[3]) so the statement differs even if you ignore private witness.
        let mut rho: Hash32 = [2u8; 32];
        rho[0] = rho[0].wrapping_add(run);

        // Vary private spend_sk (arg[5]) so witness differs too.
        let mut spend_sk: Hash32 = [4u8; 32];
        spend_sk[0] = spend_sk[0].wrapping_add(run);

        // Decide public shape per run:
        // - run0: n_out=1, withdraw=0, out_value=value
        // - run1: n_out=1, withdraw=1, out_value=value-1
        // - run2: n_out=2, withdraw=0, out_values split
        let (withdraw_amount, n_out, out_values): (u64, usize, Vec<u64>) = match run {
            0 => (0, 1, vec![value]),
            1 => (1, 1, vec![value - 1]),
            _ => (0, 2, vec![value / 2, value - (value / 2)]),
        };

        // Build Merkle membership (siblings are private in the guest, but the anchor is public).
        let recipient = recipient_from_sk(&domain, &spend_sk);
        let nf_key = nf_key_from_sk(&domain, &spend_sk);

        let cm_in = note_commitment(&domain, value, &rho, &recipient);
        let mut tree = MerkleTree::new(depth);
        tree.set_leaf(pos as usize, cm_in);
        let anchor = tree.root();
        let siblings = tree.open(pos as usize);
        let nf = nullifier(&domain, &nf_key, &rho);

        // Outputs: for each output, value/rho/pk are private; cm_out is public.
        let mut cm_outs: Vec<Hash32> = Vec::with_capacity(n_out);
        let mut out_triples: Vec<(u64, Hash32, Hash32, Hash32)> = Vec::with_capacity(n_out);
        for j in 0..n_out {
            let out_value = out_values[j];
            let mut out_rho: Hash32 = [7u8; 32];
            out_rho[0] = out_rho[0].wrapping_add(run).wrapping_add(j as u8);
            let mut out_spend_sk: Hash32 = [8u8; 32];
            out_spend_sk[0] = out_spend_sk[0].wrapping_add(run).wrapping_add(j as u8);
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

        runner.config_mut().private_indices = private_indices(depth, n_out);
        runner.config_mut().args = args;
        let cfg_val = serde_json::to_value(runner.config()).context("Failed to to_value config")?;
        configs.push(cfg_val);

        public_summaries.push((anchor, nf, cm_outs, withdraw_amount, n_out));
    }

    // Sanity: all public statements should differ (anchor/nullifier or cm_outs/shape).
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

    // --- Prover: generate 3 distinct proofs ---
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
            "[daemon_two_pass] Prover run #{}: {:?} ({} bytes) -> {}",
            i + 1,
            d,
            size,
            proof_path
        );
        proof_paths.push(proof_path);
    }

    // Assert the *inputs* differ across runs (not just file names).
    // Note: many witness values are private, and Ligero verifier may not bind to caller-provided
    // args the way you'd expect (it may verify "some witness exists" without re-checking the
    // statement against a separately-provided input). So we check:
    // - config JSON differs (we did change the requested witness/statement)
    // - proof bytes differ (prover output actually changed)
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

    // Assert the proof *contents* differ across runs.
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

    // --- Verifier: verify the 3 different proofs (matching configs) ---
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
        println!("[daemon_two_pass] Verifier run #{}: {:?}", i + 1, vd);
    }

    // --- Negative: cross-verify must FAIL (public statements differ) ---
    for i in 0..proof_paths.len() {
        for j in 0..configs.len() {
            if i == j {
                continue;
            }
            // Important: a failing guest path may terminate the verifier process (e.g. via WASI exit),
            // which would close daemon stdout and take the worker down. For the negative checks we
            // therefore use a fresh single-worker verifier daemon each time and treat "stdout closed"
            // as an expected failure signal too.
            let verifier_pool_neg = DaemonPool::new_verifier(runner.paths(), 1)
                .context("spawn verifier daemon for negative cross-check")?;
            match verifier_pool_neg.verify(configs[j].clone(), &proof_paths[i]) {
                Ok(v) => {
                    anyhow::ensure!(
                        !(v.ok && v.verify_ok == Some(true)),
                        "expected cross-verify to fail, but proof#{} verified with cfg#{} (this implies verifier is not binding to provided public inputs)",
                        i + 1,
                        j + 1
                    );
                }
                Err(e) => {
                    let s = format!("{e:#}");
                    if s.contains("daemon stdout closed unexpectedly") {
                        // Expected: verifier process exited on invalid witness/statement.
                    } else {
                        return Err(e).with_context(|| {
                            format!("cross-verify proof#{} with cfg#{}", i + 1, j + 1)
                        });
                    }
                }
            }
        }
    }

    Ok(())
}


