#!/usr/bin/env bash
# Installs LLVM 23 (and LLD for wasm-ld) on a CI macOS runner and exports LLVM_SYS_231_PREFIX
# and PATH for later steps.
#
# arm64: Homebrew's llvm@23. Some runner images ship a Homebrew whose formula list predates the
# llvm@23 alias, so a failed install retries after `brew update`.
#
# x86-64: Homebrew has no bottles for Intel macOS (it is a tier 3 configuration there), so it
# would build LLVM from source, which takes longer than the job's timeout. conda-forge still
# builds LLVM for osx-64: the pinned micromamba below installs the same 23.1 release into the
# runner's temp directory.
# Usage: tools/install_ci_llvm_macos.sh [--lld]
set -euo pipefail
lld=false
[[ "${1:-}" == "--lld" ]] && lld=true

if [[ "$(uname -m)" == x86_64 ]]; then
    llvm_version=23.1.2
    micromamba_package=micromamba-2.9.0-0.tar.bz2
    micromamba_sha256=0426ecdc41636d369f57b8fe6acbf4385a69eca45b56d9ee7d3a840a9965d44f
    tools="${RUNNER_TEMP:-/tmp}/micromamba"
    prefix="${RUNNER_TEMP:-/tmp}/llvm-$llvm_version"
    mkdir -p "$tools"
    curl --fail --show-error --silent --location --proto '=https' --tlsv1.2 \
        "https://conda.anaconda.org/conda-forge/osx-64/$micromamba_package" -o "$tools/$micromamba_package"
    echo "$micromamba_sha256  $tools/$micromamba_package" | shasum -a 256 -c -
    tar -xjf "$tools/$micromamba_package" -C "$tools" bin/micromamba
    # llvmdev: llvm-config, headers and the shared libLLVM that llvm-sys links; llvm-tools:
    # llvm-dwarfdump and llvm-symbolizer; clang and compiler-rt: the driver and runtimes for
    # `jaic build -sanitize`.
    packages=("llvmdev=$llvm_version" "libllvm23=$llvm_version" "llvm-tools=$llvm_version"
              "clang=$llvm_version" "compiler-rt=$llvm_version")
    $lld && packages+=("lld=$llvm_version")
    MAMBA_ROOT_PREFIX="$tools/root" "$tools/bin/micromamba" create --yes --quiet \
        --prefix "$prefix" --override-channels --channel conda-forge "${packages[@]}"
    # conda's libraries are found through @rpath: give every binary cargo links an rpath to
    # them (jaic loads libLLVM at start).
    echo "RUSTFLAGS=-C link-arg=-Wl,-rpath,$prefix/lib" >> "$GITHUB_ENV"
    # conda's clang has no default SDK; point it at the Command Line Tools' one.
    echo "SDKROOT=$(xcrun --show-sdk-path)" >> "$GITHUB_ENV"
    # The environment's bin also holds conda's own `cc`, `ld`, `ar`, `nm` and other cctools,
    # which must not shadow the system's: jaic links with `cc`, and with conda's linker the
    # dSYM of a split-codegen build held only one of its objects' debug information. Only
    # LLVM's own tools go on PATH.
    path_dir="${RUNNER_TEMP:-/tmp}/llvm-bin"
    mkdir -p "$path_dir"
    for tool in "$prefix"/bin/*; do
        case "$(basename "$tool")" in
            llvm-* | clang | clang-[0-9]* | clang++ | dsymutil | wasm-ld | ld.lld | ld64.lld | lld | opt | llc)
                ln -sf "$tool" "$path_dir/" ;;
        esac
    done
else
    formulae=(llvm@23)
    $lld && formulae+=(lld)
    if ! brew install "${formulae[@]}"; then
        brew update
        brew install "${formulae[@]}"
    fi
    prefix="$(brew --prefix llvm@23)"
    path_dir="$prefix/bin"
fi

version="$("$prefix/bin/llvm-config" --version)"
[[ "$version" == 23.* ]] || { echo "expected LLVM 23, got $version" >&2; exit 1; }
echo "LLVM_SYS_231_PREFIX=$prefix" >> "$GITHUB_ENV"
echo "$path_dir" >> "$GITHUB_PATH"
