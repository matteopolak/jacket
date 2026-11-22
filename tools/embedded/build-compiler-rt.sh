#!/usr/bin/env bash
# Build compiler-rt's builtins for a freestanding target (64-bit division, soft float, atomics).
#
#   tools/embedded/build-compiler-rt.sh [TRIPLE] [OUT_DIR]
#
# TRIPLE defaults to aarch64-none-elf. The archive lands in OUT_DIR/TRIPLE/libclang_rt.builtins.a
# (OUT_DIR defaults to target/compiler-rt); the last line printed is the `JAIC_COMPILER_RT=` setting
# that makes jaic link it. OUT_DIR itself also works as JAIC_COMPILER_RT, for every triple built
# into it. Copying the file to `<llvm>/lib/clang/<version>/lib/TRIPLE/` works too.
#
# Reproducible: the sources are the pinned LLVM_TAG, cloned shallowly, built with the one
# compiler-rt configuration below. Environment:
#   LLVM_TAG   release tag to build (default below)
#   LLVM_SRC   an existing llvm-project checkout to build instead of cloning (offline builds)
#   CLANG      the C compiler (default: clang), LLVM_AR the archiver (default: llvm-ar)
set -euo pipefail

LLVM_TAG="${LLVM_TAG:-llvmorg-21.1.8}"
TRIPLE="${1:-aarch64-none-elf}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT="${2:-$ROOT/target/compiler-rt}"
CLANG="${CLANG:-clang}"
LLVM_AR="${LLVM_AR:-llvm-ar}"

need() {
    command -v "$1" >/dev/null 2>&1 || {
        echo "error: '$1' is not installed. $2" >&2
        exit 1
    }
}

need "$CLANG" "Install LLVM: 'brew install llvm' (then put \$(brew --prefix llvm)/bin on PATH), or 'apt install clang'."
need "$LLVM_AR" "It ships with LLVM: 'brew install llvm', or 'apt install llvm'; or set LLVM_AR."
need cmake "Install it: 'brew install cmake' or 'apt install cmake'."
need ninja "Install it: 'brew install ninja' or 'apt install ninja-build'."
[ -n "${LLVM_SRC:-}" ] || need git "Install git, or point LLVM_SRC at an llvm-project checkout."

# A Cortex-M target is configured as compiler-rt's soft-float `armv7em` or `armv8m.main` (it does not
# know `thumbv...`) and then compiled for the core with the hard-float ABI of the program. Not as
# `armhf`: that builds the floating-point helpers with the VFP calling convention, while LLVM
# calls them (`__muldf3`) with the base one, so the soft-float variant is the one that matches.
CONFIG_TRIPLE="$TRIPLE"
ARCH_FLAGS=""
case "$TRIPLE" in
    thumbv7em-none-eabihf)
        CONFIG_TRIPLE=armv7em-none-eabi
        ARCH_FLAGS="-mcpu=cortex-m4 -mfloat-abi=soft"
        ;;
    thumbv8m.main-none-eabihf)
        CONFIG_TRIPLE=armv8m.main-none-eabi
        ARCH_FLAGS="-mcpu=cortex-m33 -mfloat-abi=soft"
        ;;
esac

WORK="$OUT/build-$TRIPLE"
SRC="${LLVM_SRC:-$OUT/llvm-project-$LLVM_TAG}"

if [ -z "${LLVM_SRC:-}" ] && [ ! -d "$SRC/compiler-rt" ]; then
    mkdir -p "$OUT"
    # Only compiler-rt and the CMake modules it includes: the full tree is over a gigabyte.
    git clone --quiet --depth 1 --branch "$LLVM_TAG" --filter=blob:none --sparse \
        https://github.com/llvm/llvm-project "$SRC"
    git -C "$SRC" sparse-checkout set compiler-rt cmake third-party llvm/cmake
fi

rm -rf "$WORK"
cmake -S "$SRC/compiler-rt" -B "$WORK" -G Ninja \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_SYSTEM_NAME=Generic \
    -DCMAKE_TRY_COMPILE_TARGET_TYPE=STATIC_LIBRARY \
    -DCMAKE_C_COMPILER="$CLANG" \
    -DCMAKE_C_COMPILER_TARGET="$CONFIG_TRIPLE" \
    -DCMAKE_ASM_COMPILER="$CLANG" \
    -DCMAKE_ASM_COMPILER_TARGET="$CONFIG_TRIPLE" \
    -DCMAKE_ASM_FLAGS="$ARCH_FLAGS" \
    -DCMAKE_AR="$(command -v "$LLVM_AR")" \
    -DCMAKE_C_FLAGS="-ffreestanding $ARCH_FLAGS" \
    -DCOMPILER_RT_BAREMETAL_BUILD=ON \
    -DCOMPILER_RT_DEFAULT_TARGET_ONLY=ON \
    -DCOMPILER_RT_BUILD_BUILTINS=ON \
    -DCOMPILER_RT_BUILD_SANITIZERS=OFF \
    -DCOMPILER_RT_BUILD_XRAY=OFF \
    -DCOMPILER_RT_BUILD_LIBFUZZER=OFF \
    -DCOMPILER_RT_BUILD_PROFILE=OFF \
    -DCOMPILER_RT_BUILD_MEMPROF=OFF \
    -DCOMPILER_RT_BUILD_ORC=OFF \
    -DCOMPILER_RT_BUILD_CTX_PROFILE=OFF \
    -DCOMPILER_RT_BUILD_GWP_ASAN=OFF \
    -DCOMPILER_RT_INCLUDE_TESTS=OFF \
    -DLLVM_CMAKE_DIR="$SRC/llvm/cmake/modules"
cmake --build "$WORK" --target builtins

ARCHIVE="$(find "$WORK" -name 'libclang_rt.builtins*.a' | head -n 1)"
[ -n "$ARCHIVE" ] || {
    echo "error: the build produced no libclang_rt.builtins archive under $WORK" >&2
    exit 1
}

mkdir -p "$OUT/$TRIPLE"
cp "$ARCHIVE" "$OUT/$TRIPLE/libclang_rt.builtins.a"
echo "built $OUT/$TRIPLE/libclang_rt.builtins.a"
echo "export JAIC_COMPILER_RT=$OUT"
