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
 *
 * NOTE:
 *   Deposits do not attest to a sender inside the pool; `sender_id` is fixed to 0x00..00.
 */

use ligetron::api::{get_args, ArgHolder};
use ligetron::bn254fr::Bn254Fr;
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
    if args.len() != 7 {
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

    let recipient = recipient_from_pk(&h, &domain, &pk_spend_recipient, &pk_ivk_recipient);
    let sender_id = [0u8; 32];
    let cm_out_fr = note_commitment_fr(&h, &domain, value, &rho, &recipient, &sender_id);
    assert_fr_eq_hash32(&cm_out_fr, &cm_out_arg);
}
