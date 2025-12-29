//! Minimal always-on worker pools for running external Ligero binaries.
//!
//! Motivation: process execution is blocking and can be expensive in callers that are async or
//! want consistent latency. We keep a fixed set of OS threads alive and dispatch jobs onto them.

use std::sync::{mpsc, OnceLock};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Clone)]
pub struct BinaryWorkerPool {
    tx: mpsc::Sender<Job>,
}

impl BinaryWorkerPool {
    pub fn new(worker_count: usize) -> Self {
        let (tx, rx) = mpsc::channel::<Job>();

        // Shared receiver for N workers.
        let rx = std::sync::Arc::new(std::sync::Mutex::new(rx));
        let n = worker_count.max(1);

        for i in 0..n {
            let rx = rx.clone();
            thread::Builder::new()
                .name(format!("ligero-bin-worker-{i}"))
                .spawn(move || loop {
                    let job = match rx.lock().ok().and_then(|guard| guard.recv().ok()) {
                        Some(j) => j,
                        None => return,
                    };
                    job();
                })
                .expect("failed to spawn ligero binary worker thread");
        }

        Self { tx }
    }

    pub fn execute<R, F>(&self, f: F) -> R
    where
        R: Send + 'static,
        F: FnOnce() -> R + Send + 'static,
    {
        let (rtx, rrx) = mpsc::channel::<R>();
        self.tx
            .send(Box::new(move || {
                let _ = rtx.send(f());
            }))
            .expect("binary worker pool is not available");
        rrx.recv().expect("binary worker pool worker terminated")
    }
}

fn default_worker_count() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

static DEFAULT_PROVER_POOL: OnceLock<BinaryWorkerPool> = OnceLock::new();
static DEFAULT_VERIFIER_POOL: OnceLock<BinaryWorkerPool> = OnceLock::new();

pub fn default_prover_pool() -> &'static BinaryWorkerPool {
    DEFAULT_PROVER_POOL.get_or_init(|| BinaryWorkerPool::new(default_worker_count()))
}

pub fn default_verifier_pool() -> &'static BinaryWorkerPool {
    DEFAULT_VERIFIER_POOL.get_or_init(|| BinaryWorkerPool::new(default_worker_count()))
}
