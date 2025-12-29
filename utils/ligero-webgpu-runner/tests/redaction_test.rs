//! Integration tests for private argument redaction.
//!
//! This crate ships redaction helpers used by the verifier path and replay-script generation.

use ligero_webgpu_runner::{redact_arg, redact_private_args, redacted_args, LigeroArg};

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

/// Helper to encode u64 as 32-byte BE hex (value in last 8 bytes)
fn u64_be32_hex(v: u64) -> String {
    let mut out = [0u8; 32];
    out[24..32].copy_from_slice(&v.to_be_bytes());
    hex::encode(out)
}

#[test]
fn test_note_spend_argument_layout_shape() {
    let depth: usize = 4; // Use smaller depth for test

    // Build argument vector matching note-spend layout
    let mut args: Vec<LigeroArg> = Vec::new();

    // 1: domain (hex) - PUBLIC
    args.push(LigeroArg::Hex { hex: hex32(0x01) });
    // 2: value (i64) - PUBLIC
    args.push(LigeroArg::I64 { i64: 500 });
    // 3: rho (hex) - PRIVATE
    args.push(LigeroArg::Hex { hex: hex32(0x02) });
    // 4: recipient (hex) - PRIVATE
    args.push(LigeroArg::Hex { hex: hex32(0x03) });
    // 5: spend_sk (hex) - PRIVATE
    args.push(LigeroArg::Hex { hex: hex32(0x04) });
    // 6: depth (i64) - PUBLIC
    args.push(LigeroArg::I64 { i64: depth as i64 });

    // 7 to 6+depth: position bits - PRIVATE
    for i in 0..depth {
        let mut bit_bytes = [0u8; 32];
        bit_bytes[31] = (i % 2) as u8; // Alternating bits
        args.push(LigeroArg::Hex {
            hex: hex::encode(bit_bytes),
        });
    }

    // 7+depth to 6+2*depth: siblings - PRIVATE
    for i in 0..depth {
        args.push(LigeroArg::Hex {
            hex: hex32(0x10 + i as u8),
        });
    }

    // 7+2*depth: anchor (str) - PUBLIC
    args.push(LigeroArg::String {
        str: format!("0x{}", hex32(0xAA)),
    });
    // 8+2*depth: nullifier (str) - PUBLIC
    args.push(LigeroArg::String {
        str: format!("0x{}", hex32(0xBB)),
    });
    // 9+2*depth: withdraw_amount (hex) - PUBLIC (for transparent withdrawals)
    args.push(LigeroArg::Hex {
        hex: u64_be32_hex(200),
    });
    // 10+2*depth: n_out (i64) - PUBLIC
    args.push(LigeroArg::I64 { i64: 1 });

    // Output args: value, rho, pk, cm
    args.push(LigeroArg::Hex {
        hex: u64_be32_hex(300),
    }); // change_value (PRIVATE in some flows)
    args.push(LigeroArg::Hex { hex: hex32(0xCC) }); // change_rho (PRIVATE)
    args.push(LigeroArg::Hex { hex: hex32(0xDD) }); // change_pk  (PRIVATE)
    args.push(LigeroArg::Hex { hex: hex32(0xEE) }); // cm_change  (PUBLIC)

    // Private indices (1-based)
    let mut private_indices: Vec<usize> = vec![3, 4, 5];
    for i in 0..depth {
        private_indices.push(7 + i); // position bits
        private_indices.push(7 + depth + i); // siblings
    }
    // Mark change_rho and change_pk as private.
    let base = 6 + 2 * depth + 4; // 0-based index of first output arg
    private_indices.push(base + 2 + 1); // +1 for 1-based, +2 to reach change_rho
    private_indices.push(base + 3 + 1); // change_pk

    let redacted = redacted_args(&args, &private_indices);
    assert_eq!(redacted.len(), args.len());

    // Public “headline” args stay unchanged.
    assert_eq!(redacted[0], args[0]);
    assert_eq!(redacted[1], args[1]);
    assert_eq!(redacted[5], args[5]);
}