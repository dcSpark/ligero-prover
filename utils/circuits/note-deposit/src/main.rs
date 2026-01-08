/*
 * Rust Guest Program: Note Deposit Verifier for Midnight Privacy Pool
 *
 * Mints a single shielded NOTE_V2 commitment to an ADDR_V2 recipient.
 *
 * - No Merkle membership proof (no spent note)
 * - No nullifier
 * - One output commitment `cm_out` bound to public `value`
 *
 * ABI / Args (argv[0] is program name):
 *   [1]  domain              — 32 bytes (PUBLIC)
 *   [2]  value               — i64 (PUBLIC, must be > 0)
 *   [3]  rho                 — 32 bytes (PRIVATE)
 *   [4]  pk_spend_recipient  — 32 bytes (PRIVATE)
 *   [5]  pk_ivk_recipient    — 32 bytes (PRIVATE)
 *   [6]  cm_out              — 32 bytes (PUBLIC; must equal NOTE_V2(...) )
 *   [7]  blacklist_root      — 32 bytes (PUBLIC)
 *   [8+] bl_siblings[k]      — BL_DEPTH × 32 bytes (PRIVATE; proves recipient leaf == 0)
 *
 * NOTE:
 *   Deposits do not attest to a sender inside the pool; `sender_id` is fixed to 0x00..00.
 */

use ligetron::api::{get_args, ArgHolder};
use ligetron::bn254fr::{Bn254Fr, addmod_checked, submod_checked};
use ligetron::poseidon2::poseidon2_hash_bytes;

type Hash32 = [u8; 32];

fn exit_with_code(code: i32) -> ! {
    std::process::exit(code)
}

#[cfg(feature = "diagnostics")]
fn fail_with_code(code: u32) -> ! {
    exit_with_code(code as i32)
}

#[cfg(not(feature = "diagnostics"))]
fn fail_with_code(_code: u32) -> ! {
    exit_with_code(71)
}

#[inline(always)]
fn hard_fail(code: u32) -> ! {
    // Force UNSAT: x == 0 AND x == 1.
    //
    // IMPORTANT: `Bn254Fr::new()` is a witness variable; to make the failure path
    // unconditionally unsatisfiable we bind it to conflicting public constants.
    let x = Bn254Fr::new();
    Bn254Fr::assert_equal_u64(&x, 0);
    Bn254Fr::assert_equal_u64(&x, 1);
    fail_with_code(code)
}

/// Convert an ASCII hex character into its 4-bit value.
///
/// This implementation is branchless and uses no lookup tables, so it does not introduce
/// secret-dependent control flow or secret-indexed memory access.
///
/// Invalid characters map to 0.
#[inline(always)]
fn hex_char_to_nibble(c: u8) -> u8 {
    // '0'..'9'
    let d = c.wrapping_sub(b'0');
    let md = (0u8).wrapping_sub((d <= 9) as u8);

    // 'a'..'f'
    let a = c.wrapping_sub(b'a');
    let ma = (0u8).wrapping_sub((a <= 5) as u8);

    // 'A'..'F'
    #[allow(non_snake_case)]
    let A = c.wrapping_sub(b'A');
    #[allow(non_snake_case)]
    let mA = (0u8).wrapping_sub((A <= 5) as u8);

    (d & md) | (a.wrapping_add(10) & ma) | (A.wrapping_add(10) & mA)
}

// 32-byte values are passed by the runner as `0x...` ASCII hex (C-string), so we decode them here.
#[inline(always)]
fn read_hash32(args: &ArgHolder, index: usize, fail_code: u32) -> Hash32 {
    let bytes = args.get_as_bytes(index);
    let mut out = [0u8; 32];
    // Fast-path: some hosts may pass raw 32 bytes directly.
    if bytes.len() == 32 {
        out.copy_from_slice(bytes);
        return out;
    }

    // Common path: Ligero passes `hex` args as a C-string `0x...` with a trailing `\0`.
    // Expected length: 2 ("0x") + 64 hex chars + 1 NUL = 67 bytes.
    let hex_bytes = if bytes.len() == 67 {
        &bytes[2..66]
    } else if bytes.len() == 66 {
        &bytes[2..66]
    } else {
        #[cfg(feature = "diagnostics")]
        {
            eprintln!("read_hash32: idx={} len={} (unexpected)", index, bytes.len());
            eprintln!("read_hash32: first_bytes={:02x?}", &bytes[..bytes.len().min(16)]);
        }
        hard_fail(fail_code);
    };

    let mut i = 0usize;
    while i < 32 {
        let hi = hex_char_to_nibble(hex_bytes[2 * i]);
        let lo = hex_char_to_nibble(hex_bytes[2 * i + 1]);
        out[i] = (hi << 4) | lo;
        i += 1;
    }

    out
}

#[inline(always)]
fn read_u64(args: &ArgHolder, index: usize, fail_code: u32) -> u64 {
    let v = args.get_as_int(index);
    if v < 0 {
        hard_fail(fail_code);
    }
    v as u64
}

#[inline(always)]
fn assert_fr_eq_hash32(computed: &Bn254Fr, expected_be: &Hash32) {
    Bn254Fr::assert_equal_bytes_be(computed, expected_be);
}

/// Convert a 32-byte big-endian hash to a Bn254Fr field element.
#[inline(always)]
fn bn254fr_from_hash32_be(h: &Hash32) -> Bn254Fr {
    let mut result = Bn254Fr::new();
    result.set_bytes_big(h);
    result
}

struct Poseidon2Core;

impl Poseidon2Core {
    #[inline(always)]
    pub fn new() -> Self {
        Self
    }

    #[inline(always)]
    pub fn hash_padded_fr(&self, preimage: &[u8]) -> Bn254Fr {
        poseidon2_hash_bytes(preimage)
    }

    #[inline(always)]
    pub fn hash_padded(&self, preimage: &[u8]) -> Hash32 {
        self.hash_padded_fr(preimage).to_bytes_be()
    }
}

/// Merkle tree node hash: H("MT_NODE_V1" || lvl || left || right)
const MT_NODE_BUF_LEN: usize = 10 + 1 + 32 + 32; // "MT_NODE_V1" + lvl + left + right = 75

#[inline(always)]
fn mt_combine(h: &Poseidon2Core, level: u8, left: &Hash32, right: &Hash32) -> Bn254Fr {
    let mut buf = [0u8; MT_NODE_BUF_LEN];
    buf[..10].copy_from_slice(b"MT_NODE_V1");
    buf[10] = level;
    buf[11..43].copy_from_slice(left);
    buf[43..75].copy_from_slice(right);
    h.hash_padded_fr(&buf)
}

/// Compute Merkle root using FIELD-LEVEL MUX operations.
/// This version works with private position bits and siblings!
fn root_from_path_field_level(
    h: &Poseidon2Core,
    leaf_bytes: &Hash32,
    pos_bits: &[Bn254Fr],
    siblings_fr: &[Bn254Fr],
    depth: usize,
) -> Bn254Fr {
    if depth == 0 {
        hard_fail(77);
    }

    let mut cur_fr = bn254fr_from_hash32_be(leaf_bytes);

    let mut left_fr = Bn254Fr::new();
    let mut right_fr = Bn254Fr::new();
    let mut delta = Bn254Fr::new();
    let mut left_bytes = [0u8; 32];
    let mut right_bytes = [0u8; 32];

    let mut lvl = 0usize;
    while lvl < depth {
        let bit = &pos_bits[lvl];
        let sib_fr = &siblings_fr[lvl];

        // delta = bit * (sib - cur)
        // left  = cur + delta
        // right = sib - delta
        submod_checked(&mut delta, sib_fr, &cur_fr);
        delta.mulmod_checked(bit);
        addmod_checked(&mut left_fr, &cur_fr, &delta);
        submod_checked(&mut right_fr, sib_fr, &delta);

        left_fr.get_bytes_big(&mut left_bytes);
        right_fr.get_bytes_big(&mut right_bytes);

        cur_fr = mt_combine(h, lvl as u8, &left_bytes, &right_bytes);
        lvl += 1;
    }

    cur_fr
}

// Blacklist sparse Merkle map depth. We derive path bits from the low bits of the recipient ID.
const BL_DEPTH: usize = 63;

#[inline(always)]
fn bl_pos_bits_from_id(id: &Hash32, depth: usize) -> Vec<Bn254Fr> {
    let mut bits = Vec::with_capacity(depth);
    let mut i = 0usize;
    while i < depth {
        let byte = id[31 - (i / 8)];
        let bit = (byte >> (i % 8)) & 1;
        bits.push(Bn254Fr::from_u32(bit as u32));
        i += 1;
    }
    bits
}

#[inline(always)]
fn read_siblings_fr(args: &ArgHolder, arg_idx: &mut usize, depth: usize, fail_code: u32) -> Vec<Bn254Fr> {
    let mut v = Vec::with_capacity(depth);
    let mut i = 0usize;
    while i < depth {
        let sib = read_hash32(args, *arg_idx, fail_code);
        *arg_idx += 1;
        v.push(bn254fr_from_hash32_be(&sib));
        i += 1;
    }
    v
}

#[inline(always)]
fn assert_not_blacklisted(
    h: &Poseidon2Core,
    id: &Hash32,
    blacklist_root: &Hash32,
    siblings_fr: &[Bn254Fr],
) {
    if siblings_fr.len() != BL_DEPTH {
        hard_fail(78);
    }

    let pos_bits = bl_pos_bits_from_id(id, BL_DEPTH);
    let leaf0: Hash32 = [0u8; 32];
    let root_fr = root_from_path_field_level(h, &leaf0, &pos_bits[..], siblings_fr, BL_DEPTH);
    assert_fr_eq_hash32(&root_fr, blacklist_root);
}

const ADDR_V2_BUF_LEN: usize = 7 + 32 + 32 + 32; // "ADDR_V2" + domain + pk_spend + pk_ivk = 103
const NOTE_CM_BUF_LEN: usize = 7 + 32 + 16 + 32 + 32 + 32; // "NOTE_V2" + domain + value + rho + recipient + sender_id = 151

fn recipient_from_pk(h: &Poseidon2Core, domain: &Hash32, pk_spend: &Hash32, pk_ivk: &Hash32) -> Hash32 {
    let mut buf = [0u8; ADDR_V2_BUF_LEN];
    buf[..7].copy_from_slice(b"ADDR_V2");
    buf[7..39].copy_from_slice(domain);
    buf[39..71].copy_from_slice(pk_spend);
    buf[71..103].copy_from_slice(pk_ivk);
    h.hash_padded(&buf)
}

fn note_commitment_fr(
    h: &Poseidon2Core,
    domain: &Hash32,
    value: u64,
    rho: &Hash32,
    recipient: &Hash32,
    sender_id: &Hash32,
) -> Bn254Fr {
    let mut buf = [0u8; NOTE_CM_BUF_LEN];
    buf[..7].copy_from_slice(b"NOTE_V2");
    buf[7..39].copy_from_slice(domain);
    // Encode value as 16-byte LE (zero-extended from u64)
    buf[39..47].copy_from_slice(&value.to_le_bytes());
    // buf[47..55] already zero from initialization (zero-extension)
    buf[55..87].copy_from_slice(rho);
    buf[87..119].copy_from_slice(recipient);
    buf[119..151].copy_from_slice(sender_id);
    h.hash_padded_fr(&buf)
}

fn main() {
    let args = get_args();
    let expected_argc = 8 + BL_DEPTH; // argv[0] + 6 base args + blacklist_root + BL_DEPTH siblings
    if args.len() != expected_argc {
        hard_fail(70);
    }

    let h = Poseidon2Core::new();

    let domain = read_hash32(&args, 1, 71);
    let value = read_u64(&args, 2, 72);
    if value == 0 {
        hard_fail(72);
    }
    // Bind the public `value` into the statement (otherwise it can be unbound if only used as bytes).
    let value_fr = Bn254Fr::from_u64(value);
    Bn254Fr::assert_equal_u64(&value_fr, value);
    let rho = read_hash32(&args, 3, 73);
    let pk_spend_recipient = read_hash32(&args, 4, 74);
    let pk_ivk_recipient = read_hash32(&args, 5, 75);
    let cm_out_arg = read_hash32(&args, 6, 76);
    let blacklist_root = read_hash32(&args, 7, 77);
    let mut arg_idx: usize = 8;
    let bl_sibs = read_siblings_fr(&args, &mut arg_idx, BL_DEPTH, 78);

    let recipient = recipient_from_pk(&h, &domain, &pk_spend_recipient, &pk_ivk_recipient);
    let sender_id = [0u8; 32];
    let cm_out_fr = note_commitment_fr(&h, &domain, value, &rho, &recipient, &sender_id);
    assert_fr_eq_hash32(&cm_out_fr, &cm_out_arg);

    // Recipient must not be blacklisted.
    assert_not_blacklisted(&h, &recipient, &blacklist_root, &bl_sibs);
}
