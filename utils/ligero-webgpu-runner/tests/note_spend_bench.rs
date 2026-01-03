//! Note-spend proving/verifying benchmark tests.
//!
//! This file intentionally contains both:
//! - a **daemon** benchmark using `webgpu_{prover,verifier} --daemon` via `DaemonPool`
//! - a **direct** benchmark using `BinaryWorkerPool` (spawns processes per request)
//!
//! The benchmarks share the same witness/statement construction code so we can do apples-to-apples
//! comparisons across strategies.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use ligetron::poseidon2_hash_bytes;
use sha2::{Digest, Sha256};

use ligero_runner::{
    daemon::DaemonPool, redact_arg, verifier, BinaryWorkerPool, LigeroArg, LigeroRunner,
    ProverRunOptions,
};

type Hash32 = [u8; 32];

// =============================================================================
// RESULTS COLLECTION FOR SUMMARY TABLE
// =============================================================================

/// Metrics extracted from prover/verifier output.
#[derive(Clone, Debug, Default)]
struct ProverMetrics {
    linear_constraints: Option<u64>,
    quadratic_constraints: Option<u64>,
    stage1_ms: Option<u64>,
    stage2_ms: Option<u64>,
    stage3_ms: Option<u64>,
    prove_result: Option<bool>,
}

#[derive(Clone, Debug, Default)]
struct VerifierMetrics {
    verify_time_ms: Option<u64>,
    verify_result: Option<bool>,
}

/// Summary of a single benchmark run.
#[derive(Clone, Debug)]
struct BenchResult {
    use_case: &'static str,
    prover_time: Duration,
    verifier_time: Duration,
    proof_size_bytes: usize,
    prover_metrics: ProverMetrics,
    verifier_metrics: VerifierMetrics,
}

lazy_static::lazy_static! {
    static ref BENCH_RESULTS: Mutex<Vec<BenchResult>> = Mutex::new(Vec::new());
}

fn parse_prover_metrics(stdout: &str) -> ProverMetrics {
    let mut m = ProverMetrics::default();
    for line in stdout.lines() {
        let clean_line = strip_ansi(line);
        let clean_trimmed = clean_line.trim();
        // Check for stage timings first (they have specific format)
        if clean_trimmed.starts_with("stage1:") {
            if let Some(ms) = extract_ms_from_timing_line(&clean_line) {
                m.stage1_ms = Some(ms);
            }
        } else if clean_trimmed.starts_with("stage2:") {
            if let Some(ms) = extract_ms_from_timing_line(&clean_line) {
                m.stage2_ms = Some(ms);
            }
        } else if clean_trimmed.starts_with("stage3:") {
            if let Some(ms) = extract_ms_from_timing_line(&clean_line) {
                m.stage3_ms = Some(ms);
            }
        } else if clean_line.contains("Num Linear constraints:") {
            if let Some(n) = clean_line
                .split_whitespace()
                .last()
                .and_then(|s| s.parse().ok())
            {
                m.linear_constraints = Some(n);
            }
        } else if clean_line.contains("Num quadratic constraints:") {
            if let Some(n) = clean_line
                .split_whitespace()
                .last()
                .and_then(|s| s.parse().ok())
            {
                m.quadratic_constraints = Some(n);
            }
        } else if clean_line.contains("Final prove result:") {
            m.prove_result = Some(clean_line.contains("true"));
        }
    }
    m
}

fn parse_verifier_metrics(stdout: &str) -> VerifierMetrics {
    let mut m = VerifierMetrics::default();
    for line in stdout.lines() {
        let clean_line = strip_ansi(line);
        if clean_line.contains("Verify time:") {
            if let Some(ms) = extract_ms_from_timing_line(&clean_line) {
                m.verify_time_ms = Some(ms);
            }
        } else if clean_line.contains("Final Verify Result:") {
            m.verify_result = Some(clean_line.contains("true"));
        }
    }
    m
}

fn strip_ansi(s: &str) -> String {
    // Remove ANSI escape sequences (e.g. "\x1b[32m") so log parsing is stable.
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == 0x1b {
            // ESC
            if i + 1 < bytes.len() && bytes[i + 1] == b'[' {
                // CSI: skip until final byte (0x40..=0x7E).
                i += 2;
                while i < bytes.len() {
                    let b = bytes[i];
                    i += 1;
                    if (0x40..=0x7e).contains(&b) {
                        break;
                    }
                }
                continue;
            }
            i += 1;
            continue;
        }

        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

fn extract_ms_from_timing_line(line: &str) -> Option<u64> {
    // e.g. "stage1: 228ms    (min: 228, max: 228, count: 1)"
    // or   "Verify time: 421ms    (min: 421, max: 421, count: 1)"
    for word in line.split_whitespace() {
        if word.ends_with("ms") {
            if let Ok(n) = word.trim_end_matches("ms").parse::<u64>() {
                return Some(n);
            }
        }
    }
    None
}

fn print_summary_table() {
    let results = BENCH_RESULTS.lock().unwrap();
    if results.is_empty() {
        return;
    }

    println!();
    println!("╔════════════════════════════════════════════════════════════════════════════════════════════════════════════╗");
    println!("║                                         BENCHMARK SUMMARY                                                  ║");
    println!("╠════════════╦═══════════╦════════════╦════════════╦═══════════════╦════════════════════╦══════════╦══════════╣");
    println!(
        "║ {:^10} ║ {:^9} ║ {:^10} ║ {:^10} ║ {:^13} ║ {:^18} ║ {:^8} ║ {:^8} ║",
        "Use Case",
        "Prover",
        "Verifier",
        "Proof Size",
        "Linear Constr",
        "Quadratic Constr",
        "Prover OK",
        "Verify OK",
    );
    println!("╠════════════╬═══════════╬════════════╬════════════╬═══════════════╬════════════════════╬══════════╬══════════╣");

    for r in results.iter() {
        let use_case = r.use_case.to_ascii_uppercase();
        let prover_ms = r.prover_time.as_millis();
        let verifier_ms = r.verifier_time.as_millis();
        let proof_kb = r.proof_size_bytes / 1024;
        let linear = r
            .prover_metrics
            .linear_constraints
            .map(|n| n.to_string())
            .unwrap_or_else(|| "N/A".to_string());
        let quad = r
            .prover_metrics
            .quadratic_constraints
            .map(|n| n.to_string())
            .unwrap_or_else(|| "N/A".to_string());
        let prover_ok = match r.prover_metrics.prove_result {
            Some(true) => "✓",
            Some(false) => "✗",
            None => "?",
        };
        let verify_ok = match r.verifier_metrics.verify_result {
            Some(true) => "✓",
            Some(false) => "✗",
            None => "?",
        };

        println!(
            "║ {:^10} ║ {:>6} ms ║ {:>7} ms ║ {:>7} KB ║ {:>13} ║ {:>18} ║ {:^8} ║ {:^8} ║",
            use_case,
            prover_ms,
            verifier_ms,
            proof_kb,
            linear,
            quad,
            prover_ok,
            verify_ok,
        );
    }

    println!("╚════════════╩═══════════╩════════════╩════════════╩═══════════════╩════════════════════╩══════════╩══════════╝");
    println!();

    // Detailed timing breakdown
    println!("┌───────────────────────────────────────────────────────────────────────────┐");
    println!("│                          PROVER STAGE BREAKDOWN                           │");
    println!("├────────────┬────────────┬────────────┬────────────┬───────────────────────┤");
    println!("│  Use Case  │  Stage 1   │  Stage 2   │  Stage 3   │   Total (stages)      │");
    println!("├────────────┼────────────┼────────────┼────────────┼───────────────────────┤");

    for r in results.iter() {
        let s1 = r.prover_metrics.stage1_ms.unwrap_or(0);
        let s2 = r.prover_metrics.stage2_ms.unwrap_or(0);
        let s3 = r.prover_metrics.stage3_ms.unwrap_or(0);
        let total = s1 + s2 + s3;
        println!(
            "│ {:^10} │ {:>7} ms │ {:>7} ms │ {:>7} ms │ {:>7} ms             │",
            r.use_case.to_uppercase(),
            s1, s2, s3, total
        );
    }

    println!("└────────────┴────────────┴────────────┴────────────┴───────────────────────┘");
    println!();
}

#[derive(Clone, Copy, Debug)]
enum NoteSpendUseCase {
    Deposit,
    Transfer,
    Withdraw,
}

impl NoteSpendUseCase {
    fn label(self) -> &'static str {
        match self {
            Self::Deposit => "deposit",
            Self::Transfer => "transfer",
            Self::Withdraw => "withdraw",
        }
    }

    fn n_out(self) -> usize {
        match self {
            Self::Deposit => 1,
            Self::Transfer => 2,
            Self::Withdraw => 1,
        }
    }
}

fn hx32(b: &Hash32) -> String {
    hex::encode(b)
}

fn hx32_short(b: &Hash32) -> String {
    let h = hx32(b);
    format!("0x{}...{}", &h[..8], &h[h.len() - 8..])
}

fn decode_hash32_hex(s: &str) -> Result<Hash32> {
    let s = s.trim();
    let s = s.strip_prefix("0x").unwrap_or(s);
    let bytes = hex::decode(s).with_context(|| format!("Failed to hex-decode 32-byte value: {s}"))?;
    anyhow::ensure!(
        bytes.len() == 32,
        "expected 32 bytes (64 hex chars), got {} bytes",
        bytes.len()
    );
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn arg_u64(args: &[LigeroArg], idx1: usize) -> Result<u64> {
    let arg0 = idx1
        .checked_sub(1)
        .context("arg_u64 expects 1-based indices")?;
    let LigeroArg::I64 { i64 } = args
        .get(arg0)
        .with_context(|| format!("missing arg index {idx1}"))?
        .clone()
    else {
        anyhow::bail!("expected i64 at arg index {idx1}");
    };
    anyhow::ensure!(i64 >= 0, "expected non-negative i64 at arg index {idx1}");
    Ok(i64 as u64)
}

fn arg_hash32(args: &[LigeroArg], idx1: usize) -> Result<Hash32> {
    let arg0 = idx1
        .checked_sub(1)
        .context("arg_hash32 expects 1-based indices")?;
    match args.get(arg0).with_context(|| format!("missing arg index {idx1}"))? {
        LigeroArg::Hex { hex } => decode_hash32_hex(hex),
        LigeroArg::String { str } => decode_hash32_hex(str),
        LigeroArg::I64 { .. } => anyhow::bail!("expected hex/string at arg index {idx1}"),
    }
}

fn print_box(title: &str, lines: &[String]) {
    const TEXT_WIDTH: usize = 80;
    const INNER: usize = TEXT_WIDTH + 2; // spaces left+right

    println!("╔{}╗", "═".repeat(INNER));
    println!("║ {:<TEXT_WIDTH$} ║", title);
    for line in lines {
        println!("║ {:<TEXT_WIDTH$} ║", line);
    }
    println!("╚{}╝", "═".repeat(INNER));
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

/// Generate labels for each argument position (1-indexed to match circuit docs).
fn arg_labels(
    depth: usize,
    n_out: usize,
    use_case: NoteSpendUseCase,
) -> Vec<(&'static str, &'static str)> {
    // Returns (name, visibility) tuples
    let mut labels: Vec<(&'static str, &'static str)> = Vec::new();

    if matches!(use_case, NoteSpendUseCase::Deposit) {
        // Deposit circuit ABI:
        // 1 domain (PUB), 2 value (PUB), 3 rho (PRIV), 4 recipient (PRIV), 5 cm_out (PUB)
        labels.push(("domain", "PUBLIC"));
        labels.push(("value", "PUBLIC"));
        labels.push(("rho", "PRIVATE"));
        labels.push(("recipient", "PRIVATE"));
        labels.push(("cm_out", "PUBLIC"));
        return labels;
    }

    // Fixed arguments [1..6]
    labels.push(("domain", "PUBLIC"));
    labels.push((
        "value",
        if matches!(use_case, NoteSpendUseCase::Deposit) {
            "PUBLIC"
        } else {
            "PRIVATE"
        },
    ));
    labels.push(("rho", "PRIVATE"));
    labels.push(("recipient", "PRIVATE"));
    labels.push(("spend_sk", "PRIVATE"));
    labels.push(("depth", "PUBLIC"));

    // pos_bits [7..7+depth)
    for _ in 0..depth {
        labels.push(("pos_bit", "PRIVATE"));
    }

    // siblings [7+depth..7+2*depth)
    for _ in 0..depth {
        labels.push(("sibling", "PRIVATE"));
    }

    // anchor, nullifier [7+2*depth, 8+2*depth]
    labels.push(("anchor", "PUBLIC"));
    labels.push(("nullifier", "PUBLIC"));

    // withdraw_amount, n_out [9+2*depth, 10+2*depth]
    labels.push(("withdraw_amount", "PUBLIC"));
    labels.push(("n_out", "PUBLIC"));

    // outputs [11+2*depth + 4*j + 0..3]
    for j in 0..n_out {
        let _ = j; // suppress unused warning
        labels.push(("value_out", "PRIVATE"));
        labels.push(("rho_out", "PRIVATE"));
        labels.push(("pk_out", "PRIVATE"));
        labels.push(("cm_out", "PUBLIC"));
    }

    labels
}

/// Print arguments with labels when verbose mode is on.
fn print_labeled_args(args: &[LigeroArg], depth: usize, n_out: usize, use_case: NoteSpendUseCase) {
    let labels = arg_labels(depth, n_out, use_case);
    let title = format!("{} ARGUMENTS", use_case.label().to_uppercase());
    println!();
    println!("┌────────────────────────────────────────────────────────────────────────────────────┐");
    println!("│ {:<82} │", title);
    println!("├─────┬──────────────────┬─────────┬─────────────────────────────────────────────────┤");
    println!("│ Idx │ Name             │ Visible │ Value                                           │");
    println!("├─────┼──────────────────┼─────────┼─────────────────────────────────────────────────┤");

    for (i, arg) in args.iter().enumerate() {
        let (name, vis) = labels.get(i).copied().unwrap_or(("unknown", "?"));
        // Display PRIVATE values in redacted form so logs/tables match what the verifier sees.
        let display_arg = if vis == "PRIVATE" {
            redact_arg(arg)
        } else {
            arg.clone()
        };

        let value_str = match display_arg {
            LigeroArg::Hex { hex } => {
                if hex.len() > 20 {
                    format!("0x{}...{}", &hex[..8], &hex[hex.len()-8..])
                } else {
                    format!("0x{}", hex)
                }
            }
            LigeroArg::I64 { i64: v } => format!("{}", v),
            LigeroArg::String { str: s } => {
                if s.len() > 24 {
                    format!("{}...{}", &s[..12], &s[s.len()-8..])
                } else {
                    s.clone()
                }
            }
        };

        println!(
            "│ {:>3} │ {:16} │ {:7} │ {:47} │",
            i + 1,
            name,
            vis,
            value_str
        );
    }

    println!("└─────┴──────────────────┴─────────┴─────────────────────────────────────────────────┘");
    println!();
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

fn private_indices_note_spend(depth: usize, use_case: NoteSpendUseCase) -> Vec<usize> {
    // NOTE: `private-indices` are 1-based indices into the args list (excluding argv[0]).
    //
    // This matches the guest's documented layout in:
    // `utils/circuits/note-spend/src/main.rs` (see top-of-file comment).
    //
    // Private (witness), by use case:
    // - Deposit: rho, recipient, spend_sk, path, outputs (value/rho/pk)
    // - Transfer: value, rho, recipient, spend_sk, path, outputs (value/rho/pk)
    // - Withdraw: value, rho, recipient, spend_sk, path, change output (value/rho/pk)
    let n_out = use_case.n_out();
    let mut idx = Vec::new();

    // value is private for transfer/withdraw, public for deposit.
    if matches!(use_case, NoteSpendUseCase::Transfer | NoteSpendUseCase::Withdraw) {
        idx.push(2); // value
    }

    idx.push(3); // rho
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

fn private_indices_note_deposit() -> Vec<usize> {
    // Deposit circuit ABI:
    // 1 domain (PUB), 2 value (PUB), 3 rho (PRIV), 4 recipient (PRIV), 5 cm_out (PUB)
    vec![3, 4]
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

fn maybe_build_note_deposit_guest(repo: &Path) -> Result<()> {
    let out = repo.join("utils/circuits/bins/note_deposit_guest.wasm");
    if out.exists() {
        return Ok(());
    }

    let guest_dir = repo.join("utils/circuits/note-deposit");
    if !guest_dir.exists() {
        anyhow::bail!("note-deposit sources not found at {}", guest_dir.display());
    }

    println!(
        "[note_deposit] note_deposit_guest.wasm not found; building via {}",
        guest_dir.join("build.sh").display()
    );

    let status = Command::new("bash")
        .arg("build.sh")
        .current_dir(&guest_dir)
        .status()
        .context("Failed to run note-deposit/build.sh")?;

    if !status.success() {
        anyhow::bail!("note-deposit/build.sh failed with status {status}");
    }

    Ok(())
}

fn build_deposit_statement(run: u8, domain: Hash32, value: u64) -> Result<Vec<LigeroArg>> {
    // Vary rho/recipient so the public commitment differs per run.
    let mut rho: Hash32 = [2u8; 32];
    rho[0] = rho[0].wrapping_add(run);

    let mut spend_sk: Hash32 = [4u8; 32];
    spend_sk[0] = spend_sk[0].wrapping_add(run);
    let recipient = recipient_from_sk(&domain, &spend_sk);

    let cm_out = note_commitment(&domain, value, &rho, &recipient);

    Ok(vec![
        LigeroArg::Hex { hex: hx32(&domain) },
        LigeroArg::I64 { i64: value as i64 },
        LigeroArg::Hex { hex: hx32(&rho) },
        LigeroArg::Hex {
            hex: hx32(&recipient),
        },
        LigeroArg::Hex { hex: hx32(&cm_out) },
    ])
}

/// Build one statement (args) plus a small public summary for sanity checks.
fn build_statement(
    run: u8,
    depth: usize,
    domain: Hash32,
    value: u64,
    pos: u64,
    use_case: NoteSpendUseCase,
) -> Result<(Vec<LigeroArg>, (Hash32, Hash32, Vec<Hash32>, u64, usize))> {
    // Vary public rho (arg[3]) so the statement differs even if you ignore private witness.
    let mut rho: Hash32 = [2u8; 32];
    rho[0] = rho[0].wrapping_add(run);

    // Vary private spend_sk (arg[5]) so witness differs too.
    let mut spend_sk: Hash32 = [4u8; 32];
    spend_sk[0] = spend_sk[0].wrapping_add(run);

    // Keep `n_out` constant across runs so `private-indices` layout is identical.
    let n_out: usize = use_case.n_out();
    let (withdraw_amount, out_values): (u64, Vec<u64>) = match use_case {
        NoteSpendUseCase::Deposit => (0, vec![value]),
        NoteSpendUseCase::Transfer => {
            let out0 = (value / 3).max(1);
            let out1 = value
                .checked_sub(out0)
                .context("value must be >= transfer out_value_0")?;
            (0, vec![out0, out1])
        }
        NoteSpendUseCase::Withdraw => {
            let withdraw_amount = (10 + run as u64).min(value);
            let change_value = value
                .checked_sub(withdraw_amount)
                .context("value must be >= withdraw_amount")?;
            (withdraw_amount, vec![change_value])
        }
    };

    let recipient = recipient_from_sk(&domain, &spend_sk);
    let nf_key = nf_key_from_sk(&domain, &spend_sk);

    let cm_in = note_commitment(&domain, value, &rho, &recipient);
    let mut tree = MerkleTree::new(depth);
    tree.set_leaf(pos as usize, cm_in);
    let anchor = tree.root();
    let siblings = tree.open(pos as usize);
    let nf = nullifier(&domain, &nf_key, &rho);

    // Outputs: value/rho/pk are private; cm_out is public.
    let mut cm_outs: Vec<Hash32> = Vec::with_capacity(n_out);
    let mut out_triples: Vec<(u64, Hash32, Hash32, Hash32)> = Vec::with_capacity(n_out);
    for (j, out_value) in out_values.into_iter().enumerate() {
        let mut out_rho: Hash32 = [7u8; 32];
        out_rho[0] = out_rho[0].wrapping_add(run);
        out_rho[1] = out_rho[1].wrapping_add(j as u8);

        let mut out_spend_sk: Hash32 = [8u8; 32];
        out_spend_sk[0] = out_spend_sk[0].wrapping_add(run);
        out_spend_sk[1] = out_spend_sk[1].wrapping_add(j as u8);

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
    let use_case = NoteSpendUseCase::Transfer;

    let mut configs: Vec<serde_json::Value> = Vec::new();
    let mut public_summaries: Vec<(Hash32, Hash32, Vec<Hash32>, u64, usize)> = Vec::new();

    for run in 0..3u8 {
        let (args, summary) = build_statement(run, depth, domain, value, pos, use_case)?;
        runner.config_mut().private_indices = private_indices_note_spend(depth, use_case);
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

fn run_note_spend_direct_bench(
    use_case: NoteSpendUseCase,
    depth: usize,
    domain: Hash32,
    value: u64,
    pos: u64,
) -> Result<()> {
    let repo = repo_root()?;
    let program = match use_case {
        NoteSpendUseCase::Deposit => {
            maybe_build_note_deposit_guest(&repo)?;
            repo.join("utils/circuits/bins/note_deposit_guest.wasm")
                .canonicalize()
                .context("Failed to canonicalize note_deposit_guest.wasm")?
        }
        NoteSpendUseCase::Transfer | NoteSpendUseCase::Withdraw => {
            maybe_build_note_spend_guest(&repo)?;
            repo.join("utils/circuits/bins/note_spend_guest.wasm")
                .canonicalize()
                .context("Failed to canonicalize note_spend_guest.wasm")?
        }
    };

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

    let label = use_case.label();
    let n_out = use_case.n_out();
    let num_runs = read_num_runs();

    // Preview run0 statement so the banner can show the exact values used by this test.
    let (run0_args, run0_priv_idx, run0_summary) = match use_case {
        NoteSpendUseCase::Deposit => {
            let args = build_deposit_statement(0, domain, value)?;
            (args, private_indices_note_deposit(), None)
        }
        NoteSpendUseCase::Transfer | NoteSpendUseCase::Withdraw => {
            let (args, summary) = build_statement(0, depth, domain, value, pos, use_case)?;
            (args, private_indices_note_spend(depth, use_case), Some(summary))
        }
    };

    // Print use case banner
    println!();
    let mut banner_lines: Vec<String> = Vec::new();
    match use_case {
        NoteSpendUseCase::Deposit => {
            let rho = arg_hash32(&run0_args, 3)?;
            let recipient = arg_hash32(&run0_args, 4)?;
            let cm_out = arg_hash32(&run0_args, 5)?;

            banner_lines.push(format!(
                "Run0: domain={}, value(pub)={}",
                hx32_short(&domain),
                value
            ));
            banner_lines.push(format!("Run0 destination(priv): recipient={}", hx32_short(&recipient)));
            banner_lines.push(format!("Run0 private rho={}", hx32_short(&rho)));
            banner_lines.push(format!("Public: cm_out={}", hx32_short(&cm_out)));
            banner_lines.push("Note: transparent origin is outside this proof".to_string());

            print_box("💰 DEPOSIT - Enter the privacy pool", &banner_lines);
        }
        NoteSpendUseCase::Transfer => {
            let (anchor, nf, cm_outs, withdraw_amount, n_out) = run0_summary
                .clone()
                .context("missing run0 summary for transfer")?;

            let sender = arg_hash32(&run0_args, 4)?;
            let out_base = 11 + 2 * depth;
            let out0_value = arg_u64(&run0_args, out_base)?;
            let out0_pk = arg_hash32(&run0_args, out_base + 2)?;
            let out0_recipient = recipient_from_pk(&domain, &out0_pk);
            let out1_value = arg_u64(&run0_args, out_base + 4)?;
            let out1_pk = arg_hash32(&run0_args, out_base + 6)?;
            let out1_recipient = recipient_from_pk(&domain, &out1_pk);

            banner_lines.push(format!(
                "Run0: domain={}, depth(pub)={depth}, pos(priv)={pos}, in_value(priv)={value}",
                hx32_short(&domain)
            ));
            banner_lines.push(format!("Run0 origin(priv): sender={}", hx32_short(&sender)));
            banner_lines.push(format!(
                "Run0 destinations(priv): {out0_value}→{}, {out1_value}→{}",
                hx32_short(&out0_recipient),
                hx32_short(&out1_recipient)
            ));
            banner_lines.push(format!("Public: withdraw_amount(pub)={withdraw_amount}, n_out(pub)={n_out}"));
            banner_lines.push(format!(
                "Public: anchor={}, nf={}",
                hx32_short(&anchor),
                hx32_short(&nf)
            ));
            if cm_outs.len() == 2 {
                banner_lines.push(format!(
                    "Public: cm_out0={}, cm_out1={}",
                    hx32_short(&cm_outs[0]),
                    hx32_short(&cm_outs[1])
                ));
            }
            if num_runs > 1 {
                banner_lines.push("Other runs vary secrets for binding test".to_string());
            }

            print_box("🔒 TRANSFER - Fully private transaction", &banner_lines);
        }
        NoteSpendUseCase::Withdraw => {
            let (anchor, nf, cm_outs, withdraw_amount, n_out) = run0_summary
                .clone()
                .context("missing run0 summary for withdraw")?;

            let sender = arg_hash32(&run0_args, 4)?;
            let out_base = 11 + 2 * depth;
            let change_value = arg_u64(&run0_args, out_base)?;
            let change_pk = arg_hash32(&run0_args, out_base + 2)?;
            let change_recipient = recipient_from_pk(&domain, &change_pk);

            banner_lines.push(format!(
                "Run0: domain={}, depth(pub)={depth}, pos(priv)={pos}",
                hx32_short(&domain)
            ));
            banner_lines.push(format!(
                "Run0: in(priv)={value}, withdraw(pub)={withdraw_amount}, change(priv)={change_value}"
            ));
            banner_lines.push(format!("Run0 origin(priv): sender={}", hx32_short(&sender)));
            banner_lines.push(format!(
                "Run0 change destination(priv): {change_value}→{}",
                hx32_short(&change_recipient)
            ));
            banner_lines.push("Note: transparent withdraw recipient is outside this proof".to_string());
            banner_lines.push(format!(
                "Public: anchor={}, nf={}, n_out(pub)={n_out}",
                hx32_short(&anchor),
                hx32_short(&nf)
            ));
            if cm_outs.len() == 1 {
                banner_lines.push(format!("Public: cm_out0={}", hx32_short(&cm_outs[0])));
            }
            if num_runs > 1 {
                banner_lines.push("Other runs vary secrets for binding test".to_string());
            }

            print_box("💸 WITHDRAW - Exit the privacy pool", &banner_lines);
        }
    }
    println!();

    println!("[{label}] Prover:   {}", runner.paths().prover_bin.display());
    println!("[{label}] Verifier: {}", runner.paths().verifier_bin.display());
    println!("[{label}] Shaders:  {}", shader_dir.display());
    println!("[{label}] Program:  {}", program.display());
    println!("[{label}] Packing:  {}", packing);
    println!("[{label}] Gzip:     {}", gzip_proof);

    // Prove N distinct proofs (controlled by LIGERO_RUNS env var, default 3).
    println!("[{label}] Running {} proof(s)", num_runs);
    let mut proofs: Vec<Vec<u8>> = Vec::new();
    let mut configs: Vec<Vec<LigeroArg>> = Vec::new();
    let mut all_priv_idx: Vec<Vec<usize>> = Vec::new();
    let mut last_prover_stdout = String::new();
    let mut last_prover_time = Duration::default();
    let mut last_proof_size = 0usize;

    for run in 0..num_runs {
        let (args, priv_idx) = if run == 0 {
            (run0_args.clone(), run0_priv_idx.clone())
        } else {
            match use_case {
                NoteSpendUseCase::Deposit => {
                    let args = build_deposit_statement(run, domain, value)?;
                    let priv_idx = private_indices_note_deposit();
                    (args, priv_idx)
                }
                NoteSpendUseCase::Transfer | NoteSpendUseCase::Withdraw => {
                    let (args, _summary) = build_statement(run, depth, domain, value, pos, use_case)?;
                    let priv_idx = private_indices_note_spend(depth, use_case);
                    (args, priv_idx)
                }
            }
        };
        runner.config_mut().private_indices = priv_idx.clone();
        runner.config_mut().args = args.clone();

        // Print labeled arguments for first run when verbose
        if run == 0 && is_verbose() {
            print_labeled_args(&args, depth, n_out, use_case);
        }

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
        last_prover_time = d;
        last_proof_size = proof.len();
        last_prover_stdout = stdout.clone();

        println!(
            "[{label}] Prover run #{}: {:?} ({} bytes)",
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

    // Verify the proofs (timed).
    let vpaths = verifier::VerifierPaths::from_explicit(
        program.clone(),
        shader_dir,
        runner.paths().verifier_bin.clone(),
        packing,
    );

    let mut last_verifier_stdout = String::new();
    let mut last_verifier_time = Duration::default();

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
        last_verifier_time = vd;
        last_verifier_stdout = vs.clone();

        assert!(ok, "verifier should report success for run #{}", i + 1);
        println!("[{label}] Verifier run #{}: {:?}", i + 1, vd);
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

    // Collect results for summary table
    let prover_metrics = parse_prover_metrics(&last_prover_stdout);
    let verifier_metrics = parse_verifier_metrics(&last_verifier_stdout);

    // Debug: show parsed stage times if parsing failed (helps diagnose ANSI/format changes).
    if is_verbose()
        && (prover_metrics.stage1_ms.is_none()
            || prover_metrics.stage2_ms.is_none()
            || prover_metrics.stage3_ms.is_none())
    {
        println!(
            "[{label}] Parsed prover stages: s1={:?}ms s2={:?}ms s3={:?}ms",
            prover_metrics.stage1_ms,
            prover_metrics.stage2_ms,
            prover_metrics.stage3_ms
        );
    }

    BENCH_RESULTS.lock().unwrap().push(BenchResult {
        use_case: label,
        prover_time: last_prover_time,
        verifier_time: last_verifier_time,
        proof_size_bytes: last_proof_size,
        prover_metrics,
        verifier_metrics,
    });

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

#[test]
fn test_note_spend_direct_bench_deposit() -> Result<()> {
    run_note_spend_direct_bench(NoteSpendUseCase::Deposit, 8, [1u8; 32], 100, 0)
}

#[test]
fn test_note_spend_direct_bench_transfer() -> Result<()> {
    run_note_spend_direct_bench(NoteSpendUseCase::Transfer, 8, [2u8; 32], 200, 1)
}

#[test]
fn test_note_spend_direct_bench_withdraw() -> Result<()> {
    let result = run_note_spend_direct_bench(NoteSpendUseCase::Withdraw, 8, [3u8; 32], 150, 2);

    // Print summary table after the last test
    // Note: This works when tests run in order (single-threaded with --test-threads=1)
    // or when withdraw is the last test to complete
    print_summary_table();

    result
}
