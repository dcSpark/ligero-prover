//! Configuration types for Ligero prover/verifier.

use serde::{Deserialize, Serialize};

use crate::LigeroArg;

/// Configuration for Ligero prover/verifier.
///
/// This struct is serialized to JSON and passed to the prover/verifier binaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LigeroConfig {
    /// Path to the WASM program
    pub program: String,

    /// Path to shader directory
    #[serde(rename = "shader-path")]
    pub shader_path: String,

    /// Packing size (FFT message packing size)
    ///
    /// Default is 8192.
    pub packing: u32,

    /// Indices of private arguments (1-based)
    ///
    /// Arguments at these indices will be redacted in proof packages
    /// and obscured when passed to the verifier.
    #[serde(rename = "private-indices")]
    pub private_indices: Vec<usize>,

    /// Program arguments
    pub args: Vec<LigeroArg>,
}

impl Default for LigeroConfig {
    fn default() -> Self {
        Self {
            program: String::new(),
            shader_path: String::new(),
            packing: 8192,
            private_indices: Vec::new(),
            args: Vec::new(),
        }
    }
}

impl LigeroConfig {
    /// Create a new config with the given program path.
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            ..Default::default()
        }
    }

    /// Set the shader path.
    pub fn with_shader_path(mut self, path: impl Into<String>) -> Self {
        self.shader_path = path.into();
        self
    }

    /// Set the packing size.
    pub fn with_packing(mut self, packing: u32) -> Self {
        self.packing = packing;
        self
    }

    /// Set the private argument indices (1-based).
    pub fn with_private_indices(mut self, indices: Vec<usize>) -> Self {
        self.private_indices = indices;
        self
    }

    /// Add a string argument.
    pub fn add_str_arg(&mut self, value: impl Into<String>) {
        self.args.push(LigeroArg::string(value));
    }

    /// Add an i64 argument.
    pub fn add_i64_arg(&mut self, value: i64) {
        self.args.push(LigeroArg::i64(value));
    }

    /// Add a u64 argument (stored as i64).
    ///
    /// # Panics
    ///
    /// Panics if value > i64::MAX
    pub fn add_u64_arg(&mut self, value: u64) {
        self.args.push(LigeroArg::u64(value));
    }

    /// Add a hex argument.
    ///
    /// Strips `0x` or `0X` prefix if present.
    pub fn add_hex_arg(&mut self, value: impl Into<String>) {
        self.args.push(LigeroArg::hex(value));
    }
}
