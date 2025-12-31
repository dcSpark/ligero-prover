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
//! - TRANSFER_TREE_DEPTH: Merkle tree depth (default: 16)
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
use ligetron::poseidon2_hash_bytes;
use rand::RngCore;

use ligero_runner::{daemon::DaemonPool, LigeroArg, LigeroConfig, LigeroPaths};

type Hash32 = [u8; 32];

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
    let out = repo.join("utils/circuits/bins/programs/note_spend_guest.wasm");
    if out.exists() {
        return Ok(out);
    }

    let guest_dir = repo.join("utils/circuits/note-spend-guest");
    anyhow::ensure!(
        guest_dir.exists(),
        "note-spend-guest sources not found at {}",
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
        .context("Failed to run note-spend-guest/build.sh")?;
    anyhow::ensure!(status.success(), "note-spend-guest/build.sh failed: {status}");

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

fn private_indices(depth: usize, n_out: usize) -> Vec<usize> {
    // 1-based indices into args list.
    let mut idx = Vec::new();
    idx.push(4); // recipient
    idx.push(5); // spend_sk
    for i in 0..depth {
        idx.push(7 + i); // pos_bits
    }
    for i in 0..depth {
        idx.push(7 + depth + i); // siblings
    }
    let outs_base = 11 + 2 * depth;
    for j in 0..n_out {
        idx.push(outs_base + 4 * j + 0); // value_out_j
        idx.push(outs_base + 4 * j + 1); // rho_out_j
        idx.push(outs_base + 4 * j + 2); // pk_out_j
    }
    idx
}

#[derive(Clone)]
struct WalletNote {
    value: u64,
    rho: Hash32,
    spend_sk: Hash32,
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
) -> TransferWitness {
    let recipient = recipient_from_sk(&domain, &note.spend_sk);
    let siblings = tree.open(pos);

    let nf_key = nf_key_from_sk(&domain, &note.spend_sk);
    let nf = nullifier(&domain, &nf_key, &note.rho);

    // Output note secrets
    let mut out_rho = [0u8; 32];
    rng.fill_bytes(&mut out_rho);
    let mut out_spend_sk = [0u8; 32];
    rng.fill_bytes(&mut out_spend_sk);
    let out_pk = pk_from_sk(&out_spend_sk);
    let out_recipient = recipient_from_pk(&domain, &out_pk);
    let cm_out = note_commitment(&domain, out_value, &out_rho, &out_recipient);

    // args layout follows `note-spend-guest/src/main.rs` top-of-file comment.
    let mut args: Vec<LigeroArg> = Vec::new();
    args.push(LigeroArg::Hex { hex: hx32(&domain) });
    args.push(LigeroArg::I64 {
        i64: note.value as i64,
    });
    args.push(LigeroArg::Hex { hex: hx32(&note.rho) });
    args.push(LigeroArg::Hex { hex: hx32(&recipient) });
    args.push(LigeroArg::Hex {
        hex: hx32(&note.spend_sk),
    });
    args.push(LigeroArg::I64 { i64: depth as i64 });

    // pos bits: field elements 0/1 as 32-byte BE.
    for lvl in 0..depth {
        let bit = ((pos >> lvl) & 1) as u8;
        let mut bit_bytes = [0u8; 32];
        bit_bytes[31] = bit;
        args.push(LigeroArg::Hex {
            hex: hex::encode(bit_bytes),
        });
    }

    // siblings
    for s in &siblings {
        args.push(LigeroArg::Hex { hex: hx32(s) });
    }

    // public: anchor + nullifier
    args.push(LigeroArg::String {
        str: format!("0x{}", hx32(&anchor)),
    });
    args.push(LigeroArg::String {
        str: format!("0x{}", hx32(&nf)),
    });

    // withdraw_amount + n_out
    args.push(LigeroArg::I64 { i64: 0 });
    args.push(LigeroArg::I64 { i64: 1 });

    // output 0
    args.push(LigeroArg::I64 {
        i64: out_value as i64,
    });
    args.push(LigeroArg::Hex { hex: hx32(&out_rho) });
    args.push(LigeroArg::Hex { hex: hx32(&out_pk) });
    args.push(LigeroArg::Hex { hex: hx32(&cm_out) });

    TransferWitness {
        wallet_idx: pos,
        args,
        proof_path,
        new_note: WalletNote {
            value: out_value,
            rho: out_rho,
            spend_sk: out_spend_sk,
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
        j.join()
            .map_err(|_| anyhow::anyhow!("thread panicked"))??;
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
    let depth = env_usize("TRANSFER_TREE_DEPTH", 16);
    let packing = env_u32("LIGERO_PACKING", 8192);
    let gzip_proof = env_bool("LIGERO_GZIP_PROOF", false);

    // Concurrency knobs: the entered value controls BOTH
    // - number of daemon processes (workers)
    // - number of concurrent requests we issue (parallelism)
    //
    // This makes "workers vs parallelism" impossible to misconfigure from the keyboard.
    let prover_default = env_usize("LIGERO_PROVER_WORKERS", 1);
    let verifier_default = env_usize("LIGERO_VERIFIER_WORKERS", 1);
    let prover_concurrency =
        prompt_usize("Prover concurrency", prover_default)?;
    let verifier_concurrency =
        prompt_usize("Verifier concurrency", verifier_default)?;
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

    let paths = LigeroPaths::discover().or_else(|_| Ok::<_, anyhow::Error>(LigeroPaths::fallback()))?;
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
        let cm_in = note_commitment(&domain, value_in, &rho, &recipient);
        tree.set_leaf(i, cm_in);
        wallets.push(WalletNote {
            value: value_in,
            rho,
            spend_sk,
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
            );
            witnesses.push(w);
        }

        let priv_idx = private_indices(depth, 1);
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
                let resp = prover_pool.prove(cfg_json).context("daemon prove request failed")?;
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


