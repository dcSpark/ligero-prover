//! Integration tests for private argument redaction.
//!
//! These tests verify the redaction logic using realistic argument layouts
//! based on the note spending circuit patterns.

use ligero_private_args::{redact_private_args, redacted_args, LigeroArg};

// ============================================================================
// Basic Redaction Tests
// ============================================================================

#[test]
fn test_redact_string() {
    let arg = LigeroArg::string("secret");
    let redacted = arg.redacted();
    assert_eq!(redacted, LigeroArg::String { str: "______".to_string() });
}

#[test]
fn test_redact_i64() {
    let arg = LigeroArg::i64(12345);
    let redacted = arg.redacted();
    assert_eq!(redacted, LigeroArg::I64 { i64: 0 });
}

#[test]
fn test_redact_i64_negative() {
    let arg = LigeroArg::i64(-999);
    let redacted = arg.redacted();
    assert_eq!(redacted, LigeroArg::I64 { i64: 0 });
}

#[test]
fn test_redact_hex() {
    let arg = LigeroArg::hex("deadbeef");
    let redacted = arg.redacted();
    assert_eq!(redacted, LigeroArg::Hex { hex: "00000000".to_string() });
}

#[test]
fn test_hex_strips_0x_prefix() {
    let arg = LigeroArg::hex("0xabcd");
    assert_eq!(arg, LigeroArg::Hex { hex: "abcd".to_string() });
}

#[test]
fn test_hex_strips_uppercase_0x_prefix() {
    let arg = LigeroArg::hex("0XABCD");
    assert_eq!(arg, LigeroArg::Hex { hex: "ABCD".to_string() });
}

// ============================================================================
// 1-Based Indexing Tests
// ============================================================================

#[test]
fn test_redact_private_args_1based_indexing() {
    let mut args = vec![
        LigeroArg::string("public"),
        LigeroArg::hex("private_hex"),
        LigeroArg::i64(42),
    ];

    // Redact index 2 (1-based) = args[1]
    redact_private_args(&mut args, &[2]);

    assert_eq!(args[0], LigeroArg::String { str: "public".to_string() });
    assert_eq!(args[1], LigeroArg::Hex { hex: "00000000000".to_string() });
    assert_eq!(args[2], LigeroArg::I64 { i64: 42 });
}

#[test]
fn test_redact_multiple_indices() {
    let mut args = vec![
        LigeroArg::i64(100),
        LigeroArg::hex("abcd"),
        LigeroArg::string("test"),
    ];

    redact_private_args(&mut args, &[1, 3]);

    assert_eq!(args[0], LigeroArg::I64 { i64: 0 });
    assert_eq!(args[1], LigeroArg::Hex { hex: "abcd".to_string() });
    assert_eq!(args[2], LigeroArg::String { str: "____".to_string() });
}

#[test]
fn test_redact_out_of_bounds_ignored() {
    let mut args = vec![LigeroArg::i64(42)];

    // Index 0 and 5 are out of bounds (1-based)
    redact_private_args(&mut args, &[0, 5]);

    // Original value unchanged
    assert_eq!(args[0], LigeroArg::I64 { i64: 42 });
}

#[test]
fn test_redacted_args_returns_copy() {
    let args = vec![
        LigeroArg::string("keep"),
        LigeroArg::string("redact"),
    ];

    let redacted = redacted_args(&args, &[2]);

    // Original unchanged
    assert_eq!(args[1], LigeroArg::String { str: "redact".to_string() });
    // Copy is redacted
    assert_eq!(redacted[1], LigeroArg::String { str: "______".to_string() });
}

// ============================================================================
// Serde Tests
// ============================================================================

#[test]
fn test_serde_roundtrip() {
    let args = vec![
        LigeroArg::string("hello"),
        LigeroArg::i64(-123),
        LigeroArg::hex("cafe"),
    ];

    let json = serde_json::to_string(&args).unwrap();
    let parsed: Vec<LigeroArg> = serde_json::from_str(&json).unwrap();

    assert_eq!(args, parsed);
}

#[test]
fn test_serde_redacted_roundtrip() {
    let args = vec![
        LigeroArg::string("secret_key"),
        LigeroArg::i64(42),
        LigeroArg::hex("deadbeefcafe"),
    ];

    let redacted = redacted_args(&args, &[1, 3]);
    let json = serde_json::to_string(&redacted).unwrap();
    let parsed: Vec<LigeroArg> = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed[0], LigeroArg::String { str: "__________".to_string() });
    assert_eq!(parsed[1], LigeroArg::I64 { i64: 42 });
    assert_eq!(parsed[2], LigeroArg::Hex { hex: "000000000000".to_string() });
}

// ============================================================================
// Realistic Note Spend Layout Tests
// ============================================================================

/// Helper to create a 64-char hex string (32 bytes)
fn hex32(fill: u8) -> String {
    hex::encode([fill; 32])
}

/// Helper to encode u64 as 32-byte BE hex (value in last 8 bytes)
fn u64_be32_hex(v: u64) -> String {
    let mut out = [0u8; 32];
    out[24..32].copy_from_slice(&v.to_be_bytes());
    hex::encode(out)
}

/// Test the argument layout used by the note-spend circuit.
///
/// Layout (from integration test):
/// ```text
///   1: domain (hex)
///   2: value (i64) - input note value (public)
///   3: rho (hex) [PRIVATE]
///   4: recipient (hex) [PRIVATE]
///   5: spend_sk (hex) [PRIVATE]
///   6: depth (i64)
///   7 to 6+depth: position bits [PRIVATE]
///   7+depth to 6+2*depth: siblings [PRIVATE]
///   7+2*depth: anchor (str) - PUBLIC
///   8+2*depth: nullifier (str) - PUBLIC
///   9+2*depth: withdraw_amount (hex) - can be public or private
///   10+2*depth: n_out (i64)
///   11+2*depth+: per-output args
/// ```
#[test]
fn test_note_spend_argument_layout() {
    let depth: usize = 4; // Use smaller depth for test

    // Build argument vector matching note-spend layout
    let mut args: Vec<LigeroArg> = Vec::new();

    // 1: domain (hex) - PUBLIC
    args.push(LigeroArg::hex(hex32(0x01)));
    // 2: value (i64) - PUBLIC
    args.push(LigeroArg::i64(500));
    // 3: rho (hex) - PRIVATE
    args.push(LigeroArg::hex(hex32(0x02)));
    // 4: recipient (hex) - PRIVATE
    args.push(LigeroArg::hex(hex32(0x03)));
    // 5: spend_sk (hex) - PRIVATE
    args.push(LigeroArg::hex(hex32(0x04)));
    // 6: depth (i64) - PUBLIC
    args.push(LigeroArg::i64(depth as i64));

    // 7 to 6+depth: position bits - PRIVATE
    for i in 0..depth {
        let mut bit_bytes = [0u8; 32];
        bit_bytes[31] = (i % 2) as u8; // Alternating bits
        args.push(LigeroArg::hex(hex::encode(bit_bytes)));
    }

    // 7+depth to 6+2*depth: siblings - PRIVATE
    for i in 0..depth {
        args.push(LigeroArg::hex(hex32(0x10 + i as u8)));
    }

    // 7+2*depth: anchor (str) - PUBLIC
    args.push(LigeroArg::string(format!("0x{}", hex32(0xAA))));
    // 8+2*depth: nullifier (str) - PUBLIC
    args.push(LigeroArg::string(format!("0x{}", hex32(0xBB))));
    // 9+2*depth: withdraw_amount (hex) - PUBLIC (for transparent withdrawals)
    args.push(LigeroArg::hex(u64_be32_hex(200)));
    // 10+2*depth: n_out (i64) - PUBLIC
    args.push(LigeroArg::i64(1));

    // Output args: value, rho, pk, cm
    // 11+2*depth: change_value (hex) - PRIVATE
    args.push(LigeroArg::hex(u64_be32_hex(300)));
    // 12+2*depth: change_rho (hex) - PRIVATE
    args.push(LigeroArg::hex(hex32(0xCC)));
    // 13+2*depth: change_pk (hex) - PRIVATE
    args.push(LigeroArg::hex(hex32(0xDD)));
    // 14+2*depth: cm_change (hex) - PUBLIC
    args.push(LigeroArg::hex(hex32(0xEE)));

    // Build private indices (1-based: add 1 to 0-based index)
    let mut private_indices: Vec<usize> = vec![3, 4, 5]; // rho (idx 2), recipient (idx 3), spend_sk (idx 4)

    // Position bits: 0-based indices 6 to 6+depth-1 → 1-based 7 to 7+depth-1
    for i in 0..depth {
        private_indices.push(7 + i);
    }

    // Siblings: 0-based indices 6+depth to 6+2*depth-1 → 1-based 7+depth to 7+2*depth-1
    for i in 0..depth {
        private_indices.push(7 + depth + i);
    }

    // Output private fields (1-based indices)
    // 0-based: change_value is at 6 + 2*depth + 4 → 1-based: 7 + 2*depth + 4
    let change_value_1based = 7 + 2 * depth + 4;
    let change_rho_1based = 7 + 2 * depth + 5;
    let change_pk_1based = 7 + 2 * depth + 6;
    private_indices.push(change_value_1based); // change_value
    private_indices.push(change_rho_1based);   // change_rho
    private_indices.push(change_pk_1based);    // change_pk

    // Redact
    let redacted = redacted_args(&args, &private_indices);

    // Total args: 6 base + depth position bits + depth siblings + 4 post-merkle + 4 output = 6 + 2*depth + 8
    assert_eq!(args.len(), 6 + 2 * depth + 8);

    // 0-based indices for array access
    let anchor_idx = 6 + 2 * depth;        // args[14] for depth=4
    let nullifier_idx = 6 + 2 * depth + 1; // args[15]
    let withdraw_idx = 6 + 2 * depth + 2;  // args[16]
    let n_out_idx = 6 + 2 * depth + 3;     // args[17]
    let change_value_idx = 6 + 2 * depth + 4; // args[18]
    let change_rho_idx = 6 + 2 * depth + 5;   // args[19]
    let change_pk_idx = 6 + 2 * depth + 6;    // args[20]
    let cm_idx = 6 + 2 * depth + 7;           // args[21]

    // Verify PUBLIC args are unchanged
    assert_eq!(redacted[0], args[0], "domain should be unchanged");
    assert_eq!(redacted[1], args[1], "value should be unchanged");
    assert_eq!(redacted[5], args[5], "depth should be unchanged");
    assert_eq!(redacted[anchor_idx], args[anchor_idx], "anchor should be unchanged");
    assert_eq!(redacted[nullifier_idx], args[nullifier_idx], "nullifier should be unchanged");
    assert_eq!(redacted[withdraw_idx], args[withdraw_idx], "withdraw_amount should be unchanged");
    assert_eq!(redacted[n_out_idx], args[n_out_idx], "n_out should be unchanged");
    assert_eq!(redacted[cm_idx], args[cm_idx], "cm_change should be unchanged");

    // Verify PRIVATE args are redacted
    // rho, recipient, spend_sk should be all zeros
    assert_eq!(redacted[2], LigeroArg::Hex { hex: "0".repeat(64) }, "rho should be redacted");
    assert_eq!(redacted[3], LigeroArg::Hex { hex: "0".repeat(64) }, "recipient should be redacted");
    assert_eq!(redacted[4], LigeroArg::Hex { hex: "0".repeat(64) }, "spend_sk should be redacted");

    // Position bits should be redacted (indices 6 to 6+depth-1)
    for i in 0..depth {
        assert_eq!(
            redacted[6 + i],
            LigeroArg::Hex { hex: "0".repeat(64) },
            "Position bit {} should be redacted",
            i
        );
    }

    // Siblings should be redacted (indices 6+depth to 6+2*depth-1)
    for i in 0..depth {
        assert_eq!(
            redacted[6 + depth + i],
            LigeroArg::Hex { hex: "0".repeat(64) },
            "Sibling {} should be redacted",
            i
        );
    }

    // Output private fields should be redacted
    assert_eq!(
        redacted[change_value_idx],
        LigeroArg::Hex { hex: "0".repeat(64) },
        "change_value should be redacted"
    );
    assert_eq!(
        redacted[change_rho_idx],
        LigeroArg::Hex { hex: "0".repeat(64) },
        "change_rho should be redacted"
    );
    assert_eq!(
        redacted[change_pk_idx],
        LigeroArg::Hex { hex: "0".repeat(64) },
        "change_pk should be redacted"
    );
}

/// Test that redaction preserves argument count and order.
#[test]
fn test_redaction_preserves_structure() {
    let args: Vec<LigeroArg> = (0..20)
        .map(|i| {
            if i % 3 == 0 {
                LigeroArg::i64(i as i64 * 100)
            } else if i % 3 == 1 {
                LigeroArg::string(format!("arg_{}", i))
            } else {
                LigeroArg::hex(format!("{:08x}", i))
            }
        })
        .collect();

    let private_indices: Vec<usize> = (1..=20).filter(|i| i % 2 == 0).collect();
    let redacted = redacted_args(&args, &private_indices);

    // Same length
    assert_eq!(redacted.len(), args.len());

    // Check each element type is preserved
    for i in 0..args.len() {
        match (&args[i], &redacted[i]) {
            (LigeroArg::I64 { .. }, LigeroArg::I64 { .. }) => {}
            (LigeroArg::String { .. }, LigeroArg::String { .. }) => {}
            (LigeroArg::Hex { .. }, LigeroArg::Hex { .. }) => {}
            _ => panic!("Type mismatch at index {}", i),
        }
    }
}

/// Test empty private indices (no redaction).
#[test]
fn test_empty_private_indices() {
    let args = vec![
        LigeroArg::string("keep"),
        LigeroArg::i64(42),
        LigeroArg::hex("abcd"),
    ];

    let redacted = redacted_args(&args, &[]);

    assert_eq!(args, redacted);
}

/// Test all indices private.
#[test]
fn test_all_indices_private() {
    let args = vec![
        LigeroArg::string("secret1"),
        LigeroArg::i64(12345),
        LigeroArg::hex("deadbeef"),
    ];

    let redacted = redacted_args(&args, &[1, 2, 3]);

    assert_eq!(redacted[0], LigeroArg::String { str: "_______".to_string() });
    assert_eq!(redacted[1], LigeroArg::I64 { i64: 0 });
    assert_eq!(redacted[2], LigeroArg::Hex { hex: "00000000".to_string() });
}

/// Test duplicate indices are handled correctly.
#[test]
fn test_duplicate_indices() {
    let mut args = vec![
        LigeroArg::string("test"),
        LigeroArg::i64(42),
    ];

    // Duplicate index 1
    redact_private_args(&mut args, &[1, 1, 1]);

    // Should still be redacted (idempotent)
    assert_eq!(args[0], LigeroArg::String { str: "____".to_string() });
    assert_eq!(args[1], LigeroArg::I64 { i64: 42 });
}

/// Test unsorted indices work correctly.
#[test]
fn test_unsorted_indices() {
    let mut args = vec![
        LigeroArg::i64(1),
        LigeroArg::i64(2),
        LigeroArg::i64(3),
        LigeroArg::i64(4),
        LigeroArg::i64(5),
    ];

    // Indices in random order
    redact_private_args(&mut args, &[5, 2, 4, 1, 3]);

    // All should be redacted
    for (i, arg) in args.iter().enumerate() {
        assert_eq!(*arg, LigeroArg::I64 { i64: 0 }, "Index {} should be redacted", i);
    }
}

/// Test very large indices are ignored.
#[test]
fn test_very_large_indices() {
    let mut args = vec![LigeroArg::i64(42)];

    redact_private_args(&mut args, &[usize::MAX, 1000000]);

    // Original unchanged
    assert_eq!(args[0], LigeroArg::I64 { i64: 42 });
}

/// Test empty args vector.
#[test]
fn test_empty_args() {
    let mut args: Vec<LigeroArg> = vec![];

    redact_private_args(&mut args, &[1, 2, 3]);

    assert!(args.is_empty());
}

/// Test redaction of zero-length strings.
#[test]
fn test_empty_string_redaction() {
    let arg = LigeroArg::string("");
    let redacted = arg.redacted();
    assert_eq!(redacted, LigeroArg::String { str: "".to_string() });
}

/// Test redaction of zero-length hex.
#[test]
fn test_empty_hex_redaction() {
    let arg = LigeroArg::Hex { hex: "".to_string() };
    let redacted = arg.redacted();
    assert_eq!(redacted, LigeroArg::Hex { hex: "".to_string() });
}
