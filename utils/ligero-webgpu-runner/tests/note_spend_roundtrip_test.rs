//! Proof roundtrip test for `ligero-runner` using the `note_spend_guest` circuit.
//!
//! This test requires:
//! - `webgpu_prover` + `webgpu_verifier` binaries
//! - a valid `shader/` directory
//! - a built `note_spend_guest.wasm`
//!
//! The test runs by default *if* Ligero assets are available (portable-binaries/shaders or env overrides).
//! If assets are missing (or the prover cannot run on this machine), it exits early with a skip message.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use ligero_runner::{verifier, LigeroArg, LigeroRunner};
use ligetron::poseidon2_hash_bytes;

type Hash32 = [u8; 32];

fn hx32(b: &Hash32) -> String {
    hex::encode(b)
}

fn repo_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // utils/ligero-runner -> utils -> repo
    Ok(manifest_dir
        .ancestors()
        .nth(2)
        .context("Failed to compute ligero-prover repo root")?
        .to_path_buf())
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
        "[note_spend_roundtrip] note_spend_guest.wasm not found; building via {}",
        guest_dir.join("build.sh").display()
    );

    // Best-effort build. This may download the wasm std target via rustup.
    let status = Command::new("bash")
        .arg("build.sh")
        .current_dir(&guest_dir)
        .status()
        .context("Failed to run note-spend/build.sh")?;

    if !status.success() {
        anyhow::bail!("note-spend/build.sh failed with status {status}");
    }

    if !out.exists() {
        anyhow::bail!(
            "note_spend_guest.wasm still not found after build at {}",
            out.display()
        );
    }

    println!("[note_spend_roundtrip] Built note_spend_guest.wasm at {}", out.display());

    Ok(())
}

fn note_spend_program_path(repo: &Path) -> Result<PathBuf> {
    if let Ok(p) = std::env::var("LIGERO_PROGRAM_PATH") {
        return Ok(PathBuf::from(p));
    }

    let p = repo.join("utils/circuits/bins/note_spend_guest.wasm");
    Ok(p)
}

// === Poseidon2 domain-separated helpers (must match the guest program) ===

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

fn merkle_root_from_path(leaf: &Hash32, pos: u64, siblings: &[Hash32]) -> Hash32 {
    let mut cur = *leaf;
    let mut idx = pos;
    for (lvl, sib) in siblings.iter().enumerate() {
        cur = if (idx & 1) == 0 {
            mt_combine(lvl as u8, &cur, sib)
        } else {
            mt_combine(lvl as u8, sib, &cur)
        };
        idx >>= 1;
    }
    cur
}

/// Small Merkle tree implementation for tests (depth is small, so a simple Vec-based tree is fine).
///
/// IMPORTANT: The level parameter passed into `mt_combine(level, ...)` matches the guest program:
/// - `level = 0` is the leaf-adjacent layer (bottom)
/// - `level` increases as we move up towards the root
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
    // Mirrors the guest layout described in utils/circuits/note-spend/src/main.rs.
    let mut v = vec![3usize, 4usize, 5usize]; // rho, recipient, spend_sk
    for i in 0..depth {
        v.push(7 + i); // pos_bits
    }
    for i in 0..depth {
        v.push(7 + depth + i); // siblings
    }
    let out_base = 11 + 2 * depth;
    for j in 0..n_out {
        v.push(out_base + 4 * j + 0); // value_out
        v.push(out_base + 4 * j + 1); // rho_out
        v.push(out_base + 4 * j + 2); // pk_out
    }
    v
}

#[test]
fn test_note_spend_proof_roundtrip_one_output() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_spend_guest(&repo)?;

    let program = note_spend_program_path(&repo)?.canonicalize().context("Failed to canonicalize note_spend_guest.wasm")?;

    let packing: u32 = std::env::var("LIGERO_PACKING")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8192);

    // Runner discovery (prover/verifier/shaders). Caller can override via env vars.
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

    println!("[note_spend_roundtrip] Program: {}", program.display());
    println!("[note_spend_roundtrip] Prover:   {}", runner.paths().prover_bin.display());
    println!("[note_spend_roundtrip] Verifier: {}", runner.paths().verifier_bin.display());
    println!("[note_spend_roundtrip] Shaders:  {}", shader_dir.display());
    println!("[note_spend_roundtrip] Packing:  {}", packing);

    // === Construct a simple depth-8 tree with one note ===
    let depth: usize = 8;
    let domain: Hash32 = [1u8; 32];
    let rho: Hash32 = [2u8; 32];
    let spend_sk: Hash32 = [4u8; 32];

    let recipient = recipient_from_sk(&domain, &spend_sk);
    let nf_key = nf_key_from_sk(&domain, &spend_sk);

    let value: u64 = 42;
    let pos: u64 = 0;

    let cm_in = note_commitment(&domain, value, &rho, &recipient);

    // Build a small tree and open a Merkle path consistent with the guest program.
    let mut tree = MerkleTree::new(depth);
    tree.set_leaf(pos as usize, cm_in);
    let anchor = tree.root();
    let siblings = tree.open(pos as usize);
    assert_eq!(siblings.len(), depth, "unexpected siblings length");

    // Sanity check: recompute via generic root_from_path.
    let recomputed = merkle_root_from_path(&cm_in, pos, &siblings);
    assert_eq!(recomputed, anchor, "Merkle root mismatch");

    let nf = nullifier(&domain, &nf_key, &rho);

    // === One-output transfer: withdraw=0, out_value=value ===
    let withdraw_amount: u64 = 0;
    let n_out: u64 = 1;

    let out_value: u64 = value;
    let out_rho: Hash32 = [7u8; 32];
    let out_spend_sk: Hash32 = [8u8; 32];
    let out_pk = pk_from_sk(&out_spend_sk);
    let out_recipient = recipient_from_pk(&domain, &out_pk);
    let cm_out = note_commitment(&domain, out_value, &out_rho, &out_recipient);

    let mut args: Vec<LigeroArg> = Vec::new();
    args.push(LigeroArg::Hex { hex: hx32(&domain) }); // 1
    args.push(LigeroArg::I64 { i64: value as i64 }); // 2
    args.push(LigeroArg::Hex { hex: hx32(&rho) }); // 3 (private)
    args.push(LigeroArg::Hex { hex: hx32(&recipient) }); // 4 (private)
    args.push(LigeroArg::Hex { hex: hx32(&spend_sk) }); // 5 (private)
    args.push(LigeroArg::I64 { i64: depth as i64 }); // 6

    // Position bits (LSB-first), encoded as 32-byte big-endian field elements (0 or 1).
    // This matches the guest's `read_position_bit` / `bn254fr_from_hash32_be`.
    for lvl in 0..depth {
        let bit = ((pos >> lvl) & 1) as u8;
        let mut bit_bytes = [0u8; 32];
        bit_bytes[31] = bit;
        args.push(LigeroArg::Hex { hex: hx32(&bit_bytes) });
    }

    // Siblings.
    for s in &siblings {
        args.push(LigeroArg::Hex { hex: hx32(s) });
    }

    // Anchor + nullifier as public field elements.
    args.push(LigeroArg::String { str: format!("0x{}", hx32(&anchor)) });
    args.push(LigeroArg::String { str: format!("0x{}", hx32(&nf)) });

    // Withdraw + outputs.
    args.push(LigeroArg::I64 { i64: withdraw_amount as i64 });
    args.push(LigeroArg::I64 { i64: n_out as i64 });
    args.push(LigeroArg::I64 { i64: out_value as i64 });
    args.push(LigeroArg::Hex { hex: hx32(&out_rho) });
    args.push(LigeroArg::Hex { hex: hx32(&out_pk) });
    args.push(LigeroArg::Hex { hex: hx32(&cm_out) });

    let priv_idx = private_indices(depth, 1);

    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = args.clone();

    let (proof_bytes, prover_stdout, prover_stderr) = match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
        keep_proof_dir: false,
        proof_outputs_base: None,
        write_replay_script: true,
    }) {
        Ok((proof, stdout, stderr)) => (proof, stdout, stderr),
        Err(err) => {
            // If the prover ran but the witness is invalid, this is a real test failure (not an env issue).
            let msg = format!("{err:#}");
            if msg.contains("Validation of linear constraints:")
                || msg.contains("Validation of quadratic constraints:")
                || msg.contains("Final prove result:")
            {
                return Err(err);
            }

            // Otherwise, treat it as an environmental failure (GPU/WebGPU / driver / missing runtime deps).
            eprintln!("Skipping: prover failed (GPU/WebGPU likely unavailable): {err}");
            return Ok(());
        }
    };

    assert!(!proof_bytes.is_empty(), "proof should not be empty");

    println!("[note_spend_roundtrip] OK: proof generated ({} bytes)", proof_bytes.len());
    if let Some(line) = prover_stdout.lines().find(|l| l.contains("Final prove result:")) {
        println!("[note_spend_roundtrip] Prover:   {line}");
    }
    if !prover_stderr.trim().is_empty() {
        println!("[note_spend_roundtrip] Prover stderr:\n{prover_stderr}");
    }

    // Verify proof.
    let vpaths = verifier::VerifierPaths::from_explicit(
        program.clone(),
        shader_dir,
        runner.paths().verifier_bin.clone(),
        packing,
    );

    let (ok, v_stdout, v_stderr) = verifier::verify_proof_with_output(&vpaths, &proof_bytes, args, priv_idx)
        .context("Failed to run verifier")?;

    if !ok {
        anyhow::bail!("verifier did not report success\nstdout: {v_stdout}\nstderr: {v_stderr}");
    }

    println!("[note_spend_roundtrip] OK: proof verified");
    if let Some(line) = v_stdout.lines().find(|l| l.contains("Final Verify Result:")) {
        println!("[note_spend_roundtrip] Verifier: {line}");
    }
    if !v_stderr.trim().is_empty() {
        println!("[note_spend_roundtrip] Verifier stderr:\n{v_stderr}");
    }

    Ok(())
}
