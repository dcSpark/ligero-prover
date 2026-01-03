#!/usr/bin/env bash
# Build script for note-deposit-guest WASM module
#
# Usage:
#   ./build.sh          # Build, copy WASM, and (if wasm2wat is available) generate WAT
#   ./build.sh --no-wat # Build and copy WASM only (skip WAT generation)
#   ./build.sh --debug  # Build with diagnostics feature enabled

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}Building note-deposit-guest...${NC}"

# Parse arguments
FEATURES=""
GENERATE_WAT=true
for arg in "$@"; do
    case $arg in
        --no-wat)
            GENERATE_WAT=false
            ;;
        --debug)
            FEATURES="--features diagnostics"
            echo -e "${YELLOW}Building with diagnostics feature enabled${NC}"
            ;;
    esac
done

# Ensure the wasm32-wasip1 target is installed
echo "Checking for wasm32-wasip1 target..."
if ! rustup target list | grep -q "wasm32-wasip1 (installed)"; then
    echo "Installing wasm32-wasip1 target..."
    rustup target add wasm32-wasip1
fi

echo "Building WASM module..."
RUSTFLAGS="-C link-arg=-s" cargo build --target wasm32-wasip1 --release $FEATURES

OUT_DIR="../bins"
mkdir -p "$OUT_DIR"

WASM_FILE="target/wasm32-wasip1/release/note_deposit_guest.wasm"

if command -v wasm-opt &> /dev/null; then
    echo "Optimizing with wasm-opt..."
    wasm-opt --enable-bulk-memory -O4 -o "$OUT_DIR/note_deposit_guest.wasm" "$WASM_FILE"
    ORIG_SIZE=$(ls -lh "$WASM_FILE" | awk '{print $5}')
    OPT_SIZE=$(ls -lh "$OUT_DIR/note_deposit_guest.wasm" | awk '{print $5}')
    echo -e "${GREEN}✓ WASM optimized: $ORIG_SIZE → $OPT_SIZE${NC}"
else
    cp "$WASM_FILE" "$OUT_DIR/note_deposit_guest.wasm"
    echo -e "${YELLOW}Warning: wasm-opt not found. Install binaryen for faster binaries.${NC}"
fi

if command -v wasm-strip &> /dev/null; then
    wasm-strip "$OUT_DIR/note_deposit_guest.wasm" 2>/dev/null || true
fi

echo -e "${GREEN}✓ WASM binary built: $OUT_DIR/note_deposit_guest.wasm${NC}"
SIZE=$(ls -lh "$OUT_DIR/note_deposit_guest.wasm" | awk '{print $5}')
echo "  Final size: $SIZE"

if [[ "$GENERATE_WAT" == "true" ]]; then
    echo "Generating WAT (WebAssembly Text) file..."
    if command -v wasm2wat &> /dev/null; then
        wasm2wat "$OUT_DIR/note_deposit_guest.wasm" -o "$OUT_DIR/note_deposit_guest.wat"
        echo -e "${GREEN}✓ WAT file generated: $OUT_DIR/note_deposit_guest.wat${NC}"
    else
        echo "Warning: wasm2wat not found. Install WABT to generate WAT files."
        echo "  https://github.com/WebAssembly/wabt"
    fi
fi

echo -e "${GREEN}Build complete!${NC}"

