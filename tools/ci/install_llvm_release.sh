#!/usr/bin/env bash
# Download (cached by archive name under $RUNNER_TEMP/llvm-download), unpack and register the official
# LLVM release archive for ci.yml's Windows setup and release.yml's static builds.
#
#   LLVM_VERSION=23.1.2 install_llvm_release.sh LLVM-23.1.2-Linux-X64.tar.xz
#
# Unpacks into $RUNNER_TEMP/llvm and appends LLVM_SYS_231_PREFIX to $GITHUB_ENV. On Windows the
# extraction uses the POSIX spelling of the paths (GNU tar reads `D:\...` as a remote host), and the
# prefix is written as `D:/...`, which Cargo and llvm-config accept.
set -euo pipefail
if [[ $# != 1 || -z "${LLVM_VERSION:-}" ]]; then
  echo "usage: LLVM_VERSION=x.y.z $0 ARCHIVE" >&2
  exit 2
fi
archive=$1
temp=$RUNNER_TEMP
prefix_of() { echo "$1"; }
if command -v cygpath > /dev/null; then
  temp=$(cygpath -u "$RUNNER_TEMP")
  prefix_of() { cygpath -m "$1"; }
fi
download=$temp/llvm-download
llvm=$temp/llvm
if [[ ! -f "$download/$archive" ]]; then
  mkdir -p "$download"
  curl -fsSL -o "$download/$archive" "https://github.com/llvm/llvm-project/releases/download/llvmorg-$LLVM_VERSION/$archive"
fi
mkdir -p "$llvm"
tar -xJf "$download/$archive" -C "$llvm" --strip-components=1
echo "LLVM_SYS_231_PREFIX=$(prefix_of "$llvm")" >> "$GITHUB_ENV"
"$llvm/bin/llvm-config" --version
echo "system libraries: $("$llvm/bin/llvm-config" --link-static --system-libs)"
