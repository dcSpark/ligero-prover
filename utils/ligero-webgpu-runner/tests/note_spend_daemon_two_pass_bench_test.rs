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
    // Indices are 1-based for the guest:
    // args:
    // 0: domain
    // 1: value
    // 2: rho
    // 3: recipient
    // 4: spend_sk                    (private)
    // 5: depth
    // 6..(6+depth-1): path bits
    // then siblings (depth items)
    // then anchor, nf
    // then withdraw_amount, n_out
    // then outputs: value, rho, pk, cm (4*n_out)
    //
    // For this test we mark spend_sk and output secret rho as private.
    let mut idx = Vec::new();
    idx.push(5); // spend_sk (argv[5] because argv[0] is program name in C++)

    // out_rho starts after:
    // domain,value,rho,recipient,spend_sk,depth => 6
    // path bits => depth
    // siblings => depth
    // anchor,nf => 2
    // withdraw_amount,n_out,out_value => 3
    // out_rho is next => 1
    let out_rho_pos = 1 + (6 + depth + depth + 2 + 3) as usize; // 1-based argv index
    // Actually argv indices are 1-based but include program name at argv[0], so:
    // In LigeroConfig private-indices expects 1-based indices for the args array (excluding argv[0]).
    // This helper matches prior tests' convention.
    // We'll just reuse the same indices generator as the non-daemon benchmark:
    // spend_sk and out_rho.
    idx.push(out_rho_pos);

    // If multiple outputs exist, mark each out_rho private too.
    // outputs are 4 fields each: value,rho,pk,cm
    for o in 1..n_out {
        idx.push(out_rho_pos + o * 4);
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

    // Build one-output spend witness.
    let depth: usize = 8;
    let domain: Hash32 = [1u8; 32];
    let rho: Hash32 = [2u8; 32];
    let spend_sk: Hash32 = [4u8; 32];

    let recipient = recipient_from_sk(&domain, &spend_sk);
    let nf_key = nf_key_from_sk(&domain, &spend_sk);

    let value: u64 = 42;
    let pos: u64 = 0;

    let cm_in = note_commitment(&domain, value, &rho, &recipient);
    let mut tree = MerkleTree::new(depth);
    tree.set_leaf(pos as usize, cm_in);
    let anchor = tree.root();
    let siblings = tree.open(pos as usize);
    let nf = nullifier(&domain, &nf_key, &rho);

    let withdraw_amount: u64 = 0;
    let n_out: u64 = 1;

    let out_value: u64 = value;
    let out_rho: Hash32 = [7u8; 32];
    let out_spend_sk: Hash32 = [8u8; 32];
    let out_pk = pk_from_sk(&out_spend_sk);
    let out_recipient = recipient_from_pk(&domain, &out_pk);
    let cm_out = note_commitment(&domain, out_value, &out_rho, &out_recipient);

    let mut args: Vec<LigeroArg> = Vec::new();
    args.push(LigeroArg::Hex { hex: hx32(&domain) });
    args.push(LigeroArg::I64 { i64: value as i64 });
    args.push(LigeroArg::Hex { hex: hx32(&rho) });
    args.push(LigeroArg::Hex {
        hex: hx32(&recipient),
    });
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
    args.push(LigeroArg::I64 { i64: out_value as i64 });
    args.push(LigeroArg::Hex { hex: hx32(&out_rho) });
    args.push(LigeroArg::Hex { hex: hx32(&out_pk) });
    args.push(LigeroArg::Hex { hex: hx32(&cm_out) });

    let priv_idx = private_indices(depth, 1);
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

    let size1 = std::fs::metadata(&proof_path_1).map(|m| m.len()).unwrap_or(0);
    let size2 = std::fs::metadata(&proof_path_2).map(|m| m.len()).unwrap_or(0);

    println!("[daemon_two_pass] Prover pass1: {:?} ({} bytes) -> {}", d1, size1, proof_path_1);
    println!("[daemon_two_pass] Prover pass2: {:?} ({} bytes) -> {}", d2, size2, proof_path_2);

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


