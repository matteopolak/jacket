#!/usr/bin/env bash
# Install the cargo-mutants release pinned in .github/pins.json (checksum-verified prebuilt binary) into
# ~/.cargo/bin. Linux x86-64 only; elsewhere use `cargo install --locked cargo-mutants@<version>`.
#
#   install_cargo_mutants.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
pin() { python3 -c "import json,sys; print(json.load(open('.github/pins.json'))[sys.argv[1]])" "$1"; }
version=$(pin cargo_mutants)
case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) target=x86_64-unknown-linux-gnu; key=cargo_mutants_sha256_linux_x64 ;;
  *) echo "no prebuilt cargo-mutants pinned for $(uname -sm); use cargo install --locked cargo-mutants@$version" >&2; exit 1 ;;
esac
dest="${CARGO_HOME:-$HOME/.cargo}/bin"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
curl -fsSL --retry 3 -o "$tmp/mutants.tgz" "https://github.com/sourcefrog/cargo-mutants/releases/download/v$version/cargo-mutants-$target.tar.gz"
actual=$( (sha256sum "$tmp/mutants.tgz" 2>/dev/null || shasum -a 256 "$tmp/mutants.tgz") | cut -d' ' -f1)
if [[ "$actual" != "$(pin "$key")" ]]; then
  echo "cargo-mutants $version ($target): checksum $actual does not match $key in .github/pins.json" >&2
  exit 1
fi
mkdir -p "$dest"
tar -xzf "$tmp/mutants.tgz" -C "$dest"
"$dest/cargo-mutants" mutants --version
