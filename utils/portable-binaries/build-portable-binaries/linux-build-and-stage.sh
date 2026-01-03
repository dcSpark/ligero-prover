#!/usr/bin/env bash
set -euo pipefail

##
# Standalone Linux portable build script (intended to run in Docker).
#
# This script:
# - Installs build dependencies
# - Clones and builds:
#     - depot_tools + Dawn (pinned commit) via gclient sync + bundled clang
#     - wabt
#     - dcspark/ligero-prover (branch)
# - Stages:
#     ligero/
#       bins/<arch>/{bin,lib}/
#       shader/               (only when producing tarball)
# - Produces a tarball by default.
#
# Usage:
#   linux-build-and-stage.sh --arch <linux-amd64|linux-arm64> [--out <dir>] [--no-tar]
#
# Output:
#   <out>/ligero/...
#   <out>/ligero-<arch>.tar.gz   (unless --no-tar)

usage() {
  cat <<'EOF'
Usage: linux-build-and-stage.sh --arch <linux-amd64|linux-arm64> [--out <dir>] [--no-tar]

Options:
  --arch <arch>   linux-amd64 or linux-arm64 (required)
  --out <dir>     Output directory (default: /out if exists, else current directory)
  --no-tar        Only stage `ligero/bins/<arch>`; do not touch `ligero/shader` and do not create tarball
  --use-local-ligero  Use this workspace's local `ligero-prover` checkout (mounted at /ligero-local) instead of cloning

Environment:
  CMAKE_JOB_COUNT  Parallel build jobs
  DAWN_GIT_REF      Dawn commit (default: cec4482eccee45696a7c0019e750c77f101ced04)
  LIGERO_REPO       Ligero prover git URL (default: https://github.com/dcspark/ligero-prover.git)
  LIGERO_BRANCH     Ligero prover git branch (default: feature/ligero-runner)
EOF
}

ARCH=""
OUT_DIR=""
NO_TAR=false
USE_LOCAL_LIGERO=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --arch)
      shift
      [[ $# -gt 0 ]] || { echo "error: --arch expects a value" >&2; usage; exit 1; }
      ARCH="$1"
      shift
      ;;
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

if [[ -z "$ARCH" ]]; then
  echo "error: --arch is required" >&2
  usage
  exit 1
fi

case "$ARCH" in
  linux-amd64|linux-arm64) ;;
  *) echo "error: unsupported arch '$ARCH'" >&2; usage; exit 1 ;;
esac

if [[ -z "$OUT_DIR" ]]; then
  if [[ -d /out ]]; then
    OUT_DIR="/out"
  else
    OUT_DIR="$PWD"
  fi
fi

OUT_DIR="$(mkdir -p "$OUT_DIR" && cd "$OUT_DIR" && pwd)"

HOST_UID="${HOST_UID:-0}"
HOST_GID="${HOST_GID:-0}"

export DEBIAN_FRONTEND=noninteractive

# Boost version to build from source (must match LIGERO_PORTABLE_ARCHIVE_VERSION expectation)
BOOST_VERSION="${BOOST_VERSION:-1.89.0}"
BOOST_VERSION_UNDERSCORE="${BOOST_VERSION//./_}"

echo "==> [${ARCH}] Installing packages..."
apt-get update
apt-get install -y --no-install-recommends \
  build-essential \
  ca-certificates \
  clang \
  cmake \
  curl \
  g++-13 \
  gcc-13 \
  git \
  libbz2-dev \
  libgl1-mesa-dev \
  libglu1-mesa-dev \
  libgmp-dev \
  liblzma-dev \
  libmpfr-dev \
  libssl-dev \
  libtbb-dev \
  libvulkan-dev \
  libwayland-dev \
  libx11-dev \
  libx11-xcb-dev \
  libxkbcommon-dev \
  libxcursor-dev \
  libxi-dev \
  libxinerama-dev \
  libxrandr-dev \
  libzstd-dev \
  ninja-build \
  patchelf \
  pkg-config \
  python3 \
  python3-venv \
  python3-jinja2 \
  python3-pip \
  unzip \
  wget \
  xz-utils \
  zlib1g-dev
rm -rf /var/lib/apt/lists/*

JOBS="${CMAKE_JOB_COUNT:-}"
if [[ -z "$JOBS" ]]; then
  if command -v nproc >/dev/null 2>&1; then
    JOBS="$(nproc)"
  else
    JOBS="4"
  fi
fi

# Build Boost from source to ensure consistent version across platforms
echo "==> [${ARCH}] Building Boost ${BOOST_VERSION} from source (using $JOBS jobs)..."
BOOST_SRC="/tmp/boost-src"
BOOST_INSTALL="/tmp/boost-install"
mkdir -p "$BOOST_SRC" "$BOOST_INSTALL"

wget -q -O "$BOOST_SRC/boost.tar.gz" \
  "https://archives.boost.io/release/${BOOST_VERSION}/source/boost_${BOOST_VERSION_UNDERSCORE}.tar.gz"
tar -xzf "$BOOST_SRC/boost.tar.gz" -C "$BOOST_SRC" --strip-components=1

cd "$BOOST_SRC"
./bootstrap.sh \
  --prefix="$BOOST_INSTALL" \
  --with-libraries=filesystem,iostreams,log,program_options,random,serialization,thread,exception,test

./b2 install \
  -j"$JOBS" \
  variant=release \
  link=shared \
  threading=multi \
  --prefix="$BOOST_INSTALL"

# Make Boost discoverable by CMake
export BOOST_ROOT="$BOOST_INSTALL"
export Boost_DIR="$BOOST_INSTALL/lib/cmake/Boost-${BOOST_VERSION}"

echo "==> [${ARCH}] Boost ${BOOST_VERSION} installed to $BOOST_INSTALL"

SYSROOT="/tmp/sysroot-${ARCH}"
TMP_ROOT="$(mktemp -d -t ligero-linux-build.XXXXXX)"
cleanup() { rm -rf "$TMP_ROOT" || true; }

# If we get interrupted/terminated, stop child processes first, then clean up.
terminate() {
  local code="${1:-130}"
  set +e
  pkill -TERM -P "$$" 2>/dev/null || true
  wait 2>/dev/null || true
  cleanup
  exit "$code"
}

trap 'terminate 130' INT
trap 'terminate 143' TERM
trap cleanup EXIT

DAWN_GIT_REF="${DAWN_GIT_REF:-cec4482eccee45696a7c0019e750c77f101ced04}"
LIGERO_REPO="${LIGERO_REPO:-https://github.com/dcspark/ligero-prover.git}"
LIGERO_BRANCH="${LIGERO_BRANCH:-feature/ligero-runner}"

DEPOT_TOOLS_DIR="$TMP_ROOT/depot_tools"
DAWN_SRC="$TMP_ROOT/dawn"
WABT_SRC="$TMP_ROOT/wabt"
LIGERO_SRC="$TMP_ROOT/ligero-prover"

DAWN_BUILD_DIR="$TMP_ROOT/dawn-build"
WABT_BUILD_DIR="$TMP_ROOT/wabt-build"
LIGERO_BUILD_DIR="$TMP_ROOT/ligero-build"

rm -rf "$SYSROOT"
mkdir -p "$SYSROOT" "$DAWN_BUILD_DIR" "$WABT_BUILD_DIR" "$LIGERO_BUILD_DIR"

echo "==> [${ARCH}] Cloning depot_tools..."
git clone --depth 1 https://chromium.googlesource.com/chromium/tools/depot_tools.git "$DEPOT_TOOLS_DIR"
export PATH="$DEPOT_TOOLS_DIR:$PATH"

echo "==> [${ARCH}] Cloning Dawn..."
git clone https://dawn.googlesource.com/dawn "$DAWN_SRC"
cd "$DAWN_SRC"
git checkout "$DAWN_GIT_REF"

cp scripts/standalone.gclient .gclient
gclient sync

LLVM_BIN="$DAWN_SRC/third_party/llvm-build/Release+Asserts/bin"
if [[ ! -x "$LLVM_BIN/clang" || ! -x "$LLVM_BIN/clang++" ]]; then
  echo "error: expected bundled clang at '$LLVM_BIN' (did gclient sync succeed?)" >&2
  exit 1
fi

DAWN_CC="$LLVM_BIN/clang"
DAWN_CXX="$LLVM_BIN/clang++"

# On some hosts / emulation setups, gclient may fetch a Linux_x64 clang even when building in an arm64 container.
# If the bundled clang can't execute (e.g. "rosetta error" or missing x86_64 loader), fall back to system clang.
if ! "$DAWN_CC" --version >/dev/null 2>&1; then
  if command -v clang >/dev/null 2>&1 && command -v clang++ >/dev/null 2>&1; then
    echo "==> [${ARCH}] Bundled clang is not runnable; falling back to system clang/clang++"
    DAWN_CC="clang"
    DAWN_CXX="clang++"
  else
    echo "error: bundled clang is not runnable and system clang/clang++ not found" >&2
    exit 1
  fi
fi

echo "==> [${ARCH}] Configuring Dawn..."
cmake -S "$DAWN_SRC" -B "$DAWN_BUILD_DIR" -G Ninja \
  -DCMAKE_C_COMPILER="$DAWN_CC" \
  -DCMAKE_CXX_COMPILER="$DAWN_CXX" \
  -DCMAKE_BUILD_TYPE=Release \
  -DDAWN_FETCH_DEPENDENCIES=ON \
  -DDAWN_ENABLE_VULKAN=ON \
  -DDAWN_BUILD_MONOLITHIC_LIBRARY=STATIC \
  -DDAWN_BUILD_TESTS=OFF \
  -DDAWN_BUILD_SAMPLES=OFF \
  -DDAWN_BUILD_PROTOBUF=OFF \
  -DTINT_BUILD_TESTS=OFF \
  -DTINT_BUILD_CMD_TOOLS=OFF \
  -DTINT_BUILD_FUZZERS=OFF \
  -DTINT_BUILD_BENCHMARKS=OFF \
  -DTINT_BUILD_TINTD=OFF \
  -DTINT_BUILD_IR_BINARY=OFF \
  -DDAWN_ENABLE_INSTALL=ON \
  -DCMAKE_INSTALL_PREFIX="$SYSROOT"

echo "==> [${ARCH}] Building Dawn..."
ninja -C "$DAWN_BUILD_DIR" -j "$JOBS"

echo "==> [${ARCH}] Installing Dawn into sysroot..."
cmake --install "$DAWN_BUILD_DIR"

echo "==> [${ARCH}] Cloning wabt..."
git clone https://github.com/WebAssembly/wabt.git "$WABT_SRC"
cd "$WABT_SRC"
git submodule update --init

echo "==> [${ARCH}] Building wabt..."
cmake -S "$WABT_SRC" -B "$WABT_BUILD_DIR" -G Ninja \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_C_COMPILER=gcc-13 \
  -DCMAKE_CXX_COMPILER=g++-13 \
  -DCMAKE_INSTALL_PREFIX="$SYSROOT" \
  -DBUILD_TESTS=OFF
cmake --build "$WABT_BUILD_DIR" --parallel "$JOBS"
cmake --install "$WABT_BUILD_DIR"

echo "==> [${ARCH}] Building Dawn via depot_tools + gclient (this may take a while)..."

echo "==> [${ARCH}] Cloning ligero-prover..."
if [[ "$USE_LOCAL_LIGERO" == "1" ]]; then
  if [[ ! -d /ligero-local || ! -f /ligero-local/CMakeLists.txt ]]; then
    echo "error: expected local ligero-prover to be mounted at /ligero-local" >&2
    exit 1
  fi
  mkdir -p "$LIGERO_SRC"
  # Copy workspace repo into temp dir without VCS metadata.
  (cd /ligero-local && tar --exclude=.git -cf - .) | (cd "$LIGERO_SRC" && tar -xf -)
else
  git clone "$LIGERO_REPO" -b "$LIGERO_BRANCH" "$LIGERO_SRC"
fi

echo "==> [${ARCH}] Patching ligero-prover for wabt compatibility..."
TRANSPILER_HPP="$LIGERO_SRC/include/transpiler.hpp"
if [[ -f "$TRANSPILER_HPP" ]] && ! grep -q "transpile_wabt_type(const wabt::Var" "$TRANSPILER_HPP"; then
  # wabt newer API uses wabt::Var for ref.null type (Var::to_type()).
  perl -0777 -i -pe 's/\}\n\n\/\/ ------------------------------------------------------------/\}\n\n\/\/ Newer wabt represents ref-null types as `wabt::Var` (which may carry an optional type).\n\/\/ Provide an overload so we can support both wabt APIs without pinning a specific version.\nvalue_kind transpile_wabt_type(const wabt::Var& var) {\n    return transpile_wabt_type(var.to_type());\n}\n\n\/\/ ------------------------------------------------------------/s' "$TRANSPILER_HPP"
fi

echo "==> [${ARCH}] Building ligero-prover..."
cmake -S "$LIGERO_SRC" -B "$LIGERO_BUILD_DIR" -G Ninja \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_C_COMPILER=gcc-13 \
  -DCMAKE_CXX_COMPILER=g++-13 \
  -DCMAKE_PREFIX_PATH="$SYSROOT;$BOOST_INSTALL" \
  -DBOOST_ROOT="$BOOST_INSTALL" \
  -DBoost_NO_SYSTEM_PATHS=ON
cmake --build "$LIGERO_BUILD_DIR" --target webgpu_prover --parallel "$JOBS"
cmake --build "$LIGERO_BUILD_DIR" --target webgpu_verifier --parallel "$JOBS"

STAGE_ROOT="$OUT_DIR"
BIN_STAGE="$STAGE_ROOT/${ARCH}"
mkdir -p "$BIN_STAGE/bin" "$BIN_STAGE/lib"

install -m 0755 "$LIGERO_BUILD_DIR/webgpu_prover" "$BIN_STAGE/bin/webgpu_prover"
install -m 0755 "$LIGERO_BUILD_DIR/webgpu_verifier" "$BIN_STAGE/bin/webgpu_verifier"

set_rpath() {
  local f="$1"
  # Make binaries prefer bundled libs
  patchelf --set-rpath '$ORIGIN/../lib' "$f" || true
}

set_rpath "$BIN_STAGE/bin/webgpu_prover"
set_rpath "$BIN_STAGE/bin/webgpu_verifier"

should_skip_linux_lib() {
  local base="$1"
  case "$base" in
    linux-vdso.so.1|ld-linux-*.so.*|ld-linux.so.*) return 0 ;;
    libc.so.*|libm.so.*|libpthread.so.*|libdl.so.*|librt.so.*) return 0 ;;
    libanl.so.*|libresolv.so.*|libutil.so.*) return 0 ;;
    *) return 1 ;;
  esac
}

copy_needed_libs_linux() {
  local elf="$1"
  ldd "$elf" | awk '/=>/ {print $3}' | while read -r p; do
    [[ -n "$p" && -f "$p" ]] || continue
    local base
    base="$(basename "$p")"
    if should_skip_linux_lib "$base"; then
      continue
    fi
    if [[ ! -f "$BIN_STAGE/lib/$base" ]]; then
      cp -L "$p" "$BIN_STAGE/lib/$base"
      chmod 0644 "$BIN_STAGE/lib/$base" || true
    fi
  done
}

echo "==> [${ARCH}] Collecting shared libs..."

# Include Boost install path so ldd can resolve Boost libraries
export LD_LIBRARY_PATH="$BOOST_INSTALL/lib:${LD_LIBRARY_PATH:-}"

copy_needed_libs_linux "$BIN_STAGE/bin/webgpu_prover"
copy_needed_libs_linux "$BIN_STAGE/bin/webgpu_verifier"

# Explicitly copy Boost libraries from our source build
echo "==> [${ARCH}] Copying Boost libraries from $BOOST_INSTALL/lib..."
for lib in "$BOOST_INSTALL/lib/"libboost_*.so*; do
  [[ -f "$lib" ]] || continue
  base="$(basename "$lib")"
  if [[ ! -f "$BIN_STAGE/lib/$base" ]]; then
    cp -L "$lib" "$BIN_STAGE/lib/$base"
    chmod 0644 "$BIN_STAGE/lib/$base" || true
  fi
done

# Also bring Dawn-provided shared libs from sysroot if any exist (best-effort).
if [[ -d "$SYSROOT/lib" ]]; then
  find "$SYSROOT/lib" -maxdepth 1 -type f -name "*.so*" -print0 2>/dev/null | while IFS= read -r -d '' lib; do
    base="$(basename "$lib")"
    if [[ ! -f "$BIN_STAGE/lib/$base" ]]; then
      cp -L "$lib" "$BIN_STAGE/lib/$base"
      chmod 0644 "$BIN_STAGE/lib/$base" || true
    fi
  done
fi

# Second pass: some copied libs bring new deps. Iterate a few times.
for _ in 1 2 3; do
  for f in "$BIN_STAGE/lib/"*.so*; do
    [[ -f "$f" ]] || continue
    copy_needed_libs_linux "$f" || true
  done
done

# Ensure bundled libs themselves can find their deps via RPATH if needed.
for f in "$BIN_STAGE/lib/"*.so*; do
  [[ -f "$f" ]] || continue
  patchelf --set-rpath '$ORIGIN' "$f" || true
done

echo "==> [${ARCH}] Stripping binaries (best-effort)..."
if command -v strip >/dev/null 2>&1; then
  strip --strip-unneeded "$BIN_STAGE/bin/webgpu_prover" || true
  strip --strip-unneeded "$BIN_STAGE/bin/webgpu_verifier" || true
fi

echo "==> [${ARCH}] Done staging to $BIN_STAGE"

if [[ "$NO_TAR" == "false" ]]; then
  echo "==> [${ARCH}] Staging shader folder..."
  mkdir -p "$STAGE_ROOT/shader"
  cp -R "$LIGERO_SRC/shader/." "$STAGE_ROOT/shader/"

  TARBALL="$OUT_DIR/ligero-${ARCH}.tar.gz"
  rm -f "$TARBALL"
  echo "==> [${ARCH}] Creating tarball: $TARBALL"
  (cd "$OUT_DIR" && tar -czf "$(basename "$TARBALL")" "${ARCH}" "shader")
fi

chown -R "$HOST_UID:$HOST_GID" "$STAGE_ROOT" || true

echo "==> [${ARCH}] Done."
echo "    Folder: $STAGE_ROOT"
if [[ "$NO_TAR" == "false" ]]; then
  echo "    Tarball: $TARBALL"
fi


