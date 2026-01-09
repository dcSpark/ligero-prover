use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// A Ligero proof package containing both the proof and serialized public output.
///
/// - `proof` is expected to be the prover's proof file bytes:
///   - default: `proof_data.gz` (Boost portable archive + gzip)
///   - when gzip is disabled: `proof_data.bin` (Boost portable archive, uncompressed)
/// - `public_output` is bincode-serialized bytes produced by the caller.
/// - `args_json` contains the (redacted) prover/verifier args as JSON.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LigeroProofPackage {
    pub proof: Vec<u8>,
    pub public_output: Vec<u8>,
    /// JSON-serialized args array (redacted according to `private_indices`).
    pub args_json: Vec<u8>,
    /// 1-based indices of private arguments.
    pub private_indices: Vec<usize>,
}

impl LigeroProofPackage {
    /// Create a new package and redact `args_json` according to `private_indices`.
    ///
    /// `args_json` must be a JSON array whose elements match Ligero's JSON arg encoding
    /// (objects like `{ "str": "..." }`, `{ "i64": 123 }`, `{ "hex": "..." }`).
    pub fn new(
        proof: Vec<u8>,
        public_output: Vec<u8>,
        args_json: Vec<u8>,
        private_indices: Vec<usize>,
    ) -> Result<Self> {
        let args_json = redact_args_json(&args_json, &private_indices)?;
        Ok(Self {
            proof,
            public_output,
            args_json,
            private_indices,
        })
    }

    pub fn is_simulation(&self) -> bool {
        self.proof.is_empty()
    }

    pub fn is_valid_gzip(&self) -> bool {
        self.proof.len() >= 2 && self.proof[0] == 0x1f && self.proof[1] == 0x8b
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).context("Failed to bincode-serialize LigeroProofPackage")
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).context("Failed to bincode-deserialize LigeroProofPackage")
    }

    pub fn public_output_as<T: DeserializeOwned>(&self) -> Result<T> {
        bincode::deserialize(&self.public_output).context("Failed to deserialize public_output")
    }

    /// Deserialize the (redacted) args JSON into a caller-provided type.
    pub fn args_as<T: DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_slice(&self.args_json).context("Failed to deserialize args_json")
    }
}

fn redact_args_json(args_json: &[u8], private_indices_1based: &[usize]) -> Result<Vec<u8>> {
    let mut args = serde_json::from_slice::<Vec<serde_json::Value>>(args_json)
        .context("args_json must be a JSON array")?;

    for &idx1 in private_indices_1based {
        if idx1 == 0 {
            continue;
        }
        let idx0 = idx1 - 1;
        if idx0 >= args.len() {
            continue;
        }
        redact_arg_value(&mut args[idx0]);
    }

    serde_json::to_vec(&args).context("Failed to serialize redacted args_json")
}

fn redact_arg_value(v: &mut serde_json::Value) {
    let serde_json::Value::Object(map) = v else {
        // Unknown encoding; replace with JSON null.
        *v = serde_json::Value::Null;
        return;
    };

    // Ligero args are untagged; exactly one of these keys is expected.
    if let Some(serde_json::Value::String(s)) = map.get("str") {
        let trimmed = s.trim();
        let redacted = if trimmed.starts_with("0x") && trimmed.len() >= 2 {
            let body = &trimmed[2..];
            format!("0x{}", "0".repeat(body.len()))
        } else if !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            "0".repeat(trimmed.len())
        } else if !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_digit()) {
            "0".repeat(trimmed.len().max(1))
        } else {
            "x".repeat(trimmed.len().max(1))
        };
        map.insert("str".to_string(), serde_json::Value::String(redacted));
        return;
    }

    if map.get("i64").is_some() {
        map.insert("i64".to_string(), serde_json::Value::Number(0.into()));
        return;
    }

    let mut handled = false;

    if let Some(serde_json::Value::String(b64)) = map.get("bytes_b64") {
        let decoded_len = general_purpose::STANDARD
            .decode(b64)
            .map(|b| b.len())
            .unwrap_or(0);
        let redacted_b64 = general_purpose::STANDARD.encode(vec![0u8; decoded_len]);
        map.insert(
            "bytes_b64".to_string(),
            serde_json::Value::String(redacted_b64),
        );
        handled = true;
    }

    if let Some(serde_json::Value::String(h)) = map.get("hex") {
        map.insert(
            "hex".to_string(),
            serde_json::Value::String("0".repeat(h.len().max(1))),
        );
        handled = true;
    }

    if handled {
        return;
    }

    // Unknown structure; set to null
    *v = serde_json::Value::Null;
}
