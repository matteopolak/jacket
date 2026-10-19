#!/usr/bin/env bash
# Install the cargo-llvm-cov release pinned in .github/pins.json (checksum-verified prebuilt binary) into
# ~/.cargo/bin. Linux x86-64/arm64 and macOS (the universal build) only.
#
#   install_llvm_cov.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
pin() { python3 -c "import json,sys; print(json.load(open('.github/pins.json'))[sys.argv[1]])" "$1"; }
version=$(pin llvm_cov)
case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) target=x86_64-unknown-linux-gnu; key=llvm_cov_sha256_linux_x64 ;;
  Linux-aarch64 | Linux-arm64) target=aarch64-unknown-linux-gnu; key=llvm_cov_sha256_linux_arm64 ;;
  Darwin-*) target=universal-apple-darwin; key=llvm_cov_sha256_macos ;;
  *) echo "no prebuilt cargo-llvm-cov for $(uname -sm); use cargo install --locked cargo-llvm-cov@$version" >&2; exit 1 ;;
esac
dest="${CARGO_HOME:-$HOME/.cargo}/bin"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
curl -fsSL --retry 3 -o "$tmp/llvm-cov.tgz" "https://github.com/taiki-e/cargo-llvm-cov/releases/download/v$version/cargo-llvm-cov-$target.tar.gz"
actual=$( (sha256sum "$tmp/llvm-cov.tgz" 2>/dev/null || shasum -a 256 "$tmp/llvm-cov.tgz") | cut -d' ' -f1)
if [[ "$actual" != "$(pin "$key")" ]]; then
  echo "cargo-llvm-cov $version ($target): checksum $actual does not match $key in .github/pins.json" >&2
  exit 1
fi
mkdir -p "$dest"
tar -xzf "$tmp/llvm-cov.tgz" -C "$dest"
"$dest/cargo-llvm-cov" llvm-cov --version | head -1
