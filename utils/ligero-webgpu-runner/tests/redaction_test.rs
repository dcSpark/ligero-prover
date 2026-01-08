//! Integration tests for private argument redaction.
//!
//! This crate ships redaction helpers used by the verifier path and replay-script generation.

use ligero_runner::{redact_arg, redact_private_args, redacted_args, LigeroArg};

// ============================================================================
// Basic Redaction Tests
// ============================================================================

#[test]
fn test_redact_string_generic() {
    let arg = LigeroArg::String {
        str: "secret".to_string(),
    };
    let redacted = redact_arg(&arg);
    assert_eq!(
        redacted,
        LigeroArg::String {
            str: "xxxxxx".to_string()
        }
    );
}

#[test]
fn test_redact_string_0x_hex_keeps_prefix() {
    let arg = LigeroArg::String {
        str: "0xABCD".to_string(),
    };
    let redacted = redact_arg(&arg);
    assert_eq!(
        redacted,
        LigeroArg::String {
            str: "0x0000".to_string()
        }
    );
}

#[test]
fn test_redact_i64() {
    let arg = LigeroArg::I64 { i64: 12345 };
    let redacted = redact_arg(&arg);
    assert_eq!(redacted, LigeroArg::I64 { i64: 0 });
}

#[test]
fn test_redact_hex() {
    let arg = LigeroArg::Hex {
        hex: "deadbeef".to_string(),
    };
    let redacted = redact_arg(&arg);
    assert_eq!(
        redacted,
        LigeroArg::Hex {
            hex: "00000000".to_string()
        }
    );
}

// ============================================================================
// 1-Based Indexing Tests
// ============================================================================

#[test]
fn test_redact_private_args_1based_indexing() {
    let mut args = vec![
        LigeroArg::String {
            str: "public".to_string(),
        },
        LigeroArg::Hex {
            hex: "abcd".to_string(),
        },
        LigeroArg::I64 { i64: 42 },
    ];

    // Redact index 2 (1-based) = args[1]
    redact_private_args(&mut args, &[2]);

    assert_eq!(
        args[0],
        LigeroArg::String {
            str: "public".to_string()
        }
    );
    assert_eq!(
        args[1],
        LigeroArg::Hex {
            hex: "0000".to_string()
        }
    );
    assert_eq!(args[2], LigeroArg::I64 { i64: 42 });
}

#[test]
fn test_redact_out_of_bounds_ignored() {
    let mut args = vec![LigeroArg::I64 { i64: 42 }];

    // Index 0 and 5 are out of bounds (1-based)
    redact_private_args(&mut args, &[0, 5]);

    // Original value unchanged
    assert_eq!(args[0], LigeroArg::I64 { i64: 42 });
}

#[test]
fn test_redacted_args_returns_copy() {
    let args = vec![
        LigeroArg::String {
            str: "keep".to_string(),
        },
        LigeroArg::String {
            str: "redact".to_string(),
        },
    ];

    let redacted = redacted_args(&args, &[2]);

    // Original unchanged
    assert_eq!(
        args[1],
        LigeroArg::String {
            str: "redact".to_string()
        }
    );
    // Copy is redacted
    assert_eq!(
        redacted[1],
        LigeroArg::String {
            str: "xxxxxx".to_string()
        }
    );
}

// ============================================================================
// Serde Tests
// ============================================================================

#[test]
fn test_serde_roundtrip() {
    let args = vec![
        LigeroArg::String {
            str: "hello".to_string(),
        },
        LigeroArg::I64 { i64: -123 },
        LigeroArg::Hex {
            hex: "cafe".to_string(),
        },
    ];

    let json = serde_json::to_string(&args).unwrap();
    let parsed: Vec<LigeroArg> = serde_json::from_str(&json).unwrap();

    assert_eq!(args, parsed);
}

#[test]
fn test_serde_redacted_roundtrip() {
    let args = vec![
        LigeroArg::String {
            str: "secret_key".to_string(),
        },
        LigeroArg::I64 { i64: 42 },
        LigeroArg::Hex {
            hex: "deadbeefcafe".to_string(),
        },
    ];

    let redacted = redacted_args(&args, &[1, 3]);
    let json = serde_json::to_string(&redacted).unwrap();
    let parsed: Vec<LigeroArg> = serde_json::from_str(&json).unwrap();

    assert_eq!(
        parsed[0],
        LigeroArg::String {
            str: "xxxxxxxxxx".to_string()
        }
    );
    assert_eq!(parsed[1], LigeroArg::I64 { i64: 42 });
    assert_eq!(
        parsed[2],
        LigeroArg::Hex {
            hex: "000000000000".to_string()
        }
    );
}

// ============================================================================
// Realistic Note Spend Layout Tests (shape + “public stays public”)
// ============================================================================

/// Helper to create a 64-char hex string (32 bytes)
fn hex32(fill: u8) -> String {
    hex::encode([fill; 32])
}

#[test]
fn test_note_spend_argument_layout_shape() {
    let depth: usize = 4; // Use smaller depth for test
    let n_in: usize = 1;
    let n_out: usize = 1;

    // Build argument vector matching note-spend v2 layout (see utils/circuits/note-spend/src/main.rs)
    let mut args: Vec<LigeroArg> = Vec::new();

    // 1: domain (hex) - PUBLIC
    args.push(LigeroArg::Hex { hex: hex32(0x01) });

    // 2: spend_sk (hex) - PRIVATE
    args.push(LigeroArg::Hex { hex: hex32(0x04) });
    // 3: pk_ivk_owner (hex) - PRIVATE
    args.push(LigeroArg::Hex { hex: hex32(0x05) });
    // 4: depth (i64) - PUBLIC
    args.push(LigeroArg::I64 { i64: depth as i64 });
    // 5: anchor (hex) - PUBLIC
    args.push(LigeroArg::Hex { hex: hex32(0xAA) });
    // 6: n_in (i64) - PUBLIC
    args.push(LigeroArg::I64 { i64: n_in as i64 });

    // Input 0:
    // 7: value_in_0 (i64) - PRIVATE
    args.push(LigeroArg::I64 { i64: 500 });
    // 8: rho_in_0 (hex) - PRIVATE
    args.push(LigeroArg::Hex { hex: hex32(0x02) });
    // 9: sender_id_in_0 (hex) - PRIVATE
    args.push(LigeroArg::Hex { hex: hex32(0x06) });

    // 10..(9+depth): position bits - PRIVATE
    for i in 0..depth {
        let mut bit_bytes = [0u8; 32];
        bit_bytes[31] = (i % 2) as u8; // Alternating bits
        args.push(LigeroArg::Hex {
            hex: hex::encode(bit_bytes),
        });
    }

    // siblings - PRIVATE
    for i in 0..depth {
        args.push(LigeroArg::Hex {
            hex: hex32(0x10 + i as u8),
        });
    }

    // nullifier_0 (hex) - PUBLIC
    args.push(LigeroArg::Hex { hex: hex32(0xBB) });

    // withdraw_amount (i64) - PUBLIC
    args.push(LigeroArg::I64 { i64: 200 });
    // withdraw_to (hex) - PUBLIC
    args.push(LigeroArg::Hex { hex: hex32(0x00) });
    // n_out (i64) - PUBLIC
    args.push(LigeroArg::I64 { i64: n_out as i64 });

    // Output 0:
    // value_out_0 (i64) - PRIVATE
    args.push(LigeroArg::I64 { i64: 300 });
    // rho_out_0 (hex) - PRIVATE
    args.push(LigeroArg::Hex { hex: hex32(0xCC) });
    // pk_spend_out_0 (hex) - PRIVATE
    args.push(LigeroArg::Hex { hex: hex32(0xDD) });
    // pk_ivk_out_0 (hex) - PRIVATE
    args.push(LigeroArg::Hex { hex: hex32(0xDE) });
    // cm_out_0 (hex) - PUBLIC
    args.push(LigeroArg::Hex { hex: hex32(0xEE) });
    // inv_enforce (hex) - PRIVATE
    args.push(LigeroArg::Hex { hex: hex32(0xEF) });

    // Private indices (1-based)
    let mut private_indices: Vec<usize> = Vec::new();
    private_indices.push(2); // spend_sk
    private_indices.push(3); // pk_ivk_owner
    private_indices.push(7); // value_in_0
    private_indices.push(8); // rho_in_0
    private_indices.push(9); // sender_id_in_0
    for i in 0..depth {
        private_indices.push(10 + i); // pos_bits
        private_indices.push(10 + depth + i); // siblings
    }
    // Outputs: value, rho, pk_spend, pk_ivk are private
    let per_in = 4 + 2 * depth;
    let withdraw_idx = 7 + n_in * per_in;
    let out0_base = withdraw_idx + 3; // 1-based index of value_out_0
    private_indices.push(out0_base + 0); // value_out_0
    private_indices.push(out0_base + 1); // rho_out_0
    private_indices.push(out0_base + 2); // pk_spend_out_0
    private_indices.push(out0_base + 3); // pk_ivk_out_0
    private_indices.push(out0_base + 5); // inv_enforce

    let redacted = redacted_args(&args, &private_indices);
    assert_eq!(redacted.len(), args.len());

    // Public “headline” args stay unchanged.
    assert_eq!(redacted[0], args[0]);
    assert_eq!(redacted[3], args[3]); // depth
    assert_eq!(redacted[4], args[4]); // anchor
    assert_eq!(redacted[5], args[5]); // n_in
    assert_eq!(redacted[17], args[17]); // nullifier_0
    assert_eq!(redacted[18], args[18]); // withdraw_amount
    assert_eq!(redacted[19], args[19]); // withdraw_to
    assert_eq!(redacted[20], args[20]); // n_out
    assert_eq!(redacted[25], args[25]); // cm_out_0
}
