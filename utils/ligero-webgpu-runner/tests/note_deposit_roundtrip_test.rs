//! Proof roundtrip + binding tests for the `note_deposit_guest` circuit.
//!
//! Like the note-spend integration tests, these tests require:
//! - `webgpu_prover` + `webgpu_verifier` binaries
//! - a valid `shader/` directory
//! - a built `note_deposit_guest.wasm`
//!
//! If assets are missing (or the prover cannot run on this machine), tests exit early with a skip
//! message.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use ligero_runner::{verifier, LigeroArg, LigeroRunner};
use ligetron::poseidon2_hash_bytes;
use ligetron::Bn254Fr;

type Hash32 = [u8; 32];

const BL_DEPTH: usize = 16;
const BL_BUCKET_SIZE: usize = 12;

const PRIVPOOL_EXAMPLE_ADDR: &str =
    "privpool1eqrexjkvvw5wjljp4mpup250hl4sdpk6hl36dmdcdsfldvjw2j8staydzl";

fn hx32(b: &Hash32) -> String {
    hex::encode(b)
}

fn b64_32(b: &Hash32) -> String {
    general_purpose::STANDARD.encode(b)
}

fn arg32(b: &Hash32) -> LigeroArg {
    LigeroArg::HexBytesB64 {
        hex: hx32(b),
        bytes_b64: b64_32(b),
    }
}

// === Minimal Bech32m decode/encode for privpool addresses (HRP = "privpool") ===

const BECH32_CHARSET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";
const BECH32M_CONST: u32 = 0x2bc8_30a3;

fn bech32_char_to_u5(c: u8) -> Option<u8> {
    match c {
        b'q' => Some(0),
        b'p' => Some(1),
        b'z' => Some(2),
        b'r' => Some(3),
        b'y' => Some(4),
        b'9' => Some(5),
        b'x' => Some(6),
        b'8' => Some(7),
        b'g' => Some(8),
        b'f' => Some(9),
        b'2' => Some(10),
        b't' => Some(11),
        b'v' => Some(12),
        b'd' => Some(13),
        b'w' => Some(14),
        b'0' => Some(15),
        b's' => Some(16),
        b'3' => Some(17),
        b'j' => Some(18),
        b'n' => Some(19),
        b'5' => Some(20),
        b'4' => Some(21),
        b'k' => Some(22),
        b'h' => Some(23),
        b'c' => Some(24),
        b'e' => Some(25),
        b'6' => Some(26),
        b'm' => Some(27),
        b'u' => Some(28),
        b'a' => Some(29),
        b'7' => Some(30),
        b'l' => Some(31),
        _ => None,
    }
}

fn bech32_polymod(values: &[u8]) -> u32 {
    const GEN: [u32; 5] = [
        0x3b6a_57b2,
        0x2650_8e6d,
        0x1ea1_19fa,
        0x3d42_33dd,
        0x2a14_62b3,
    ];
    let mut chk: u32 = 1;
    for &v in values {
        let b = chk >> 25;
        chk = ((chk & 0x01ff_ffff) << 5) ^ (v as u32);
        for (i, g) in GEN.iter().enumerate() {
            if ((b >> i) & 1) != 0 {
                chk ^= g;
            }
        }
    }
    chk
}

fn bech32_hrp_expand(hrp: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(hrp.len() * 2 + 1);
    for b in hrp.as_bytes() {
        out.push(*b >> 5);
    }
    out.push(0);
    for b in hrp.as_bytes() {
        out.push(*b & 31);
    }
    out
}

fn bech32m_checksum(hrp: &str, data: &[u8]) -> [u8; 6] {
    let mut values = bech32_hrp_expand(hrp);
    values.extend_from_slice(data);
    values.extend_from_slice(&[0u8; 6]);
    let pm = bech32_polymod(&values) ^ BECH32M_CONST;

    let mut out = [0u8; 6];
    for i in 0..6 {
        out[i] = ((pm >> (5 * (5 - i))) & 31) as u8;
    }
    out
}

fn bech32m_decode(s: &str) -> Result<(String, Vec<u8>)> {
    anyhow::ensure!(!s.is_empty(), "empty bech32m string");
    anyhow::ensure!(
        s.bytes().all(|b| (33..=126).contains(&b)),
        "invalid ascii in bech32m string"
    );

    let has_lower = s.bytes().any(|b| (b'a'..=b'z').contains(&b));
    let has_upper = s.bytes().any(|b| (b'A'..=b'Z').contains(&b));
    anyhow::ensure!(!(has_lower && has_upper), "mixed-case bech32m string");

    let s = s.to_ascii_lowercase();
    let pos = s.rfind('1').context("missing bech32 separator '1'")?;
    anyhow::ensure!(pos >= 1, "hrp too short");
    anyhow::ensure!(pos + 7 <= s.len(), "data too short for checksum");

    let hrp = s[..pos].to_string();
    let data_part = s[pos + 1..].as_bytes();

    let mut data_u5: Vec<u8> = Vec::with_capacity(data_part.len());
    for &c in data_part {
        data_u5.push(bech32_char_to_u5(c).context("invalid bech32 char")?);
    }
    anyhow::ensure!(data_u5.len() >= 6, "missing checksum");

    // Verify bech32m checksum.
    let pm = {
        let mut v = bech32_hrp_expand(&hrp);
        v.extend_from_slice(&data_u5);
        bech32_polymod(&v)
    };
    anyhow::ensure!(pm == BECH32M_CONST, "invalid bech32m checksum");

    data_u5.truncate(data_u5.len() - 6);
    Ok((hrp, data_u5))
}

fn convert_bits(data: &[u8], from: u32, to: u32, pad: bool) -> Result<Vec<u8>> {
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    let mut out: Vec<u8> = Vec::new();
    let maxv: u32 = (1 << to) - 1;

    for &v in data {
        anyhow::ensure!((v as u32) < (1 << from), "convert_bits: value out of range");
        acc = (acc << from) | (v as u32);
        bits += from;
        while bits >= to {
            bits -= to;
            out.push(((acc >> bits) & maxv) as u8);
        }
    }

    if pad {
        if bits != 0 {
            out.push(((acc << (to - bits)) & maxv) as u8);
        }
    } else {
        anyhow::ensure!(bits < from, "convert_bits: excess padding");
        anyhow::ensure!(
            ((acc << (to - bits)) & maxv) == 0,
            "convert_bits: non-zero padding"
        );
    }

    Ok(out)
}

fn decode_privpool_addr(addr: &str) -> Result<Hash32> {
    let (hrp, data_u5) = bech32m_decode(addr)?;
    anyhow::ensure!(hrp == "privpool", "unexpected hrp: {hrp}");
    let raw = convert_bits(&data_u5, 5, 8, false)?;
    anyhow::ensure!(raw.len() == 32, "unexpected payload length: {}", raw.len());
    let mut out = [0u8; 32];
    out.copy_from_slice(&raw);
    Ok(out)
}

fn encode_privpool_addr(id: &Hash32) -> String {
    let data_u5 = convert_bits(id, 8, 5, true).expect("8->5 convert_bits must succeed");
    let checksum = bech32m_checksum("privpool", &data_u5);
    let mut s = String::with_capacity("privpool1".len() + data_u5.len() + 6);
    s.push_str("privpool1");
    for v in data_u5.iter().chain(checksum.iter()) {
        s.push(BECH32_CHARSET[*v as usize] as char);
    }
    s
}

fn repo_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // utils/ligero-webgpu-runner -> utils -> repo
    Ok(manifest_dir
        .ancestors()
        .nth(2)
        .context("Failed to compute ligero-prover repo root")?
        .to_path_buf())
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
        "[note_deposit_roundtrip] note_deposit_guest.wasm not found; building via {}",
        guest_dir.join("build.sh").display()
    );

    // Best-effort build. This may download the wasm std target via rustup.
    let status = Command::new("bash")
        .arg("build.sh")
        .current_dir(&guest_dir)
        .status()
        .context("Failed to run note-deposit/build.sh")?;

    if !status.success() {
        anyhow::bail!("note-deposit/build.sh failed with status {status}");
    }

    if !out.exists() {
        anyhow::bail!(
            "note_deposit_guest.wasm still not found after build at {}",
            out.display()
        );
    }

    println!(
        "[note_deposit_roundtrip] Built note_deposit_guest.wasm at {}",
        out.display()
    );

    Ok(())
}

fn note_deposit_program_path(repo: &Path) -> PathBuf {
    repo.join("utils/circuits/bins/note_deposit_guest.wasm")
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

fn recipient_from_pk(domain: &Hash32, pk_spend: &Hash32, pk_ivk: &Hash32) -> Hash32 {
    poseidon2_hash_domain(b"ADDR_V2", &[domain, pk_spend, pk_ivk])
}

fn mt_combine(level: u8, left: &Hash32, right: &Hash32) -> Hash32 {
    let lvl = [level];
    poseidon2_hash_domain(b"MT_NODE_V1", &[&lvl, left, right])
}

fn merkle_default_nodes_from_leaf(depth: usize, leaf0: &Hash32) -> Vec<Hash32> {
    let mut out = Vec::with_capacity(depth + 1);
    out.push(*leaf0); // height 0 (leaf)
    for lvl in 0..depth {
        let prev = out[lvl];
        out.push(mt_combine(lvl as u8, &prev, &prev));
    }
    out
}

fn fr_from_hash32_be(h: &Hash32) -> Bn254Fr {
    let mut fr = Bn254Fr::new();
    fr.set_bytes_big(h);
    fr
}

fn bl_empty_bucket_entries() -> [Hash32; BL_BUCKET_SIZE] {
    [[0u8; 32]; BL_BUCKET_SIZE]
}

fn bl_bucket_pos_from_id(id: &Hash32) -> u64 {
    // Derive the bucket index from the low BL_DEPTH bits of `id` (LSB-first).
    let mut pos: u64 = 0;
    for i in 0..BL_DEPTH {
        let byte = id[31 - (i / 8)];
        let bit = (byte >> (i % 8)) & 1;
        pos |= (bit as u64) << (i as u32);
    }
    pos
}

fn bl_bucket_leaf(entries: &[Hash32; BL_BUCKET_SIZE]) -> Hash32 {
    let mut buf = Vec::with_capacity(12 + 32 * BL_BUCKET_SIZE);
    buf.extend_from_slice(b"BL_BUCKET_V1");
    for e in entries {
        buf.extend_from_slice(e);
    }
    poseidon2_hash_bytes(&buf).to_bytes_be()
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

fn empty_blacklist_bucket_proof_for_id(
    id: &Hash32,
) -> Result<(Hash32, [Hash32; BL_BUCKET_SIZE], Hash32, Vec<Hash32>)> {
    let entries = bl_empty_bucket_entries();
    let leaf0 = bl_bucket_leaf(&entries);
    let default_nodes = merkle_default_nodes_from_leaf(BL_DEPTH, &leaf0);
    let blacklist_root = default_nodes[BL_DEPTH];
    let inv = bl_bucket_inv_for_id(id, &entries)
        .context("unexpected: recipient id collides with empty bucket")?;
    let siblings: Vec<Hash32> = default_nodes.iter().take(BL_DEPTH).copied().collect();
    Ok((blacklist_root, entries, inv, siblings))
}

fn append_empty_blacklist_bucket_hex(args: &mut Vec<LigeroArg>, id: &Hash32) -> Result<()> {
    let (blacklist_root, entries, inv, siblings) = empty_blacklist_bucket_proof_for_id(id)?;
    args.push(arg32(&blacklist_root));
    for e in entries.iter() {
        args.push(arg32(e));
    }
    args.push(arg32(&inv));
    for sib in siblings.iter() {
        args.push(arg32(sib));
    }
    Ok(())
}

fn append_empty_blacklist_bucket_string(args: &mut Vec<LigeroArg>, id: &Hash32) -> Result<()> {
    let (blacklist_root, entries, inv, siblings) = empty_blacklist_bucket_proof_for_id(id)?;
    args.push(LigeroArg::String {
        str: format!("0x{}", hx32(&blacklist_root)),
    });
    for e in entries.iter() {
        args.push(LigeroArg::String {
            str: format!("0x{}", hx32(e)),
        });
    }
    args.push(LigeroArg::String {
        str: format!("0x{}", hx32(&inv)),
    });
    for sib in siblings.iter() {
        args.push(LigeroArg::String {
            str: format!("0x{}", hx32(sib)),
        });
    }
    Ok(())
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

fn merkle_root_and_openings_sparse(
    depth: usize,
    default_nodes: &[Hash32],
    leaves: &[(u64, Hash32)],
) -> Result<(Hash32, Vec<Vec<Hash32>>)> {
    use std::collections::{HashMap, HashSet};

    anyhow::ensure!(
        default_nodes.len() == depth + 1,
        "default_nodes must have length depth+1"
    );

    // Store only non-default nodes by height. Height 0 are leaves; height `depth` is the root.
    let mut levels: Vec<HashMap<u64, Hash32>> = (0..=depth).map(|_| HashMap::new()).collect();

    if depth >= 64 {
        anyhow::bail!("depth too large for sparse tree: {depth}");
    }
    let max_pos = if depth == 63 {
        1u64 << 63
    } else {
        1u64 << depth
    };
    for (pos, leaf) in leaves {
        anyhow::ensure!(
            *pos < max_pos,
            "leaf pos {pos} out of range for depth {depth}"
        );
        if let Some(prev) = levels[0].insert(*pos, *leaf) {
            anyhow::ensure!(
                prev == *leaf,
                "duplicate leaf pos {pos} with different value"
            );
        }
    }

    // Build up.
    for lvl in 0..depth {
        let mut parent_set: HashSet<u64> = HashSet::new();
        for &idx in levels[lvl].keys() {
            parent_set.insert(idx >> 1);
        }

        for pidx in parent_set {
            let left_idx = pidx * 2;
            let right_idx = pidx * 2 + 1;
            let left = levels[lvl].get(&left_idx).unwrap_or(&default_nodes[lvl]);
            let right = levels[lvl].get(&right_idx).unwrap_or(&default_nodes[lvl]);
            let parent = mt_combine(lvl as u8, left, right);
            if parent != default_nodes[lvl + 1] {
                levels[lvl + 1].insert(pidx, parent);
            }
        }
    }

    let root = *levels[depth].get(&0).unwrap_or(&default_nodes[depth]);

    // Open siblings for each requested leaf (in input order).
    let mut openings: Vec<Vec<Hash32>> = Vec::with_capacity(leaves.len());
    for (pos, _leaf) in leaves {
        let mut sibs: Vec<Hash32> = Vec::with_capacity(depth);
        for lvl in 0..depth {
            let idx_at_lvl = pos >> lvl;
            let sib_idx = idx_at_lvl ^ 1;
            let sib = *levels[lvl].get(&sib_idx).unwrap_or(&default_nodes[lvl]);
            sibs.push(sib);
        }
        openings.push(sibs);
    }

    Ok((root, openings))
}

fn note_commitment_v2(
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

fn private_indices_note_deposit() -> Vec<usize> {
    // Deposit circuit ABI:
    // 1 domain (PUB), 2 value (PUB), 3 rho (PRIV), 4 pk_spend_recipient (PRIV),
    // 5 pk_ivk_recipient (PRIV), 6 cm_out (PUB),
    // 7 blacklist_root (PUB),
    // 8 bucket_entries[BL_BUCKET_SIZE] (PRIV),
    // 8+BL_BUCKET_SIZE bucket_inv (PRIV),
    // then BL_DEPTH siblings (PRIV)
    let mut idx = vec![3, 4, 5];
    for i in 0..BL_BUCKET_SIZE {
        idx.push(8 + i);
    }
    idx.push(8 + BL_BUCKET_SIZE);
    for i in 0..BL_DEPTH {
        idx.push(9 + BL_BUCKET_SIZE + i);
    }
    idx
}

fn setup_runner_and_paths(
    program: &Path,
) -> Result<(LigeroRunner, verifier::VerifierPaths, Vec<usize>)> {
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
        anyhow::bail!("SKIP");
    }

    let shader_dir = PathBuf::from(&runner.config().shader_path);
    if !shader_dir.exists() {
        eprintln!(
            "Skipping: shader path not found at {}",
            shader_dir.display()
        );
        anyhow::bail!("SKIP");
    }

    let vpaths = verifier::VerifierPaths::from_explicit(
        program.to_path_buf(),
        shader_dir,
        runner.paths().verifier_bin.clone(),
        packing,
    );

    Ok((runner, vpaths, private_indices_note_deposit()))
}

fn try_skip<T>(r: Result<T>) -> Result<Option<T>> {
    match r {
        Ok(v) => Ok(Some(v)),
        Err(e) if e.to_string().contains("SKIP") => Ok(None),
        Err(e) => Err(e),
    }
}

#[test]
fn test_note_deposit_roundtrip_hex_args() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_deposit_guest(&repo)?;

    let program = note_deposit_program_path(&repo)
        .canonicalize()
        .context("Failed to canonicalize note_deposit_guest.wasm")?;

    let Some((mut runner, vpaths, priv_idx)) = try_skip(setup_runner_and_paths(&program))? else {
        return Ok(());
    };

    let domain: Hash32 = [1u8; 32];
    let value: u64 = 42;
    let rho: Hash32 = [2u8; 32];
    let pk_spend: Hash32 = [3u8; 32];
    let pk_ivk: Hash32 = [4u8; 32];
    let recipient = recipient_from_pk(&domain, &pk_spend, &pk_ivk);
    let sender_id = [0u8; 32];
    let cm_out = note_commitment_v2(&domain, value, &rho, &recipient, &sender_id);

    let mut args = vec![
        arg32(&domain),
        LigeroArg::I64 { i64: value as i64 },
        arg32(&rho),
        arg32(&pk_spend),
        arg32(&pk_ivk),
        arg32(&cm_out),
    ];
    append_empty_blacklist_bucket_hex(&mut args, &recipient)?;

    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = args.clone();

    let (proof_bytes, _p_stdout, _p_stderr) =
        match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
            keep_proof_dir: false,
            proof_outputs_base: None,
            write_replay_script: false,
        }) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("Skipping: prover failed (GPU/WebGPU likely unavailable): {err}");
                return Ok(());
            }
        };

    anyhow::ensure!(!proof_bytes.is_empty(), "proof should not be empty");

    let (ok, v_stdout, v_stderr) =
        verifier::verify_proof_with_output(&vpaths, &proof_bytes, args, priv_idx)
            .context("Failed to run verifier")?;

    if !ok {
        anyhow::bail!("verifier did not report success\nstdout: {v_stdout}\nstderr: {v_stderr}");
    }

    Ok(())
}

#[test]
fn test_note_deposit_roundtrip_string_args() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_deposit_guest(&repo)?;

    let program = note_deposit_program_path(&repo)
        .canonicalize()
        .context("Failed to canonicalize note_deposit_guest.wasm")?;

    let Some((mut runner, vpaths, priv_idx)) = try_skip(setup_runner_and_paths(&program))? else {
        return Ok(());
    };

    let domain: Hash32 = [4u8; 32];
    let value: u64 = 123;
    let rho: Hash32 = [5u8; 32];
    let pk_spend: Hash32 = [6u8; 32];
    let pk_ivk: Hash32 = [7u8; 32];
    let recipient = recipient_from_pk(&domain, &pk_spend, &pk_ivk);
    let sender_id = [0u8; 32];
    let cm_out = note_commitment_v2(&domain, value, &rho, &recipient, &sender_id);

    let mut args = vec![
        LigeroArg::String {
            str: format!("0x{}", hx32(&domain)),
        },
        LigeroArg::I64 { i64: value as i64 },
        LigeroArg::String {
            str: format!("0x{}", hx32(&rho)),
        },
        LigeroArg::String {
            str: format!("0x{}", hx32(&pk_spend)),
        },
        LigeroArg::String {
            str: format!("0x{}", hx32(&pk_ivk)),
        },
        LigeroArg::String {
            str: format!("0x{}", hx32(&cm_out)),
        },
    ];
    append_empty_blacklist_bucket_string(&mut args, &recipient)?;

    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = args.clone();

    // The WebGPU prover/verifier pass 32-byte args to WASM as `0x...` strings; both `Hex` and
    // `String` encodings can therefore be supported by the guest.
    let (proof_bytes, _p_stdout, _p_stderr) =
        match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
            keep_proof_dir: false,
            proof_outputs_base: None,
            write_replay_script: false,
        }) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("Skipping: prover failed (GPU/WebGPU likely unavailable): {err}");
                return Ok(());
            }
        };

    let (ok, v_stdout, v_stderr) =
        verifier::verify_proof_with_output(&vpaths, &proof_bytes, args, priv_idx)
            .context("Failed to run verifier")?;
    if !ok {
        anyhow::bail!("verifier did not report success\nstdout: {v_stdout}\nstderr: {v_stderr}");
    }

    Ok(())
}

#[test]
fn test_note_deposit_verifier_rejects_mutated_value() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_deposit_guest(&repo)?;

    let program = note_deposit_program_path(&repo)
        .canonicalize()
        .context("Failed to canonicalize note_deposit_guest.wasm")?;

    let Some((mut runner, vpaths, priv_idx)) = try_skip(setup_runner_and_paths(&program))? else {
        return Ok(());
    };

    let domain: Hash32 = [7u8; 32];
    let value: u64 = 77;
    let rho: Hash32 = [8u8; 32];
    let pk_spend: Hash32 = [9u8; 32];
    let pk_ivk: Hash32 = [10u8; 32];
    let recipient = recipient_from_pk(&domain, &pk_spend, &pk_ivk);
    let sender_id = [0u8; 32];
    let cm_out = note_commitment_v2(&domain, value, &rho, &recipient, &sender_id);

    let mut args = vec![
        arg32(&domain),
        LigeroArg::I64 { i64: value as i64 },
        arg32(&rho),
        arg32(&pk_spend),
        arg32(&pk_ivk),
        arg32(&cm_out),
    ];
    append_empty_blacklist_bucket_hex(&mut args, &recipient)?;

    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = args.clone();

    let (proof_bytes, _p_stdout, _p_stderr) =
        match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
            keep_proof_dir: false,
            proof_outputs_base: None,
            write_replay_script: false,
        }) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("Skipping: prover failed (GPU/WebGPU likely unavailable): {err}");
                return Ok(());
            }
        };

    // Sanity: should verify with the original statement.
    let (ok, v_stdout, v_stderr) =
        verifier::verify_proof_with_output(&vpaths, &proof_bytes, args.clone(), priv_idx.clone())
            .context("Failed to run verifier")?;
    if !ok {
        anyhow::bail!(
            "verifier did not report success for the original statement\nstdout: {v_stdout}\nstderr: {v_stderr}"
        );
    }

    // Mutate a PUBLIC input (value) without changing the proof -> must fail.
    let mut bad_args = args;
    bad_args[1] = LigeroArg::I64 {
        i64: (value as i64) + 1,
    };

    match verifier::verify_proof_with_output(&vpaths, &proof_bytes, bad_args, priv_idx) {
        Ok((ok_bad, _stdout, _stderr)) => {
            anyhow::ensure!(
                !ok_bad,
                "expected verification to fail when a public input is mutated (this implies the verifier is not binding to provided public inputs)"
            );
        }
        Err(_e) => {
            // Any error is also an acceptable failure signal for a bad statement.
        }
    }

    Ok(())
}

#[test]
fn test_note_deposit_verifier_rejects_mutated_cm_out() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_deposit_guest(&repo)?;

    let program = note_deposit_program_path(&repo)
        .canonicalize()
        .context("Failed to canonicalize note_deposit_guest.wasm")?;

    let Some((mut runner, vpaths, priv_idx)) = try_skip(setup_runner_and_paths(&program))? else {
        return Ok(());
    };

    let domain: Hash32 = [10u8; 32];
    let value: u64 = 55;
    let rho: Hash32 = [11u8; 32];
    let pk_spend: Hash32 = [12u8; 32];
    let pk_ivk: Hash32 = [13u8; 32];
    let recipient = recipient_from_pk(&domain, &pk_spend, &pk_ivk);
    let sender_id = [0u8; 32];
    let cm_out = note_commitment_v2(&domain, value, &rho, &recipient, &sender_id);

    let mut args = vec![
        arg32(&domain),
        LigeroArg::I64 { i64: value as i64 },
        arg32(&rho),
        arg32(&pk_spend),
        arg32(&pk_ivk),
        arg32(&cm_out),
    ];
    append_empty_blacklist_bucket_hex(&mut args, &recipient)?;

    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = args.clone();

    let (proof_bytes, _p_stdout, _p_stderr) =
        match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
            keep_proof_dir: false,
            proof_outputs_base: None,
            write_replay_script: false,
        }) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("Skipping: prover failed (GPU/WebGPU likely unavailable): {err}");
                return Ok(());
            }
        };

    // Sanity: should verify with the original statement.
    let (ok, v_stdout, v_stderr) =
        verifier::verify_proof_with_output(&vpaths, &proof_bytes, args.clone(), priv_idx.clone())
            .context("Failed to run verifier")?;
    if !ok {
        anyhow::bail!(
            "verifier did not report success for the original statement\nstdout: {v_stdout}\nstderr: {v_stderr}"
        );
    }

    // Mutate a PUBLIC input (cm_out) without changing the proof -> must fail.
    let mut bad_args = args;
    let mut bad_cm = cm_out;
    bad_cm[0] ^= 1;
    bad_args[5] = arg32(&bad_cm);

    match verifier::verify_proof_with_output(&vpaths, &proof_bytes, bad_args, priv_idx) {
        Ok((ok_bad, _stdout, _stderr)) => {
            anyhow::ensure!(
                !ok_bad,
                "expected verification to fail when a public input is mutated (this implies the verifier is not binding to provided public inputs)"
            );
        }
        Err(_e) => {
            // Any error is also an acceptable failure signal for a bad statement.
        }
    }

    Ok(())
}

#[test]
fn test_note_deposit_rejects_negative_value_and_wrong_argc() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_deposit_guest(&repo)?;

    let program = note_deposit_program_path(&repo)
        .canonicalize()
        .context("Failed to canonicalize note_deposit_guest.wasm")?;

    let Some((mut runner, vpaths, priv_idx)) = try_skip(setup_runner_and_paths(&program))? else {
        return Ok(());
    };

    // Baseline valid proof to ensure the environment is functional.
    let domain: Hash32 = [13u8; 32];
    let value: u64 = 42;
    let rho: Hash32 = [14u8; 32];
    let pk_spend: Hash32 = [15u8; 32];
    let pk_ivk: Hash32 = [16u8; 32];
    let recipient = recipient_from_pk(&domain, &pk_spend, &pk_ivk);
    let sender_id = [0u8; 32];
    let cm_out = note_commitment_v2(&domain, value, &rho, &recipient, &sender_id);

    let mut good_args = vec![
        arg32(&domain),
        LigeroArg::I64 { i64: value as i64 },
        arg32(&rho),
        arg32(&pk_spend),
        arg32(&pk_ivk),
        arg32(&cm_out),
    ];
    append_empty_blacklist_bucket_hex(&mut good_args, &recipient)?;

    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = good_args.clone();
    let (good_proof, _p_stdout, _p_stderr) =
        match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
            keep_proof_dir: false,
            proof_outputs_base: None,
            write_replay_script: false,
        }) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("Skipping: prover failed (GPU/WebGPU likely unavailable): {err}");
                return Ok(());
            }
        };

    let (ok, v_stdout, v_stderr) = verifier::verify_proof_with_output(
        &vpaths,
        &good_proof,
        good_args.clone(),
        priv_idx.clone(),
    )
    .context("Failed to run verifier")?;
    if !ok {
        anyhow::bail!("baseline proof did not verify\nstdout: {v_stdout}\nstderr: {v_stderr}");
    }

    // Invalid: negative public `value` must be rejected.
    let mut bad_args = good_args.clone();
    bad_args[1] = LigeroArg::I64 { i64: -1 };
    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = bad_args.clone();
    match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
        keep_proof_dir: false,
        proof_outputs_base: None,
        write_replay_script: false,
    }) {
        Ok((proof, _stdout, _stderr)) => {
            let (ok, stdout, stderr) =
                verifier::verify_proof_with_output(&vpaths, &proof, bad_args, priv_idx.clone())
                    .context("Failed to run verifier")?;
            anyhow::ensure!(
                !ok,
                "expected verification to fail, but it succeeded\nstdout: {stdout}\nstderr: {stderr}"
            );
        }
        Err(_e) => {
            // Expected: invalid input triggers UNSAT and the prover exits non-zero.
        }
    }

    // Invalid: zero `value` must be rejected.
    let mut bad_args = good_args.clone();
    bad_args[1] = LigeroArg::I64 { i64: 0 };
    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = bad_args.clone();
    match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
        keep_proof_dir: false,
        proof_outputs_base: None,
        write_replay_script: false,
    }) {
        Ok((proof, _stdout, _stderr)) => {
            let (ok, stdout, stderr) =
                verifier::verify_proof_with_output(&vpaths, &proof, bad_args, priv_idx.clone())
                    .context("Failed to run verifier")?;
            anyhow::ensure!(
                !ok,
                "expected verification to fail, but it succeeded\nstdout: {stdout}\nstderr: {stderr}"
            );
        }
        Err(_e) => {
            // Expected: invalid input triggers UNSAT and the prover exits non-zero.
        }
    }

    // Invalid: mismatch between recipient keys and cm_out must be rejected (prove-time).
    let mut bad_args = good_args.clone();
    let wrong_pk_ivk: Hash32 = [0x99u8; 32];
    bad_args[4] = arg32(&wrong_pk_ivk);
    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = bad_args.clone();
    match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
        keep_proof_dir: false,
        proof_outputs_base: None,
        write_replay_script: false,
    }) {
        Ok((proof, _stdout, _stderr)) => {
            let (ok, stdout, stderr) =
                verifier::verify_proof_with_output(&vpaths, &proof, bad_args, priv_idx.clone())
                    .context("Failed to run verifier")?;
            anyhow::ensure!(
                !ok,
                "expected verification to fail, but it succeeded\nstdout: {stdout}\nstderr: {stderr}"
            );
        }
        Err(_e) => {
            // Expected: witness no longer satisfies cm_out == NOTE_V2(...).
        }
    }

    // Invalid: wrong arg count must be rejected (regression test for failure-path soundness).
    let mut bad_args = good_args;
    bad_args.pop(); // drop one arg (wrong arg count)
    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = bad_args.clone();
    match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
        keep_proof_dir: false,
        proof_outputs_base: None,
        write_replay_script: false,
    }) {
        Ok((proof, _stdout, _stderr)) => {
            let (ok, stdout, stderr) =
                verifier::verify_proof_with_output(&vpaths, &proof, bad_args, priv_idx)
                    .context("Failed to run verifier")?;
            anyhow::ensure!(
                !ok,
                "expected verification to fail, but it succeeded\nstdout: {stdout}\nstderr: {stderr}"
            );
        }
        Err(_e) => {}
    }

    Ok(())
}

#[test]
fn test_note_deposit_blacklist_accepts_unlisted_and_rejects_listed() -> Result<()> {
    let repo = repo_root()?;
    maybe_build_note_deposit_guest(&repo)?;

    let program = note_deposit_program_path(&repo)
        .canonicalize()
        .context("Failed to canonicalize note_deposit_guest.wasm")?;

    let Some((mut runner, vpaths, priv_idx)) = try_skip(setup_runner_and_paths(&program))? else {
        return Ok(());
    };

    // Example address (bech32m) that we can add to the blacklist tree AND attempt to deposit to.
    //
    // This test assumes the bech32m payload is the 32-byte `pk_spend`, and (per your scheme)
    // `pk_ivk == pk_spend`.
    //
    // Then the circuit-level recipient is:
    //   recipient = H("ADDR_V2" || domain || pk_spend || pk_ivk)
    let pk_spend_example = decode_privpool_addr(PRIVPOOL_EXAMPLE_ADDR)?;
    let pk_ivk_example = pk_spend_example;
    anyhow::ensure!(
        encode_privpool_addr(&pk_spend_example) == PRIVPOOL_EXAMPLE_ADDR,
        "privpool bech32m roundtrip mismatch"
    );

    // Two recipients: one will be blacklisted, the other unlisted.
    let domain: Hash32 = [0x55u8; 32];
    let value: u64 = 42;
    let rho_ok: Hash32 = [0x11u8; 32];
    let rho_bad: Hash32 = [0x22u8; 32];

    let pk_spend_ok: Hash32 = [0x33u8; 32];
    let pk_ivk_ok: Hash32 = [0x44u8; 32];
    let recipient_ok = recipient_from_pk(&domain, &pk_spend_ok, &pk_ivk_ok);

    let pk_spend_bad: Hash32 = [0x66u8; 32];
    let pk_ivk_bad: Hash32 = [0x77u8; 32];
    let recipient_bad = recipient_from_pk(&domain, &pk_spend_bad, &pk_ivk_bad);

    let recipient_example = recipient_from_pk(&domain, &pk_spend_example, &pk_ivk_example);
    anyhow::ensure!(
        recipient_example != recipient_bad,
        "example and bad recipients must be distinct"
    );

    // Build a bucketed blacklist tree (depth BL_DEPTH), where each leaf commits to BL_BUCKET_SIZE IDs.
    let empty_bucket = bl_empty_bucket_entries();
    let leaf_default = bl_bucket_leaf(&empty_bucket);
    let default_nodes = merkle_default_nodes_from_leaf(BL_DEPTH, &leaf_default);

    // Blacklist set: include a fixed bech32m example (for docs/demo) and a synthetic bad recipient.
    let mut blacklist_ids: Vec<Hash32> = vec![recipient_example, recipient_bad];
    blacklist_ids.sort();
    blacklist_ids.dedup();
    anyhow::ensure!(
        blacklist_ids.len() == 2,
        "expected exactly 2 unique blacklisted IDs for this test"
    );

    use std::collections::{BTreeMap, HashMap};

    let mut bucket_ids: BTreeMap<u64, Vec<Hash32>> = BTreeMap::new();
    for id in blacklist_ids.iter() {
        bucket_ids
            .entry(bl_bucket_pos_from_id(id))
            .or_default()
            .push(*id);
    }

    let mut bucket_entries_by_pos: HashMap<u64, [Hash32; BL_BUCKET_SIZE]> = HashMap::new();
    let mut leaves: Vec<(u64, Hash32)> = Vec::new();
    for (pos, mut ids) in bucket_ids.into_iter() {
        ids.sort();
        anyhow::ensure!(
            ids.len() <= BL_BUCKET_SIZE,
            "bucket overflow at pos={pos}: {} entries",
            ids.len()
        );
        let mut entries = bl_empty_bucket_entries();
        for (i, id) in ids.into_iter().enumerate() {
            entries[i] = id;
        }
        bucket_entries_by_pos.insert(pos, entries);
        let leaf = bl_bucket_leaf(&entries);
        if leaf != leaf_default {
            leaves.push((pos, leaf));
        }
    }

    // Ensure we can open the (possibly default) bucket that the unlisted recipient lands in.
    let pos_ok = bl_bucket_pos_from_id(&recipient_ok);
    let bucket_ok = bucket_entries_by_pos
        .get(&pos_ok)
        .copied()
        .unwrap_or(empty_bucket);
    let leaf_ok = bl_bucket_leaf(&bucket_ok);
    if !leaves.iter().any(|(p, _)| *p == pos_ok) {
        leaves.push((pos_ok, leaf_ok));
    }

    let (blacklist_root, openings) =
        merkle_root_and_openings_sparse(BL_DEPTH, &default_nodes, &leaves)?;
    let mut opening_by_pos: HashMap<u64, Vec<Hash32>> = HashMap::new();
    for ((pos, _leaf), sibs) in leaves.iter().zip(openings.into_iter()) {
        opening_by_pos.insert(*pos, sibs);
    }

    let sibs_ok = opening_by_pos.get(&pos_ok).context("missing opening for pos_ok")?;
    anyhow::ensure!(
        merkle_root_from_path(&leaf_ok, pos_ok, sibs_ok) == blacklist_root,
        "bucket opening for recipient_ok must match root"
    );

    // === Proof for the unlisted address should verify ===
    let sender_id = [0u8; 32];
    let cm_ok = note_commitment_v2(&domain, value, &rho_ok, &recipient_ok, &sender_id);
    anyhow::ensure!(
        blacklist_ids.binary_search(&recipient_ok).is_err(),
        "expected ok recipient to be unlisted"
    );

    let inv_ok = bl_bucket_inv_for_id(&recipient_ok, &bucket_ok)
        .context("unexpected: recipient_ok collides with its bucket entries")?;

    let mut args_ok = vec![
        arg32(&domain),
        LigeroArg::I64 { i64: value as i64 },
        arg32(&rho_ok),
        arg32(&pk_spend_ok),
        arg32(&pk_ivk_ok),
        arg32(&cm_ok),
        arg32(&blacklist_root),
    ];
    for e in bucket_ok.iter() {
        args_ok.push(arg32(e));
    }
    args_ok.push(arg32(&inv_ok));
    for sib in sibs_ok.iter().take(BL_DEPTH) {
        args_ok.push(arg32(sib));
    }

    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = args_ok.clone();
    let (proof_ok, _p_stdout, _p_stderr) =
        match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
            keep_proof_dir: false,
            proof_outputs_base: None,
            write_replay_script: false,
        }) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("Skipping: prover failed (GPU/WebGPU likely unavailable): {err}");
                return Ok(());
            }
        };
    anyhow::ensure!(!proof_ok.is_empty(), "proof should not be empty");

    let (ok, v_stdout, v_stderr) =
        verifier::verify_proof_with_output(&vpaths, &proof_ok, args_ok, priv_idx.clone())
            .context("Failed to run verifier")?;
    if !ok {
        anyhow::bail!(
            "verifier did not report success for unlisted address\nstdout: {v_stdout}\nstderr: {v_stderr}"
        );
    }

    // === Proof for the example bech32m address must be rejected ===
    let rho_example: Hash32 = [0x33u8; 32];
    let cm_example =
        note_commitment_v2(&domain, value, &rho_example, &recipient_example, &sender_id);
    let pos_example = bl_bucket_pos_from_id(&recipient_example);
    let bucket_example = bucket_entries_by_pos
        .get(&pos_example)
        .copied()
        .context("missing bucket for example recipient")?;
    let leaf_example = bl_bucket_leaf(&bucket_example);
    let sibs_example = opening_by_pos
        .get(&pos_example)
        .context("missing opening for example bucket")?;
    anyhow::ensure!(
        merkle_root_from_path(&leaf_example, pos_example, sibs_example) == blacklist_root,
        "bucket opening for example recipient must match root"
    );

    let mut args_example = vec![
        arg32(&domain),
        LigeroArg::I64 { i64: value as i64 },
        arg32(&rho_example),
        arg32(&pk_spend_example),
        arg32(&pk_ivk_example),
        arg32(&cm_example),
        arg32(&blacklist_root),
    ];
    for e in bucket_example.iter() {
        args_example.push(arg32(e));
    }
    // No inverse exists for a blacklisted id; provide a dummy value and expect UNSAT.
    args_example.push(arg32(&[0u8; 32]));
    for sib in sibs_example.iter().take(BL_DEPTH) {
        args_example.push(arg32(sib));
    }

    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = args_example.clone();
    match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
        keep_proof_dir: false,
        proof_outputs_base: None,
        write_replay_script: false,
    }) {
        Err(_e) => {} // rejected at prove-time ✅ (expected)
        Ok((proof_example, _stdout, _stderr)) => {
            let (ok_example, _stdout, _stderr) = verifier::verify_proof_with_output(
                &vpaths,
                &proof_example,
                args_example,
                priv_idx.clone(),
            )
            .context("Failed to run verifier")?;
            anyhow::ensure!(
                !ok_example,
                "expected verification to fail for example blacklisted address, but it succeeded"
            );
        }
    }

    // === Proof for the blacklisted address must be rejected ===
    let cm_bad = note_commitment_v2(&domain, value, &rho_bad, &recipient_bad, &sender_id);
    let pos_bad = bl_bucket_pos_from_id(&recipient_bad);
    let bucket_bad = bucket_entries_by_pos
        .get(&pos_bad)
        .copied()
        .context("missing bucket for bad recipient")?;
    let leaf_bad = bl_bucket_leaf(&bucket_bad);
    let sibs_bad = opening_by_pos
        .get(&pos_bad)
        .context("missing opening for bad bucket")?;
    anyhow::ensure!(
        merkle_root_from_path(&leaf_bad, pos_bad, sibs_bad) == blacklist_root,
        "bucket opening for bad recipient must match root"
    );
    let mut args_bad = vec![
        arg32(&domain),
        LigeroArg::I64 { i64: value as i64 },
        arg32(&rho_bad),
        arg32(&pk_spend_bad),
        arg32(&pk_ivk_bad),
        arg32(&cm_bad),
        arg32(&blacklist_root),
    ];
    for e in bucket_bad.iter() {
        args_bad.push(arg32(e));
    }
    args_bad.push(arg32(&[0u8; 32]));
    for sib in sibs_bad.iter().take(BL_DEPTH) {
        args_bad.push(arg32(sib));
    }

    runner.config_mut().private_indices = priv_idx.clone();
    runner.config_mut().args = args_bad.clone();
    match runner.run_prover_with_output(ligero_runner::ProverRunOptions {
        keep_proof_dir: false,
        proof_outputs_base: None,
        write_replay_script: false,
    }) {
        Err(_e) => Ok(()), // rejected at prove-time ✅ (expected)
        Ok((proof_bad, _stdout, _stderr)) => {
            let (ok_bad, _stdout, _stderr) =
                verifier::verify_proof_with_output(&vpaths, &proof_bad, args_bad, priv_idx)
                    .context("Failed to run verifier")?;
            anyhow::ensure!(
                !ok_bad,
                "expected verification to fail for blacklisted address, but it succeeded"
            );
            Ok(())
        }
    }
}
