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

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Enable verbose output by default for this script
export LIGERO_VERBOSE="${LIGERO_VERBOSE:-1}"

# Run only 1 proof by default (set LIGERO_RUNS=3 for full cross-verification test)
export LIGERO_RUNS="${LIGERO_RUNS:-1}"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Note Spend Direct Benchmark ===${NC}"
echo ""

# Step 1: Rebuild the circuit
echo -e "${BLUE}[1/2] Rebuilding note-spend circuit...${NC}"
CIRCUIT_BUILD_SCRIPT="$REPO_ROOT/utils/circuits/note-spend/build.sh"

if [[ ! -f "$CIRCUIT_BUILD_SCRIPT" ]]; then
    echo -e "${RED}Error: Circuit build script not found at $CIRCUIT_BUILD_SCRIPT${NC}"
    exit 1
fi

(cd "$REPO_ROOT/utils/circuits/note-spend" && bash build.sh)
echo -e "${GREEN}✓ Circuit rebuilt successfully${NC}"
echo ""

# Step 2: Run the benchmark
echo -e "${BLUE}[2/2] Running test_note_spend_direct_bench...${NC}"
cd "$REPO_ROOT/utils/ligero-webgpu-runner"
cargo test --test note_spend_bench test_note_spend_direct_bench -- --nocapture

echo ""
echo -e "${GREEN}=== Benchmark complete ===${NC}"
