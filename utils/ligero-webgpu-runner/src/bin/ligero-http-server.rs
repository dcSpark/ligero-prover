//! HTTP server for Ligero proving/verifying service.
//!
//! Exposes two synchronous POST endpoints:
//! - `/prove` - Generate a proof for a given circuit and arguments
//! - `/verify` - Verify an existing proof
//!
//! Request format:
//! ```json
//! {
//!   "circuit": "note_spend",
//!   "args": [ /* LigeroArg array */ ],
//!   "proof": null  // for proving, base64 string for verification
//! }
//! ```
//!
//! Response format:
//! ```json
//! {
//!   "success": true,
//!   "exitCode": 0,
//!   "proof": "base64..."  // null on verification
//! }
//! ```

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use tiny_http::{Header, Method, Request, Response, Server};

use ligero_runner::{
    verifier::{verify_proof_with_output, VerifierPaths},
    LigeroArg, LigeroPaths, LigeroRunner, ProverRunOptions,
};

/// Request body for `/prove` and `/verify` endpoints.
#[derive(Debug, Clone, Deserialize)]
struct ProveVerifyRequest {
    /// Circuit name (e.g., "note_spend", "note_deposit", "value_validator")
    circuit: String,
    /// Arguments to pass to the prover/verifier
    args: Vec<LigeroArg>,
    /// Base64-encoded proof bytes (null/absent for proving, required for verification)
    #[serde(default)]
    proof: Option<String>,
    /// Optional private argument indices (1-based)
    #[serde(default, rename = "privateIndices")]
    private_indices: Vec<usize>,
}

/// Response body for `/prove` and `/verify` endpoints.
#[derive(Debug, Clone, Serialize)]
struct ProveVerifyResponse {
    /// Whether the operation succeeded
    success: bool,
    /// Exit code (0 on success)
    #[serde(rename = "exitCode")]
    exit_code: i32,
    /// Base64-encoded proof bytes (only for `/prove`, null for `/verify`)
    #[serde(skip_serializing_if = "Option::is_none")]
    proof: Option<String>,
    /// Error message if the operation failed
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Shared state for the HTTP server.
struct ServerState {
    paths: LigeroPaths,
    proof_outputs_base: PathBuf,
    keep_proof_dir: bool,
}

impl ServerState {
    fn new(paths: LigeroPaths, proof_outputs_base: Option<PathBuf>, keep_proof_dir: bool) -> Self {
        let proof_outputs_base = proof_outputs_base.unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("proof_outputs")
        });
        Self {
            paths,
            proof_outputs_base,
            keep_proof_dir,
        }
    }

    /// Resolve circuit name to a program path.
    fn resolve_circuit(&self, circuit: &str) -> Result<String> {
        // Try common naming patterns
        let candidates = [
            format!("{}_guest", circuit),
            format!("{}_guest.wasm", circuit),
            circuit.to_string(),
            format!("{}.wasm", circuit),
        ];

        for candidate in &candidates {
            if let Ok(path) = ligero_runner::resolve_program(candidate) {
                return Ok(path.to_string_lossy().into_owned());
            }
        }

        anyhow::bail!(
            "Could not resolve circuit '{}'. Tried: {:?}",
            circuit,
            candidates
        )
    }

    /// Run the prover for the given request.
    fn prove(&self, req: &ProveVerifyRequest) -> ProveVerifyResponse {
        let program = match self.resolve_circuit(&req.circuit) {
            Ok(p) => p,
            Err(e) => {
                return ProveVerifyResponse {
                    success: false,
                    exit_code: 1,
                    proof: None,
                    error: Some(format!("Failed to resolve circuit: {}", e)),
                }
            }
        };

        let mut runner = LigeroRunner::new_with_paths(&program, self.paths.clone());

        // Set private indices if provided
        if !req.private_indices.is_empty() {
            runner = runner.with_private_indices(req.private_indices.clone());
        }

        // Add arguments
        for arg in &req.args {
            runner.config_mut().args.push(arg.clone());
        }

        // Generate a unique proof dir ID
        let proof_id = format!(
            "http_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        runner.set_proof_dir_id(proof_id);

        let options = ProverRunOptions {
            keep_proof_dir: self.keep_proof_dir,
            proof_outputs_base: Some(self.proof_outputs_base.clone()),
            write_replay_script: true,
        };

        match runner.run_prover_with_options(options) {
            Ok(proof_bytes) => {
                let proof_b64 = BASE64.encode(&proof_bytes);
                ProveVerifyResponse {
                    success: true,
                    exit_code: 0,
                    proof: Some(proof_b64),
                    error: None,
                }
            }
            Err(e) => ProveVerifyResponse {
                success: false,
                exit_code: 1,
                proof: None,
                error: Some(format!("Prover failed: {}", e)),
            },
        }
    }

    /// Run the verifier for the given request.
    fn verify(&self, req: &ProveVerifyRequest) -> ProveVerifyResponse {
        let proof_b64 = match &req.proof {
            Some(p) => p,
            None => {
                return ProveVerifyResponse {
                    success: false,
                    exit_code: 1,
                    proof: None,
                    error: Some("Proof is required for verification".to_string()),
                }
            }
        };

        let proof_bytes = match BASE64.decode(proof_b64) {
            Ok(b) => b,
            Err(e) => {
                return ProveVerifyResponse {
                    success: false,
                    exit_code: 1,
                    proof: None,
                    error: Some(format!("Failed to decode proof: {}", e)),
                }
            }
        };

        let program = match self.resolve_circuit(&req.circuit) {
            Ok(p) => p,
            Err(e) => {
                return ProveVerifyResponse {
                    success: false,
                    exit_code: 1,
                    proof: None,
                    error: Some(format!("Failed to resolve circuit: {}", e)),
                }
            }
        };

        let verifier_paths = VerifierPaths::from_explicit(
            PathBuf::from(&program),
            self.paths.shader_dir.clone(),
            self.paths.verifier_bin.clone(),
            8192, // default packing
        );

        match verify_proof_with_output(
            &verifier_paths,
            &proof_bytes,
            req.args.clone(),
            req.private_indices.clone(),
        ) {
            Ok((success, _stdout, _stderr)) => ProveVerifyResponse {
                success,
                exit_code: if success { 0 } else { 1 },
                proof: None,
                error: if success {
                    None
                } else {
                    Some("Verification failed".to_string())
                },
            },
            Err(e) => ProveVerifyResponse {
                success: false,
                exit_code: 1,
                proof: None,
                error: Some(format!("Verifier failed: {}", e)),
            },
        }
    }
}

fn json_response<T: Serialize>(status_code: u16, body: &T) -> Response<std::io::Cursor<Vec<u8>>> {
    let json_bytes = serde_json::to_vec(body).unwrap_or_else(|_| b"{}".to_vec());
    let data_length = json_bytes.len();
    let content_type = Header::from_bytes("Content-Type", "application/json").unwrap();
    // Use Response::new directly with explicit data_length to avoid chunked encoding
    tiny_http::Response::new(
        tiny_http::StatusCode(status_code),
        vec![content_type],
        std::io::Cursor::new(json_bytes),
        Some(data_length),
        None,
    )
}

fn handle_request(state: &ServerState, mut request: Request) {
    let path = request.url().to_string();
    let method = request.method().clone();

    // Handle health check separately (allows GET and no body)
    if path == "/health" {
        let resp = ProveVerifyResponse {
            success: true,
            exit_code: 0,
            proof: None,
            error: None,
        };
        let _ = request.respond(json_response(200, &resp));
        return;
    }

    // Only accept POST requests for other endpoints
    if method != Method::Post {
        let resp = ProveVerifyResponse {
            success: false,
            exit_code: 1,
            proof: None,
            error: Some("Only POST method is allowed".to_string()),
        };
        let _ = request.respond(json_response(405, &resp));
        return;
    }

    // Read request body
    let mut body = String::new();
    if let Err(e) = request.as_reader().read_to_string(&mut body) {
        let resp = ProveVerifyResponse {
            success: false,
            exit_code: 1,
            proof: None,
            error: Some(format!("Failed to read request body: {}", e)),
        };
        let _ = request.respond(json_response(400, &resp));
        return;
    }

    // Parse request body
    let req: ProveVerifyRequest = match serde_json::from_str(&body) {
        Ok(r) => r,
        Err(e) => {
            let resp = ProveVerifyResponse {
                success: false,
                exit_code: 1,
                proof: None,
                error: Some(format!("Failed to parse request body: {}", e)),
            };
            let _ = request.respond(json_response(400, &resp));
            return;
        }
    };

    // Route to appropriate handler
    let resp = match path.as_str() {
        "/prove" => {
            tracing::info!(circuit = %req.circuit, "Processing prove request");
            state.prove(&req)
        }
        "/verify" => {
            tracing::info!(circuit = %req.circuit, "Processing verify request");
            state.verify(&req)
        }
        _ => ProveVerifyResponse {
            success: false,
            exit_code: 1,
            proof: None,
            error: Some(format!("Unknown endpoint: {}", path)),
        },
    };

    let status = if resp.success { 200 } else { 400 };
    let _ = request.respond(json_response(status, &resp));
}

fn parse_usize(flag: &str, v: Option<&String>) -> Result<usize> {
    let s = v.with_context(|| format!("missing value for {flag}"))?;
    s.parse::<usize>()
        .with_context(|| format!("invalid {flag}={s}"))
}

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Minimal argv parsing
    let args: Vec<String> = std::env::args().collect();

    let mut bind_addr: SocketAddr = "127.0.0.1:1313".parse().unwrap();
    let mut proof_outputs: Option<PathBuf> = None;
    let mut keep_proof_dir = false;
    let mut num_threads: Option<usize> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--bind" | "-b" => {
                i += 1;
                let s = args.get(i).context("missing value for --bind")?;
                bind_addr = s.parse::<SocketAddr>().context("invalid --bind address")?;
            }
            "--proof-outputs" => {
                i += 1;
                let s = args.get(i).context("missing value for --proof-outputs")?;
                proof_outputs = Some(PathBuf::from(s));
            }
            "--keep-proof-dir" => {
                keep_proof_dir = true;
            }
            "--threads" | "-t" => {
                i += 1;
                num_threads = Some(parse_usize("--threads", args.get(i))?);
            }
            "--help" | "-h" => {
                eprintln!(
                    r#"Ligero HTTP Proving/Verifying Service

Usage:
  ligero-http-server [OPTIONS]

Options:
  -b, --bind <ADDR>          Bind address (default: 127.0.0.1:1313)
  --proof-outputs <PATH>     Base directory for proof outputs
  --keep-proof-dir           Keep proof directories after completion
  -t, --threads <N>          Number of worker threads (default: CPU count)
  -h, --help                 Show this help message

Endpoints:
  POST /prove   Generate a proof
  POST /verify  Verify an existing proof
  GET  /health  Health check

Request format:
  {{
    "circuit": "note_spend",
    "args": [ {{ "str": "value" }}, {{ "i64": 123 }}, {{ "hex": "deadbeef" }} ],
    "proof": null,         // null for proving, base64 string for verification
    "privateIndices": [1]  // optional, 1-based indices of private args
  }}

Response format:
  {{
    "success": true,
    "exitCode": 0,
    "proof": "base64...",  // null on verification
    "error": null          // error message if failed
  }}

Environment variables:
  LIGERO_ROOT             Path to ligero-prover repository
  LIGERO_PROGRAMS_DIR     Directory containing .wasm circuit files
  LIGERO_SHADER_PATH      Path to shader directory
  LIGERO_PROVER_BIN       Path to webgpu_prover binary
  LIGERO_VERIFIER_BIN     Path to webgpu_verifier binary
"#
                );
                return Ok(());
            }
            other => anyhow::bail!("unknown argument: {other}"),
        }
        i += 1;
    }

    // Discover Ligero paths
    let paths = LigeroPaths::discover().unwrap_or_else(|e| {
        tracing::warn!("Failed to discover Ligero paths: {}, using fallback", e);
        LigeroPaths::fallback()
    });

    tracing::info!("Ligero paths:");
    tracing::info!("  Prover binary: {}", paths.prover_bin.display());
    tracing::info!("  Verifier binary: {}", paths.verifier_bin.display());
    tracing::info!("  Shader directory: {}", paths.shader_dir.display());

    let state = Arc::new(ServerState::new(paths, proof_outputs, keep_proof_dir));

    // Create the HTTP server
    let server = Server::http(bind_addr)
        .map_err(|e| anyhow::anyhow!("Failed to bind HTTP server: {}", e))?;

    let num_threads = num_threads.unwrap_or_else(|| {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    });

    tracing::info!(
        "Starting Ligero HTTP server on {} with {} worker threads",
        bind_addr,
        num_threads
    );

    // Spawn worker threads
    let server = Arc::new(server);
    let mut handles = Vec::with_capacity(num_threads);

    for _ in 0..num_threads {
        let server = Arc::clone(&server);
        let state = Arc::clone(&state);
        handles.push(std::thread::spawn(move || {
            loop {
                match server.recv() {
                    Ok(request) => {
                        handle_request(&state, request);
                    }
                    Err(e) => {
                        tracing::error!("Error receiving request: {}", e);
                        break;
                    }
                }
            }
        }));
    }

    // Wait for all threads
    for handle in handles {
        let _ = handle.join();
    }

    Ok(())
}
