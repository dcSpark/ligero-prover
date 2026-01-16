//! Integration tests for the Ligero HTTP proving/verifying server.
//!
//! These tests require:
//! - `webgpu_prover` + `webgpu_verifier` binaries (or skipped if not available)
//! - a valid `shader/` directory
//! - a built `note_spend_guest.wasm` (or value_validator_rust.wasm for simpler tests)
//!
//! Run with:
//!   cargo test --manifest-path utils/ligero-webgpu-runner/Cargo.toml http_server_test -- --nocapture

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use base64::Engine;
use serde::Deserialize;
use serde_json::json;

/// Port counter to avoid conflicts between tests running in parallel.
static PORT_COUNTER: AtomicU16 = AtomicU16::new(19800);

fn next_port() -> u16 {
    PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
}

fn repo_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    Ok(manifest_dir
        .ancestors()
        .nth(2)
        .context("Failed to compute ligero-prover repo root")?
        .to_path_buf())
}

/// Decode HTTP chunked transfer encoding.
fn decode_chunked(data: &[u8]) -> Result<String> {
    let mut result = Vec::new();
    let mut pos = 0;

    while pos < data.len() {
        // Find the end of the chunk size line
        let chunk_size_end = data[pos..]
            .windows(2)
            .position(|w| w == b"\r\n")
            .map(|p| pos + p)
            .unwrap_or(data.len());

        if chunk_size_end >= data.len() {
            break;
        }

        // Parse chunk size (hex)
        let chunk_size_str = String::from_utf8_lossy(&data[pos..chunk_size_end]);
        // Handle optional chunk extensions (after semicolon)
        let chunk_size_hex = chunk_size_str.split(';').next().unwrap_or("").trim();
        let chunk_size = usize::from_str_radix(chunk_size_hex, 16).unwrap_or(0);

        if chunk_size == 0 {
            // Last chunk
            break;
        }

        // Skip past the chunk size line
        let chunk_start = chunk_size_end + 2;
        let chunk_end = chunk_start + chunk_size;

        if chunk_end > data.len() {
            // Incomplete chunk, use what we have
            result.extend_from_slice(&data[chunk_start..]);
            break;
        }

        result.extend_from_slice(&data[chunk_start..chunk_end]);

        // Move past the chunk data and trailing CRLF
        pos = chunk_end + 2;
    }

    Ok(String::from_utf8_lossy(&result).to_string())
}

fn http_server_binary() -> Result<PathBuf> {
    // Check target/debug first, then target/release.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let debug_bin = manifest_dir.join("target/debug/ligero-http-server");
    if debug_bin.exists() {
        return Ok(debug_bin);
    }
    let release_bin = manifest_dir.join("target/release/ligero-http-server");
    if release_bin.exists() {
        return Ok(release_bin);
    }

    // Try workspace target directory.
    let workspace_debug = manifest_dir
        .ancestors()
        .nth(2)
        .map(|p| p.join("target/debug/ligero-http-server"));
    if let Some(ref p) = workspace_debug {
        if p.exists() {
            return Ok(p.clone());
        }
    }

    anyhow::bail!(
        "ligero-http-server binary not found. Build with: cargo build --bin ligero-http-server"
    )
}

fn check_binaries_exist() -> bool {
    let repo = match repo_root() {
        Ok(r) => r,
        Err(_) => return false,
    };

    // Check for portable binaries or build directory.
    let prover_paths = [
        repo.join("utils/portable-binaries/macos-arm64/bin/webgpu_prover"),
        repo.join("utils/portable-binaries/linux-amd64/bin/webgpu_prover"),
        repo.join("utils/portable-binaries/linux-arm64/bin/webgpu_prover"),
        repo.join("build/webgpu_prover"),
    ];

    prover_paths.iter().any(|p| p.exists())
}

/// HTTP response for prove/verify endpoints.
#[derive(Debug, Clone, Deserialize)]
struct ProveVerifyResponse {
    success: bool,
    #[serde(rename = "exitCode")]
    exit_code: i32,
    proof: Option<String>,
    error: Option<String>,
}

/// Helper to start the HTTP server and return a handle.
struct HttpServerHandle {
    child: Child,
    addr: SocketAddr,
}

impl HttpServerHandle {
    fn start(port: u16) -> Result<Self> {
        let binary = http_server_binary()?;
        let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;

        let child = Command::new(&binary)
            .args(["-b", &addr.to_string(), "-t", "2"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to start HTTP server at {}", binary.display()))?;

        // Wait for server to be ready.
        let mut retries = 20;
        while retries > 0 {
            thread::sleep(Duration::from_millis(100));
            if TcpStream::connect_timeout(&addr, Duration::from_millis(100)).is_ok() {
                return Ok(Self { child, addr });
            }
            retries -= 1;
        }

        anyhow::bail!("HTTP server did not start within timeout");
    }

    #[allow(dead_code)]
    fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.addr, path)
    }

    /// Send an HTTP request and return the response body.
    fn request(&self, method: &str, path: &str, body: Option<&str>) -> Result<(u16, String)> {
        let mut stream = TcpStream::connect_timeout(&self.addr, Duration::from_secs(5))
            .context("Failed to connect to HTTP server")?;

        stream
            .set_read_timeout(Some(Duration::from_secs(300)))
            .ok();
        stream
            .set_write_timeout(Some(Duration::from_secs(30)))
            .ok();

        let body_bytes = body.unwrap_or("").as_bytes();
        let request = format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            method,
            path,
            self.addr,
            body_bytes.len()
        );

        stream.write_all(request.as_bytes())?;
        stream.write_all(body_bytes)?;
        stream.flush()?;

        let mut response = Vec::new();
        stream.read_to_end(&mut response)?;

        // Split headers and body
        let response_str = String::from_utf8_lossy(&response);
        let header_end = response_str.find("\r\n\r\n").unwrap_or(response_str.len());
        let headers = &response_str[..header_end];
        let body_start = header_end + 4;

        // Extract status code.
        let status_line = headers.lines().next().unwrap_or("");
        let status_code: u16 = status_line
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        // Check if chunked transfer encoding
        let is_chunked = headers
            .to_lowercase()
            .contains("transfer-encoding: chunked");

        let body = if is_chunked && body_start < response.len() {
            // Decode chunked transfer encoding
            decode_chunked(&response[body_start..])?
        } else if body_start < response.len() {
            String::from_utf8_lossy(&response[body_start..]).to_string()
        } else {
            String::new()
        };

        Ok((status_code, body))
    }

    fn post_json<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<(u16, T)> {
        let body_str = serde_json::to_string(body)?;
        let (status, response_body) = self.request("POST", path, Some(&body_str))?;
        let parsed: T = serde_json::from_str(&response_body)
            .with_context(|| format!("Failed to parse response: {}", response_body))?;
        Ok((status, parsed))
    }
}

impl Drop for HttpServerHandle {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// ============================================================================
// Test Cases
// ============================================================================

#[test]
fn test_http_server_health_endpoint() -> Result<()> {
    if http_server_binary().is_err() {
        eprintln!("Skipping: HTTP server binary not found");
        return Ok(());
    }

    let port = next_port();
    let server = HttpServerHandle::start(port)?;

    let (status, body) = server.request("GET", "/health", None)?;

    assert_eq!(status, 200, "health endpoint should return 200");

    let resp: ProveVerifyResponse = serde_json::from_str(&body)?;
    assert!(resp.success, "health check should succeed");
    assert_eq!(resp.exit_code, 0, "health check exit code should be 0");

    println!("[test_health] OK: health endpoint works");
    Ok(())
}

#[test]
fn test_http_server_unknown_endpoint() -> Result<()> {
    if http_server_binary().is_err() {
        eprintln!("Skipping: HTTP server binary not found");
        return Ok(());
    }

    let port = next_port();
    let server = HttpServerHandle::start(port)?;

    let body = json!({
        "circuit": "test",
        "args": []
    });

    let (status, resp): (u16, ProveVerifyResponse) =
        server.post_json("/unknown_endpoint", &body)?;

    assert_eq!(status, 400, "unknown endpoint should return 400");
    assert!(!resp.success, "unknown endpoint should fail");
    assert!(
        resp.error
            .as_ref()
            .map(|e| e.contains("Unknown endpoint"))
            .unwrap_or(false),
        "error should mention unknown endpoint"
    );

    println!("[test_unknown_endpoint] OK: unknown endpoint handled correctly");
    Ok(())
}

#[test]
fn test_http_server_invalid_json() -> Result<()> {
    if http_server_binary().is_err() {
        eprintln!("Skipping: HTTP server binary not found");
        return Ok(());
    }

    let port = next_port();
    let server = HttpServerHandle::start(port)?;

    let (status, body) = server.request("POST", "/prove", Some("not valid json"))?;

    assert_eq!(status, 400, "invalid JSON should return 400");

    let resp: ProveVerifyResponse = serde_json::from_str(&body)?;
    assert!(!resp.success, "invalid JSON should fail");
    assert!(
        resp.error
            .as_ref()
            .map(|e| e.contains("parse"))
            .unwrap_or(false),
        "error should mention parse failure"
    );

    println!("[test_invalid_json] OK: invalid JSON handled correctly");
    Ok(())
}

#[test]
fn test_http_server_missing_circuit() -> Result<()> {
    if http_server_binary().is_err() {
        eprintln!("Skipping: HTTP server binary not found");
        return Ok(());
    }

    let port = next_port();
    let server = HttpServerHandle::start(port)?;

    // Missing required 'circuit' field.
    let body = json!({
        "args": []
    });

    let (status, resp): (u16, ProveVerifyResponse) = server.post_json("/prove", &body)?;

    assert_eq!(status, 400, "missing circuit should return 400");
    assert!(!resp.success, "missing circuit should fail");
    assert!(
        resp.error
            .as_ref()
            .map(|e| e.contains("circuit") || e.contains("missing field"))
            .unwrap_or(false),
        "error should mention missing circuit: {:?}",
        resp.error
    );

    println!("[test_missing_circuit] OK: missing circuit handled correctly");
    Ok(())
}

#[test]
fn test_http_server_unknown_circuit() -> Result<()> {
    if http_server_binary().is_err() {
        eprintln!("Skipping: HTTP server binary not found");
        return Ok(());
    }

    let port = next_port();
    let server = HttpServerHandle::start(port)?;

    let body = json!({
        "circuit": "nonexistent_circuit_xyz",
        "args": []
    });

    let (status, resp): (u16, ProveVerifyResponse) = server.post_json("/prove", &body)?;

    assert_eq!(status, 400, "unknown circuit should return 400");
    assert!(!resp.success, "unknown circuit should fail");
    assert!(
        resp.error
            .as_ref()
            .map(|e| e.contains("resolve circuit") || e.contains("Could not resolve"))
            .unwrap_or(false),
        "error should mention circuit resolution: {:?}",
        resp.error
    );

    println!("[test_unknown_circuit] OK: unknown circuit handled correctly");
    Ok(())
}

#[test]
fn test_http_server_verify_missing_proof() -> Result<()> {
    if http_server_binary().is_err() {
        eprintln!("Skipping: HTTP server binary not found");
        return Ok(());
    }

    let port = next_port();
    let server = HttpServerHandle::start(port)?;

    let body = json!({
        "circuit": "note_spend",
        "args": []
        // proof is missing
    });

    let (status, resp): (u16, ProveVerifyResponse) = server.post_json("/verify", &body)?;

    assert_eq!(status, 400, "verify without proof should return 400");
    assert!(!resp.success, "verify without proof should fail");
    assert!(
        resp.error
            .as_ref()
            .map(|e| e.contains("Proof is required"))
            .unwrap_or(false),
        "error should mention missing proof: {:?}",
        resp.error
    );

    println!("[test_verify_missing_proof] OK: verify without proof handled correctly");
    Ok(())
}

#[test]
fn test_http_server_verify_invalid_base64_proof() -> Result<()> {
    if http_server_binary().is_err() {
        eprintln!("Skipping: HTTP server binary not found");
        return Ok(());
    }

    let port = next_port();
    let server = HttpServerHandle::start(port)?;

    let body = json!({
        "circuit": "note_spend",
        "args": [],
        "proof": "not_valid_base64!!!"
    });

    let (status, resp): (u16, ProveVerifyResponse) = server.post_json("/verify", &body)?;

    assert_eq!(status, 400, "invalid base64 proof should return 400");
    assert!(!resp.success, "invalid base64 proof should fail");
    assert!(
        resp.error
            .as_ref()
            .map(|e| e.contains("decode proof") || e.contains("base64"))
            .unwrap_or(false),
        "error should mention decoding: {:?}",
        resp.error
    );

    println!("[test_verify_invalid_base64] OK: invalid base64 proof handled correctly");
    Ok(())
}

#[test]
fn test_http_server_method_not_allowed() -> Result<()> {
    if http_server_binary().is_err() {
        eprintln!("Skipping: HTTP server binary not found");
        return Ok(());
    }

    let port = next_port();
    let server = HttpServerHandle::start(port)?;

    // GET request to /prove (should only accept POST)
    let (status, body) = server.request("GET", "/prove", None)?;

    assert_eq!(status, 405, "GET on /prove should return 405");

    let resp: ProveVerifyResponse = serde_json::from_str(&body)?;
    assert!(!resp.success, "GET on /prove should fail");
    assert!(
        resp.error
            .as_ref()
            .map(|e| e.contains("POST"))
            .unwrap_or(false),
        "error should mention POST requirement: {:?}",
        resp.error
    );

    println!("[test_method_not_allowed] OK: method not allowed handled correctly");
    Ok(())
}

// ============================================================================
// Proving/Verification Tests (require GPU environment)
// ============================================================================

/// Helper to build note_spend_guest args for a simple transfer.
mod note_spend_helpers {
    use super::*;
    use ligetron::poseidon2_hash_bytes;
    use ligetron::Bn254Fr;

    pub type Hash32 = [u8; 32];

    const BL_DEPTH: usize = 16;
    const BL_BUCKET_SIZE: usize = 12;

    fn hx32(b: &Hash32) -> String {
        hex::encode(b)
    }

    fn poseidon2_hash_domain(tag: &[u8], parts: &[&[u8]]) -> Hash32 {
        let mut buf_len = tag.len();
        for part in parts {
            buf_len += part.len();
        }
        let mut tmp = Vec::with_capacity(buf_len);
        tmp.extend_from_slice(tag);
        for part in parts {
            tmp.extend_from_slice(part);
        }
        poseidon2_hash_bytes(&tmp).to_bytes_be()
    }

    fn mt_combine(level: u8, left: &Hash32, right: &Hash32) -> Hash32 {
        let lvl = [level];
        poseidon2_hash_domain(b"MT_NODE_V1", &[&lvl, left, right])
    }

    fn merkle_default_nodes_from_leaf(depth: usize, leaf0: &Hash32) -> Vec<Hash32> {
        let mut out = Vec::with_capacity(depth + 1);
        out.push(*leaf0);
        for lvl in 0..depth {
            let prev = out[lvl];
            out.push(mt_combine(lvl as u8, &prev, &prev));
        }
        out
    }

    fn bl_empty_bucket_entries() -> [Hash32; BL_BUCKET_SIZE] {
        [[0u8; 32]; BL_BUCKET_SIZE]
    }

    fn bl_bucket_leaf(entries: &[Hash32; BL_BUCKET_SIZE]) -> Hash32 {
        let mut buf = Vec::with_capacity(12 + 32 * BL_BUCKET_SIZE);
        buf.extend_from_slice(b"BL_BUCKET_V1");
        for e in entries {
            buf.extend_from_slice(e);
        }
        poseidon2_hash_bytes(&buf).to_bytes_be()
    }

    fn fr_from_hash32_be(h: &Hash32) -> Bn254Fr {
        let mut fr = Bn254Fr::new();
        fr.set_bytes_big(h);
        fr
    }

    fn bl_bucket_inv_for_id(
        id: &Hash32,
        bucket_entries: &[Hash32; BL_BUCKET_SIZE],
    ) -> Option<Hash32> {
        let id_fr = fr_from_hash32_be(id);
        let mut prod = Bn254Fr::from_u32(1);
        for e in bucket_entries.iter() {
            let e_fr = fr_from_hash32_be(e);
            let mut delta = id_fr.clone();
            delta.submod_checked(&e_fr);
            prod.mulmod_checked(&delta);
        }
        if prod.is_zero() {
            return None;
        }
        let mut inv = prod.clone();
        inv.inverse();
        Some(inv.to_bytes_be())
    }

    pub fn note_commitment(
        domain: &Hash32,
        value: u64,
        rho: &Hash32,
        recipient: &Hash32,
        sender_id: &Hash32,
    ) -> Hash32 {
        let mut v16 = [0u8; 16];
        v16[..8].copy_from_slice(&value.to_le_bytes());
        poseidon2_hash_domain(b"NOTE_V2", &[domain, &v16, rho, recipient, sender_id])
    }

    pub fn pk_from_sk(spend_sk: &Hash32) -> Hash32 {
        poseidon2_hash_domain(b"PK_V1", &[spend_sk])
    }

    pub fn ivk_seed(domain: &Hash32, spend_sk: &Hash32) -> Hash32 {
        poseidon2_hash_domain(b"IVK_SEED_V1", &[domain, spend_sk])
    }

    pub fn recipient_from_pk(domain: &Hash32, pk_spend: &Hash32, pk_ivk: &Hash32) -> Hash32 {
        poseidon2_hash_domain(b"ADDR_V2", &[domain, pk_spend, pk_ivk])
    }

    pub fn recipient_from_sk(domain: &Hash32, spend_sk: &Hash32, pk_ivk: &Hash32) -> Hash32 {
        recipient_from_pk(domain, &pk_from_sk(spend_sk), pk_ivk)
    }

    pub fn nf_key_from_sk(domain: &Hash32, spend_sk: &Hash32) -> Hash32 {
        poseidon2_hash_domain(b"NFKEY_V1", &[domain, spend_sk])
    }

    pub fn nullifier(domain: &Hash32, nf_key: &Hash32, rho: &Hash32) -> Hash32 {
        poseidon2_hash_domain(b"PRF_NF_V1", &[domain, nf_key, rho])
    }

    pub fn compute_inv_enforce(
        in_values: &[u64],
        in_rhos: &[Hash32],
        out_values: &[u64],
        out_rhos: &[Hash32],
    ) -> Result<Hash32> {
        let mut prod = Bn254Fr::from_u32(1);
        for &v in in_values {
            prod.mulmod_checked(&Bn254Fr::from_u64(v));
        }
        for &v in out_values {
            prod.mulmod_checked(&Bn254Fr::from_u64(v));
        }
        for out_rho in out_rhos {
            let out_fr = fr_from_hash32_be(out_rho);
            for in_rho in in_rhos {
                let in_fr = fr_from_hash32_be(in_rho);
                let mut delta = out_fr.clone();
                delta.submod_checked(&in_fr);
                prod.mulmod_checked(&delta);
            }
        }
        if out_rhos.len() == 2 {
            let out0 = fr_from_hash32_be(&out_rhos[0]);
            let out1 = fr_from_hash32_be(&out_rhos[1]);
            let mut delta = out0.clone();
            delta.submod_checked(&out1);
            prod.mulmod_checked(&delta);
        }
        anyhow::ensure!(!prod.is_zero(), "inv_enforce undefined");
        let mut inv = prod.clone();
        inv.inverse();
        Ok(inv.to_bytes_be())
    }

    pub struct MerkleTree {
        depth: usize,
        leaves: Vec<Hash32>,
    }

    impl MerkleTree {
        pub fn new(depth: usize) -> Self {
            let size = 1usize << depth;
            Self {
                depth,
                leaves: vec![[0u8; 32]; size],
            }
        }

        pub fn set_leaf(&mut self, pos: usize, leaf: Hash32) {
            self.leaves[pos] = leaf;
        }

        pub fn root(&self) -> Hash32 {
            let mut level = self.leaves.clone();
            for lvl in 0..self.depth {
                let mut next = Vec::with_capacity(level.len() / 2);
                for i in (0..level.len()).step_by(2) {
                    next.push(mt_combine(lvl as u8, &level[i], &level[i + 1]));
                }
                level = next;
            }
            level[0]
        }

        pub fn open(&self, pos: usize) -> Vec<Hash32> {
            let mut siblings = Vec::with_capacity(self.depth);
            let mut idx = pos;
            let mut level = self.leaves.clone();
            for lvl in 0..self.depth {
                let sib_idx = if (idx & 1) == 0 { idx + 1 } else { idx - 1 };
                siblings.push(level[sib_idx]);
                let mut next = Vec::with_capacity(level.len() / 2);
                for i in (0..level.len()).step_by(2) {
                    next.push(mt_combine(lvl as u8, &level[i], &level[i + 1]));
                }
                level = next;
                idx >>= 1;
            }
            siblings
        }
    }

    pub fn private_indices(depth: usize, n_in: usize, n_out: usize, is_transfer: bool) -> Vec<usize> {
        let mut idx = vec![2usize, 3usize];
        let per_in = 5usize + depth;
        for i in 0..n_in {
            let base = 7 + i * per_in;
            idx.push(base);
            idx.push(base + 1);
            idx.push(base + 2);
            idx.push(base + 3);
            for k in 0..depth {
                idx.push(base + 4 + k);
            }
        }
        let withdraw_idx = 7 + n_in * per_in;
        let outs_base = withdraw_idx + 3;
        for j in 0..n_out {
            idx.push(outs_base + 5 * j + 0);
            idx.push(outs_base + 5 * j + 1);
            idx.push(outs_base + 5 * j + 2);
            idx.push(outs_base + 5 * j + 3);
        }
        let inv_enforce_idx = outs_base + 5 * n_out;
        idx.push(inv_enforce_idx);
        let bl_checks = if is_transfer { 2usize } else { 1usize };
        let mut cur = inv_enforce_idx + 2;
        for _ in 0..bl_checks {
            for i in 0..BL_BUCKET_SIZE {
                idx.push(cur + i);
            }
            idx.push(cur + BL_BUCKET_SIZE);
            cur += BL_BUCKET_SIZE + 1;
            for k in 0..BL_DEPTH {
                idx.push(cur + k);
            }
            cur += BL_DEPTH;
        }
        idx
    }

    pub fn build_note_spend_args(depth: usize) -> Result<(Vec<serde_json::Value>, Vec<usize>)> {
        let domain: Hash32 = [1u8; 32];
        let rho_in: Hash32 = [2u8; 32];
        let spend_sk: Hash32 = [4u8; 32];
        let pk_ivk_owner = ivk_seed(&domain, &spend_sk);
        let recipient_owner = recipient_from_sk(&domain, &spend_sk, &pk_ivk_owner);
        let sender_id_current = recipient_owner;
        let nf_key = nf_key_from_sk(&domain, &spend_sk);

        let value: u64 = 42;
        let pos: u64 = 0;
        let sender_id_in: Hash32 = [6u8; 32];
        let cm_in = note_commitment(&domain, value, &rho_in, &recipient_owner, &sender_id_in);

        let mut tree = MerkleTree::new(depth);
        tree.set_leaf(pos as usize, cm_in);
        let anchor = tree.root();
        let siblings = tree.open(pos as usize);
        let nf = nullifier(&domain, &nf_key, &rho_in);

        let n_in: u64 = 1;
        let withdraw_amount: u64 = 0;
        let n_out: u64 = 1;
        let out_value: u64 = value;
        let out_rho: Hash32 = [7u8; 32];
        let out_spend_sk: Hash32 = [8u8; 32];
        let out_pk_spend = pk_from_sk(&out_spend_sk);
        let out_pk_ivk = ivk_seed(&domain, &out_spend_sk);
        let out_recipient = recipient_from_pk(&domain, &out_pk_spend, &out_pk_ivk);
        let cm_out = note_commitment(
            &domain,
            out_value,
            &out_rho,
            &out_recipient,
            &sender_id_current,
        );

        let inv_enforce = compute_inv_enforce(&[value], &[rho_in], &[out_value], &[out_rho])?;

        let mut args: Vec<serde_json::Value> = Vec::new();
        args.push(json!({ "hex": hx32(&domain) }));
        args.push(json!({ "hex": hx32(&spend_sk) }));
        args.push(json!({ "hex": hx32(&pk_ivk_owner) }));
        args.push(json!({ "i64": depth as i64 }));
        args.push(json!({ "hex": hx32(&anchor) }));
        args.push(json!({ "i64": n_in as i64 }));
        args.push(json!({ "i64": value as i64 }));
        args.push(json!({ "hex": hx32(&rho_in) }));
        args.push(json!({ "hex": hx32(&sender_id_in) }));
        args.push(json!({ "i64": pos as i64 }));
        for s in &siblings {
            args.push(json!({ "hex": hx32(s) }));
        }
        args.push(json!({ "hex": hx32(&nf) }));
        args.push(json!({ "i64": withdraw_amount as i64 }));
        args.push(json!({ "hex": hx32(&[0u8; 32]) }));
        args.push(json!({ "i64": n_out as i64 }));
        args.push(json!({ "i64": out_value as i64 }));
        args.push(json!({ "hex": hx32(&out_rho) }));
        args.push(json!({ "hex": hx32(&out_pk_spend) }));
        args.push(json!({ "hex": hx32(&out_pk_ivk) }));
        args.push(json!({ "hex": hx32(&cm_out) }));
        args.push(json!({ "hex": hx32(&inv_enforce) }));

        // Blacklist args.
        let bucket_entries = bl_empty_bucket_entries();
        let leaf0 = bl_bucket_leaf(&bucket_entries);
        let default_nodes = merkle_default_nodes_from_leaf(BL_DEPTH, &leaf0);
        let blacklist_root = default_nodes[BL_DEPTH];
        let bl_siblings: Vec<Hash32> = default_nodes.iter().take(BL_DEPTH).copied().collect();

        args.push(json!({ "hex": hx32(&blacklist_root) }));

        // Two blacklist checks for transfer (sender_id_current, out_recipient).
        for id in [sender_id_current, out_recipient] {
            for e in bucket_entries.iter() {
                args.push(json!({ "hex": hx32(e) }));
            }
            let inv = bl_bucket_inv_for_id(&id, &bucket_entries)
                .context("unexpected: id collides with empty blacklist bucket")?;
            args.push(json!({ "hex": hx32(&inv) }));
            for sib in bl_siblings.iter() {
                args.push(json!({ "hex": hx32(sib) }));
            }
        }

        let priv_idx = private_indices(depth, 1, 1, true);
        Ok((args, priv_idx))
    }
}

#[test]
fn test_http_server_prove_and_verify_roundtrip() -> Result<()> {
    if http_server_binary().is_err() {
        eprintln!("Skipping: HTTP server binary not found");
        return Ok(());
    }

    if !check_binaries_exist() {
        eprintln!("Skipping: Ligero prover/verifier binaries not found");
        return Ok(());
    }

    let port = next_port();
    let server = HttpServerHandle::start(port)?;

    // Build note_spend args with depth=8 for faster test.
    let depth = 8;
    let (args, priv_idx) = note_spend_helpers::build_note_spend_args(depth)?;

    // === PROVE ===
    println!("[test_roundtrip] Sending prove request...");
    let prove_body = json!({
        "circuit": "note_spend",
        "args": args,
        "privateIndices": priv_idx
    });

    let (status, prove_resp): (u16, ProveVerifyResponse) =
        server.post_json("/prove", &prove_body)?;

    // If the prover fails due to GPU unavailability, skip the test.
    if !prove_resp.success {
        if let Some(ref err) = prove_resp.error {
            if err.contains("GPU")
                || err.contains("WebGPU")
                || err.contains("Failed to execute")
                || err.contains("prover failed")
            {
                eprintln!(
                    "Skipping: prover failed (GPU/WebGPU likely unavailable): {}",
                    err
                );
                return Ok(());
            }
        }
        anyhow::bail!(
            "Prove request failed with status {}: {:?}",
            status,
            prove_resp.error
        );
    }

    assert_eq!(status, 200, "prove should return 200 on success");
    assert!(prove_resp.success, "prove should succeed");
    assert_eq!(prove_resp.exit_code, 0, "prove exit code should be 0");

    let proof = prove_resp
        .proof
        .as_ref()
        .context("prove response should contain proof")?;
    assert!(!proof.is_empty(), "proof should not be empty");

    println!(
        "[test_roundtrip] OK: proof generated ({} base64 chars)",
        proof.len()
    );

    // === VERIFY ===
    println!("[test_roundtrip] Sending verify request...");
    let verify_body = json!({
        "circuit": "note_spend",
        "args": args,
        "proof": proof,
        "privateIndices": priv_idx
    });

    let (verify_status, verify_resp): (u16, ProveVerifyResponse) =
        server.post_json("/verify", &verify_body)?;

    assert_eq!(verify_status, 200, "verify should return 200 on success");
    assert!(verify_resp.success, "verify should succeed");
    assert_eq!(verify_resp.exit_code, 0, "verify exit code should be 0");
    assert!(
        verify_resp.proof.is_none(),
        "verify should not return proof"
    );

    println!("[test_roundtrip] OK: proof verified successfully");
    Ok(())
}

#[test]
fn test_http_server_verify_fails_with_wrong_proof() -> Result<()> {
    if http_server_binary().is_err() {
        eprintln!("Skipping: HTTP server binary not found");
        return Ok(());
    }

    if !check_binaries_exist() {
        eprintln!("Skipping: Ligero prover/verifier binaries not found");
        return Ok(());
    }

    let port = next_port();
    let server = HttpServerHandle::start(port)?;

    let depth = 8;
    let (args, priv_idx) = note_spend_helpers::build_note_spend_args(depth)?;

    // First generate a valid proof.
    println!("[test_verify_wrong_proof] Generating valid proof...");
    let prove_body = json!({
        "circuit": "note_spend",
        "args": args,
        "privateIndices": priv_idx
    });

    let (_status, prove_resp): (u16, ProveVerifyResponse) =
        server.post_json("/prove", &prove_body)?;

    if !prove_resp.success {
        if let Some(ref err) = prove_resp.error {
            if err.contains("GPU")
                || err.contains("WebGPU")
                || err.contains("Failed to execute")
            {
                eprintln!(
                    "Skipping: prover failed (GPU/WebGPU likely unavailable): {}",
                    err
                );
                return Ok(());
            }
        }
        anyhow::bail!("Prove request failed: {:?}", prove_resp.error);
    }

    let proof = prove_resp
        .proof
        .as_ref()
        .context("prove response should contain proof")?;

    // Corrupt the proof by changing some bytes.
    let mut proof_bytes = base64::engine::general_purpose::STANDARD.decode(proof)?;
    if proof_bytes.len() > 100 {
        // Flip some bytes in the middle of the proof.
        for i in 50..60 {
            proof_bytes[i] ^= 0xFF;
        }
    }
    let corrupted_proof = base64::engine::general_purpose::STANDARD.encode(&proof_bytes);

    // Verify with corrupted proof - should fail.
    println!("[test_verify_wrong_proof] Sending verify request with corrupted proof...");
    let verify_body = json!({
        "circuit": "note_spend",
        "args": args,
        "proof": corrupted_proof,
        "privateIndices": priv_idx
    });

    let (verify_status, verify_resp): (u16, ProveVerifyResponse) =
        server.post_json("/verify", &verify_body)?;

    // The verify should fail (return success=false or error).
    assert!(
        !verify_resp.success || verify_status != 200,
        "verify with corrupted proof should fail"
    );

    println!("[test_verify_wrong_proof] OK: corrupted proof correctly rejected");
    Ok(())
}

#[test]
fn test_http_server_verify_fails_with_mutated_public_input() -> Result<()> {
    if http_server_binary().is_err() {
        eprintln!("Skipping: HTTP server binary not found");
        return Ok(());
    }

    if !check_binaries_exist() {
        eprintln!("Skipping: Ligero prover/verifier binaries not found");
        return Ok(());
    }

    let port = next_port();
    let server = HttpServerHandle::start(port)?;

    let depth = 8;
    let (args, priv_idx) = note_spend_helpers::build_note_spend_args(depth)?;

    // Generate a valid proof.
    println!("[test_verify_mutated_input] Generating valid proof...");
    let prove_body = json!({
        "circuit": "note_spend",
        "args": args,
        "privateIndices": priv_idx
    });

    let (_prove_status, prove_resp): (u16, ProveVerifyResponse) =
        server.post_json("/prove", &prove_body)?;

    if !prove_resp.success {
        if let Some(ref err) = prove_resp.error {
            if err.contains("GPU")
                || err.contains("WebGPU")
                || err.contains("Failed to execute")
            {
                eprintln!(
                    "Skipping: prover failed (GPU/WebGPU likely unavailable): {}",
                    err
                );
                return Ok(());
            }
        }
        anyhow::bail!("Prove request failed: {:?}", prove_resp.error);
    }

    let proof = prove_resp.proof.as_ref().context("missing proof")?;

    // Mutate the anchor (public input at index 4).
    let mut mutated_args = args.clone();
    mutated_args[4] = json!({ "hex": "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef" });

    // Verify with mutated public input - should fail.
    println!("[test_verify_mutated_input] Verifying with mutated anchor...");
    let verify_body = json!({
        "circuit": "note_spend",
        "args": mutated_args,
        "proof": proof,
        "privateIndices": priv_idx
    });

    let (verify_status, verify_resp): (u16, ProveVerifyResponse) =
        server.post_json("/verify", &verify_body)?;

    assert!(
        !verify_resp.success || verify_status != 200,
        "verify with mutated public input should fail"
    );

    println!("[test_verify_mutated_input] OK: mutated public input correctly rejected");
    Ok(())
}
