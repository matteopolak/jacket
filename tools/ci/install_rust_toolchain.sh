#!/usr/bin/env bash
# Install the Rust toolchain that rust-toolchain.toml pins (minimal profile), so the channel is written
# in one place and every workflow gets it from there.
#
#   install_rust_toolchain.sh [--component NAME]... [--target TRIPLE]...
#
# Run from the checkout (rust-toolchain.toml is read from the working directory).
set -euo pipefail
channel=$(sed -n 's/^channel *= *"\(.*\)"/\1/p' rust-toolchain.toml)
if [[ -z "$channel" ]]; then
  echo "no channel in rust-toolchain.toml" >&2
  exit 1
fi
components=() targets=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --component) components+=(--component "$2"); shift 2 ;;
    --target) targets+=("$2"); shift 2 ;;
    *) echo "usage: $0 [--component NAME]... [--target TRIPLE]..." >&2; exit 2 ;;
  esac
done
rustup toolchain install "$channel" --profile minimal ${components[@]+"${components[@]}"}
for target in ${targets[@]+"${targets[@]}"}; do
  rustup target add "$target" --toolchain "$channel"
done
