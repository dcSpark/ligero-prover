#!/usr/bin/env bash
# Script to rebuild the note-spend circuit and run the direct benchmark
#
# Usage:
#   ./scripts/run_note_spend_direct_bench.sh
#
# Environment variables:
#   LIGERO_VERBOSE=1  - Show detailed prover/verifier output (default: 1)
#   LIGERO_RUNS=N     - Number of proofs to generate (default: 1, use 3 for cross-verification)
#   LIGERO_PACKING=N  - Override packing size (default: 8192)
#   LIGERO_ENABLE_VIEWERS=1 - Enable viewer attestations (FVK) for transfer/withdraw (default: 1)
#   LIGERO_SHOW_VIEWER_PAYLOAD=1 - Print decrypted viewer payload for TRANSFER run0 (default: 1)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Enable verbose output by default for this script
export LIGERO_VERBOSE="${LIGERO_VERBOSE:-1}"

# Run only 1 proof by default (set LIGERO_RUNS=3 for full cross-verification test)
export LIGERO_RUNS="${LIGERO_RUNS:-1}"

# Enable viewer attestations by default for this benchmark (transfer/withdraw only).
export LIGERO_ENABLE_VIEWERS="${LIGERO_ENABLE_VIEWERS:-1}"

# Print decrypted viewer payload for transfer by default (sensitive; set to 0 to disable).
export LIGERO_SHOW_VIEWER_PAYLOAD="${LIGERO_SHOW_VIEWER_PAYLOAD:-1}"

# Prefer locally-built binaries (if present) over packaged portable ones.
if [[ -x "$REPO_ROOT/build/webgpu_prover" ]]; then
    export LIGERO_PROVER_BIN="$REPO_ROOT/build/webgpu_prover"
fi

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Note Spend Direct Benchmark ===${NC}"
echo -e "${BLUE}Viewers (FVK attestations): ${LIGERO_ENABLE_VIEWERS}${NC}"
echo -e "${BLUE}Show decrypted viewer payload (TRANSFER run0): ${LIGERO_SHOW_VIEWER_PAYLOAD}${NC}"
echo ""

# Step 1: Rebuild the circuits
echo -e "${BLUE}[1/2] Rebuilding deposit + spend circuits...${NC}"

DEPOSIT_BUILD_SCRIPT="$REPO_ROOT/utils/circuits/note-deposit/build.sh"
SPEND_BUILD_SCRIPT="$REPO_ROOT/utils/circuits/note-spend/build.sh"

if [[ ! -f "$DEPOSIT_BUILD_SCRIPT" ]]; then
    echo -e "${RED}Error: Deposit build script not found at $DEPOSIT_BUILD_SCRIPT${NC}"
    exit 1
fi
if [[ ! -f "$SPEND_BUILD_SCRIPT" ]]; then
    echo -e "${RED}Error: Spend build script not found at $SPEND_BUILD_SCRIPT${NC}"
    exit 1
fi

(cd "$REPO_ROOT/utils/circuits/note-deposit" && bash build.sh)
(cd "$REPO_ROOT/utils/circuits/note-spend" && bash build.sh)
echo -e "${GREEN}✓ Circuits rebuilt successfully${NC}"
echo ""

# Step 2: Run the benchmarks (Deposit / Transfer / Withdraw)
echo -e "${BLUE}[2/2] Running direct benches (deposit/transfer/withdraw)...${NC}"
cd "$REPO_ROOT/utils/ligero-webgpu-runner"
# Run benches serially to avoid GPU/prover contention skewing timings.
cargo test --test note_spend_bench test_note_spend_direct_bench_ -- --nocapture --test-threads=1

echo ""
echo -e "${GREEN}=== Benchmark complete ===${NC}"
