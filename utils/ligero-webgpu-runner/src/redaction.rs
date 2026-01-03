//! Helpers to redact private Ligero arguments before logging or emitting replay scripts.
//!
//! The goal is to preserve *parseability* (e.g. keep hex strings looking like hex, keep numbers
//! looking like numbers) while removing sensitive material.

use crate::LigeroArg;

/// Redact a single Ligero argument while preserving basic parseability.
///
/// Mirrors the historical `verifier.rs` redaction behavior:
/// - `String`:
///   - if it starts with `0x...`, replace the body with zeros (keep `0x`)
///   - if it is pure hex, replace with zeros
///   - if it is pure decimal, replace with zeros
///   - otherwise replace with `x` (same length; minimum 1)
/// - `I64`: replaced with `0`
/// - `Hex`: replaced with zeros (same length; minimum 1)
pub fn redact_arg(arg: &LigeroArg) -> LigeroArg {
    match arg {
        LigeroArg::String { str: s } => {
            let trimmed = s.trim();
            if trimmed.starts_with("0x") && trimmed.len() >= 2 {
                let body = &trimmed[2..];
                return LigeroArg::String {
                    str: format!("0x{}", "0".repeat(body.len())),
                };
            }

            let is_hex = !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_hexdigit());
            if is_hex {
                return LigeroArg::String {
                    str: "0".repeat(trimmed.len()),
                };
            }

            let is_dec = !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_digit());
            if is_dec {
                return LigeroArg::String {
                    str: "0".repeat(trimmed.len().max(1)),
                };
            }

            LigeroArg::String {
                str: "x".repeat(trimmed.len().max(1)),
            }
        }
        LigeroArg::I64 { .. } => LigeroArg::I64 { i64: 0 },
        LigeroArg::Hex { hex: h } => LigeroArg::Hex {
            hex: "0".repeat(h.len().max(1)),
        },
    }
}

/// Redact the arguments at the given 1-based indices (out-of-range indices are ignored).
pub fn redact_private_args(args: &mut [LigeroArg], private_indices_1based: &[usize]) {
    for &idx1 in private_indices_1based {
        if idx1 == 0 {
            continue;
        }
        let idx0 = idx1 - 1;
        if let Some(a) = args.get_mut(idx0) {
            *a = redact_arg(a);
        }
    }
}

/// Return a redacted copy of the args, redacting the given 1-based indices.
pub fn redacted_args(args: &[LigeroArg], private_indices_1based: &[usize]) -> Vec<LigeroArg> {
    let mut out = args.to_vec();
    redact_private_args(&mut out, private_indices_1based);
    out
}


