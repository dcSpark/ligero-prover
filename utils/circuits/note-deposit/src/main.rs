/*
 * Rust Guest Program: Note Deposit Verifier for Midnight Privacy Pool
 *
 * DEPOSIT verifies creation of a single shielded note from a transparent source:
 *   - No Merkle membership proof (no spent note)
 *   - No nullifier
 *   - One output commitment `cm_out` bound to public `value`
 *
 * ABI / Args (argv[0] is program name):
 *   [1]  domain     — hex arg → 32 bytes (PUBLIC)
 *   [2]  value      — i64 arg → 8 bytes  (PUBLIC)
 *   [3]  rho        — hex arg → 32 bytes (PRIVATE)
 *   [4]  recipient  — hex arg → 32 bytes (PRIVATE)
 *   [5]  cm_out     — hex arg → 32 bytes (PUBLIC; must equal computed)
 *
 * SECURITY NOTE:
 *   We must bind PUBLIC values into the statement with explicit constraints.
 *   In particular, Poseidon2's byte absorption uses unconstrained byte->field
 *   conversions, so we construct and bind the public parts of the NOTE preimage
 *   as constants (31-byte chunks) and only leave the private tail as witness.
 */

use ligetron::api::{get_args, ArgHolder};
use ligetron::bn254fr::{addmod_checked, Bn254Fr};
use ligetron::poseidon2::Poseidon2Context;

type Hash32 = [u8; 32];

// Public NOTE preimage chunks (31 bytes each) are built into these buffers so
// `bn254fr_assert_equal_bytes` can safely treat them as uniformly-public memory.
//
// IMPORTANT: Ligero tracks secrecy at the byte level; stack slots can end up
// "mixed" (some bytes secret, some public) due to reuse. Passing such a slice to
// `assert_equal_bytes_be` will trap. Keeping these buffers in static memory and
// only ever writing public bytes avoids that.
static mut NOTE_CHUNK0_BYTES: [u8; 31] = [0u8; 31];
static mut NOTE_CHUNK1_PREFIX_BYTES: [u8; 31] = [0u8; 31];

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
    let zero = Bn254Fr::new();
    let one = Bn254Fr::from_u32(1);
    Bn254Fr::assert_equal(&zero, &one);
    fail_with_code(code)
}

#[inline(always)]
fn read_hash32(args: &ArgHolder, index: usize, fail_code: u32) -> Hash32 {
    let bytes = args.get_as_bytes(index);
    let mut out = [0u8; 32];
    if bytes.len() == 32 {
        out.copy_from_slice(bytes);
        return out;
    }

    // Fallback: decode ASCII hex (accept optional 0x/0X prefix).
    let hex_bytes = if bytes.len() >= 2 && bytes[0] == b'0' && (bytes[1] == b'x' || bytes[1] == b'X') {
        &bytes[2..]
    } else {
        bytes
    };

    if hex_bytes.is_empty() {
        hard_fail(fail_code);
    }

    for i in 0..32 {
        let idx = i * 2;
        let hi = if idx < hex_bytes.len() {
            hex_char_to_nibble(hex_bytes[idx])
        } else {
            0
        };
        let lo = if idx + 1 < hex_bytes.len() {
            hex_char_to_nibble(hex_bytes[idx + 1])
        } else {
            0
        };
        out[i] = (hi << 4) | lo;
    }

    out
}

/// Convert ASCII hex char -> nibble WITHOUT table lookups (no secret-dependent memory access).
/// Valid: '0'..'9','a'..'f','A'..'F'. Invalid maps to 0.
#[inline(always)]
fn hex_char_to_nibble(c: u8) -> u8 {
    let d = c.wrapping_sub(b'0');
    let md = (0u8).wrapping_sub((d <= 9) as u8);

    let a = c.wrapping_sub(b'a');
    let ma = (0u8).wrapping_sub((a <= 5) as u8);

    #[allow(non_snake_case)]
    let A = c.wrapping_sub(b'A');
    #[allow(non_snake_case)]
    let mA = (0u8).wrapping_sub((A <= 5) as u8);

    (d & md) | (a.wrapping_add(10) & ma) | (A.wrapping_add(10) & mA)
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
fn fr_from_bytes_be(bytes: &[u8]) -> Bn254Fr {
    let mut out = Bn254Fr::new();
    out.set_bytes_big(bytes);
    out
}

#[inline(always)]
fn assert_fr_eq_hash32(computed: &Bn254Fr, expected_be: &Hash32) {
    Bn254Fr::assert_equal_bytes_be(computed, expected_be);
}

/// NOTE commitment: H("NOTE_V1" || domain || value_16_le || rho || recipient)
///
/// This reproduces `poseidon2_hash_bytes(preimage)` but binds the PUBLIC parts
/// (domain and value) by absorbing the first two 31-byte chunks as constants:
///   chunk0 = "NOTE_V1" || domain[0..24]
///   chunk1 = domain[24..32] || value_16_le || rho[0..7]
///
/// Remaining bytes (private): rho[7..32] || recipient (57 bytes) are absorbed via
/// `digest_update_bytes` and finalized with standard 0x80 padding.
fn note_commitment_fr_bound(domain: &Hash32, value: u64, rho: &Hash32, recipient: &Hash32) -> Bn254Fr {
    // chunk0 = "NOTE_V1"(7) || domain[0..24](24) = 31 bytes
    let chunk0_fr = unsafe {
        NOTE_CHUNK0_BYTES[..7].copy_from_slice(b"NOTE_V1");
        NOTE_CHUNK0_BYTES[7..31].copy_from_slice(&domain[..24]);
        let chunk0_bytes = std::slice::from_raw_parts(
            (&raw const NOTE_CHUNK0_BYTES) as *const u8,
            31,
        );
        let chunk0_fr = fr_from_bytes_be(chunk0_bytes);
        Bn254Fr::assert_equal_bytes_be(&chunk0_fr, chunk0_bytes);
        chunk0_fr
    };

    // value_16_le = u64 LE, zero-extended to 16 bytes.
    let mut v16 = [0u8; 16];
    v16[..8].copy_from_slice(&value.to_le_bytes());

    // chunk1 = domain[24..32](8) || value_16_le(16) || rho[0..7](7) = 31 bytes
    // Bind prefix (domain tail + value + 7 zeros) as a constant; add rho_prefix as witness.
    let chunk1_prefix_fr = unsafe {
        NOTE_CHUNK1_PREFIX_BYTES[..8].copy_from_slice(&domain[24..32]);
        NOTE_CHUNK1_PREFIX_BYTES[8..24].copy_from_slice(&v16);
        NOTE_CHUNK1_PREFIX_BYTES[24..31].fill(0);
        let prefix_bytes = std::slice::from_raw_parts(
            (&raw const NOTE_CHUNK1_PREFIX_BYTES) as *const u8,
            31,
        );
        let prefix_fr = fr_from_bytes_be(prefix_bytes);
        Bn254Fr::assert_equal_bytes_be(&prefix_fr, prefix_bytes);
        prefix_fr
    };

    let rho_prefix_fr = fr_from_bytes_be(&rho[..7]);
    let mut chunk1_fr = Bn254Fr::new();
    addmod_checked(&mut chunk1_fr, &chunk1_prefix_fr, &rho_prefix_fr);

    // Remaining bytes from preimage offset 62 onwards:
    // rho[7..32] (25 bytes) || recipient (32 bytes) = 57 bytes.
    let mut rest = [0u8; 57];
    rest[..25].copy_from_slice(&rho[7..32]);
    rest[25..57].copy_from_slice(recipient);

    let mut ctx = Poseidon2Context::new();
    ctx.digest_update(&chunk0_fr);
    ctx.digest_update(&chunk1_fr);
    ctx.digest_update_bytes(&rest);
    ctx.digest_final()
}

fn main() {
    let args = get_args();
    if args.len() != 6 {
        hard_fail(70);
    }

    let domain = read_hash32(&args, 1, 71);
    let value = read_u64(&args, 2, 72);
    let rho = read_hash32(&args, 3, 73);
    let recipient = read_hash32(&args, 4, 74);
    let cm_out_arg = read_hash32(&args, 5, 75);

    let cm_out_fr = note_commitment_fr_bound(&domain, value, &rho, &recipient);
    assert_fr_eq_hash32(&cm_out_fr, &cm_out_arg);
}
