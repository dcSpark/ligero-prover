//! Argument types and redaction logic for Ligero proofs.

use serde::{Deserialize, Serialize};

/// Argument type for Ligero prover/verifier.
///
/// Ligero supports three argument types:
/// - `String`: UTF-8 string values
/// - `I64`: 64-bit signed integers
/// - `Hex`: Hexadecimal-encoded byte strings (without 0x prefix)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LigeroArg {
    /// String argument
    #[serde(rename = "str")]
    String {
        /// String value
        str: String,
    },
    /// 64-bit signed integer argument
    #[serde(rename = "i64")]
    I64 {
        /// Integer value
        i64: i64,
    },
    /// Hexadecimal argument (without 0x prefix)
    #[serde(rename = "hex")]
    Hex {
        /// Hex string value (no 0x prefix)
        hex: String,
    },
}

impl LigeroArg {
    /// Create a new string argument.
    pub fn string(s: impl Into<String>) -> Self {
        LigeroArg::String { str: s.into() }
    }

    /// Create a new i64 argument.
    pub fn i64(value: i64) -> Self {
        LigeroArg::I64 { i64: value }
    }

    /// Create a new u64 argument (stored as i64).
    ///
    /// # Panics
    ///
    /// Panics if value > i64::MAX
    pub fn u64(value: u64) -> Self {
        assert!(
            value <= i64::MAX as u64,
            "u64 value too large for i64 encoding"
        );
        LigeroArg::I64 { i64: value as i64 }
    }

    /// Create a new hex argument.
    ///
    /// Strips `0x` or `0X` prefix if present.
    pub fn hex(s: impl Into<String>) -> Self {
        let value = s.into();
        let hex = if value.starts_with("0x") || value.starts_with("0X") {
            value[2..].to_string()
        } else {
            value
        };
        LigeroArg::Hex { hex }
    }

    /// Create a redacted version of this argument.
    ///
    /// Returns a new argument of the same type and length, but with
    /// placeholder values:
    /// - Strings → `"_"` repeated to match length
    /// - Integers → `0`
    /// - Hex → `"0"` repeated to match length
    pub fn redacted(&self) -> Self {
        match self {
            LigeroArg::String { str: s } => LigeroArg::String {
                str: "_".repeat(s.len()),
            },
            LigeroArg::I64 { .. } => LigeroArg::I64 { i64: 0 },
            LigeroArg::Hex { hex: h } => LigeroArg::Hex {
                hex: "0".repeat(h.len()),
            },
        }
    }
}

/// Redact arguments at the specified indices (1-based indexing).
///
/// Replaces private values with same-length placeholders:
/// - Strings → `"_"` repeated
/// - Integers → `0`
/// - Hex → `"0"` repeated
///
/// This preserves type and length consistency required by the Ligero verifier.
///
/// # Arguments
///
/// * `args` - Mutable slice of arguments to redact in place
/// * `private_indices` - Indices of private arguments (1-based, matching Ligero's native format)
///
/// # Example
///
/// ```rust
/// use ligero_private_args::{LigeroArg, redact_private_args};
///
/// let mut args = vec![
///     LigeroArg::i64(100),
///     LigeroArg::hex("abcd1234"),
///     LigeroArg::string("secret"),
/// ];
///
/// // Redact args at indices 2 and 3 (1-based)
/// redact_private_args(&mut args, &[2, 3]);
///
/// assert!(matches!(&args[0], LigeroArg::I64 { i64: 100 })); // Unchanged
/// assert!(matches!(&args[1], LigeroArg::Hex { hex } if hex == "00000000"));
/// assert!(matches!(&args[2], LigeroArg::String { str } if str == "______"));
/// ```
pub fn redact_private_args(args: &mut [LigeroArg], private_indices: &[usize]) {
    for &idx in private_indices {
        // 1-based indexing: idx 1 = args[0]
        if idx > 0 && idx <= args.len() {
            let arg_idx = idx - 1;
            args[arg_idx] = args[arg_idx].redacted();
        }
    }
}

/// Create a redacted copy of the arguments.
///
/// Same as [`redact_private_args`] but returns a new vector instead of
/// modifying in place.
///
/// # Arguments
///
/// * `args` - Slice of arguments to copy and redact
/// * `private_indices` - Indices of private arguments (1-based)
///
/// # Returns
///
/// A new vector with private arguments replaced by placeholders.
pub fn redacted_args(args: &[LigeroArg], private_indices: &[usize]) -> Vec<LigeroArg> {
    let mut result = args.to_vec();
    redact_private_args(&mut result, private_indices);
    result
}
