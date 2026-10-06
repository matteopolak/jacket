#!/usr/bin/env bash
# Installs Homebrew LLVM 23 (and LLD for wasm-ld) on a CI macOS runner and exports
# LLVM_SYS_231_PREFIX and PATH for later steps. Some runner images ship a Homebrew whose
# formula list predates the llvm@23 alias, so a failed install retries after `brew update`.
# Usage: tools/install_ci_llvm_macos.sh [--lld]
set -euo pipefail
formulae=(llvm@23)
[[ "${1:-}" == "--lld" ]] && formulae+=(lld)
if ! brew install "${formulae[@]}"; then
    brew update
    brew install "${formulae[@]}"
fi
prefix="$(brew --prefix llvm@23)"
version="$("$prefix/bin/llvm-config" --version)"
[[ "$version" == 23.* ]] || { echo "expected LLVM 23, got $version" >&2; exit 1; }
echo "LLVM_SYS_231_PREFIX=$prefix" >> "$GITHUB_ENV"
echo "$prefix/bin" >> "$GITHUB_PATH"
