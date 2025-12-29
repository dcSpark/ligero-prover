//! Daemon-mode helpers for `webgpu_prover` / `webgpu_verifier`.
//!
//! This module implements:
//! - A pool of long-lived child processes running `webgpu_prover --daemon` and `webgpu_verifier --daemon`
//! - A small proxy server that exposes those daemons via Unix sockets and/or TCP.
//!
//! ## C++ protocol (stdin/stdout)
//! The C++ daemons speak NDJSON:
//! - stdin:  one JSON request per line (the existing Ligero config JSON, with optional extra fields)
//! - stdout: one JSON response per line: `{ ok, exit_code, proof_path?, verify_ok?, error?, id? }`
//!
//! ## Socket protocol (client/server)
//! For TCP/Unix sockets we use a length-prefixed framing:
//! - u32 (big-endian) byte length, then that many bytes of UTF-8 JSON payload.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::LigeroPaths;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DaemonRequest {
    /// Run the prover with the provided Ligero config JSON.
    Prove { id: Option<String>, config: Value },
    /// Run the verifier with the provided Ligero config JSON and proof path.
    Verify {
        id: Option<String>,
        config: Value,
        proof_path: String,
    },
    /// Health check.
    Health { id: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonResponse {
    pub id: Option<String>,
    pub ok: bool,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub exit_code: Option<i64>,
    #[serde(default)]
    pub proof_path: Option<String>,
    #[serde(default)]
    pub verify_ok: Option<bool>,
}

fn canonicalize_if_possible(p: &str) -> String {
    std::fs::canonicalize(p)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| p.to_string())
}

fn canonicalize_config_paths(mut cfg: Value) -> Value {
    // Best-effort: if program/shader-path are strings, make them absolute.
    if let Value::Object(ref mut map) = cfg {
        if let Some(Value::String(program)) = map.get("program").cloned() {
            map.insert("program".to_string(), Value::String(canonicalize_if_possible(&program)));
        }
        if let Some(Value::String(shader)) = map.get("shader-path").cloned() {
            map.insert(
                "shader-path".to_string(),
                Value::String(canonicalize_if_possible(&shader)),
            );
        }
        if let Some(Value::String(proof_path)) = map.get("proof-path").cloned() {
            map.insert(
                "proof-path".to_string(),
                Value::String(canonicalize_if_possible(&proof_path)),
            );
        }
    }
    cfg
}

struct DaemonIo {
    child: Child,
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
}

/// A single daemon worker process.
pub struct DaemonWorker {
    io: Mutex<DaemonIo>,
}

impl DaemonWorker {
    fn spawn(bin: &Path, args: &[&str], cwd: &Path) -> Result<Self> {
        let mut child = Command::new(bin)
            .args(args)
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to spawn daemon {}", bin.display()))?;

        // Feature detection: binaries that don't support `--daemon` will treat it as JSON input,
        // print a parse error and exit immediately. Daemon-capable binaries should stay alive and
        // block on stdin waiting for requests.
        thread::sleep(Duration::from_millis(50));
        if let Some(status) = child
            .try_wait()
            .context("Failed to check daemon child status")?
        {
            // Don't spam the caller's stderr on feature-detection failures; capture it instead.
            let mut stderr_buf = Vec::new();
            if let Some(mut stderr) = child.stderr.take() {
                let _ = stderr.read_to_end(&mut stderr_buf);
            }
            let stderr_s = String::from_utf8_lossy(&stderr_buf).trim().to_string();
            return Err(anyhow!(
                "{} exited immediately (status={}); is this binary built with --daemon support?\n{}",
                bin.display(),
                status,
                stderr_s
            ));
        }

        // Forward daemon stderr to parent stderr for visibility (stdout is reserved for protocol).
        if let Some(mut stderr) = child.stderr.take() {
            thread::spawn(move || {
                let mut out = std::io::stderr();
                let _ = std::io::copy(&mut stderr, &mut out);
            });
        }

        let stdin = child.stdin.take().ok_or_else(|| anyhow!("failed to open stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow!("failed to open stdout"))?;

        Ok(Self {
            io: Mutex::new(DaemonIo {
                child,
                stdin: BufWriter::new(stdin),
                stdout: BufReader::new(stdout),
            }),
        })
    }

    fn request_json_line(&self, config: &Value) -> Result<DaemonResponse> {
        let mut io = self.io.lock().unwrap();

        let line = serde_json::to_string(config).context("Failed to serialize daemon request")?;
        io.stdin
            .write_all(line.as_bytes())
            .context("Failed to write daemon request")?;
        io.stdin
            .write_all(b"\n")
            .context("Failed to write daemon newline")?;
        io.stdin.flush().context("Failed to flush daemon stdin")?;

        let mut resp_line = String::new();
        let n = io
            .stdout
            .read_line(&mut resp_line)
            .context("Failed to read daemon response line")?;
        if n == 0 {
            return Err(anyhow!("daemon stdout closed unexpectedly"));
        }

        let resp: DaemonResponse =
            serde_json::from_str(resp_line.trim_end()).context("Failed to parse daemon response")?;
        Ok(resp)
    }
}

impl Drop for DaemonWorker {
    fn drop(&mut self) {
        if let Ok(mut io) = self.io.lock() {
            // Best-effort terminate.
            let _ = io.child.kill();
            let _ = io.child.wait();
        }
    }
}

#[derive(Clone)]
pub struct DaemonPool {
    workers: Arc<Vec<Arc<DaemonWorker>>>,
    available: Arc<(Mutex<VecDeque<usize>>, Condvar)>,
}

impl DaemonPool {
    pub fn new_prover(paths: &LigeroPaths, n: usize) -> Result<Self> {
        let n = n.max(1);
        let mut workers = Vec::with_capacity(n);
        for _ in 0..n {
            workers.push(Arc::new(DaemonWorker::spawn(
                &paths.prover_bin,
                &["--daemon"],
                &paths.bins_dir,
            )?));
        }
        Ok(Self::from_workers(workers))
    }

    pub fn new_verifier(paths: &LigeroPaths, n: usize) -> Result<Self> {
        let n = n.max(1);
        let mut workers = Vec::with_capacity(n);
        for _ in 0..n {
            workers.push(Arc::new(DaemonWorker::spawn(
                &paths.verifier_bin,
                &["--daemon"],
                &paths.bins_dir,
            )?));
        }
        Ok(Self::from_workers(workers))
    }

    fn from_workers(workers: Vec<Arc<DaemonWorker>>) -> Self {
        let mut q = VecDeque::with_capacity(workers.len());
        for i in 0..workers.len() {
            q.push_back(i);
        }
        Self {
            workers: Arc::new(workers),
            available: Arc::new((Mutex::new(q), Condvar::new())),
        }
    }

    fn acquire(&self) -> usize {
        let (lock, cv) = &*self.available;
        let mut q = lock.lock().unwrap();
        loop {
            if let Some(i) = q.pop_front() {
                return i;
            }
            q = cv.wait(q).unwrap();
        }
    }

    fn release(&self, i: usize) {
        let (lock, cv) = &*self.available;
        let mut q = lock.lock().unwrap();
        q.push_back(i);
        cv.notify_one();
    }

    pub fn prove(&self, config: Value) -> Result<DaemonResponse> {
        let i = self.acquire();
        let worker = self.workers[i].clone();
        let res = worker.request_json_line(&canonicalize_config_paths(config));
        self.release(i);
        res
    }

    pub fn verify(&self, mut config: Value, proof_path: &str) -> Result<DaemonResponse> {
        // Ensure verifier gets a proof path.
        if let Value::Object(ref mut map) = config {
            map.insert(
                "proof-path".to_string(),
                Value::String(proof_path.to_string()),
            );
        }
        let i = self.acquire();
        let worker = self.workers[i].clone();
        let res = worker.request_json_line(&canonicalize_config_paths(config));
        self.release(i);
        res
    }
}

fn read_frame<R: Read>(r: &mut R) -> Result<Vec<u8>> {
    let mut len_buf = [0u8; 4];
    r.read_exact(&mut len_buf).context("Failed to read frame len")?;
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf).context("Failed to read frame body")?;
    Ok(buf)
}

fn write_frame<W: Write>(w: &mut W, bytes: &[u8]) -> Result<()> {
    let len = u32::try_from(bytes.len()).context("frame too large")?;
    w.write_all(&len.to_be_bytes()).context("Failed to write frame len")?;
    w.write_all(bytes).context("Failed to write frame body")?;
    w.flush().ok();
    Ok(())
}

fn handle_stream(mut stream: impl Read + Write, prover: &DaemonPool, verifier: &DaemonPool) -> Result<()> {
    let req_bytes = read_frame(&mut stream)?;
    let req: DaemonRequest = serde_json::from_slice(&req_bytes).context("Failed to parse request")?;

    let resp = match req {
        DaemonRequest::Health { id } => DaemonResponse {
            id,
            ok: true,
            error: None,
            exit_code: None,
            proof_path: None,
            verify_ok: None,
        },
        DaemonRequest::Prove { id, config } => {
            let mut r = prover.prove(config)?;
            r.id = id.or(r.id);
            r
        }
        DaemonRequest::Verify {
            id,
            config,
            proof_path,
        } => {
            let mut r = verifier.verify(config, &proof_path)?;
            r.id = id.or(r.id);
            r
        }
    };

    let out = serde_json::to_vec(&resp).context("Failed to serialize response")?;
    write_frame(&mut stream, &out)?;
    Ok(())
}

/// A proxy server that exposes prover/verifier daemon pools via Unix sockets and/or TCP.
#[derive(Clone)]
pub struct DaemonServer {
    prover: DaemonPool,
    verifier: DaemonPool,
}

impl DaemonServer {
    pub fn new(paths: &LigeroPaths, prover_workers: usize, verifier_workers: usize) -> Result<Self> {
        Ok(Self {
            prover: DaemonPool::new_prover(paths, prover_workers)?,
            verifier: DaemonPool::new_verifier(paths, verifier_workers)?,
        })
    }

    pub fn serve_tcp(&self, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr).with_context(|| format!("bind tcp {}", addr))?;
        let prover = self.prover.clone();
        let verifier = self.verifier.clone();

        for conn in listener.incoming() {
            let conn = conn?;
            let p = prover.clone();
            let v = verifier.clone();
            thread::spawn(move || {
                let _ = handle_stream(conn, &p, &v);
            });
        }
        Ok(())
    }

    #[cfg(unix)]
    pub fn serve_unix(&self, path: &Path) -> Result<()> {
        use std::os::unix::net::UnixListener;
        // Remove stale socket path.
        let _ = std::fs::remove_file(path);
        let listener =
            UnixListener::bind(path).with_context(|| format!("bind unix {}", path.display()))?;

        let prover = self.prover.clone();
        let verifier = self.verifier.clone();

        for conn in listener.incoming() {
            let conn = conn?;
            let p = prover.clone();
            let v = verifier.clone();
            thread::spawn(move || {
                let _ = handle_stream(conn, &p, &v);
            });
        }
        Ok(())
    }
}


