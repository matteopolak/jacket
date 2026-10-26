#!/usr/bin/env bash
# macOS arm64 release build: link the official LLVM against Homebrew's static zstd and with its own lld.
# Appends SDKROOT and the cargo linker variables to GITHUB_ENV; needs RUNNER_TEMP/llvm in place.
set -e
brew install zstd
ls -l /opt/homebrew/lib/libzstd.a
# llvm-sys turns that absolute path into `-L/opt/homebrew/lib -lzstd`, for which ld
# prefers the dylib beside the archive (and jaic would then need Homebrew's zstd at run
# time). Without the dylib links in that folder, the archive is linked.
rm -f /opt/homebrew/lib/libzstd*.dylib
ls "$RUNNER_TEMP/llvm/bin/ld64.lld"
find "$RUNNER_TEMP/llvm/lib" \( -name 'libc++*' -o -name 'libunwind*' \) -print -delete
{
  echo "SDKROOT=$(xcrun --show-sdk-path)"
  echo "CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER=$RUNNER_TEMP/llvm/bin/clang"
  echo "CARGO_TARGET_AARCH64_APPLE_DARWIN_RUSTFLAGS=-C link-arg=-fuse-ld=lld"
} >> "$GITHUB_ENV"
