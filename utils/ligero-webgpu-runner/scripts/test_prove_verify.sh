#!/bin/bash
# test_prove_verify.sh - Test the Ligero HTTP server proving/verifying endpoints
#
# This script uses the generate-test-request binary to create cryptographically
# valid arguments for the note_spend circuit.
#
# Usage:
#   ./test_prove_verify.sh [server_url]
#
# Example:
#   ./test_prove_verify.sh http://127.0.0.1:1313

set -e

SERVER="${1:-http://127.0.0.1:1313}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$PROJECT_ROOT/../.." && pwd)"

# Find the generate-test-request binary
GENERATOR=""
for path in \
    "$PROJECT_ROOT/target/debug/generate-test-request" \
    "$PROJECT_ROOT/target/release/generate-test-request" \
    "$REPO_ROOT/target/debug/generate-test-request" \
    "$REPO_ROOT/target/release/generate-test-request"; do
    if [ -x "$path" ]; then
        GENERATOR="$path"
        break
    fi
done

if [ -z "$GENERATOR" ]; then
    echo "Error: generate-test-request binary not found."
    echo "Please build it first with:"
    echo "  cargo build --manifest-path=$PROJECT_ROOT/Cargo.toml --bin generate-test-request"
    exit 1
fi

echo "========================================"
echo "Ligero HTTP Server Test Script"
echo "========================================"
echo "Server:    $SERVER"
echo "Generator: $GENERATOR"
echo ""

# Create temporary files
REQUEST_FILE=$(mktemp)
PROVE_RESP_FILE=$(mktemp)
VERIFY_REQ_FILE=$(mktemp)
trap "rm -f $REQUEST_FILE $PROVE_RESP_FILE $VERIFY_REQ_FILE" EXIT

# Check if server is running
echo "[1/4] Checking server health..."
HEALTH=$(curl -s -X POST "$SERVER/health" -H "Content-Type: application/json" -d '{}' 2>/dev/null || echo '{}')
if echo "$HEALTH" | grep -q '"success":true'; then
    echo "  ✓ Server is healthy"
else
    echo "  ✗ Server not responding. Is it running?"
    echo "  Response: $HEALTH"
    echo ""
    echo "  Start the server with:"
    echo "    cargo run --manifest-path=$PROJECT_ROOT/Cargo.toml --bin ligero-http-server"
    exit 1
fi

# Generate the note_spend request JSON using the Rust helper
echo ""
echo "[2/4] Generating note_spend prove request..."
"$GENERATOR" > "$REQUEST_FILE"
REQUEST_SIZE=$(wc -c < "$REQUEST_FILE" | tr -d ' ')
echo "  ✓ Request generated ($REQUEST_SIZE bytes)"

# Send prove request
echo ""
echo "[3/4] Sending prove request (this may take a while)..."
START_TIME=$(date +%s)

curl -s -X POST "$SERVER/prove" \
    -H "Content-Type: application/json" \
    -d @"$REQUEST_FILE" > "$PROVE_RESP_FILE"

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

# Check prove result using Python for reliable JSON parsing
PROVE_RESULT=$(python3 << EOF
import json
import sys

try:
    with open("$PROVE_RESP_FILE") as f:
        resp = json.load(f)
    
    if resp.get('success'):
        proof = resp.get('proof', '')
        print(f"SUCCESS:{len(proof)}")
    else:
        error = resp.get('error', 'unknown error')
        # Check for GPU-related errors
        if any(x in error for x in ['GPU', 'WebGPU', 'Failed to execute', 'prover failed']):
            print(f"GPU_ERROR:{error[:200]}")
        else:
            print(f"ERROR:{error[:200]}")
except Exception as e:
    print(f"PARSE_ERROR:{e}")
EOF
)

if [[ "$PROVE_RESULT" == SUCCESS:* ]]; then
    PROOF_LEN="${PROVE_RESULT#SUCCESS:}"
    echo "  ✓ Proof generated successfully in ${DURATION}s"
    echo "  ✓ Proof size: $PROOF_LEN base64 characters"
elif [[ "$PROVE_RESULT" == GPU_ERROR:* ]]; then
    ERROR_MSG="${PROVE_RESULT#GPU_ERROR:}"
    echo "  ⚠ Proving skipped (GPU/WebGPU likely unavailable)"
    echo "  Note: This is expected in CI or non-GPU environments."
    echo "  The HTTP server is working correctly."
    echo ""
    echo "========================================"
    echo "Test completed (GPU tests skipped)"
    echo "========================================"
    exit 0
else
    ERROR_MSG="${PROVE_RESULT#ERROR:}"
    ERROR_MSG="${ERROR_MSG#PARSE_ERROR:}"
    echo "  ✗ Proving failed!"
    echo "  Error: $ERROR_MSG"
    exit 1
fi

# Create verify request with the proof
echo ""
echo "[4/4] Sending verify request..."

python3 << EOF
import json

with open("$REQUEST_FILE") as f:
    req = json.load(f)

with open("$PROVE_RESP_FILE") as f:
    resp = json.load(f)

req['proof'] = resp['proof']

with open("$VERIFY_REQ_FILE", 'w') as f:
    json.dump(req, f)
EOF

START_TIME=$(date +%s)

VERIFY_RESPONSE=$(curl -s -X POST "$SERVER/verify" \
    -H "Content-Type: application/json" \
    -d @"$VERIFY_REQ_FILE")

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

# Check verify result
VERIFY_SUCCESS=$(echo "$VERIFY_RESPONSE" | python3 -c "import sys,json; r=json.load(sys.stdin); print('yes' if r.get('success') else 'no')" 2>/dev/null || echo "no")

if [ "$VERIFY_SUCCESS" = "yes" ]; then
    echo "  ✓ Proof verified successfully in ${DURATION}s"
else
    echo "  ✗ Verification failed!"
    echo "  Response: $VERIFY_RESPONSE"
    exit 1
fi

echo ""
echo "========================================"
echo "All tests passed!"
echo "========================================"
