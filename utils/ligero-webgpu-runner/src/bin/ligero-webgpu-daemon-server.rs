use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::{Context, Result};

use ligero_runner::{daemon::DaemonServer, LigeroPaths};

fn parse_usize(flag: &str, v: Option<&String>) -> Result<usize> {
    let s = v.with_context(|| format!("missing value for {flag}"))?;
    s.parse::<usize>()
        .with_context(|| format!("invalid {flag}={s}"))
}

fn main() -> Result<()> {
    // Minimal argv parsing (no clap to keep deps small).
    // Example:
    //   ligero-webgpu-daemon-server --unix /tmp/ligero.sock --prover-workers 4 --verifier-workers 4
    //   ligero-webgpu-daemon-server --tcp 127.0.0.1:7777
    let args: Vec<String> = std::env::args().collect();

    let mut unix_path: Option<PathBuf> = None;
    let mut tcp_addr: Option<SocketAddr> = None;
    let mut prover_workers: Option<usize> = None;
    let mut verifier_workers: Option<usize> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--unix" => {
                i += 1;
                unix_path = Some(PathBuf::from(
                    args.get(i).context("missing value for --unix")?,
                ));
            }
            "--tcp" => {
                i += 1;
                let s = args.get(i).context("missing value for --tcp")?;
                tcp_addr = Some(s.parse::<SocketAddr>().context("invalid --tcp addr")?);
            }
            "--prover-workers" => {
                i += 1;
                prover_workers = Some(parse_usize("--prover-workers", args.get(i))?);
            }
            "--verifier-workers" => {
                i += 1;
                verifier_workers = Some(parse_usize("--verifier-workers", args.get(i))?);
            }
            "--help" | "-h" => {
                eprintln!(
                    "Usage:\n  ligero-webgpu-daemon-server [--unix PATH] [--tcp ADDR] [--prover-workers N] [--verifier-workers N]\n\nExamples:\n  ligero-webgpu-daemon-server --unix /tmp/ligero.sock\n  ligero-webgpu-daemon-server --tcp 127.0.0.1:7777\n"
                );
                return Ok(());
            }
            other => anyhow::bail!("unknown arg: {other}"),
        }
        i += 1;
    }

    if unix_path.is_none() && tcp_addr.is_none() {
        anyhow::bail!("must provide at least one of --unix or --tcp");
    }

    let default_n = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let prover_workers = prover_workers.unwrap_or(default_n);
    let verifier_workers = verifier_workers.unwrap_or(default_n);

    let paths = LigeroPaths::discover().unwrap_or_else(|_| LigeroPaths::fallback());

    let server = DaemonServer::new(&paths, prover_workers, verifier_workers)?;

    if let Some(addr) = tcp_addr {
        eprintln!("Serving TCP on {addr} (prover={prover_workers}, verifier={verifier_workers})");
        let server_tcp = server.clone();
        std::thread::spawn(move || {
            let _ = server_tcp.serve_tcp(addr);
        });
    }

    #[cfg(unix)]
    if let Some(ref path) = unix_path {
        eprintln!(
            "Serving Unix socket at {} (prover={}, verifier={})",
            path.display(),
            prover_workers,
            verifier_workers
        );
        // Blocks forever.
        server.serve_unix(path)?;
    }

    // If only TCP was configured, block forever.
    if tcp_addr.is_some() && unix_path.is_none() {
        std::thread::park();
    }

    Ok(())
}


