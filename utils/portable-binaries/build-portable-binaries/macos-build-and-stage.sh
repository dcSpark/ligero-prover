#!/usr/bin/env bash
set -euo pipefail

##
# Standalone macOS (arm64) portable build script.
#
# This script:
# - Ensures Homebrew + required formulae
# - Clones and builds:
#     - Dawn (pinned commit)
#     - wabt
#     - dcspark/ligero-prover (branch)
# - Stages:
#     ligero/
#       bins/macos-arm64/{bin,lib}/
#       shader/
# - Produces a tarball by default.
#
# Usage:
#   macos-build-and-stage.sh [--out <dir>] [--no-tar]
#
# Output:
#   <out>/ligero/...
#   <out>/ligero-macos-arm64.tar.gz   (unless --no-tar)

usage() {
  cat <<'EOF'
Usage: macos-build-and-stage.sh [--out <dir>] [--no-tar] [--cache-dir <dir>]

Options:
  --out <dir>         Output directory (default: current directory)
  --no-tar            Only stage the `ligero/` directory; do not create tarball
  --use-local-ligero  Use this local `ligero-prover` checkout instead of cloning
  --cache-dir <dir>   Directory to cache downloaded repos (default: ~/.cache/ligero-build)

Environment:
  CMAKE_JOB_COUNT     Parallel build jobs
  DAWN_GIT_REF        Dawn commit (default: cec4482eccee45696a7c0019e750c77f101ced04)
  LIGERO_REPO         Ligero prover git URL (default: https://github.com/dcspark/ligero-prover.git)
  LIGERO_BRANCH       Ligero prover git branch (default: main)
  FORCE_DOWNLOAD      Set to 1 to force re-downloading all repos even if cached
EOF
}

OUT_DIR="$PWD"
NO_TAR=false
USE_LOCAL_LIGERO=0
CACHE_DIR="${HOME}/.cache/ligero-build"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out)
      shift
      [[ $# -gt 0 ]] || { echo "error: --out expects a path" >&2; usage; exit 1; }
      OUT_DIR="$1"
      shift
      ;;
    --no-tar)
      NO_TAR=true
      shift
      ;;
    --use-local-ligero)
      USE_LOCAL_LIGERO=1
      shift
      ;;
    --cache-dir)
      shift
      [[ $# -gt 0 ]] || { echo "error: --cache-dir expects a path" >&2; usage; exit 1; }
      CACHE_DIR="$1"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown arg '$1'" >&2
      usage
      exit 1
      ;;
  esac
done

if [[ "$(uname -s)" != "Darwin" || "$(uname -m)" != "arm64" ]]; then
  echo "error: must run on macOS arm64" >&2
  exit 1
fi

DAWN_GIT_REF="${DAWN_GIT_REF:-cec4482eccee45696a7c0019e750c77f101ced04}"
LIGERO_REPO="${LIGERO_REPO:-https://github.com/dcspark/ligero-prover.git}"
LIGERO_BRANCH="${LIGERO_BRANCH:-main}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

JOBS="${CMAKE_JOB_COUNT:-}"
if [[ -z "$JOBS" ]]; then
  JOBS="$(sysctl -n hw.ncpu 2>/dev/null || echo 8)"
fi

mkdir -p "$OUT_DIR"
OUT_DIR="$(cd "$OUT_DIR" && pwd)"

if ! command -v brew >/dev/null 2>&1; then
  echo "error: Homebrew is required but not found." >&2
  echo "Install it with:" >&2
  echo '  /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"' >&2
  exit 1
fi

echo "==> Ensuring required Homebrew formulae..."
REQUIRED_FORMULAE=(cmake gmp mpfr libomp llvm boost)
for f in "${REQUIRED_FORMULAE[@]}"; do
  if ! brew list --formula "$f" >/dev/null 2>&1; then
    echo "==> Installing $f..."
    brew install "$f"
  fi
done

# Verify Boost version is >= 1.84 (archive version 19)
# This is required for cross-platform proof compatibility with Linux builds.
BOOST_VERSION="$(brew info --json=v2 boost | python3 -c 'import sys,json; print(json.load(sys.stdin)["formulae"][0]["versions"]["stable"])' 2>/dev/null || echo "unknown")"
echo "==> Homebrew Boost version: $BOOST_VERSION"
# Extract major.minor version for comparison
BOOST_MAJOR="${BOOST_VERSION%%.*}"
BOOST_REST="${BOOST_VERSION#*.}"
BOOST_MINOR="${BOOST_REST%%.*}"
if [[ "$BOOST_MAJOR" -lt 1 ]] || { [[ "$BOOST_MAJOR" -eq 1 ]] && [[ "$BOOST_MINOR" -lt 84 ]]; }; then
  echo "warning: Boost version $BOOST_VERSION is older than 1.84."
  echo "         Cross-platform proof compatibility requires Boost >= 1.84 (archive version 19)."
  echo "         Consider running: brew upgrade boost"
fi

# IMPORTANT:
# Homebrew LLVM uses its own libc++ headers which can mismatch the system libc++ at link-time,
# causing errors like: `Undefined symbols ... std::__1::__hash_memory`.
# Default to AppleClang (matches system libc++). Opt into Homebrew LLVM via USE_HOMEBREW_LLVM=1.
if [[ "${USE_HOMEBREW_LLVM:-0}" == "1" ]]; then
  LLVM_PREFIX="$(brew --prefix llvm)"
  export CC="${CC:-$LLVM_PREFIX/bin/clang}"
  export CXX="${CXX:-$LLVM_PREFIX/bin/clang++}"
else
  export CC="${CC:-$(xcrun --find clang)}"
  export CXX="${CXX:-$(xcrun --find clang++)}"
fi

if [[ ! -x "$CC" || ! -x "$CXX" ]]; then
  echo "error: C/C++ compilers not found (CC='$CC', CXX='$CXX')" >&2
  exit 1
fi

# Create cache directory for downloaded repos (persistent across builds)
mkdir -p "$CACHE_DIR"
CACHE_DIR="$(cd "$CACHE_DIR" && pwd)"

# Temp directory for build artifacts (cleaned up on exit)
TMP_ROOT="$(mktemp -d -t ligero-macos-build.XXXXXX)"
cleanup() {
  rm -rf "$TMP_ROOT"
}
trap cleanup EXIT

# Source directories (cached)
DAWN_SRC="$CACHE_DIR/dawn"
WABT_SRC="$CACHE_DIR/wabt"
LIGERO_SRC="$TMP_ROOT/ligero-prover"  # ligero-prover is special (--use-local-ligero or branch)

# Build directories (temporary)
SYSROOT="$TMP_ROOT/sysroot"
DAWN_BUILD="$TMP_ROOT/dawn-build"
WABT_BUILD="$TMP_ROOT/wabt-build"
LIGERO_BUILD="$TMP_ROOT/ligero-build"

mkdir -p "$SYSROOT" "$DAWN_BUILD" "$WABT_BUILD" "$LIGERO_BUILD"

FORCE_DOWNLOAD="${FORCE_DOWNLOAD:-0}"

# Clone or update Dawn
if [[ "$FORCE_DOWNLOAD" == "1" ]] && [[ -d "$DAWN_SRC" ]]; then
  echo "==> FORCE_DOWNLOAD: removing cached Dawn..."
  rm -rf "$DAWN_SRC"
fi

if [[ -d "$DAWN_SRC/.git" ]]; then
  echo "==> Using cached Dawn at $DAWN_SRC"
  cd "$DAWN_SRC"
  CURRENT_REF="$(git rev-parse HEAD 2>/dev/null || echo "")"
  if [[ "$CURRENT_REF" != "$DAWN_GIT_REF" ]]; then
    echo "==> Updating Dawn to $DAWN_GIT_REF..."
    git fetch origin
    git checkout "$DAWN_GIT_REF"
  fi
else
  echo "==> Cloning Dawn..."
  git clone https://dawn.googlesource.com/dawn "$DAWN_SRC"
  cd "$DAWN_SRC"
  git checkout "$DAWN_GIT_REF"
fi

echo "==> Building Dawn..."
cmake -S "$DAWN_SRC" -B "$DAWN_BUILD" \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_C_COMPILER="$CC" \
  -DCMAKE_CXX_COMPILER="$CXX" \
  -DDAWN_FETCH_DEPENDENCIES=ON \
  -DDAWN_BUILD_MONOLITHIC_LIBRARY=STATIC \
  -DDAWN_ENABLE_INSTALL=ON \
  -DCMAKE_INSTALL_PREFIX="$SYSROOT"
cmake --build "$DAWN_BUILD" --parallel "$JOBS"
cmake --install "$DAWN_BUILD"

# Clone or update wabt
if [[ "$FORCE_DOWNLOAD" == "1" ]] && [[ -d "$WABT_SRC" ]]; then
  echo "==> FORCE_DOWNLOAD: removing cached wabt..."
  rm -rf "$WABT_SRC"
fi

if [[ -d "$WABT_SRC/.git" ]]; then
  echo "==> Using cached wabt at $WABT_SRC"
  cd "$WABT_SRC"
else
  echo "==> Cloning wabt..."
  git clone https://github.com/WebAssembly/wabt.git "$WABT_SRC"
  cd "$WABT_SRC"
  git submodule update --init
fi

echo "==> Building wabt..."
cmake -S "$WABT_SRC" -B "$WABT_BUILD" \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_C_COMPILER="$CC" \
  -DCMAKE_CXX_COMPILER="$CXX" \
  -DCMAKE_INSTALL_PREFIX="$SYSROOT"
cmake --build "$WABT_BUILD" --parallel "$JOBS"
cmake --install "$WABT_BUILD"

# Clone or copy ligero-prover
echo "==> Preparing ligero-prover..."
if [[ "$USE_LOCAL_LIGERO" == "1" ]]; then
  if [[ ! -f "$REPO_ROOT/CMakeLists.txt" ]]; then
    echo "error: expected to be running inside a ligero-prover checkout (repo root: $REPO_ROOT)" >&2
    exit 1
  fi
  mkdir -p "$LIGERO_SRC"
  (cd "$REPO_ROOT" && tar --exclude=.git -cf - .) | (cd "$LIGERO_SRC" && tar -xf -)
else
  git clone "$LIGERO_REPO" -b "$LIGERO_BRANCH" "$LIGERO_SRC"
fi

echo "==> Patching ligero-prover for wabt compatibility..."
TRANSPILER_HPP="$LIGERO_SRC/include/transpiler.hpp"
if [[ -f "$TRANSPILER_HPP" ]] && ! grep -q "transpile_wabt_type(const wabt::Var" "$TRANSPILER_HPP"; then
  # wabt newer API uses wabt::Var for ref.null type (Var::to_type()).
  perl -0777 -i -pe 's/\}\n\n\/\/ ------------------------------------------------------------/\}\n\n\/\/ Newer wabt represents ref-null types as `wabt::Var` (which may carry an optional type).\n\/\/ Provide an overload so we can support both wabt APIs without pinning a specific version.\nvalue_kind transpile_wabt_type(const wabt::Var& var) {\n    return transpile_wabt_type(var.to_type());\n}\n\n\/\/ ------------------------------------------------------------/s' "$TRANSPILER_HPP"
fi

echo "==> Building ligero-prover..."
BREW_PREFIX="$(brew --prefix)"
cmake -S "$LIGERO_SRC" -B "$LIGERO_BUILD" \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_C_COMPILER="$CC" \
  -DCMAKE_CXX_COMPILER="$CXX" \
  -DCMAKE_PREFIX_PATH="$SYSROOT;$BREW_PREFIX"
cmake --build "$LIGERO_BUILD" --target webgpu_prover --parallel "$JOBS"
cmake --build "$LIGERO_BUILD" --target webgpu_verifier --parallel "$JOBS"

STAGE_ROOT="$OUT_DIR"
BIN_STAGE="$STAGE_ROOT/macos-arm64"

# Do not delete the whole stage root, as this script may be used as part of a multi-arch build.
mkdir -p "$STAGE_ROOT/shader"
rm -rf "$BIN_STAGE"
mkdir -p "$BIN_STAGE/bin" "$BIN_STAGE/lib"

echo "==> Staging shader folder..."
if ! find "$STAGE_ROOT/shader" -mindepth 1 -maxdepth 1 | read -r _; then
  cp -R "$LIGERO_SRC/shader/." "$STAGE_ROOT/shader/"
fi

echo "==> Staging binaries..."
install -m 0755 "$LIGERO_BUILD/webgpu_prover" "$BIN_STAGE/bin/webgpu_prover"
install -m 0755 "$LIGERO_BUILD/webgpu_verifier" "$BIN_STAGE/bin/webgpu_verifier"

# Prefer bundled libs.
install_name_tool -add_rpath "@executable_path/../lib" "$BIN_STAGE/bin/webgpu_prover" || true
install_name_tool -add_rpath "@executable_path/../lib" "$BIN_STAGE/bin/webgpu_verifier" || true

is_system_dylib() {
  case "$1" in
    /usr/lib/*|/System/*) return 0 ;;
    *) return 1 ;;
  esac
}

deps_for() {
  # Print dependency paths from otool -L output, excluding the first line (the binary itself)
  otool -L "$1" | tail -n +2 | awk '{print $1}'
}

copy_dylib() {
  local p="$1"
  [[ -f "$p" ]] || return 0
  local base
  base="$(basename "$p")"
  if [[ ! -f "$BIN_STAGE/lib/$base" ]]; then
    cp -L "$p" "$BIN_STAGE/lib/$base"
    chmod 0644 "$BIN_STAGE/lib/$base" || true
    # Make the dylib itself load relative deps
    install_name_tool -id "@rpath/$base" "$BIN_STAGE/lib/$base" || true
    install_name_tool -add_rpath "@loader_path" "$BIN_STAGE/lib/$base" || true
  fi
}

rewrite_dep() {
  local target="$1"
  local old="$2"
  local base
  base="$(basename "$old")"
  install_name_tool -change "$old" "@rpath/$base" "$target" || true
}

echo "==> Collecting dylibs..."
for bin in "$BIN_STAGE/bin/webgpu_prover" "$BIN_STAGE/bin/webgpu_verifier"; do
  while read -r dep; do
    [[ -n "$dep" ]] || continue
    if is_system_dylib "$dep"; then
      continue
    fi
    copy_dylib "$dep"
    rewrite_dep "$bin" "$dep"
  done < <(deps_for "$bin")
done

# Iterate on copied dylibs too (best-effort)
for _ in 1 2 3; do
  for lib in "$BIN_STAGE/lib/"*.dylib; do
    [[ -f "$lib" ]] || continue
    while read -r dep; do
      [[ -n "$dep" ]] || continue
      if is_system_dylib "$dep"; then
        continue
      fi
      copy_dylib "$dep"
      rewrite_dep "$lib" "$dep"
    done < <(deps_for "$lib")
  done
done

if [[ "$NO_TAR" == "false" ]]; then
  TARBALL="$OUT_DIR/ligero-macos-arm64.tar.gz"
  rm -f "$TARBALL"
  echo "==> Creating tarball: $TARBALL"
  (cd "$OUT_DIR" && tar -czf "$(basename "$TARBALL")" "macos-arm64" "shader")
fi

echo "==> Done."
echo "    Folder: $STAGE_ROOT"
if [[ "$NO_TAR" == "false" ]]; then
  echo "    Tarball: $TARBALL"
fi


