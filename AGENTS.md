# Repository Guidelines

This contributor guide is scoped to `utils/` (circuits, runner, and packaging). The core prover lives in `src/` and `shader/`, and the SDKs live in `sdk/`; reference those when debugging binary/shader/ABI issues.

## Project Structure (Utils-Focused)
- `utils/circuits/`: Rust WASI guest programs (`note-spend`, `note-deposit`) plus checked-in build outputs in `utils/circuits/bins/*.wasm` (and `*.wat` when generated).
- `utils/ligero-webgpu-runner/`: Rust crate (`ligero-runner`) that shells out to `webgpu_prover`/`webgpu_verifier`, manages proof artifacts, and supports “simulation mode” tests.
- `utils/portable-binaries/`: Staged portable `webgpu_prover`/`webgpu_verifier` binaries + shared libs, plus a packaged `shader/` snapshot for redistribution.

## Build, Test, and Bench Commands
Build circuits (writes to `utils/circuits/bins/`):
```bash
bash utils/circuits/note-spend/build.sh [--no-wat|--debug]
bash utils/circuits/note-deposit/build.sh [--no-wat|--debug]
```

Run runner tests:
```bash
cargo test --manifest-path utils/ligero-webgpu-runner/Cargo.toml
```

Validate circuit changes + performance review (recommended before PRs):
```bash
./scripts/run_note_spend_direct_bench.sh
```
Key knobs: `LIGERO_RUNS=3`, `LIGERO_PACKING=8192`, `LIGERO_VERBOSE=0|1`, `LIGERO_ENABLE_VIEWERS=0|1`.

Build portable binaries (Linux arches require Docker; use local shaders to avoid git clones):
```bash
bash utils/portable-binaries/build-portable-binaries.sh --use-local-ligero
```

## Debugging & Configuration Tips
- Override discovery with `LIGERO_PROVER_BIN` (or `LIGERO_PROVER_BINARY_PATH`) or set `LIGERO_ROOT=<repo-root>` to help the runner find binaries and `shader/`.
- If you do changes to the circuits, you need to rebuild the wasm version.

## Coding, Commits, and PRs
- Format Rust with `cargo fmt`; keep crates `edition = "2021"` conventions.
- Don’t commit generated proof artifacts (`utils/ligero-webgpu-runner/proof_outputs/` is gitignored).
- PRs that touch circuits should include updated `utils/circuits/bins/` artifacts and note the exact validation run (at minimum `cargo test` and `./scripts/run_note_spend_direct_bench.sh`).
