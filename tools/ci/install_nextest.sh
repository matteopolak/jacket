#!/usr/bin/env bash
# Install the cargo-nextest release pinned in .github/pins.json (checksum-verified prebuilt binary) into
# ~/.cargo/bin, which is on PATH wherever rustup is. Linux x86-64/arm64 and macOS only; the Windows jobs
# use `cargo test`.
#
#   install_nextest.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
pin() { python3 -c "import json,sys; print(json.load(open('.github/pins.json'))[sys.argv[1]])" "$1"; }
version=$(pin nextest)
case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) platform=linux; key=nextest_sha256_linux_x64 ;;
  Linux-aarch64 | Linux-arm64) platform=linux-arm; key=nextest_sha256_linux_arm64 ;;
  Darwin-*) platform=mac; key=nextest_sha256_macos ;;
  *) echo "no prebuilt cargo-nextest for $(uname -sm); use cargo install --locked cargo-nextest@$version" >&2; exit 1 ;;
esac
dest="${CARGO_HOME:-$HOME/.cargo}/bin"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
curl -fsSL --retry 3 -o "$tmp/nextest.tgz" "https://get.nexte.st/$version/$platform"
actual=$( (sha256sum "$tmp/nextest.tgz" 2>/dev/null || shasum -a 256 "$tmp/nextest.tgz") | cut -d' ' -f1)
if [[ "$actual" != "$(pin "$key")" ]]; then
  echo "cargo-nextest $version ($platform): checksum $actual does not match $key in .github/pins.json" >&2
  exit 1
fi
mkdir -p "$dest"
tar -xzf "$tmp/nextest.tgz" -C "$dest"
"$dest/cargo-nextest" nextest --version | head -1
