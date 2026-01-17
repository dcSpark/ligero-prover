//! Generate valid note_spend test request JSON for curl testing.
//!
//! This tool generates cryptographically valid arguments for the note_spend circuit
//! that can be used to test the HTTP proving server.
//!
//! Usage:
//!   cargo run --bin generate-test-request > request.json
//!   curl -X POST http://127.0.0.1:1313/prove -H "Content-Type: application/json" -d @request.json

use ligetron::{poseidon2_hash_bytes, Bn254Fr};
use serde_json::json;

type Hash32 = [u8; 32];

const BL_BUCKET_SIZE: usize = 12;
const BL_DEPTH: usize = 16;

fn hx32(b: &Hash32) -> String {
    hex::encode(b)
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

fn merkle_default_nodes_from_leaf(depth: usize, leaf0: &Hash32) -> Vec<Hash32> {
    let mut out = Vec::with_capacity(depth + 1);
    out.push(*leaf0);
    for lvl in 0..depth {
        let prev = out[lvl];
        out.push(mt_combine(lvl as u8, &prev, &prev));
    }
    out
}

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

fn fr_from_hash32_be(h: &Hash32) -> Bn254Fr {
    let mut fr = Bn254Fr::new();
    fr.set_bytes_big(h);
    fr
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

fn recipient_from_sk(domain: &Hash32, spend_sk: &Hash32, pk_ivk: &Hash32) -> Hash32 {
    recipient_from_pk(domain, &pk_from_sk(spend_sk), pk_ivk)
}

fn nf_key_from_sk(domain: &Hash32, spend_sk: &Hash32) -> Hash32 {
    poseidon2_hash_domain(b"NFKEY_V1", &[domain, spend_sk])
}

fn nullifier(domain: &Hash32, nf_key: &Hash32, rho: &Hash32) -> Hash32 {
    poseidon2_hash_domain(b"PRF_NF_V1", &[domain, nf_key, rho])
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

fn private_indices(depth: usize, n_in: usize, n_out: usize, is_transfer: bool) -> Vec<usize> {
    let mut idx = vec![2usize, 3usize];
    let per_in = 5usize + depth;
    for i in 0..n_in {
        let base = 7 + i * per_in;
        idx.push(base);
        idx.push(base + 1);
        idx.push(base + 2);
        idx.push(base + 3);
        for k in 0..depth {
            idx.push(base + 4 + k);
        }
    }
    let withdraw_idx = 7 + n_in * per_in;
    let outs_base = withdraw_idx + 3;
    for j in 0..n_out {
        idx.push(outs_base + 5 * j);
        idx.push(outs_base + 5 * j + 1);
        idx.push(outs_base + 5 * j + 2);
        idx.push(outs_base + 5 * j + 3);
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

fn main() {
    let depth = 8usize;

    // Generate test values (same as integration tests)
    let domain: Hash32 = [1u8; 32];
    let rho_in: Hash32 = [2u8; 32];
    let spend_sk: Hash32 = [4u8; 32];
    let pk_ivk_owner = ivk_seed(&domain, &spend_sk);
    let recipient_owner = recipient_from_sk(&domain, &spend_sk, &pk_ivk_owner);
    let sender_id_current = recipient_owner;
    let nf_key = nf_key_from_sk(&domain, &spend_sk);

    let value: u64 = 42;
    let pos: u64 = 0;
    let sender_id_in: Hash32 = [6u8; 32];
    let cm_in = note_commitment(&domain, value, &rho_in, &recipient_owner, &sender_id_in);

    let mut tree = MerkleTree::new(depth);
    tree.set_leaf(pos as usize, cm_in);
    let anchor = tree.root();
    let siblings = tree.open(pos as usize);
    let nf = nullifier(&domain, &nf_key, &rho_in);

    let n_in: u64 = 1;
    let withdraw_amount: u64 = 0;
    let n_out: u64 = 1;
    let out_value: u64 = value;
    let out_rho: Hash32 = [7u8; 32];
    let out_spend_sk: Hash32 = [8u8; 32];
    let out_pk_spend = pk_from_sk(&out_spend_sk);
    let out_pk_ivk = ivk_seed(&domain, &out_spend_sk);
    let out_recipient = recipient_from_pk(&domain, &out_pk_spend, &out_pk_ivk);
    let cm_out = note_commitment(&domain, out_value, &out_rho, &out_recipient, &sender_id_current);

    let inv_enforce = compute_inv_enforce(&[value], &[rho_in], &[out_value], &[out_rho]);

    // Build args array
    let mut args: Vec<serde_json::Value> = Vec::new();
    args.push(json!({ "hex": hx32(&domain) }));
    args.push(json!({ "hex": hx32(&spend_sk) }));
    args.push(json!({ "hex": hx32(&pk_ivk_owner) }));
    args.push(json!({ "i64": depth as i64 }));
    args.push(json!({ "hex": hx32(&anchor) }));
    args.push(json!({ "i64": n_in as i64 }));
    args.push(json!({ "i64": value as i64 }));
    args.push(json!({ "hex": hx32(&rho_in) }));
    args.push(json!({ "hex": hx32(&sender_id_in) }));
    args.push(json!({ "i64": pos as i64 }));
    for s in &siblings {
        args.push(json!({ "hex": hx32(s) }));
    }
    args.push(json!({ "hex": hx32(&nf) }));
    args.push(json!({ "i64": withdraw_amount as i64 }));
    args.push(json!({ "hex": hx32(&[0u8; 32]) }));
    args.push(json!({ "i64": n_out as i64 }));
    args.push(json!({ "i64": out_value as i64 }));
    args.push(json!({ "hex": hx32(&out_rho) }));
    args.push(json!({ "hex": hx32(&out_pk_spend) }));
    args.push(json!({ "hex": hx32(&out_pk_ivk) }));
    args.push(json!({ "hex": hx32(&cm_out) }));
    args.push(json!({ "hex": hx32(&inv_enforce) }));

    // Blacklist args
    let bucket_entries = bl_empty_bucket_entries();
    let leaf0 = bl_bucket_leaf(&bucket_entries);
    let default_nodes = merkle_default_nodes_from_leaf(BL_DEPTH, &leaf0);
    let blacklist_root = default_nodes[BL_DEPTH];
    let bl_siblings: Vec<Hash32> = default_nodes.iter().take(BL_DEPTH).copied().collect();

    args.push(json!({ "hex": hx32(&blacklist_root) }));

    // Two blacklist checks for transfer (sender_id_current, out_recipient)
    for id in [sender_id_current, out_recipient] {
        for e in bucket_entries.iter() {
            args.push(json!({ "hex": hx32(e) }));
        }
        let inv = bl_bucket_inv_for_id(&id, &bucket_entries)
            .expect("unexpected: id collides with empty blacklist bucket");
        args.push(json!({ "hex": hx32(&inv) }));
        for sib in bl_siblings.iter() {
            args.push(json!({ "hex": hx32(sib) }));
        }
    }

    let priv_idx = private_indices(depth, 1, 1, true);

    // Output the full request JSON
    let request = json!({
        "circuit": "note_spend",
        "args": args,
        "privateIndices": priv_idx
    });

    println!("{}", serde_json::to_string_pretty(&request).unwrap());
}
