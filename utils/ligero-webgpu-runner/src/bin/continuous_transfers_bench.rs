//! Minimal throughput benchmark for "Midnight privacy transfers" (note-spend circuit) without Sovereign SDK.
//!
//! What it does (per round):
//! - Build a Merkle tree of N input notes (one per wallet)
//! - Generate N note-spend proofs in parallel (daemon mode)
//! - Verify N note-spend proofs in parallel (daemon mode)
//! - Print total and per-proof timings for prove and verify
//!
//! Configuration (environment variables):
//! - TRANSFER_N: number of proofs per round (default: 16) (fallback if stdin prompt is skipped)
//! - TRANSFER_ROUNDS: number of rounds (default: 1) (fallback if stdin prompt is skipped)
//! - TRANSFER_TREE_DEPTH: Merkle tree depth (default: 8)
//! - LIGERO_PACKING: packing (default: 8192)
//! - LIGERO_GZIP_PROOF: whether to gzip proof files (default: false)
//! - LIGERO_PROVER_WORKERS: default value for interactive prover concurrency (default: 1)
//! - LIGERO_VERIFIER_WORKERS: default value for interactive verifier concurrency (default: 1)

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use ligetron::{poseidon2_hash_bytes, Bn254Fr};
use rand::RngCore;

use ligero_runner::{daemon::DaemonPool, LigeroArg, LigeroConfig, LigeroPaths};

type Hash32 = [u8; 32];

// Blacklist constants (must match circuit)
const BL_BUCKET_SIZE: usize = 12;
const BL_DEPTH: usize = 16;

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_u32(name: &str, default: u32) -> u32 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_bool(name: &str, default: bool) -> bool {
    std::env::var(name)
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(default)
}

fn prompt_usize(prompt: &str, default: usize) -> Result<usize> {
    use std::io::{self, Write};
    eprint!("{prompt} [{default}]: ");
    io::stdout().flush().ok();
    let mut line = String::new();
    io::stdin().read_line(&mut line).ok();
    let s = line.trim();
    if s.is_empty() {
        return Ok(default);
    }
    s.parse::<usize>()
        .with_context(|| format!("invalid number: '{s}'"))
}

fn repo_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    Ok(manifest_dir
        .ancestors()
        .nth(2)
        .context("Failed to compute ligero-prover repo root")?
        .to_path_buf())
}

fn maybe_build_note_spend_guest(repo: &Path) -> Result<PathBuf> {
    let out = repo.join("utils/circuits/bins/note_spend_guest.wasm");
    if out.exists() {
        return Ok(out);
    }

    let guest_dir = repo.join("utils/circuits/note-spend");
    anyhow::ensure!(
        guest_dir.exists(),
        "note-spend sources not found at {}",
        guest_dir.display()
    );

    eprintln!(
        "[bench] note_spend_guest.wasm not found; building via {}",
        guest_dir.join("build.sh").display()
    );

    let status = Command::new("bash")
        .arg("build.sh")
        .current_dir(&guest_dir)
        .status()
        .context("Failed to run note-spend/build.sh")?;
    anyhow::ensure!(status.success(), "note-spend/build.sh failed: {status}");

    Ok(out)
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

fn recipient_from_sk(domain: &Hash32, spend_sk: &Hash32) -> Hash32 {
    let pk_spend = pk_from_sk(spend_sk);
    let pk_ivk = ivk_seed(domain, spend_sk);
    recipient_from_pk(domain, &pk_spend, &pk_ivk)
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
) -> Hash32 {
    let mut prod = Bn254Fr::from_u32(1);
    for &v in in_values {
        prod.mulmod_checked(&Bn254Fr::from_u64(v));
    }
    for &v in out_values {
        prod.mulmod_checked(&Bn254Fr::from_u64(v));
    }
    for out_rho in out_rhos {
        let out_fr = fr_from_hash32_be(out_rho);
        for in_rho in in_rhos {
            let in_fr = fr_from_hash32_be(in_rho);
            let mut delta = out_fr.clone();
            delta.submod_checked(&in_fr);
            prod.mulmod_checked(&delta);
        }
    }
    if out_rhos.len() == 2 {
        let out0 = fr_from_hash32_be(&out_rhos[0]);
        let out1 = fr_from_hash32_be(&out_rhos[1]);
        let mut delta = out0.clone();
        delta.submod_checked(&out1);
        prod.mulmod_checked(&delta);
    }
    assert!(!prod.is_zero(), "inv_enforce undefined");
    let mut inv = prod.clone();
    inv.inverse();
    inv.to_bytes_be()
}

// --- Blacklist helpers ---

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

fn merkle_default_nodes_from_leaf(depth: usize, leaf0: &Hash32) -> Vec<Hash32> {
    let mut out = Vec::with_capacity(depth + 1);
    out.push(*leaf0);
    for lvl in 0..depth {
        let prev = out[lvl];
        out.push(mt_combine(lvl as u8, &prev, &prev));
    }
    out
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

fn hx32(b: &Hash32) -> String {
    hex::encode(b)
}

/// Incremental merkle tree storing all levels so `open()` is O(depth) and `root()` is O(1).
struct MerkleTree {
    depth: usize,
    levels: Vec<Vec<Hash32>>, // level 0 = leaves, level depth = root (len=1)
}

impl MerkleTree {
    fn new(depth: usize) -> Self {
        let size = 1usize << depth;
        let mut levels = Vec::with_capacity(depth + 1);
        levels.push(vec![[0u8; 32]; size]);
        for lvl in 0..depth {
            let prev = levels[lvl].len();
            levels.push(vec![[0u8; 32]; prev / 2]);
        }
        let mut t = Self { depth, levels };
        t.recompute_all();
        t
    }

    fn recompute_all(&mut self) {
        for lvl in 0..self.depth {
            for i in 0..self.levels[lvl + 1].len() {
                let left = self.levels[lvl][2 * i];
                let right = self.levels[lvl][2 * i + 1];
                self.levels[lvl + 1][i] = mt_combine(lvl as u8, &left, &right);
            }
        }
    }

    fn set_leaf(&mut self, pos: usize, leaf: Hash32) {
        self.levels[0][pos] = leaf;
        let mut idx = pos;
        for lvl in 0..self.depth {
            let parent = idx / 2;
            let left = self.levels[lvl][parent * 2];
            let right = self.levels[lvl][parent * 2 + 1];
            self.levels[lvl + 1][parent] = mt_combine(lvl as u8, &left, &right);
            idx = parent;
        }
    }

    fn root(&self) -> Hash32 {
        self.levels[self.depth][0]
    }

    fn open(&self, pos: usize) -> Vec<Hash32> {
        let mut siblings = Vec::with_capacity(self.depth);
        let mut idx = pos;
        for lvl in 0..self.depth {
            let sib_idx = if (idx & 1) == 0 { idx + 1 } else { idx - 1 };
            siblings.push(self.levels[lvl][sib_idx]);
            idx >>= 1;
        }
        siblings
    }
}

fn private_indices(depth: usize, n_in: usize, n_out: usize, is_transfer: bool) -> Vec<usize> {
    // 1-based indices into args list matching V2 protocol.
    let mut idx = vec![2usize, 3usize]; // spend_sk, pk_ivk
    let per_in = 5usize + depth;
    for i in 0..n_in {
        let base = 7 + i * per_in;
        idx.push(base);     // value_in
        idx.push(base + 1); // rho_in
        idx.push(base + 2); // sender_id_in
        idx.push(base + 3); // pos
        for k in 0..depth {
            idx.push(base + 4 + k); // siblings
        }
    }
    let withdraw_idx = 7 + n_in * per_in;
    let outs_base = withdraw_idx + 3;
    for j in 0..n_out {
        idx.push(outs_base + 5 * j);     // value_out
        idx.push(outs_base + 5 * j + 1); // rho_out
        idx.push(outs_base + 5 * j + 2); // pk_spend_out
        idx.push(outs_base + 5 * j + 3); // pk_ivk_out
    }
    let inv_enforce_idx = outs_base + 5 * n_out;
    idx.push(inv_enforce_idx);
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

#[derive(Clone)]
struct WalletNote {
    value: u64,
    rho: Hash32,
    spend_sk: Hash32,
    sender_id: Hash32,
}

#[derive(Clone)]
struct TransferWitness {
    wallet_idx: usize,
    args: Vec<LigeroArg>,
    proof_path: PathBuf,
    new_note: WalletNote,
    new_cm: Hash32,
}

fn build_transfer_witness(
    depth: usize,
    domain: Hash32,
    anchor: Hash32,
    tree: &MerkleTree,
    pos: usize,
    note: &WalletNote,
    out_value: u64,
    proof_path: PathBuf,
    rng: &mut impl RngCore,
    blacklist_root: &Hash32,
    bl_bucket_entries: &[Hash32; BL_BUCKET_SIZE],
    bl_siblings: &[Hash32],
) -> TransferWitness {
    let pk_ivk_owner = ivk_seed(&domain, &note.spend_sk);
    let recipient_owner = recipient_from_sk(&domain, &note.spend_sk);
    let sender_id_current = recipient_owner; // for transfer, sender is self

    let siblings = tree.open(pos);
    let nf_key = nf_key_from_sk(&domain, &note.spend_sk);
    let nf = nullifier(&domain, &nf_key, &note.rho);

    // Output note secrets
    let mut out_rho = [0u8; 32];
    rng.fill_bytes(&mut out_rho);
    let mut out_spend_sk = [0u8; 32];
    rng.fill_bytes(&mut out_spend_sk);
    let out_pk_spend = pk_from_sk(&out_spend_sk);
    let out_pk_ivk = ivk_seed(&domain, &out_spend_sk);
    let out_recipient = recipient_from_pk(&domain, &out_pk_spend, &out_pk_ivk);
    let cm_out = note_commitment(&domain, out_value, &out_rho, &out_recipient, &sender_id_current);

    // Compute inv_enforce
    let inv_enforce = compute_inv_enforce(
        &[note.value],
        &[note.rho],
        &[out_value],
        &[out_rho],
    );

    // V2 argument layout
    let mut args: Vec<LigeroArg> = Vec::new();
    // 1: domain
    args.push(LigeroArg::Hex { hex: hx32(&domain) });
    // 2: spend_sk (private)
    args.push(LigeroArg::Hex { hex: hx32(&note.spend_sk) });
    // 3: pk_ivk_owner (private)
    args.push(LigeroArg::Hex { hex: hx32(&pk_ivk_owner) });
    // 4: depth
    args.push(LigeroArg::I64 { i64: depth as i64 });
    // 5: anchor (public)
    args.push(LigeroArg::Hex { hex: hx32(&anchor) });
    // 6: n_in
    args.push(LigeroArg::I64 { i64: 1 });

    // Input note 0
    // 7: value_in (private)
    args.push(LigeroArg::I64 { i64: note.value as i64 });
    // 8: rho_in (private)
    args.push(LigeroArg::Hex { hex: hx32(&note.rho) });
    // 9: sender_id_in (private)
    args.push(LigeroArg::Hex { hex: hx32(&note.sender_id) });
    // 10: pos (private)
    args.push(LigeroArg::I64 { i64: pos as i64 });
    // 11..(11+depth): siblings (private)
    for s in &siblings {
        args.push(LigeroArg::Hex { hex: hx32(s) });
    }
    // nullifier (public)
    args.push(LigeroArg::Hex { hex: hx32(&nf) });

    // withdraw_amount, withdraw_recipient (public)
    args.push(LigeroArg::I64 { i64: 0 });
    args.push(LigeroArg::Hex { hex: hx32(&[0u8; 32]) });

    // n_out
    args.push(LigeroArg::I64 { i64: 1 });

    // Output note 0
    // value_out (private)
    args.push(LigeroArg::I64 { i64: out_value as i64 });
    // rho_out (private)
    args.push(LigeroArg::Hex { hex: hx32(&out_rho) });
    // pk_spend_out (private)
    args.push(LigeroArg::Hex { hex: hx32(&out_pk_spend) });
    // pk_ivk_out (private)
    args.push(LigeroArg::Hex { hex: hx32(&out_pk_ivk) });
    // cm_out (public)
    args.push(LigeroArg::Hex { hex: hx32(&cm_out) });

    // inv_enforce (private)
    args.push(LigeroArg::Hex { hex: hx32(&inv_enforce) });

    // blacklist_root (public)
    args.push(LigeroArg::Hex { hex: hx32(blacklist_root) });

    // Blacklist checks for transfer: sender_id_current, out_recipient
    for id in [sender_id_current, out_recipient] {
        // bucket entries (private)
        for e in bl_bucket_entries.iter() {
            args.push(LigeroArg::Hex { hex: hx32(e) });
        }
        // bucket inv (private)
        let inv = bl_bucket_inv_for_id(&id, bl_bucket_entries)
            .expect("unexpected: id collides with empty blacklist bucket");
        args.push(LigeroArg::Hex { hex: hx32(&inv) });
        // blacklist siblings (private)
        for sib in bl_siblings.iter() {
            args.push(LigeroArg::Hex { hex: hx32(sib) });
        }
    }

    TransferWitness {
        wallet_idx: pos,
        args,
        proof_path,
        new_note: WalletNote {
            value: out_value,
            rho: out_rho,
            spend_sk: out_spend_sk,
            sender_id: sender_id_current,
        },
        new_cm: cm_out,
    }
}

fn run_parallel<T, R>(
    items: Vec<T>,
    threads: usize,
    f: impl Fn(T) -> Result<R> + Send + Sync + 'static,
) -> Result<Vec<R>>
where
    T: Clone + Send + Sync + 'static,
    R: Send + 'static,
{
    let threads = threads.max(1);
    let n = items.len();
    let idx = Arc::new(AtomicUsize::new(0));
    let items = Arc::new(items);
    let out: Arc<Mutex<Vec<Option<R>>>> = Arc::new(Mutex::new({
        let mut v = Vec::with_capacity(n);
        v.resize_with(n, || None);
        v
    }));
    let f = Arc::new(f);

    let mut joins = Vec::with_capacity(threads);
    for _ in 0..threads {
        let idx = idx.clone();
        let items = items.clone();
        let out = out.clone();
        let f = f.clone();
        joins.push(std::thread::spawn(move || -> Result<()> {
            loop {
                let i = idx.fetch_add(1, Ordering::Relaxed);
                if i >= items.len() {
                    break;
                }
                let r = f(items[i].clone())?;
                out.lock().unwrap()[i] = Some(r);
            }
            Ok(())
        }));
    }

    for j in joins {
        j.join().map_err(|_| anyhow::anyhow!("thread panicked"))??;
    }

    let mut locked = out.lock().unwrap();
    let mut res = Vec::with_capacity(n);
    for (i, v) in locked.drain(..).enumerate() {
        res.push(v.with_context(|| format!("missing result at index {i}"))?);
    }
    Ok(res)
}

fn main() -> Result<()> {
    let n_default = env_usize("TRANSFER_N", 16);
    let rounds_default = env_usize("TRANSFER_ROUNDS", 1);
    let n = prompt_usize("Transfers per round", n_default)?;
    let rounds = prompt_usize("Number of rounds (0 = run indefinitely)", rounds_default)?;
    let depth = env_usize("TRANSFER_TREE_DEPTH", 8);
    let packing = env_u32("LIGERO_PACKING", 8192);
    let gzip_proof = env_bool("LIGERO_GZIP_PROOF", false);

    // Concurrency knobs: the entered value controls BOTH
    // - number of daemon processes (workers)
    // - number of concurrent requests we issue (parallelism)
    //
    // This makes "workers vs parallelism" impossible to misconfigure from the keyboard.
    let prover_default = env_usize("LIGERO_PROVER_WORKERS", 1);
    let verifier_default = env_usize("LIGERO_VERIFIER_WORKERS", 1);
    let prover_concurrency = prompt_usize("Prover concurrency", prover_default)?;
    let verifier_concurrency = prompt_usize("Verifier concurrency", verifier_default)?;
    anyhow::ensure!(prover_concurrency > 0, "Prover concurrency must be > 0");
    anyhow::ensure!(verifier_concurrency > 0, "Verifier concurrency must be > 0");

    let prover_workers = prover_concurrency;
    let verifier_workers = verifier_concurrency;
    let prove_par = prover_concurrency;
    let verify_par = verifier_concurrency;

    anyhow::ensure!(n > 0, "TRANSFER_N must be > 0");
    anyhow::ensure!(depth >= 1, "TRANSFER_TREE_DEPTH must be >= 1");
    anyhow::ensure!(n <= (1usize << depth), "TRANSFER_N must be <= 2^depth");

    let repo = repo_root()?;
    let program = maybe_build_note_spend_guest(&repo)?;

    let paths =
        LigeroPaths::discover().or_else(|_| Ok::<_, anyhow::Error>(LigeroPaths::fallback()))?;
    let shader_path = paths.shader_dir.to_string_lossy().to_string();

    eprintln!(
        "[bench] n={} rounds={} depth={} packing={} gzip_proof={} prover_workers={} verifier_workers={} prove_par={} verify_par={}",
        n, rounds, depth, packing, gzip_proof, prover_workers, verifier_workers, prove_par, verify_par
    );
    eprintln!(
        "[bench] prover_bin={} verifier_bin={} program={} shader_dir={}",
        paths.prover_bin.display(),
        paths.verifier_bin.display(),
        program.display(),
        paths.shader_dir.display()
    );

    let prover_pool = DaemonPool::new_prover(&paths, prover_workers)
        .context("failed to start prover daemon pool")?;
    let verifier_pool = DaemonPool::new_verifier(&paths, verifier_workers)
        .context("failed to start verifier daemon pool")?;

    // Keep proof files for the entire run under one temp dir.
    let proof_dir = tempfile::tempdir().context("failed to create proof output dir")?;
    eprintln!("[bench] proof_dir={}", proof_dir.path().display());

    // Domain used by the guest/circuit.
    let domain: Hash32 = [1u8; 32];

    // Initialize blacklist data (empty blacklist).
    let bl_bucket_entries = bl_empty_bucket_entries();
    let leaf0 = bl_bucket_leaf(&bl_bucket_entries);
    let bl_default_nodes = merkle_default_nodes_from_leaf(BL_DEPTH, &leaf0);
    let blacklist_root = bl_default_nodes[BL_DEPTH];
    let bl_siblings: Vec<Hash32> = bl_default_nodes.iter().take(BL_DEPTH).copied().collect();

    // Initialize N wallet notes and build the initial tree.
    let mut rng = rand::thread_rng();
    let value_in: u64 = 100;
    let mut wallets: Vec<WalletNote> = Vec::with_capacity(n);
    let mut tree = MerkleTree::new(depth);
    for i in 0..n {
        let mut rho = [0u8; 32];
        rng.fill_bytes(&mut rho);
        let mut spend_sk = [0u8; 32];
        rng.fill_bytes(&mut spend_sk);
        let recipient = recipient_from_sk(&domain, &spend_sk);
        // Initial sender_id is random (simulates deposit)
        let mut sender_id = [0u8; 32];
        rng.fill_bytes(&mut sender_id);
        let cm_in = note_commitment(&domain, value_in, &rho, &recipient, &sender_id);
        tree.set_leaf(i, cm_in);
        wallets.push(WalletNote {
            value: value_in,
            rho,
            spend_sk,
            sender_id,
        });
    }

    // Benchmark rounds.
    let mut round: usize = 0;
    loop {
        if rounds != 0 && round >= rounds {
            break;
        }
        eprintln!("\n[round {}/{}] building witnesses", round + 1, rounds);
        let anchor = tree.root();

        // Build all witnesses for this round (sequential, lightweight).
        let mut witnesses: Vec<TransferWitness> = Vec::with_capacity(n);
        for i in 0..n {
            let proof_name = if gzip_proof {
                format!("proof_r{round}_i{i}.gz")
            } else {
                format!("proof_r{round}_i{i}.bin")
            };
            let proof_path = proof_dir.path().join(proof_name);
            let out_value = wallets[i].value; // pure transfer: no withdraw
            let w = build_transfer_witness(
                depth,
                domain,
                anchor,
                &tree,
                i,
                &wallets[i],
                out_value,
                proof_path,
                &mut rng,
                &blacklist_root,
                &bl_bucket_entries,
                &bl_siblings,
            );
            witnesses.push(w);
        }

        let priv_idx = private_indices(depth, 1, 1, true);
        let program_s = program.to_string_lossy().to_string();

        eprintln!("[round {}/{}] proving {} proofs...", round + 1, rounds, n);
        let prove_start = Instant::now();
        let prove_results = run_parallel(witnesses, prove_par, {
            let prover_pool = prover_pool.clone();
            let shader_path = shader_path.clone();
            let program_s = program_s.clone();
            let priv_idx = priv_idx.clone();
            move |w: TransferWitness| -> Result<TransferWitness> {
                let cfg = LigeroConfig {
                    program: program_s.clone(),
                    shader_path: shader_path.clone(),
                    gpu_threads: None,
                    packing,
                    gzip_proof,
                    proof_path: Some(w.proof_path.to_string_lossy().to_string()),
                    private_indices: priv_idx.clone(),
                    args: w.args.clone(),
                };
                let cfg_json = serde_json::to_value(&cfg)?;
                let resp = prover_pool
                    .prove(cfg_json)
                    .context("daemon prove request failed")?;
                if !resp.ok {
                    anyhow::bail!(
                        "prover daemon returned ok=false (exit_code={:?}): {}",
                        resp.exit_code,
                        resp.error.unwrap_or_else(|| "unknown error".to_string())
                    );
                }
                // Proof is written to our proof_path.
                anyhow::ensure!(
                    w.proof_path.exists(),
                    "expected proof file not found at {}",
                    w.proof_path.display()
                );
                Ok(w)
            }
        })?;
        let prove_elapsed = prove_start.elapsed();

        let prove_ms = prove_elapsed.as_secs_f64() * 1000.0;
        eprintln!(
            "[round {}/{}] prove_total_ms={:.2} prove_per_ms={:.2} throughput={:.2} proofs/s",
            round + 1,
            rounds,
            prove_ms,
            prove_ms / n as f64,
            (n as f64) / prove_elapsed.as_secs_f64()
        );

        eprintln!("[round {}/{}] verifying {} proofs...", round + 1, rounds, n);
        let verify_start = Instant::now();
        let verify_ok = run_parallel(prove_results.clone(), verify_par, {
            let verifier_pool = verifier_pool.clone();
            let shader_path = shader_path.clone();
            let program_s = program_s.clone();
            let priv_idx = priv_idx.clone();
            move |w: TransferWitness| -> Result<()> {
                let cfg = LigeroConfig {
                    program: program_s.clone(),
                    shader_path: shader_path.clone(),
                    gpu_threads: None,
                    packing,
                    gzip_proof,
                    proof_path: None,
                    private_indices: priv_idx.clone(),
                    args: w.args.clone(),
                };
                let cfg_json = serde_json::to_value(&cfg)?;
                let proof_path_s = w.proof_path.to_string_lossy().to_string();
                let resp = verifier_pool
                    .verify(cfg_json, &proof_path_s)
                    .context("daemon verify request failed")?;
                if !resp.ok {
                    anyhow::bail!(
                        "verifier daemon returned ok=false (exit_code={:?}): {}",
                        resp.exit_code,
                        resp.error.unwrap_or_else(|| "unknown error".to_string())
                    );
                }
                if let Some(false) = resp.verify_ok {
                    anyhow::bail!("verifier reported verify_ok=false");
                }
                Ok(())
            }
        })?;
        drop(verify_ok);
        let verify_elapsed = verify_start.elapsed();

        let verify_ms = verify_elapsed.as_secs_f64() * 1000.0;
        eprintln!(
            "[round {}/{}] verify_total_ms={:.2} verify_per_ms={:.2} throughput={:.2} proofs/s",
            round + 1,
            rounds,
            verify_ms,
            verify_ms / n as f64,
            (n as f64) / verify_elapsed.as_secs_f64()
        );

        // Update wallet state + tree to the newly created output notes (so the next round is realistic).
        for w in prove_results {
            wallets[w.wallet_idx] = w.new_note;
            tree.set_leaf(w.wallet_idx, w.new_cm);
        }

        // Small pacing to make logs readable when running many rounds.
        std::thread::sleep(Duration::from_millis(10));

        round += 1;
    }

    Ok(())
}
