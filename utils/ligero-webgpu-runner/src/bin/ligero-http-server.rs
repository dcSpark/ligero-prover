//! HTTP server for Ligero proving/verifying service using daemon mode.
//!
//! This server maintains long-lived `webgpu_prover --daemon` and `webgpu_verifier --daemon`
//! processes for high-throughput proving and verification. This avoids the overhead of
//! spawning a new process for each request.
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
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tiny_http::{Header, Method, Request, Response, Server};

/// Global atomic counter to ensure unique proof directory IDs even under high concurrency.
static PROOF_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

use ligero_runner::{
    daemon::DaemonPool,
    LigeroArg, LigeroPaths,
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
    /// Optional packing size (defaults to 8192)
    #[serde(default)]
    packing: Option<u32>,
    /// Whether to gzip the proof (defaults to false)
    #[serde(default)]
    gzip: Option<bool>,
}

/// Redact a JSON argument value while preserving its type.
fn redact_json_arg(arg: &Value) -> Value {
    if let Value::Object(obj) = arg {
        if obj.contains_key("str") || obj.contains_key("string") {
            serde_json::json!({"str": "REDACTED"})
        } else if obj.contains_key("i64") {
            serde_json::json!({"i64": 0})
        } else if obj.contains_key("hex") {
            serde_json::json!({"hex": "0000000000000000000000000000000000000000000000000000000000000000"})
        } else if obj.contains_key("hex_bytes_b64") || obj.contains_key("hexBytesB64") {
            serde_json::json!({"hex_bytes_b64": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="})
        } else {
            // Unknown type, replace with hex zero
            serde_json::json!({"hex": "0000000000000000000000000000000000000000000000000000000000000000"})
        }
    } else {
        // Fallback for unexpected structure
        serde_json::json!({"hex": "0000000000000000000000000000000000000000000000000000000000000000"})
    }
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

/// Shared state for the HTTP server with daemon pools.
struct ServerState {
    paths: LigeroPaths,
    prover_pool: DaemonPool,
    verifier_pool: DaemonPool,
    proof_outputs_base: PathBuf,
    keep_proof_dir: bool,
}

impl ServerState {
    fn new(
        paths: LigeroPaths,
        prover_pool: DaemonPool,
        verifier_pool: DaemonPool,
        proof_outputs_base: Option<PathBuf>,
        keep_proof_dir: bool,
    ) -> Self {
        let proof_outputs_base = proof_outputs_base.unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("proof_outputs")
        });
        Self {
            paths,
            prover_pool,
            verifier_pool,
            proof_outputs_base,
            keep_proof_dir,
        }
    }

    /// Resolve circuit name to a program path.
    fn resolve_circuit(&self, circuit: &str) -> Result<String> {
        // If the input looks like a path (contains / or \) or ends with .wasm,
        // try to resolve it directly first without modifications.
        if circuit.contains('/') || circuit.contains('\\') || circuit.ends_with(".wasm") {
            if let Ok(path) = ligero_runner::resolve_program(circuit) {
                return Ok(path.to_string_lossy().into_owned());
            }
        }

        // Try common naming patterns for circuit names
        let candidates = [
            circuit.to_string(),
            format!("{}_guest", circuit),
            format!("{}_guest.wasm", circuit),
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

    /// Build a Ligero config JSON Value for the daemon.
    fn build_config(&self, req: &ProveVerifyRequest, program: &str, use_gzip: bool) -> Value {
        let mut config = serde_json::json!({
            "program": program,
            "shader-path": self.paths.shader_dir.to_string_lossy(),
            "packing": req.packing.unwrap_or(8192),
            "gzip-proof": use_gzip,
            "args": req.args,
        });

        if !req.private_indices.is_empty() {
            config["private-indices"] = serde_json::json!(req.private_indices);
        }

        config
    }

    /// Run the prover for the given request using daemon pool.
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

        // Generate a unique proof output path for this request.
        let counter = PROOF_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let proof_dir_name = format!("ligero_proof_http_{}_{}", timestamp, counter);
        let proof_dir = self.proof_outputs_base.join(&proof_dir_name);

        // Create the proof output directory
        if let Err(e) = std::fs::create_dir_all(&proof_dir) {
            return ProveVerifyResponse {
                success: false,
                exit_code: 1,
                proof: None,
                error: Some(format!("Failed to create proof directory: {}", e)),
            };
        }

        // Determine if we should use gzip (defaults to false)
        let use_gzip = req.gzip.unwrap_or(false);

        // Build config with ABSOLUTE proof output path (daemon runs in bins_dir)
        let mut config = self.build_config(req, &program, use_gzip);
        // Canonicalize to get absolute path for the proof file
        let proof_filename = if use_gzip { "proof_data.gz" } else { "proof_data.bin" };
        let proof_path_abs = proof_dir
            .canonicalize()
            .unwrap_or_else(|_| proof_dir.clone())
            .join(proof_filename);
        config["proof-path"] = Value::String(proof_path_abs.to_string_lossy().to_string());

        // Send to prover daemon
        match self.prover_pool.prove(config) {
            Ok(resp) => {
                if resp.ok {
                    // Read the proof file - prefer the path from daemon response, fallback to our absolute path
                    let proof_file = resp
                        .proof_path
                        .as_ref()
                        .map(PathBuf::from)
                        .unwrap_or(proof_path_abs);

                    match std::fs::read(&proof_file) {
                        Ok(proof_bytes) => {
                            // Clean up proof directory unless keeping it
                            if !self.keep_proof_dir {
                                let _ = std::fs::remove_dir_all(&proof_dir);
                            }

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
                            error: Some(format!(
                                "Prover daemon succeeded but failed to read proof: {}\nproof dir: {}",
                                e,
                                proof_dir.display()
                            )),
                        },
                    }
                } else {
                    ProveVerifyResponse {
                        success: false,
                        exit_code: resp.exit_code.unwrap_or(1) as i32,
                        proof: None,
                        error: Some(format!(
                            "Prover daemon failed: {}\nproof dir: {}",
                            resp.error.unwrap_or_else(|| "unknown error".to_string()),
                            proof_dir.display()
                        )),
                    }
                }
            }
            Err(e) => ProveVerifyResponse {
                success: false,
                exit_code: 1,
                proof: None,
                error: Some(format!("Prover daemon request failed: {}", e)),
            },
        }
    }

    /// Run the verifier for the given request using daemon pool.
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

        // Write proof to a temp file for the verifier daemon
        let temp_dir = match tempfile::tempdir() {
            Ok(d) => d,
            Err(e) => {
                return ProveVerifyResponse {
                    success: false,
                    exit_code: 1,
                    proof: None,
                    error: Some(format!("Failed to create temp directory: {}", e)),
                }
            }
        };

        // Detect if proof is gzip-compressed
        let is_gzip = proof_bytes.len() >= 2 && proof_bytes[0] == 0x1f && proof_bytes[1] == 0x8b;
        let proof_filename = if is_gzip {
            "proof_data.gz"
        } else {
            "proof_data.bin"
        };
        let proof_path = temp_dir.path().join(proof_filename);

        if let Err(e) = std::fs::write(&proof_path, &proof_bytes) {
            return ProveVerifyResponse {
                success: false,
                exit_code: 1,
                proof: None,
                error: Some(format!("Failed to write proof file: {}", e)),
            };
        }

        // Build config for verifier (with redacted private args)
        // Use detected gzip status from the proof bytes
        let mut config = self.build_config(req, &program, is_gzip);

        // Redact private arguments for verification, preserving argument types
        if let Value::Array(ref mut args) = config["args"] {
            for &idx in &req.private_indices {
                if idx > 0 && idx <= args.len() {
                    let arg_idx = idx - 1;
                    // Preserve the type when redacting
                    args[arg_idx] = redact_json_arg(&args[arg_idx]);
                }
            }
        }

        // Send to verifier daemon
        match self.verifier_pool.verify(config, &proof_path.to_string_lossy()) {
            Ok(resp) => {
                let success = resp.ok && resp.verify_ok == Some(true);
                ProveVerifyResponse {
                    success,
                    exit_code: if success { 0 } else { resp.exit_code.unwrap_or(1) as i32 },
                    proof: None,
                    error: if success {
                        None
                    } else {
                        Some(resp.error.unwrap_or_else(|| "Verification failed".to_string()))
                    },
                }
            }
            Err(e) => ProveVerifyResponse {
                success: false,
                exit_code: 1,
                proof: None,
                error: Some(format!("Verifier daemon request failed: {}", e)),
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
    let mut prover_workers: Option<usize> = None;
    let mut verifier_workers: Option<usize> = None;

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
            "--prover-workers" | "-p" => {
                i += 1;
                prover_workers = Some(parse_usize("--prover-workers", args.get(i))?);
            }
            "--verifier-workers" | "-v" => {
                i += 1;
                verifier_workers = Some(parse_usize("--verifier-workers", args.get(i))?);
            }
            "--help" | "-h" => {
                eprintln!(
                    r#"Ligero HTTP Proving/Verifying Service (Daemon Mode)

This server uses long-lived daemon processes for high-throughput proving
and verification, avoiding process spawn overhead per request.

Usage:
  ligero-http-server [OPTIONS]

Options:
  -b, --bind <ADDR>            Bind address (default: 127.0.0.1:1313)
  --proof-outputs <PATH>       Base directory for proof outputs
  --keep-proof-dir             Keep proof directories after completion
  -t, --threads <N>            Number of HTTP worker threads (default: CPU count)
  -p, --prover-workers <N>     Number of prover daemon workers (default: CPU count)
  -v, --verifier-workers <N>   Number of verifier daemon workers (default: CPU count)
  -h, --help                   Show this help message

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
    tracing::info!("  Bins directory: {}", paths.bins_dir.display());

    // Calculate default worker counts
    let default_workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let prover_workers = prover_workers.unwrap_or(default_workers);
    let verifier_workers = verifier_workers.unwrap_or(default_workers);

    // Create daemon pools
    tracing::info!("Starting {} prover daemon workers...", prover_workers);
    let prover_pool = DaemonPool::new_prover(&paths, prover_workers)
        .context("Failed to start prover daemon pool")?;
    tracing::info!("✓ Prover daemon pool started");

    tracing::info!("Starting {} verifier daemon workers...", verifier_workers);
    let verifier_pool = DaemonPool::new_verifier(&paths, verifier_workers)
        .context("Failed to start verifier daemon pool")?;
    tracing::info!("✓ Verifier daemon pool started");

    let state = Arc::new(ServerState::new(
        paths,
        prover_pool,
        verifier_pool,
        proof_outputs,
        keep_proof_dir,
    ));

    // Create the HTTP server
    let server = Server::http(bind_addr)
        .map_err(|e| anyhow::anyhow!("Failed to bind HTTP server: {}", e))?;

    let num_threads = num_threads.unwrap_or(default_workers);

    tracing::info!(
        "Starting Ligero HTTP server on {} with {} HTTP threads, {} prover daemons, {} verifier daemons",
        bind_addr,
        num_threads,
        prover_workers,
        verifier_workers
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
