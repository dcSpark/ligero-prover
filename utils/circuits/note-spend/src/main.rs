/*
 * Rust Guest Program: Note Spend Verifier for Midnight Privacy Pool (outputs + balance)
 *
 * NOTE: DEPOSIT is implemented as a separate, cheaper guest program:
 *   - `utils/circuits/note-deposit` → `utils/circuits/bins/note_deposit_guest.wasm`
 * This guest verifies spends (TRANSFER / WITHDRAW) of an existing note.
 *
 * Verifies a single-input spend with up to two shielded outputs:
 *   1) Merkle root (anchor) from note commitment + auth path
 *   2) PRF-based nullifier: Poseidon2("PRF_NF_V1" || domain || nf_key || rho)
 *   3) Output note commitments (0..=2): Poseidon2("NOTE_V1" || domain || value || rho || recipient)
 *   4) Balance: input_value == withdraw_amount + sum(output_values)
 *
 * =============================================================================
 * BUSINESS REQUIREMENTS - Privacy Pool Transaction Types
 * =============================================================================
 *
 * This circuit supports three transaction types in a shielded payment system:
 *
 * ┌─────────────────────────────────────────────────────────────────────────────┐
 * │ DEPOSIT - Enter the privacy pool                                           │
 * │                                                                             │
 * │   Public:  value (input amount), origin (sender address on transparent)    │
 * │   Private: recipient (who receives the shielded note)                       │
 * │                                                                             │
 * │   Use case: User deposits 100 tokens from their public address into the    │
 * │   shielded pool. Everyone sees the deposit amount and source, but the      │
 * │   recipient's shielded address is hidden.                                  │
 * │                                                                             │
 * │   Circuit config: n_out=1, withdraw_amount=0                               │
 * │   The input comes from a transparent source (not a spent note).            │
 * └─────────────────────────────────────────────────────────────────────────────┘
 *
 * ┌─────────────────────────────────────────────────────────────────────────────┐
 * │ TRANSFER - Fully private transaction within the pool                       │
 * │                                                                             │
 * │   Public:  anchor (state root), nullifier (prevents double-spend),         │
 * │            cm_out (output commitments)                                      │
 * │   Private: value, origin (which note is spent), recipient                   │
 * │                                                                             │
 * │   Use case: Alice sends 50 tokens to Bob. Observers see that *some*        │
 * │   transaction occurred (nullifier published, new commitments added),       │
 * │   but cannot determine the amount, sender, or recipient.                   │
 * │                                                                             │
 * │   Circuit config: n_out=1 or 2, withdraw_amount=0                          │
 * │   Can have 2 outputs for change (e.g., spend 100, send 50, keep 50).       │
 * └─────────────────────────────────────────────────────────────────────────────┘
 *
 * ┌─────────────────────────────────────────────────────────────────────────────┐
 * │ WITHDRAW - Exit the privacy pool                                           │
 * │                                                                             │
 * │   Public:  withdraw_amount (value leaving pool), recipient (transparent),  │
 * │            anchor, nullifier, cm_out (change commitment if any)            │
 * │   Private: input value, origin (which note), change value                  │
 * │                                                                             │
 * │   Use case: User withdraws 30 tokens to their public address. Observers    │
 * │   see the withdrawal amount and destination, but don't know which note     │
 * │   was spent or the original balance (change is re-shielded).               │
 * │                                                                             │
 * │   Circuit config: n_out=0 or 1, withdraw_amount>0                          │
 * │   - n_out=0: full withdrawal (entire note value exits)                     │
 * │   - n_out=1: partial withdrawal (change goes to shielded output)           │
 * │                                                                             │
 * │   Balance constraint: input_value = withdraw_amount + sum(output_values)   │
 * └─────────────────────────────────────────────────────────────────────────────┘
 *
 * =============================================================================
 *
 * ARGUMENT LAYOUT (WASI args_get, 1-indexed):
 * ============================================
 * Mixed ABI - hex args for 32-byte values, i64 for integers, and strings for human-readable hex:
 *   - LigeroArg::Hex { hex: "0x..." }   → Passed as ASCII hex string to guest, parsed here
 *   - LigeroArg::I64 { i64: N }         → 8 bytes little-endian
 *   - LigeroArg::String { str: "0x..." } → Passed as ASCII string to guest, parsed here
 *
 * Arguments:
 *   [1]  domain         — hex arg → 32 bytes
 *   [2]  value          — i64 arg → 8 bytes (input note value as u64)
 *   [3]  rho            — hex arg → 32 bytes
 *   [4]  recipient      — hex arg → 32 bytes  [PRIVATE]
 *                        MUST equal recipient_from_sk(domain, spend_sk)
 *   [5]  spend_sk       — hex arg → 32 bytes  [PRIVATE]
 *                        Used to (a) authorize spend via recipient binding
 *                        and (b) derive nf_key := H("NFKEY_V1"||domain||spend_sk)
 *   [6]  depth          — i64 arg → 8 bytes
 *   [7..7+depth)        — pos_bits[i] — hex arg → 32 bytes each  [PRIVATE]
 *                        Position bits as field elements (0x00...00 or 0x00...01)
 *   [7+depth..7+2*depth) — siblings[i] — hex arg → 32 bytes each  [PRIVATE]
 *   [7+2*depth]  anchor       — str/hex arg → 32 bytes (expected Merkle root)
 *   [8+2*depth]  nullifier    — str/hex arg → 32 bytes (expected nullifier)
 *   [9+2*depth]  withdraw_amount — i64 arg → 8 bytes
 *   [10+2*depth] n_out           — i64 arg → 8 bytes (0, 1, or 2)
 *   For each j in [0..n_out):
 *     [11+2*depth + 4*j + 0] value_out_j  — i64 arg → 8 bytes   [PRIVATE]
 *     [11+2*depth + 4*j + 1] rho_out_j    — hex arg → 32 bytes  [PRIVATE]
 *     [11+2*depth + 4*j + 2] pk_out_j     — hex arg → 32 bytes  [PRIVATE]
 *                                          recipient is DERIVED: H("ADDR_V1"||domain||pk_out)
 *     [11+2*depth + 4*j + 3] cm_out_j     — hex arg → 32 bytes (PUBLIC; must equal computed)
 *
 * Expected argc = 11 + 2*depth + 4*n_out (argc includes argv[0]).
 *
 * SECURITY NOTES:
 *   1) All validation paths inject UNSAT constraints before exit (hard_fail)
 *   2) Balance check uses field-level constraint, not runtime boolean comparison
 *   3) Position bits are constrained to be boolean (0 or 1)
 *   4) Merkle path uses field-level MUX to avoid witness-dependent constraints
 *
 * Hashing uses Ligetron's Poseidon2 via bn254fr host functions (Ligero-compatible).
 *
 * Viewer plaintexts (Level B) are extended to include an attested sender_id:
 *   [ domain | value | rho | recipient | sender_id ]
 * 
 */

// =============================================================================
// Ligetron SDK requires std - the heavy Poseidon2 computation is done via
// host functions anyway, so std overhead is minimal.
// =============================================================================

// Ligetron SDK imports
use ligetron::api::{get_args, ArgHolder};
use ligetron::bn254fr::{Bn254Fr, addmod_checked, submod_checked};
use ligetron::poseidon2::poseidon2_hash_bytes;

/// Exit the program with the given code.
fn exit_with_code(code: i32) -> ! {
    std::process::exit(code)
}

/// Conditional exit with detailed error codes in diagnostics mode.
/// In production (no diagnostics feature), all failures exit with code 71.
///
/// Error code conventions (diagnostics mode):
///   70-79: Argument parsing errors
///   80-89: Constraint verification failures
///   90-99: Viewer attestation errors

#[cfg(feature = "diagnostics")]
fn fail_with_code(code: u32) -> ! {
    exit_with_code(code as i32)
}

#[cfg(not(feature = "diagnostics"))]
fn fail_with_code(_code: u32) -> ! {
    exit_with_code(71)
}

/// Hard failure that injects an UNSAT constraint before exiting.
/// 
/// SECURITY: This is critical for soundness! Without the UNSAT constraint,
/// a malicious prover could trigger a failure path and still get a valid proof
/// for a "truncated" circuit (if the zkVM doesn't enforce exit code checks).
/// 
/// The constraint 0 == 1 is unsatisfiable, ensuring the proof will fail verification.
#[inline(always)]
fn hard_fail(code: u32) -> ! {
    // Force UNSAT: 0 == 1
    let zero = Bn254Fr::new();
    let one = Bn254Fr::from_u32(1);
    Bn254Fr::assert_equal(&zero, &one);
    fail_with_code(code)
}

type Hash32 = [u8; 32];

// =============================================================================
// Ligetron-compatible Poseidon2Core shim
// Uses Ligetron's Poseidon2 implementation via bn254fr host functions.
// Uses Ligetron's Poseidon2 implementation via bn254fr host functions.
// =============================================================================

struct Poseidon2Core;

impl Poseidon2Core {
    #[inline(always)]
    pub fn new() -> Self {
        Self
    }

    /// Return Poseidon2 digest as a field element (constraint-friendly).
    #[inline(always)]
    pub fn hash_padded_fr(&self, preimage: &[u8]) -> Bn254Fr {
        poseidon2_hash_bytes(preimage)
    }

    /// Return Poseidon2 digest as 32-byte BE (for Merkle/preimage composition).
    #[inline(always)]
    pub fn hash_padded(&self, preimage: &[u8]) -> Hash32 {
        let digest = self.hash_padded_fr(preimage);
        bn254fr_to_hash32(&digest)
    }
}

/// Convert a Bn254Fr field element to a 32-byte hash.
/// Uses big-endian byte order for compatibility with existing test vectors.
#[inline(always)]
fn bn254fr_to_hash32(x: &Bn254Fr) -> Hash32 {
    x.to_bytes_be()
}

/// Convert a 32-byte big-endian hash to a Bn254Fr field element.
/// Uses hex encoding to avoid value-dependent parsing.
#[inline(always)]
fn bn254fr_from_hash32_be(h: &Hash32) -> Bn254Fr {
    let mut result = Bn254Fr::new();
    result.set_bytes_big(h);
    result
}

/// Assert that a computed field element equals an expected 32-byte digest (public).
#[inline(always)]
fn assert_fr_eq_hash32(computed: &Bn254Fr, expected_be: &Hash32) {
    // Bind the expected digest bytes into the statement as a constant and constrain `computed` to it.
    //
    // NOTE: This is critical for soundness when the verifier reconstructs the constraint system
    // without evaluating private inputs: parsing bytes into a field element via `set_bytes_big`
    // does *not* by itself constrain the value to equal those bytes.
    Bn254Fr::assert_equal_bytes_be(computed, expected_be);
}

// ============================================================================
// FIELD-LEVEL SELECT FOR OBLIVIOUS MERKLE PATH
// Uses arithmetic selection with private bits (no witness-dependent branching).
// ============================================================================

/// Enforce that a field element is a boolean (0 or 1).
/// Constraint: cond * (cond - 1) == 0
/// 
/// This is critical for soundness: if position bits are not constrained
/// to be boolean, a malicious prover could use other field values to
/// manipulate the Merkle path computation.
#[inline(always)]
fn assert_bit(cond: &Bn254Fr) {
    // Create constant 1
    let one = Bn254Fr::from_u32(1);
    // t = cond - 1
    let mut t = Bn254Fr::new();
    submod_checked(&mut t, cond, &one);
    // t = cond * (cond - 1)
    t.mulmod_checked(cond);
    // Assert t == 0 (only true if cond is 0 or 1)
    let zero = Bn254Fr::new();
    Bn254Fr::assert_equal(&t, &zero);
}

/// Read a position bit from a 32-byte hex argument.
/// The bit should be passed as 0x000...0000 (for 0) or 0x000...0001 (for 1).
/// Returns a Bn254Fr that is either 0 or 1.
#[inline(always)]
fn read_position_bit(args: &ArgHolder, index: usize) -> Bn254Fr {
    let b = read_hash32(args, index);
    bn254fr_from_hash32_be(&b)
}

// ============================================================================
// OPTIMIZED: All values stored as u64 (not u128) to avoid expensive 128-bit ops.
// Values are encoded to 16-byte LE with zero-extension for protocol compatibility.
// ============================================================================


// ============================================================================
// ARGUMENT HELPERS: Read typed args from ArgHolder.
// Ligero prover passes:
//   - LigeroArg::Hex { hex: "0x..." } as raw 32 bytes (unhexed)
//   - LigeroArg::I64 { i64: N }       as 8 bytes little-endian
// ============================================================================

/// Read a 32-byte hash from ArgHolder at given index.
/// Convert ASCII hex char -> nibble WITHOUT table lookups (no secret-dependent memory access).
/// Valid: '0'..'9','a'..'f','A'..'F'. Invalid maps to 0.
/// This uses pure arithmetic masks to avoid any secret-indexed memory access.
#[inline(always)]
fn hex_char_to_nibble(c: u8) -> u8 {
    // d in 0..=255 (wrapping); md = 0xFF iff c in '0'..'9'
    let d = c.wrapping_sub(b'0');
    let md = (0u8).wrapping_sub((d <= 9) as u8);

    // a in 0..=255; ma = 0xFF iff c in 'a'..'f'
    let a = c.wrapping_sub(b'a');
    let ma = (0u8).wrapping_sub((a <= 5) as u8);

    // A in 0..=255; mA = 0xFF iff c in 'A'..'F'
    #[allow(non_snake_case)]
    let A = c.wrapping_sub(b'A');
    #[allow(non_snake_case)]
    let mA = (0u8).wrapping_sub((A <= 5) as u8);

    // Select without branches/tables; only one mask can be 0xFF for valid hex.
    (d & md) | (a.wrapping_add(10) & ma) | (A.wrapping_add(10) & mA)
}

/// Reads a 32-byte hash from a hex arg using constant-time decoding (oblivious).
/// IMPORTANT for Ligero private inputs:
///   - avoid secret-dependent branching AND secret-dependent memory lookups
///   - accept both (a) raw 32-byte ABI and (b) ASCII "0x.." fallback
#[inline(always)]
fn read_hash32(args: &ArgHolder, index: usize) -> Hash32 {
    let bytes = args.get_as_bytes(index);
    let mut out = [0u8; 32];

    // Fast path: some Ligetron ABIs provide `hex` as raw bytes already.
    if bytes.len() == 32 {
        out.copy_from_slice(bytes);
        return out;
    }

    // Fallback: decode ASCII hex (check for 0x prefix properly).
    let hex_bytes = if bytes.len() >= 2 && bytes[0] == b'0' && (bytes[1] == b'x' || bytes[1] == b'X') {
        &bytes[2..]
    } else {
        bytes
    };

    for i in 0..32 {
        let idx = i * 2;
        let hi = if idx < hex_bytes.len() { hex_bytes[idx] } else { b'0' };
        let lo = if idx + 1 < hex_bytes.len() { hex_bytes[idx + 1] } else { b'0' };
        out[i] = (hex_char_to_nibble(hi) << 4) | hex_char_to_nibble(lo);
    }

    out
}

/// Read a non-negative i64 as u64, failing with error code if negative.
#[inline(always)]
fn read_u64(args: &ArgHolder, index: usize, fail_code: u32) -> u64 {
    let v = args.get_as_int(index);
    if v < 0 { hard_fail(fail_code); }
    v as u64
}

/// Read a u32 from an i64 arg, validating range.
#[inline(always)]
fn read_u32(args: &ArgHolder, index: usize, fail_code: u32) -> u32 {
    let v = args.get_as_int(index);
    if v < 0 || v > u32::MAX as i64 { hard_fail(fail_code); }
    v as u32
}

// ============================================================================
// OPTIMIZED HASH FUNCTIONS: Fixed-size buffers, single hasher instance
// Each hash type has a dedicated function with exact buffer size.
// ============================================================================

// Fixed buffer sizes for each hash type (tag + data)
const MT_NODE_BUF_LEN: usize = 10 + 1 + 32 + 32;   // "MT_NODE_V1" + lvl + left + right = 75
const NOTE_CM_BUF_LEN: usize = 7 + 32 + 16 + 32 + 32; // "NOTE_V1" + domain + value + rho + recipient = 119
const PRF_NF_BUF_LEN: usize = 9 + 32 + 32 + 32;    // "PRF_NF_V1" + domain + nf_key + rho = 105
const PK_BUF_LEN: usize = 5 + 32;                   // "PK_V1" + spend_sk = 37
const ADDR_BUF_LEN: usize = 7 + 32 + 32;           // "ADDR_V1" + domain + pk = 71
const NFKEY_BUF_LEN: usize = 8 + 32 + 32;          // "NFKEY_V1" + domain + spend_sk = 72
const FVK_COMMIT_BUF_LEN: usize = 13 + 32;         // "FVK_COMMIT_V1" + fvk = 45
const VIEW_KDF_BUF_LEN: usize = 11 + 32 + 32;      // "VIEW_KDF_V1" + fvk + cm = 75
const VIEW_STREAM_BUF_LEN: usize = 14 + 32 + 4;    // "VIEW_STREAM_V1" + k + ctr = 50
const CT_HASH_BUF_LEN: usize = 10 + 144;           // "CT_HASH_V1" + ct = 154
const VIEW_MAC_BUF_LEN: usize = 11 + 32 + 32 + 32; // "VIEW_MAC_V1" + k + cm + ct_hash = 107

/// Merkle tree node hash: H("MT_NODE_V1" || lvl || left || right)
/// Fixed 75-byte preimage.

fn mt_combine(h: &Poseidon2Core, level: u8, left: &Hash32, right: &Hash32) -> (Bn254Fr, Hash32) {
    let mut buf = [0u8; MT_NODE_BUF_LEN];
    buf[..10].copy_from_slice(b"MT_NODE_V1");
    buf[10] = level;
    buf[11..43].copy_from_slice(left);
    buf[43..75].copy_from_slice(right);
    let fr = h.hash_padded_fr(&buf);
    let bytes = bn254fr_to_hash32(&fr);
    (fr, bytes)
}

/// Note commitment: H("NOTE_V1" || domain || value_16 || rho || recipient)
/// Fixed 119-byte preimage. Value is u64 zero-extended to 16 bytes.

fn note_commitment(h: &Poseidon2Core, domain: &Hash32, value: u64, rho: &Hash32, recipient: &Hash32) -> (Bn254Fr, Hash32) {
    let mut buf = [0u8; NOTE_CM_BUF_LEN];
    buf[..7].copy_from_slice(b"NOTE_V1");
    buf[7..39].copy_from_slice(domain);
    // Encode value as 16-byte LE (zero-extended from u64)
    buf[39..47].copy_from_slice(&value.to_le_bytes());
    // buf[47..55] already zero from initialization (zero-extension)
    buf[55..87].copy_from_slice(rho);
    buf[87..119].copy_from_slice(recipient);
    let fr = h.hash_padded_fr(&buf);
    let bytes = bn254fr_to_hash32(&fr);
    (fr, bytes)
}

/// Nullifier: H("PRF_NF_V1" || domain || nf_key || rho)
/// Fixed 105-byte preimage.

fn nullifier(h: &Poseidon2Core, domain: &Hash32, nf_key: &Hash32, rho: &Hash32) -> (Bn254Fr, Hash32) {
    let mut buf = [0u8; PRF_NF_BUF_LEN];
    buf[..9].copy_from_slice(b"PRF_NF_V1");
    buf[9..41].copy_from_slice(domain);
    buf[41..73].copy_from_slice(nf_key);
    buf[73..105].copy_from_slice(rho);
    let fr = h.hash_padded_fr(&buf);
    let bytes = bn254fr_to_hash32(&fr);
    (fr, bytes)
}

/// pk = H("PK_V1" || spend_sk)
/// Fixed 37-byte preimage.

fn pk_from_sk(h: &Poseidon2Core, spend_sk: &Hash32) -> Hash32 {
    let mut buf = [0u8; PK_BUF_LEN];
    buf[..5].copy_from_slice(b"PK_V1");
    buf[5..37].copy_from_slice(spend_sk);
    h.hash_padded(&buf)
}

/// recipient_addr = H("ADDR_V1" || domain || pk)
/// Fixed 71-byte preimage.

fn recipient_from_pk(h: &Poseidon2Core, domain: &Hash32, pk: &Hash32) -> Hash32 {
    let mut buf = [0u8; ADDR_BUF_LEN];
    buf[..7].copy_from_slice(b"ADDR_V1");
    buf[7..39].copy_from_slice(domain);
    buf[39..71].copy_from_slice(pk);
    h.hash_padded(&buf)
}

/// recipient_addr = H("ADDR_V1" || domain || pk)
/// Returns both (field element, 32-byte hash) to avoid re-parsing hashes into the field.
#[inline(always)]
fn recipient_from_pk_fr(h: &Poseidon2Core, domain: &Hash32, pk: &Hash32) -> (Bn254Fr, Hash32) {
    let mut buf = [0u8; ADDR_BUF_LEN];
    buf[..7].copy_from_slice(b"ADDR_V1");
    buf[7..39].copy_from_slice(domain);
    buf[39..71].copy_from_slice(pk);
    let fr = h.hash_padded_fr(&buf);
    let bytes = bn254fr_to_hash32(&fr);
    (fr, bytes)
}

/// recipient_addr from spend_sk (sk -> pk -> recipient)

#[allow(dead_code)]
fn recipient_from_sk(h: &Poseidon2Core, domain: &Hash32, spend_sk: &Hash32) -> Hash32 {
    let pk = pk_from_sk(h, spend_sk);
    recipient_from_pk(h, domain, &pk)
}

/// recipient_addr from spend_sk (sk -> pk -> recipient), returning (field element, bytes).
#[inline(always)]
fn recipient_from_sk_fr(h: &Poseidon2Core, domain: &Hash32, spend_sk: &Hash32) -> (Bn254Fr, Hash32) {
    let pk = pk_from_sk(h, spend_sk);
    recipient_from_pk_fr(h, domain, &pk)
}

/// nf_key = H("NFKEY_V1" || domain || spend_sk)
/// Fixed 72-byte preimage.

fn nf_key_from_sk(h: &Poseidon2Core, domain: &Hash32, spend_sk: &Hash32) -> Hash32 {
    let mut buf = [0u8; NFKEY_BUF_LEN];
    buf[..8].copy_from_slice(b"NFKEY_V1");
    buf[8..40].copy_from_slice(domain);
    buf[40..72].copy_from_slice(spend_sk);
    h.hash_padded(&buf)
}

// ============================================================================
// OBLIVIOUS MERKLE PATH: Avoid secret-dependent branching.
// If `pos` is private, `if (pos_bit) { ... }` creates secret branching which
// is expensive in Ligetron. Instead, use constant-time conditional swap.
// ============================================================================

/// Constant-time conditional swap of two 32-byte arrays.
/// If bit == 1, swaps a and b. If bit == 0, does nothing.
/// Uses XOR-based swap to avoid branching.
#[inline(always)]
fn cswap32(a: &mut [u8; 32], b: &mut [u8; 32], bit: u8) {
    // mask = 0x00 if bit=0, 0xFF if bit=1
    let mask = (0u8).wrapping_sub(bit & 1);
    let mut i = 0;
    while i < 32 {
        let t = mask & (a[i] ^ b[i]);
        a[i] ^= t;
        b[i] ^= t;
        i += 1;
    }
}

/// Compute Merkle root from leaf and authentication path using OBLIVIOUS algorithm.
/// No secret-dependent branching: uses conditional swap based on position bit.
/// This is critical for zkVM performance when `pos` is a private input.
/// DEPRECATED: Old byte-level version that doesn't work with private position.
/// Kept for reference - use root_from_path_field_level instead.
#[allow(dead_code)]
fn root_from_path_oblivious_old(h: &Poseidon2Core, leaf: &Hash32, pos: u64, siblings: &[Hash32], depth: u32) -> (Bn254Fr, Hash32) {
    if depth == 0 { hard_fail(77); }
    let mut cur = *leaf;
    let mut idx = pos;
    let mut lvl = 0u32;
    let mut cur_fr = Bn254Fr::new();
    while lvl < depth {
        let mut left = cur;
        let mut right = siblings[lvl as usize];
        let bit = (idx & 1) as u8;
        cswap32(&mut left, &mut right, bit);
        let (fr, bytes) = mt_combine(h, lvl as u8, &left, &right);
        cur = bytes;
        cur_fr = fr;
        idx >>= 1;
        lvl += 1;
    }
    (cur_fr, cur)
}

/// Compute Merkle root using FIELD-LEVEL MUX operations.
/// This version works with private position bits and siblings!
/// 
/// Arguments:
/// - h: Poseidon2 hasher instance
/// - leaf_bytes: The leaf commitment as 32-byte hash
/// - pos_bits: Position bits as field elements (0 or 1 each), one per level
/// - siblings_fr: Sibling hashes as field elements, one per level
/// - depth: Number of levels in the Merkle path
/// 
/// The position bits determine which side the current node is on at each level:
/// - bit=0: current is left child, sibling is right child
/// - bit=1: current is right child, sibling is left child
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

    // Initialize current node from leaf
    let mut cur_fr = bn254fr_from_hash32_be(leaf_bytes);

    // Reuse temporaries; this also reduces per-level host overhead.
    let mut left_fr = Bn254Fr::new();
    let mut right_fr = Bn254Fr::new();
    let mut delta = Bn254Fr::new();
    let mut left_bytes = [0u8; 32];
    let mut right_bytes = [0u8; 32];

    let mut lvl = 0usize;
    while lvl < depth {
        let bit = &pos_bits[lvl];
        let sib_fr = &siblings_fr[lvl];

        // 1-mul select:
        // delta = bit * (sib - cur)
        // left  = cur + delta
        // right = sib - delta
        submod_checked(&mut delta, sib_fr, &cur_fr);
        delta.mulmod_checked(bit);
        addmod_checked(&mut left_fr, &cur_fr, &delta);
        submod_checked(&mut right_fr, sib_fr, &delta);

        left_fr.get_bytes_big(&mut left_bytes);
        right_fr.get_bytes_big(&mut right_bytes);

        // Compute hash using byte preimage.
        let (next_fr, _next_bytes) = mt_combine(h, lvl as u8, &left_bytes, &right_bytes);
        cur_fr = next_fr;
        lvl += 1;
    }

    cur_fr
}

// === Level B: Viewer Attestation Functions ===

/// FVK commitment: H("FVK_COMMIT_V1" || fvk)
/// Fixed 45-byte preimage.

fn fvk_commit(h: &Poseidon2Core, fvk: &Hash32) -> (Bn254Fr, Hash32) {
    let mut buf = [0u8; FVK_COMMIT_BUF_LEN];
    buf[..13].copy_from_slice(b"FVK_COMMIT_V1");
    buf[13..45].copy_from_slice(fvk);
    let fr = h.hash_padded_fr(&buf);
    let bytes = bn254fr_to_hash32(&fr);
    (fr, bytes)
}

/// View KDF: H("VIEW_KDF_V1" || fvk || cm)
/// Fixed 75-byte preimage.

fn view_kdf(h: &Poseidon2Core, fvk: &Hash32, cm: &Hash32) -> Hash32 {
    let mut buf = [0u8; VIEW_KDF_BUF_LEN];
    buf[..11].copy_from_slice(b"VIEW_KDF_V1");
    buf[11..43].copy_from_slice(fvk);
    buf[43..75].copy_from_slice(cm);
    h.hash_padded(&buf)
}

/// Stream block: H("VIEW_STREAM_V1" || k || ctr)
/// Fixed 50-byte preimage.

fn stream_block(h: &Poseidon2Core, k: &Hash32, ctr: u32) -> Hash32 {
    let mut buf = [0u8; VIEW_STREAM_BUF_LEN];
    buf[..14].copy_from_slice(b"VIEW_STREAM_V1");
    buf[14..46].copy_from_slice(k);
    buf[46..50].copy_from_slice(&ctr.to_le_bytes());
    h.hash_padded(&buf)
}

/// Stream XOR encrypt for exactly 144 bytes (NOTE_PLAIN_LEN).
/// Optimized: 5 hash calls for 144 bytes (4 full blocks + 16-byte remainder).

fn stream_xor_encrypt_144(h: &Poseidon2Core, k: &Hash32, pt: &[u8; 144], ct_out: &mut [u8; 144]) {
    // Block 0: bytes 0-31
    let ks0 = stream_block(h, k, 0);
    let mut i = 0;
    while i < 32 { ct_out[i] = pt[i] ^ ks0[i]; i += 1; }
    
    // Block 1: bytes 32-63
    let ks1 = stream_block(h, k, 1);
    i = 0;
    while i < 32 { ct_out[32 + i] = pt[32 + i] ^ ks1[i]; i += 1; }
    
    // Block 2: bytes 64-95
    let ks2 = stream_block(h, k, 2);
    i = 0;
    while i < 32 { ct_out[64 + i] = pt[64 + i] ^ ks2[i]; i += 1; }
    
    // Block 3: bytes 96-127
    let ks3 = stream_block(h, k, 3);
    i = 0;
    while i < 32 { ct_out[96 + i] = pt[96 + i] ^ ks3[i]; i += 1; }
    
    // Block 4: bytes 128-143 (16-byte remainder)
    let ks4 = stream_block(h, k, 4);
    i = 0;
    while i < 16 { ct_out[128 + i] = pt[128 + i] ^ ks4[i]; i += 1; }
}

/// Ciphertext hash: H("CT_HASH_V1" || ct)
/// Fixed 154-byte preimage for 144-byte ciphertext.

fn ct_hash(h: &Poseidon2Core, ct: &[u8; 144]) -> (Bn254Fr, Hash32) {
    let mut buf = [0u8; CT_HASH_BUF_LEN];
    buf[..10].copy_from_slice(b"CT_HASH_V1");
    buf[10..154].copy_from_slice(ct);
    let fr = h.hash_padded_fr(&buf);
    let bytes = bn254fr_to_hash32(&fr);
    (fr, bytes)
}

/// View MAC: H("VIEW_MAC_V1" || k || cm || ct_hash)
/// Fixed 107-byte preimage.

fn view_mac(h: &Poseidon2Core, k: &Hash32, cm: &Hash32, ct_h: &Hash32) -> (Bn254Fr, Hash32) {
    let mut buf = [0u8; VIEW_MAC_BUF_LEN];
    buf[..11].copy_from_slice(b"VIEW_MAC_V1");
    buf[11..43].copy_from_slice(k);
    buf[43..75].copy_from_slice(cm);
    buf[75..107].copy_from_slice(ct_h);
    let fr = h.hash_padded_fr(&buf);
    let bytes = bn254fr_to_hash32(&fr);
    (fr, bytes)
}

/// Encode note plaintext for viewer encryption.
/// [ domain(32) | value_le_16 | rho(32) | recipient(32) | sender_id(32) ] => 144 bytes
/// Value is u64 zero-extended to 16 bytes.

fn encode_note_plain(domain: &Hash32, value: u64, rho: &Hash32, recipient: &Hash32, sender_id: &Hash32, out: &mut [u8; 144]) {
    out[0..32].copy_from_slice(domain);
    // Encode value as 16-byte LE (u64 zero-extended to 16 bytes)
    out[32..40].copy_from_slice(&value.to_le_bytes());
    // Explicitly zero the high 8 bytes for self-contained correctness
    // (don't rely on caller to pre-zero the buffer)
    out[40..48].copy_from_slice(&[0u8; 8]);
    out[48..80].copy_from_slice(rho);
    out[80..112].copy_from_slice(recipient);
    out[112..144].copy_from_slice(sender_id);
}

/// Maximum Merkle tree depth supported by the circuit.
/// Must be ≤ 63 to ensure the bound check `pos >= (1u64 << depth)` is safe
/// (shifting by 64 would overflow a u64).
const MAX_DEPTH: usize = 63;
const MAX_OUTS: usize = 2;
const MAX_VIEWERS: usize = 8;
const NOTE_PLAIN_LEN: usize = 144; // 32 + 16 + 32 + 32 + 32 (domain + value + rho + recipient + sender_id)

fn main() {
    // Get command line arguments via ligetron SDK
    let args = get_args();
    let argc = args.len() as u32;

    // Create single hasher instance, reuse for all hashes.
    let h = Poseidon2Core::new();

    // 1) domain [hex arg -> 32 bytes]
    let domain = read_hash32(&args, 1);

    // 2) input value [i64 arg -> 8 bytes]
    let value = read_u64(&args, 2, 72);

    // 3) rho [hex arg -> 32 bytes]
    let rho = read_hash32(&args, 3);

    // 4) recipient [PRIVATE] [hex arg -> 32 bytes]
    let recipient_arg = read_hash32(&args, 4);

    // 5) spend_sk [PRIVATE] [hex arg -> 32 bytes]
    let spend_sk = read_hash32(&args, 5);

    // Derive recipient from spend_sk (authorization binding)
    let (recipient_expected_fr, recipient_expected) = recipient_from_sk_fr(&h, &domain, &spend_sk);
    // Enforce: recipient_arg == recipient_from_sk(domain, spend_sk)
    //
    // SECURITY: This is a constraint (not a runtime branch) so the verifier's
    // redacted-private-input execution still builds the same circuit.
    let recipient_arg_fr = bn254fr_from_hash32_be(&recipient_arg);
    Bn254Fr::assert_equal(&recipient_arg_fr, &recipient_expected_fr);

    // Sender identity (attested) = input-note owner identity
    let sender_id = recipient_expected;

    // === NEW ARGUMENT LAYOUT FOR FIELD-LEVEL MERKLE PATH ===
    // Position is now passed as individual bits (one per level) instead of a single integer.
    // This enables private position without breaking constraints.
    //
    // Layout from arg 6 onwards:
    //   6: depth (i64)
    //   7 to 7+depth-1: position bits [PRIVATE] (hex, each 0x00...00 or 0x00...01)
    //   7+depth to 7+2*depth-1: siblings [PRIVATE] (hex)
    //   7+2*depth: anchor (str)
    //   8+2*depth: nullifier (str)
    //   9+2*depth: withdraw_amount (i64)
    //   10+2*depth: n_out (i64)
    //   ... outputs ...

    // 6) depth [i64 arg -> 8 bytes] - now comes BEFORE position bits
    let depth_u32 = read_u32(&args, 6, 77);
    if depth_u32 > MAX_DEPTH as u32 { hard_fail(77); }
    let depth = depth_u32 as usize;

    // 7 to 7+depth-1) position bits [PRIVATE] [hex args -> field elements]
    // Each bit is either 0x00...00 (for 0) or 0x00...01 (for 1)
    let mut pos_bits: Vec<Bn254Fr> = Vec::with_capacity(depth);
    for i in 0..depth {
        let bit = read_position_bit(&args, 7 + i);
        // Enforce position bit is actually boolean (0 or 1)
        // This is critical for soundness - prevents malicious provers from
        // using arbitrary field values to manipulate the Merkle path
        assert_bit(&bit);
        pos_bits.push(bit);
    }

    // 7+depth to 7+2*depth-1) siblings [PRIVATE] [hex args -> 32 bytes each]
    // Read both as bytes (for hash preimage) and as field elements (for MUX)
    let mut siblings_fr: Vec<Bn254Fr> = Vec::with_capacity(depth);
    for i in 0..depth {
        let sibling = read_hash32(&args, 7 + depth + i);
        siblings_fr.push(bn254fr_from_hash32_be(&sibling));
    }

    // 7+2*depth) anchor [str/hex arg -> 32 bytes]
    // NOTE: We bind it as bytes (not as a parsed field element) to ensure it is part of the
    // public statement in the proof system.
    let anchor_arg = read_hash32(&args, 7 + 2 * depth);

    // 8+2*depth) nullifier [str/hex arg -> 32 bytes]
    let nullifier_arg = read_hash32(&args, 8 + 2 * depth);

    // 9+2*depth) withdraw amount [i64 arg -> 8 bytes]
    let withdraw_amount = read_u64(&args, 9 + 2 * depth, 82);

    // 10+2*depth) n_out in {0,1,2} [i64 arg -> 8 bytes]
    let n_out_u32 = read_u32(&args, 10 + 2 * depth, 83);
    if n_out_u32 > MAX_OUTS as u32 { hard_fail(83); }
    let n_out = n_out_u32 as usize;

    // Enforce business transaction-type rules:
    // - withdraw_amount == 0  => n_out ∈ {1,2} (deposit/transfer must create outputs)
    // - withdraw_amount  > 0  => n_out ∈ {0,1} (withdraw can have at most one change note)
    //
    // These are PUBLIC checks (withdraw_amount + n_out are public), so branching is safe.
    if withdraw_amount == 0 {
        if n_out == 0 { hard_fail(87); }
    } else if n_out > 1 {
        hard_fail(87);
    }

    // Expected argc without viewers
    // argv[0] (program name) + 1 domain + 1 value + 1 rho + 1 recipient + 1 spend_sk + 1 depth
    //          + depth pos_bits + depth siblings + 1 anchor + 1 nullifier
    //          + 1 withdraw + 1 n_out + 4*n_out output args
    // = 1 + 6 + 2*depth + 2 + 2 + 4*n_out = 11 + 2*depth + 4*n_out
    let expected_base = 11u32 + 2 * depth_u32 + 4u32 * n_out_u32;
    
    // Must have at least the base args
    if argc < expected_base { hard_fail(84); }

    // Store output data for viewer encryption — use u64 for values
    struct OutPlain {
        v: u64,
        rho: Hash32,
        rcp: Hash32,
        cm: Hash32,
    }
    let mut outs: [OutPlain; MAX_OUTS] = [
        OutPlain { v: 0, rho: [0; 32], rcp: [0; 32], cm: [0; 32] },
        OutPlain { v: 0, rho: [0; 32], rcp: [0; 32], cm: [0; 32] },
    ];

    // Parse & verify outputs — use u64 arithmetic throughout
    // Output args start at index 11 + 2*depth
    let mut out_sum: u64 = 0;
    for j in 0..n_out {
        let base = 11 + 2 * depth + 4 * j;

        // value_out_j [PRIVATE] [i64 arg -> 8 bytes]
        let vj = read_u64(&args, base + 0, 85);
        out_sum = out_sum.checked_add(vj).unwrap_or_else(|| hard_fail(86));

        // rho_out_j [PRIVATE] [hex arg -> 32 bytes]
        let rho_j = read_hash32(&args, base + 1);

        // pk_out_j [PRIVATE] [hex arg -> 32 bytes]
        let pk_out_j = read_hash32(&args, base + 2);
        
        // Derive recipient from pk_out_j (ensures only valid privacy addresses can receive)
        let rcp_j = recipient_from_pk(&h, &domain, &pk_out_j);

        // cm_out_j (PUBLIC) [hex arg -> 32 bytes]
        let cm_arg = read_hash32(&args, base + 3);

        let (cm_cmp_fr, _cm_cmp_bytes) = note_commitment(&h, &domain, vj, &rho_j, &rcp_j);
        // Use field-level constraint instead of byte equality
        assert_fr_eq_hash32(&cm_cmp_fr, &cm_arg);

        // Store output data for later viewer encryption
        outs[j] = OutPlain { v: vj, rho: rho_j, rcp: rcp_j, cm: cm_arg };
    }

    // Compute input note commitment and anchor using FIELD-LEVEL Merkle path
    // Uses MUX operations that work with private position bits and siblings!
    // Use recipient_expected (derived from spend_sk) instead of recipient arg
    let (_cm_in_fr, cm_in_bytes) = note_commitment(&h, &domain, value, &rho, &recipient_expected);
    
    // Use field-level Merkle path computation
    let anchor_computed_fr = root_from_path_field_level(
        &h,
        &cm_in_bytes,
        &pos_bits[..depth],
        &siblings_fr[..depth],
        depth,
    );
    // Use field-level constraint (bytes-bound): anchor_computed == anchor_arg
    assert_fr_eq_hash32(&anchor_computed_fr, &anchor_arg);

    // Compute PRF nullifier and check (nf_key is derived; not prover-chosen)
    let nf_key = nf_key_from_sk(&h, &domain, &spend_sk);
    let (nf_computed_fr, _nf_bytes) = nullifier(&h, &domain, &nf_key, &rho);
    // Use field-level constraint (bytes-bound): nullifier_computed == nullifier_arg
    assert_fr_eq_hash32(&nf_computed_fr, &nullifier_arg);

    // Balance: input value must equal withdraw + sum(outputs)
    // CRITICAL: Use FIELD-LEVEL constraint, not runtime boolean comparison!
    // A runtime boolean like `assert_one((value == rhs) as i32)` would create
    // witness-dependent constraints that fail verification when verifier runs
    // with obscured private inputs.
    //
    // Instead, we express the balance as: value_fr == withdraw_fr + out_sum_fr
    // This creates uniform constraints regardless of actual values.
    
    // First check for overflow at runtime (inject UNSAT if overflow)
    let _rhs_check = withdraw_amount.checked_add(out_sum).unwrap_or_else(|| hard_fail(90));
    
    // Convert amounts to field elements
    let value_fr = Bn254Fr::from_u64(value);
    let withdraw_fr = Bn254Fr::from_u64(withdraw_amount);
    let out_sum_fr = Bn254Fr::from_u64(out_sum);

    // Bind the public `withdraw_amount` into the statement.
    Bn254Fr::assert_equal_u64(&withdraw_fr, withdraw_amount);
    
    // Compute RHS as field element: withdraw + sum(outputs)
    let mut rhs_fr = Bn254Fr::new();
    addmod_checked(&mut rhs_fr, &withdraw_fr, &out_sum_fr);
    
    // Field constraint: value == withdraw + sum(outputs)
    Bn254Fr::assert_equal(&value_fr, &rhs_fr);

    // --- Level B: Viewer Attestations ---
    // If viewers are declared, verify ct_hash + mac for each (output, viewer)

    let base_after_outs = expected_base as usize;

    // If we have exactly the base args, no viewer attestations
    if argc == expected_base {
        return;
    }

    // Otherwise argc > expected_base, so we must have viewer attestations
    // n_viewers [i64 arg -> 8 bytes]
    let n_viewers: usize = {
        let v = read_u32(&args, base_after_outs, 91) as usize;
        if v > MAX_VIEWERS { hard_fail(91); }
        v
    };

    // Expected argc with viewer attestations:
    //   expected_base + 1 (m_viewers)
    //   + m_viewers * ( 1 public fvk_commit + 1 private fvk + 2*n_out public digests )
    let extra_per_viewer = 1 + 1 + 2 * n_out;
    let expected_argc_b = expected_base + 1u32 + (n_viewers as u32) * (extra_per_viewer as u32);
    if argc != expected_argc_b { hard_fail(92); }

    // Precompute plaintexts once per output, reuse across all viewers.
    let mut out_pts: [[u8; NOTE_PLAIN_LEN]; MAX_OUTS] = [[0u8; NOTE_PLAIN_LEN]; MAX_OUTS];
    for j in 0..n_out {
        encode_note_plain(&domain, outs[j].v, &outs[j].rho, &outs[j].rcp, &sender_id, &mut out_pts[j]);
    }

    // Work buffer for ciphertext only (plaintext is precomputed above)
    let mut ct_buf = [0u8; NOTE_PLAIN_LEN];

    let mut arg_idx = base_after_outs + 1; // start right after m_viewers
    for _vi in 0..n_viewers {
        // 1) Public fvk_commitment [hex arg -> 32 bytes]
        let fvk_commit_arg = read_hash32(&args, arg_idx);
        arg_idx += 1;

        // 2) Private fvk [hex arg -> 32 bytes]
        let fvk = read_hash32(&args, arg_idx);
        arg_idx += 1;

        // Check binding H(fvk) == fvk_commitment (public)
        let (fvk_c_fr, _fvk_c_bytes) = fvk_commit(&h, &fvk);
        // Use field-level constraint
        assert_fr_eq_hash32(&fvk_c_fr, &fvk_commit_arg);

        // 3) For each output, compute ct_hash + mac and compare to public args
        for j in 0..n_out {
            let outp = &outs[j];

            // Key from (fvk, cm_j)
            let k = view_kdf(&h, &fvk, &outp.cm);

            // Encrypt deterministically using precomputed plaintext
            stream_xor_encrypt_144(&h, &k, &out_pts[j], &mut ct_buf);

            // Compute digests
            let (ct_h_fr, ct_h_bytes) = ct_hash(&h, &ct_buf);
            let (macv_fr, _macv_bytes) = view_mac(&h, &k, &outp.cm, &ct_h_bytes);

            // ct_hash (PUBLIC) [hex arg -> 32 bytes]
            let ct_hash_arg = read_hash32(&args, arg_idx);
            arg_idx += 1;
            // Use field-level constraint
            assert_fr_eq_hash32(&ct_h_fr, &ct_hash_arg);

            // mac (PUBLIC) [hex arg -> 32 bytes]
            let mac_arg = read_hash32(&args, arg_idx);
            arg_idx += 1;
            // Use field-level constraint
            assert_fr_eq_hash32(&macv_fr, &mac_arg);
        }
    }
    // Success - all constraints verified
}
