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
use ligetron::Bn254Fr;
use sha2::{Digest, Sha256};

use ligero_runner::{
    daemon::DaemonPool, verifier, BinaryWorkerPool, LigeroArg, LigeroRunner, ProverRunOptions,
};

type Hash32 = [u8; 32];
const BL_DEPTH: usize = 63;

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
    println!("╔═════════════════════════════════════════════════════════════════════════════════════════════════════════════════╗");
    println!("║                                         BENCHMARK SUMMARY                                                       ║");
    println!("╠════════════╦═══════════╦════════════╦════════════╦═══════════════╦════════════════════╦════════════╦════════════╣");
    println!(
        "║ {:^10} ║ {:^9} ║ {:^10} ║ {:^10} ║ {:^13} ║ {:^18} ║ {:^10} ║ {:^10} ║",
        "Use Case",
        "Prover",
        "Verifier",
        "Proof Size",
        "Linear Constr",
        "Quadratic Constr",
        "Prover OK",
        "Verify OK",
    );
    println!("╠════════════╬═══════════╬════════════╬════════════╬═══════════════╬════════════════════╬════════════╬════════════╣");

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
            "║ {:^10} ║ {:>6} ms ║ {:>7} ms ║ {:>7} KB ║ {:>13} ║ {:>18} ║ {:^10} ║ {:^10} ║",
            use_case, prover_ms, verifier_ms, proof_kb, linear, quad, prover_ok, verify_ok,
        );
    }

    println!("╚════════════╩═══════════╩════════════╩════════════╩═══════════════╩════════════════════╩════════════╩════════════╝");
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
            "│ {:^10} │ {:>7} ms │ {:>7} ms │ {:>7} ms │ {:>6} ms             │",
            r.use_case.to_uppercase(),
            s1,
            s2,
            s3,
            total
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
    let bytes =
        hex::decode(s).with_context(|| format!("Failed to hex-decode 32-byte value: {s}"))?;
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
    match args
        .get(arg0)
        .with_context(|| format!("missing arg index {idx1}"))?
    {
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

fn enable_viewers() -> bool {
    std::env::var("LIGERO_ENABLE_VIEWERS")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Generate labels for each argument position (1-indexed to match circuit docs).
fn arg_labels(args: &[LigeroArg], use_case: NoteSpendUseCase) -> Vec<(&'static str, &'static str)> {
    // Returns (name, visibility) tuples (1-based positions excluding argv[0]).
    let mut labels: Vec<(&'static str, &'static str)> = Vec::new();

    if matches!(use_case, NoteSpendUseCase::Deposit) {
        // Deposit circuit ABI:
        // 1 domain (PUB), 2 value (PUB), 3 rho (PRIV), 4 pk_spend_recipient (PRIV),
        // 5 pk_ivk_recipient (PRIV), 6 cm_out (PUB),
        // 7 blacklist_root (PUB), 8..(8+BL_DEPTH-1) bl_siblings (PRIV)
        labels.push(("domain", "PUBLIC"));
        labels.push(("value", "PUBLIC"));
        labels.push(("rho", "PRIVATE"));
        labels.push(("pk_spend_recipient", "PRIVATE"));
        labels.push(("pk_ivk_recipient", "PRIVATE"));
        labels.push(("cm_out", "PUBLIC"));
        labels.push(("blacklist_root", "PUBLIC"));
        for _ in 0..BL_DEPTH {
            labels.push(("bl_sibling", "PRIVATE"));
        }
        return labels;
    }

    // note-spend v2 ABI (transfer/withdraw).
    //
    // Header:
    // 1 domain (PUB)
    // 2 spend_sk (PRIV)
    // 3 pk_ivk_owner (PRIV)
    // 4 depth (PUB)
    // 5 anchor (PUB)
    // 6 n_in (PUB)
    labels.push(("domain", "PUBLIC"));
    labels.push(("spend_sk", "PRIVATE"));
    labels.push(("pk_ivk_owner", "PRIVATE"));
    labels.push(("depth", "PUBLIC"));
    labels.push(("anchor", "PUBLIC"));
    labels.push(("n_in", "PUBLIC"));

    let depth = arg_u64(args, 4).ok().unwrap_or(0) as usize;
    let n_in = arg_u64(args, 6).ok().unwrap_or(0) as usize;
    let per_in = 4usize + 2usize * depth;
    let withdraw_idx = 7usize + n_in * per_in;
    let n_out = arg_u64(args, withdraw_idx + 2).ok().unwrap_or(0) as usize;

    // Inputs.
    for _i in 0..n_in {
        labels.push(("value_in", "PRIVATE"));
        labels.push(("rho_in", "PRIVATE"));
        labels.push(("sender_id_in", "PRIVATE"));
        for _ in 0..depth {
            labels.push(("pos_bit", "PRIVATE"));
        }
        for _ in 0..depth {
            labels.push(("sibling", "PRIVATE"));
        }
        labels.push(("nullifier", "PUBLIC"));
    }

    // Withdraw + n_out.
    labels.push(("withdraw_amount", "PUBLIC"));
    labels.push(("withdraw_to", "PUBLIC"));
    labels.push(("n_out", "PUBLIC"));

    // Outputs.
    for _j in 0..n_out {
        labels.push(("value_out", "PRIVATE"));
        labels.push(("rho_out", "PRIVATE"));
        labels.push(("pk_spend_out", "PRIVATE"));
        labels.push(("pk_ivk_out", "PRIVATE"));
        labels.push(("cm_out", "PUBLIC"));
    }

    // inv_enforce witness (PRIVATE) - present even when there are no viewers.
    labels.push(("inv_enforce", "PRIVATE"));

    // Blacklist section (root public, siblings private).
    labels.push(("blacklist_root", "PUBLIC"));
    for _ in 0..BL_DEPTH {
        labels.push(("bl_sibling_sender", "PRIVATE"));
    }
    for _j in 0..n_out {
        for _ in 0..BL_DEPTH {
            labels.push(("bl_sibling_out", "PRIVATE"));
        }
    }

    // Optional viewer attestations (Level B).
    if labels.len() < args.len() {
        labels.push(("n_viewers", "PUBLIC"));
        let n_viewers_idx1 = labels.len();
        let n_viewers = arg_u64(args, n_viewers_idx1).ok().unwrap_or(0) as usize;
        for _v in 0..n_viewers {
            labels.push(("fvk_commit", "PUBLIC"));
            labels.push(("fvk", "PRIVATE"));
            for _j in 0..n_out {
                labels.push(("ct_hash", "PUBLIC"));
                labels.push(("mac", "PUBLIC"));
            }
        }
    }

    labels
}

/// Print arguments with labels when verbose mode is on.
fn print_labeled_args(args: &[LigeroArg], use_case: NoteSpendUseCase) {
    let labels = arg_labels(args, use_case);
    let title = format!("{} ARGUMENTS", use_case.label().to_uppercase());
    println!();
    println!(
        "┌────────────────────────────────────────────────────────────────────────────────────┐"
    );
    println!("│ {:<82} │", title);
    println!(
        "├─────┬──────────────────┬─────────┬─────────────────────────────────────────────────┤"
    );
    println!(
        "│ Idx │ Name             │ Visible │ Value                                           │"
    );
    println!(
        "├─────┼──────────────────┼─────────┼─────────────────────────────────────────────────┤"
    );

    for (i, arg) in args.iter().enumerate() {
        let (name, vis) = labels.get(i).copied().unwrap_or(("unknown", "?"));
        let value_str = if vis == "PRIVATE" {
            "<redacted>".to_string()
        } else {
            match arg.clone() {
                LigeroArg::Hex { hex } => {
                    if hex.len() > 20 {
                        format!("0x{}...{}", &hex[..8], &hex[hex.len() - 8..])
                    } else {
                        format!("0x{}", hex)
                    }
                }
                LigeroArg::I64 { i64: v } => format!("{}", v),
                LigeroArg::String { str: s } => {
                    if s.len() > 24 {
                        format!("{}...{}", &s[..12], &s[s.len() - 8..])
                    } else {
                        s.clone()
                    }
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

    println!(
        "└─────┴──────────────────┴─────────┴─────────────────────────────────────────────────┘"
    );
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

fn merkle_default_nodes(depth: usize) -> Vec<Hash32> {
    let mut out = Vec::with_capacity(depth + 1);
    out.push([0u8; 32]); // height 0 (leaf)
    for lvl in 0..depth {
        let prev = out[lvl];
        out.push(mt_combine(lvl as u8, &prev, &prev));
    }
    out
}

fn append_empty_blacklist(args: &mut Vec<LigeroArg>, n_out: usize) {
    let bl_defaults = merkle_default_nodes(BL_DEPTH);
    let blacklist_root = bl_defaults[BL_DEPTH];
    args.push(LigeroArg::Hex {
        hex: hx32(&blacklist_root),
    }); // blacklist_root (pub)
    for sib in bl_defaults.iter().take(BL_DEPTH) {
        args.push(LigeroArg::Hex { hex: hx32(sib) }); // sender siblings (priv)
    }
    for _j in 0..n_out {
        for sib in bl_defaults.iter().take(BL_DEPTH) {
            args.push(LigeroArg::Hex { hex: hx32(sib) }); // output recipient siblings (priv)
        }
    }
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

fn recipient_from_pk(domain: &Hash32, pk_spend: &Hash32, pk_ivk: &Hash32) -> Hash32 {
    poseidon2_hash_domain(b"ADDR_V2", &[domain, pk_spend, pk_ivk])
}

fn ivk_seed(domain: &Hash32, spend_sk: &Hash32) -> Hash32 {
    // Wallet-side this would be clamped and base-multiplied (X25519) to get a real pk_ivk,
    // but the circuit treats `pk_ivk` as an opaque 32-byte value and only binds it into ADDR_V2.
    poseidon2_hash_domain(b"IVK_SEED_V1", &[domain, spend_sk])
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

fn private_indices_note_spend(depth: usize, n_in: usize, n_out: usize) -> Vec<usize> {
    // NOTE: `private-indices` are 1-based indices into the args list (excluding argv[0]).
    //
    // This matches `utils/circuits/note-spend/src/main.rs` v2 ABI:
    //   [1] domain (PUB)
    //   [2] spend_sk (PRIV)
    //   [3] pk_ivk_owner (PRIV)
    //   [4] depth (PUB)
    //   [5] anchor (PUB)
    //   [6] n_in (PUB)
    //   For each input i:
    //     value_in_i     (PRIV)
    //     rho_in_i       (PRIV)
    //     sender_id_in_i (PRIV)
    //     pos_bits       (PRIV) × depth
    //     siblings       (PRIV) × depth
    //     nullifier_i    (PUB)
    //   withdraw_amount (PUB)
    //   withdraw_to     (PUB)
    //   n_out           (PUB)
    //   For each output j:
    //     value_out_j    (PRIV)
    //     rho_out_j      (PRIV)
    //     pk_spend_out_j (PRIV)
    //     pk_ivk_out_j   (PRIV)
    //     cm_out_j       (PUB)
    //   inv_enforce     (PRIV)
    //   blacklist_root  (PUB)
    //   bl_siblings_sender[BL_DEPTH] (PRIV)
    //   bl_siblings_out_j[BL_DEPTH]  (PRIV, per output)
    let mut idx = Vec::new();
    idx.push(2); // spend_sk
    idx.push(3); // pk_ivk_owner

    let per_in = 4 + 2 * depth; // value + rho + sender_id + pos_bits[depth] + siblings[depth] + nullifier
    for i in 0..n_in {
        let base = 7 + i * per_in;
        idx.push(base); // value_in_i
        idx.push(base + 1); // rho_in_i
        idx.push(base + 2); // sender_id_in_i

        // pos_bits_i
        for k in 0..depth {
            idx.push(base + 3 + k);
        }

        // siblings_i
        for k in 0..depth {
            idx.push(base + 3 + depth + k);
        }

        // nullifier_i is public (base + 3 + 2*depth)
    }

    // Outputs start after withdraw_amount + withdraw_to + n_out.
    let withdraw_idx = 7 + n_in * per_in;
    let outs_base = withdraw_idx + 3;
    for j in 0..n_out {
        idx.push(outs_base + 5 * j + 0); // value_out_j
        idx.push(outs_base + 5 * j + 1); // rho_out_j
        idx.push(outs_base + 5 * j + 2); // pk_spend_out_j
        idx.push(outs_base + 5 * j + 3); // pk_ivk_out_j
                                         // cm_out_j is public (outs_base + 5*j + 4)
    }

    // inv_enforce comes right after the outputs.
    let inv_enforce_idx = outs_base + 5 * n_out;
    idx.push(inv_enforce_idx);

    let bl_sender_start = inv_enforce_idx + 2;
    for k in 0..BL_DEPTH {
        idx.push(bl_sender_start + k);
    }
    let bl_out_start = bl_sender_start + BL_DEPTH;
    for j in 0..n_out {
        for k in 0..BL_DEPTH {
            idx.push(bl_out_start + j * BL_DEPTH + k);
        }
    }

    idx
}

fn private_indices_note_deposit() -> Vec<usize> {
    // Deposit circuit ABI:
    // 1 domain (PUB), 2 value (PUB), 3 rho (PRIV),
    // 4 pk_spend_recipient (PRIV), 5 pk_ivk_recipient (PRIV), 6 cm_out (PUB),
    // 7 blacklist_root (PUB), 8..(8+BL_DEPTH-1) bl_siblings (PRIV)
    let mut idx = vec![3, 4, 5];
    for i in 0..BL_DEPTH {
        idx.push(8 + i);
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
    let pk_spend = pk_from_sk(&spend_sk);
    let pk_ivk = ivk_seed(&domain, &spend_sk);
    let recipient = recipient_from_pk(&domain, &pk_spend, &pk_ivk);

    let sender_id = [0u8; 32];
    let cm_out = note_commitment(&domain, value, &rho, &recipient, &sender_id);

    let mut args = vec![
        LigeroArg::Hex { hex: hx32(&domain) },
        LigeroArg::I64 { i64: value as i64 },
        LigeroArg::Hex { hex: hx32(&rho) },
        LigeroArg::Hex { hex: hx32(&pk_spend) },
        LigeroArg::Hex { hex: hx32(&pk_ivk) },
        LigeroArg::Hex { hex: hx32(&cm_out) },
    ];
    append_empty_blacklist(&mut args, 0);
    Ok(args)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NoteSpendPublicSummary {
    anchor: Hash32,
    nullifiers: Vec<Hash32>,
    cm_outs: Vec<Hash32>,
    withdraw_amount: u64,
    n_in: usize,
    n_out: usize,
}

/// Build one statement (args) plus a small public summary for sanity checks.
fn build_statement(
    run: u8,
    depth: usize,
    domain: Hash32,
    value: u64,
    pos: u64,
    use_case: NoteSpendUseCase,
    include_viewers: bool,
) -> Result<(Vec<LigeroArg>, NoteSpendPublicSummary, Vec<usize>)> {
    anyhow::ensure!(
        !matches!(use_case, NoteSpendUseCase::Deposit),
        "deposit uses note-deposit guest; build_statement is for spend (transfer/withdraw)"
    );

    // Keep `n_in` and `n_out` constant across runs so the `private-indices` layout is identical.
    let n_in: usize = 1;
    let n_out: usize = use_case.n_out();

    // Vary private rho/spend_sk per run so the public statement differs (anchor/nf/cm_outs).
    let mut rho: Hash32 = [2u8; 32];
    rho[0] = rho[0].wrapping_add(run);

    let mut spend_sk: Hash32 = [4u8; 32];
    spend_sk[0] = spend_sk[0].wrapping_add(run);

    let pk_ivk_owner = ivk_seed(&domain, &spend_sk);
    let pk_spend_owner = pk_from_sk(&spend_sk);
    let recipient_owner = recipient_from_pk(&domain, &pk_spend_owner, &pk_ivk_owner);
    let sender_id = recipient_owner;
    let mut sender_id_in: Hash32 = [6u8; 32];
    sender_id_in[0] = sender_id_in[0].wrapping_add(run);

    let (withdraw_amount, out_values): (u64, Vec<u64>) = match use_case {
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
        NoteSpendUseCase::Deposit => unreachable!("guarded above"),
    };

    // Transparent withdraw destination (PUBLIC). Transfers have no withdraw, so keep it canonical.
    let withdraw_to: Hash32 = match use_case {
        NoteSpendUseCase::Transfer => [0u8; 32],
        NoteSpendUseCase::Withdraw => {
            let mut to = [9u8; 32];
            to[0] = to[0].wrapping_add(run);
            to
        }
        NoteSpendUseCase::Deposit => unreachable!("guarded above"),
    };

    // Input note (single-input spend for the bench).
    let nf_key = nf_key_from_sk(&domain, &spend_sk);
    let cm_in = note_commitment(&domain, value, &rho, &recipient_owner, &sender_id_in);
    let mut tree = MerkleTree::new(depth);
    tree.set_leaf(pos as usize, cm_in);
    let anchor = tree.root();
    let siblings = tree.open(pos as usize);
    let nf = nullifier(&domain, &nf_key, &rho);

    // Outputs: (value, rho, pk_spend, pk_ivk) are private; cm_out is public.
    let mut cm_outs: Vec<Hash32> = Vec::with_capacity(n_out);
    let mut out_rhos: Vec<Hash32> = Vec::with_capacity(n_out);
    let mut out_triples: Vec<(u64, Hash32, Hash32, Hash32, Hash32, Hash32)> =
        Vec::with_capacity(n_out);
    for (j, out_value) in out_values.iter().copied().enumerate() {
        let mut out_rho: Hash32 = [7u8; 32];
        out_rho[0] = out_rho[0].wrapping_add(run);
        out_rho[1] = out_rho[1].wrapping_add(j as u8);

        let (out_pk_spend, out_pk_ivk, out_recipient) = match use_case {
            // Change outputs go back to the spender (self).
            NoteSpendUseCase::Withdraw => (pk_spend_owner, pk_ivk_owner, recipient_owner),
            NoteSpendUseCase::Transfer if j == 1 => (pk_spend_owner, pk_ivk_owner, recipient_owner),
            _ => {
                let mut out_spend_sk: Hash32 = [8u8; 32];
                out_spend_sk[0] = out_spend_sk[0].wrapping_add(run);
                out_spend_sk[1] = out_spend_sk[1].wrapping_add(j as u8);

                let pk_spend = pk_from_sk(&out_spend_sk);
                let pk_ivk = ivk_seed(&domain, &out_spend_sk);
                let recipient = recipient_from_pk(&domain, &pk_spend, &pk_ivk);
                (pk_spend, pk_ivk, recipient)
            }
        };
        let cm_out = note_commitment(&domain, out_value, &out_rho, &out_recipient, &sender_id);

        cm_outs.push(cm_out);
        out_rhos.push(out_rho);
        out_triples.push((out_value, out_rho, out_pk_spend, out_pk_ivk, out_recipient, cm_out));
    }

    let in_rhos = [rho];
    let inv_enforce = compute_inv_enforce(&[value], &in_rhos, &out_values, &out_rhos)?;

    // --- Build args for note_spend_guest v2 layout ---
    let mut args: Vec<LigeroArg> = Vec::new();
    args.push(LigeroArg::Hex { hex: hx32(&domain) }); // 1 domain (PUB)
    args.push(LigeroArg::Hex {
        hex: hx32(&spend_sk),
    }); // 2 spend_sk (PRIV)
    args.push(LigeroArg::Hex {
        hex: hx32(&pk_ivk_owner),
    }); // 3 pk_ivk_owner (PRIV)
    args.push(LigeroArg::I64 { i64: depth as i64 }); // 4 depth (PUB)
    args.push(LigeroArg::Hex { hex: hx32(&anchor) }); // 5 anchor (PUB)
    args.push(LigeroArg::I64 { i64: n_in as i64 }); // 6 n_in (PUB)

    // Input 0 (value, rho, sender_id_in, pos_bits[depth], siblings[depth], nullifier)
    args.push(LigeroArg::I64 { i64: value as i64 }); // value_in_0 (PRIV)
    args.push(LigeroArg::Hex { hex: hx32(&rho) }); // rho_in_0 (PRIV)
    args.push(LigeroArg::Hex {
        hex: hx32(&sender_id_in),
    }); // sender_id_in_0 (PRIV)
    for lvl in 0..depth {
        let bit = ((pos >> lvl) & 1) as u8;
        let mut bit_bytes = [0u8; 32];
        bit_bytes[31] = bit;
        args.push(LigeroArg::Hex {
            hex: hx32(&bit_bytes),
        }); // pos_bit_0_l
    }
    for s in &siblings {
        args.push(LigeroArg::Hex { hex: hx32(s) }); // sibling_0_l
    }
    args.push(LigeroArg::Hex { hex: hx32(&nf) }); // nullifier_0 (PUB)

    // Withdraw + outputs.
    args.push(LigeroArg::I64 {
        i64: withdraw_amount as i64,
    });
    args.push(LigeroArg::Hex {
        hex: hx32(&withdraw_to),
    });
    args.push(LigeroArg::I64 { i64: n_out as i64 });

    for (out_value, out_rho, out_pk_spend, out_pk_ivk, _out_recipient, cm_out) in out_triples.iter()
    {
        args.push(LigeroArg::I64 {
            i64: *out_value as i64,
        });
        args.push(LigeroArg::Hex {
            hex: hx32(out_rho),
        });
        args.push(LigeroArg::Hex {
            hex: hx32(out_pk_spend),
        });
        args.push(LigeroArg::Hex {
            hex: hx32(out_pk_ivk),
        });
        args.push(LigeroArg::Hex { hex: hx32(cm_out) });
    }

    // inv_enforce (PRIVATE) - witness that enforces nonzero values + rho uniqueness constraints.
    args.push(LigeroArg::Hex {
        hex: hx32(&inv_enforce),
    });

    append_empty_blacklist(&mut args, n_out);

    let mut keep_private_indices: Vec<usize> = Vec::new();
    if include_viewers {
        // Add 1 viewer attestation (fvk is private; digests are public).
        let n_viewers: u64 = 1;
        let mut fvk: Hash32 = [11u8; 32];
        fvk[0] = fvk[0].wrapping_add(run);
        let fvk_commitment = fvk_commit(&fvk);

        args.push(LigeroArg::I64 {
            i64: n_viewers as i64,
        }); // n_viewers (pub)
        args.push(LigeroArg::Hex {
            hex: hx32(&fvk_commitment),
        }); // fvk_commit (pub)

        let fvk_idx1 = args.len() + 1;
        args.push(LigeroArg::Hex { hex: hx32(&fvk) }); // fvk (priv)
        keep_private_indices.push(fvk_idx1);

        for (out_value, out_rho, _out_pk_spend, _out_pk_ivk, out_recipient, cm_out) in out_triples
        {
            let k = view_kdf(&fvk, &cm_out);
            let pt = encode_note_plain(&domain, out_value, &out_rho, &out_recipient, &sender_id);
            let ct = stream_xor_encrypt_144(&k, &pt);
            let ct_h = ct_hash(&ct);
            let mac = view_mac(&k, &cm_out, &ct_h);

            args.push(LigeroArg::Hex { hex: hx32(&ct_h) }); // ct_hash (pub)
            args.push(LigeroArg::Hex { hex: hx32(&mac) }); // mac (pub)
        }
    }

    Ok((
        args,
        NoteSpendPublicSummary {
            anchor,
            nullifiers: vec![nf],
            cm_outs,
            withdraw_amount,
            n_in,
            n_out,
        },
        keep_private_indices,
    ))
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
        eprintln!(
            "Skipping: shader path not found at {}",
            shader_dir.display()
        );
        return Ok(());
    }

    println!("[daemon] Prover:   {}", runner.paths().prover_bin.display());
    println!(
        "[daemon] Verifier: {}",
        runner.paths().verifier_bin.display()
    );
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
    let mut public_summaries: Vec<NoteSpendPublicSummary> = Vec::new();

    for run in 0..3u8 {
        let (args, summary, _keep_priv_idx) =
            build_statement(run, depth, domain, value, pos, use_case, false)?;
        runner.config_mut().private_indices =
            private_indices_note_spend(depth, summary.n_in, summary.n_out);
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
        eprintln!(
            "Skipping: shader path not found at {}",
            shader_dir.display()
        );
        return Ok(());
    }

    let label = use_case.label();
    let num_runs = read_num_runs();

    // Preview run0 statement so the banner can show the exact values used by this test.
    let include_viewers =
        enable_viewers() && matches!(use_case, NoteSpendUseCase::Transfer | NoteSpendUseCase::Withdraw);

    let (run0_args, run0_priv_idx, run0_keep_priv_idx, run0_summary) = match use_case {
        NoteSpendUseCase::Deposit => {
            let args = build_deposit_statement(0, domain, value)?;
            (args, private_indices_note_deposit(), Vec::new(), None)
        }
        NoteSpendUseCase::Transfer | NoteSpendUseCase::Withdraw => {
            let (args, summary, keep_priv_idx) =
                build_statement(0, depth, domain, value, pos, use_case, include_viewers)?;
            (
                args,
                private_indices_note_spend(depth, summary.n_in, summary.n_out),
                keep_priv_idx,
                Some(summary),
            )
        }
    };

    // Print use case banner
    println!();
    let mut banner_lines: Vec<String> = Vec::new();
    match use_case {
        NoteSpendUseCase::Deposit => {
            let rho = arg_hash32(&run0_args, 3)?;
            let pk_spend_recipient = arg_hash32(&run0_args, 4)?;
            let pk_ivk_recipient = arg_hash32(&run0_args, 5)?;
            let recipient = recipient_from_pk(&domain, &pk_spend_recipient, &pk_ivk_recipient);
            let cm_out = arg_hash32(&run0_args, 6)?;

            banner_lines.push(format!(
                "Run0: domain={}, value(pub)={}",
                hx32_short(&domain),
                value
            ));
            banner_lines.push(format!(
                "Run0 destination(priv): recipient={}",
                hx32_short(&recipient),
            ));
            banner_lines.push(format!(
                "Run0 recipient keys(priv): pk_spend={}, pk_ivk={}",
                hx32_short(&pk_spend_recipient),
                hx32_short(&pk_ivk_recipient),
            ));
            banner_lines.push(format!("Run0 private rho={}", hx32_short(&rho)));
            banner_lines.push(format!("Public: cm_out={}", hx32_short(&cm_out)));
            banner_lines.push("Note: transparent origin is outside this proof".to_string());
            banner_lines.push("Note: sender_id is fixed to 0x00..00".to_string());

            print_box("💰 DEPOSIT - Enter the privacy pool", &banner_lines);
        }
        NoteSpendUseCase::Transfer => {
            let summary = run0_summary
                .clone()
                .context("missing run0 summary for transfer")?;

            let spend_sk = arg_hash32(&run0_args, 2)?;
            let pk_ivk_owner = arg_hash32(&run0_args, 3)?;
            let sender = recipient_from_pk(&domain, &pk_from_sk(&spend_sk), &pk_ivk_owner);

            let per_in = 4 + 2 * depth;
            let withdraw_idx = 7 + summary.n_in * per_in;
            let out_base = withdraw_idx + 3;

            let out0_value = arg_u64(&run0_args, out_base)?;
            let out0_pk_spend = arg_hash32(&run0_args, out_base + 2)?;
            let out0_pk_ivk = arg_hash32(&run0_args, out_base + 3)?;
            let out0_recipient = recipient_from_pk(&domain, &out0_pk_spend, &out0_pk_ivk);

            let out1_value = arg_u64(&run0_args, out_base + 5)?;
            let out1_pk_spend = arg_hash32(&run0_args, out_base + 7)?;
            let out1_pk_ivk = arg_hash32(&run0_args, out_base + 8)?;
            let out1_recipient = recipient_from_pk(&domain, &out1_pk_spend, &out1_pk_ivk);

            banner_lines.push(format!(
                "Run0: domain={}, depth(pub)={depth}, n_in(pub)={}, pos0(priv)={pos}, in0_value(priv)={value}",
                hx32_short(&domain),
                summary.n_in
            ));
            banner_lines.push(
                "Note: PRIVATE args are redacted in the args table and verifier output".to_string(),
            );
            banner_lines.push(format!("Run0 origin(priv): sender={}", hx32_short(&sender)));

            if out1_recipient == sender {
                banner_lines.push(format!(
                    "Run0 destinations(priv): pay {out0_value}→{}, change {out1_value}→self({})",
                    hx32_short(&out0_recipient),
                    hx32_short(&sender),
                ));
            } else {
                banner_lines.push(format!(
                    "Run0 destinations(priv): pay {out0_value}→{}, change {out1_value}→{}",
                    hx32_short(&out0_recipient),
                    hx32_short(&out1_recipient),
                ));
            }

            let withdraw_to = arg_hash32(&run0_args, withdraw_idx + 1)?;
            banner_lines.push(format!(
                "Public: withdraw_amount(pub)={}, withdraw_to(pub)={}, n_out(pub)={}",
                summary.withdraw_amount,
                hx32_short(&withdraw_to),
                summary.n_out
            ));
            banner_lines.push(format!(
                "Public: anchor={}, nf0={}",
                hx32_short(&summary.anchor),
                hx32_short(
                    summary
                        .nullifiers
                        .first()
                        .context("missing nullifier in summary")?
                )
            ));
            banner_lines.push(format!(
                "Public: cm_out0={}, cm_out1={}",
                hx32_short(
                    summary
                        .cm_outs
                        .first()
                        .context("missing cm_out0 in summary")?
                ),
                hx32_short(
                    summary
                        .cm_outs
                        .get(1)
                        .context("missing cm_out1 in summary")?
                )
            ));
            if include_viewers {
                let idx_n_viewers = out_base + 5 * summary.n_out + 2 + BL_DEPTH * (1 + summary.n_out);
                let n_viewers = arg_u64(&run0_args, idx_n_viewers)?;
                let fvk_commit = arg_hash32(&run0_args, idx_n_viewers + 1)?;
                banner_lines.push(format!(
                    "Viewer(pub): n_viewers={n_viewers}, fvk_commit={}",
                    hx32_short(&fvk_commit)
                ));
            }
            if num_runs > 1 {
                banner_lines.push("Other runs vary secrets for binding test".to_string());
            }

            print_box("🔒 TRANSFER - Fully private transaction", &banner_lines);
        }
        NoteSpendUseCase::Withdraw => {
            let summary = run0_summary
                .clone()
                .context("missing run0 summary for withdraw")?;

            let spend_sk = arg_hash32(&run0_args, 2)?;
            let pk_ivk_owner = arg_hash32(&run0_args, 3)?;
            let sender = recipient_from_pk(&domain, &pk_from_sk(&spend_sk), &pk_ivk_owner);

            let per_in = 4 + 2 * depth;
            let withdraw_idx = 7 + summary.n_in * per_in;
            let out_base = withdraw_idx + 3;

            let change_value = arg_u64(&run0_args, out_base)?;
            let change_pk_spend = arg_hash32(&run0_args, out_base + 2)?;
            let change_pk_ivk = arg_hash32(&run0_args, out_base + 3)?;
            let change_recipient = recipient_from_pk(&domain, &change_pk_spend, &change_pk_ivk);

            banner_lines.push(format!(
                "Run0: domain={}, depth(pub)={depth}, n_in(pub)={}, pos0(priv)={pos}",
                hx32_short(&domain),
                summary.n_in
            ));
            banner_lines.push(
                "Note: PRIVATE args are redacted in the args table and verifier output".to_string(),
            );
            banner_lines.push(format!(
                "Run0: in0(priv)={value}, withdraw(pub)={}, change(priv)={change_value}",
                summary.withdraw_amount
            ));
            let withdraw_to = arg_hash32(&run0_args, withdraw_idx + 1)?;
            banner_lines.push(format!(
                "Public: withdraw_to={}",
                hx32_short(&withdraw_to)
            ));
            banner_lines.push(format!("Run0 origin(priv): sender={}", hx32_short(&sender)));

            if change_recipient == sender {
                banner_lines.push(format!(
                    "Run0 change destination(priv): {change_value}→self({})",
                    hx32_short(&sender),
                ));
            } else {
                banner_lines.push(format!(
                    "Run0 change destination(priv): {change_value}→{}",
                    hx32_short(&change_recipient)
                ));
            }
            banner_lines.push(format!(
                "Public: anchor={}, nf0={}, n_out(pub)={}",
                hx32_short(&summary.anchor),
                hx32_short(
                    summary
                        .nullifiers
                        .first()
                        .context("missing nullifier in summary")?
                ),
                summary.n_out
            ));
            banner_lines.push(format!(
                "Public: cm_out0={}",
                hx32_short(
                    summary
                        .cm_outs
                        .first()
                        .context("missing cm_out0 in summary")?
                )
            ));
            if include_viewers {
                let idx_n_viewers = out_base + 5 * summary.n_out + 2 + BL_DEPTH * (1 + summary.n_out);
                let n_viewers = arg_u64(&run0_args, idx_n_viewers)?;
                let fvk_commit = arg_hash32(&run0_args, idx_n_viewers + 1)?;
                banner_lines.push(format!(
                    "Viewer(pub): n_viewers={n_viewers}, fvk_commit={}",
                    hx32_short(&fvk_commit)
                ));
            }
            if num_runs > 1 {
                banner_lines.push("Other runs vary secrets for binding test".to_string());
            }

            print_box("💸 WITHDRAW - Exit the privacy pool", &banner_lines);
        }
    }
    println!();

    println!(
        "[{label}] Prover:   {}",
        runner.paths().prover_bin.display()
    );
    println!(
        "[{label}] Verifier: {}",
        runner.paths().verifier_bin.display()
    );
    println!("[{label}] Shaders:  {}", shader_dir.display());
    println!("[{label}] Program:  {}", program.display());
    println!("[{label}] Packing:  {}", packing);
    println!("[{label}] Gzip:     {}", gzip_proof);

    // Prove N distinct proofs (controlled by LIGERO_RUNS env var, default 3).
    println!("[{label}] Running {} proof(s)", num_runs);
    let mut proofs: Vec<Vec<u8>> = Vec::new();
    let mut configs: Vec<Vec<LigeroArg>> = Vec::new();
    let mut all_priv_idx: Vec<Vec<usize>> = Vec::new();
    let mut all_keep_priv_idx: Vec<Vec<usize>> = Vec::new();
    let mut last_prover_stdout = String::new();
    let mut last_prover_time = Duration::default();
    let mut last_proof_size = 0usize;

    for run in 0..num_runs {
        let (args, mut priv_idx, keep_priv_idx) = if run == 0 {
            (
                run0_args.clone(),
                run0_priv_idx.clone(),
                run0_keep_priv_idx.clone(),
            )
        } else {
            match use_case {
                NoteSpendUseCase::Deposit => {
                    let args = build_deposit_statement(run, domain, value)?;
                    let priv_idx = private_indices_note_deposit();
                    (args, priv_idx, Vec::new())
                }
                NoteSpendUseCase::Transfer | NoteSpendUseCase::Withdraw => {
                    let (args, summary, keep_priv_idx) =
                        build_statement(run, depth, domain, value, pos, use_case, include_viewers)?;
                    let priv_idx = private_indices_note_spend(depth, summary.n_in, summary.n_out);
                    (args, priv_idx, keep_priv_idx)
                }
            }
        };

        // Some circuit extensions require a subset of private inputs to be present for the verifier
        // binary (e.g. viewer attestations need `fvk` to reconstruct the same instance).
        priv_idx.extend_from_slice(&keep_priv_idx);
        priv_idx.sort_unstable();
        priv_idx.dedup();
        runner.config_mut().private_indices = priv_idx.clone();
        runner.config_mut().args = args.clone();

        // Print labeled arguments for first run when verbose
        if run == 0 && is_verbose() {
            print_labeled_args(&args, use_case);
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
        all_keep_priv_idx.push(keep_priv_idx);
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

    for (i, (((proof, args), priv_idx), keep_priv_idx)) in proofs
        .iter()
        .zip(configs.iter())
        .zip(all_priv_idx.iter())
        .zip(all_keep_priv_idx.iter())
        .enumerate()
    {
        let tv = Instant::now();
        let (ok, vs, ve) = verifier::verify_proof_with_output_keep_private_args_in_pool(
            &verifier_pool,
            &vpaths,
            proof,
            args.clone(),
            priv_idx.clone(),
            keep_priv_idx.clone(),
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
            prover_metrics.stage1_ms, prover_metrics.stage2_ms, prover_metrics.stage3_ms
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
                match verifier::verify_proof_with_output_keep_private_args_in_pool(
                    &verifier_pool,
                    &vpaths,
                    &proofs[i],
                    configs[j].clone(),
                    all_priv_idx[j].clone(),
                    all_keep_priv_idx[j].clone(),
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
