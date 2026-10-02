#!/usr/bin/env bash
# Install signed LLVM packages on the clean Ubuntu 24.04 hosted CI runner.
set -euo pipefail

source /etc/os-release
if [[ "$ID" != ubuntu || "$VERSION_ID" != 24.04 || "$(uname -m)" != x86_64 ]]; then
  echo 'This setup supports only Ubuntu 24.04 x86_64.' >&2
  exit 1
fi

task_temp=$(mktemp -d)
trap 'rm -rf "$task_temp"' EXIT
curl --fail --show-error --silent --location --proto '=https' --tlsv1.2 \
  https://apt.llvm.org/llvm-snapshot.gpg.key -o "$task_temp/key.asc"
fingerprints=$(gpg --batch --show-keys --with-colons "$task_temp/key.asc" \
  | awk -F: '$1 == "pub" { primary=1; next } primary && $1 == "fpr" { print $10; primary=0 }')
if [[ "$fingerprints" != 6084F3CF814B57C1CF12EFD515CF4D18AF4F7421 ]]; then
  echo 'Unexpected LLVM archive signing key; review upstream key rotation.' >&2
  exit 1
fi
gpg --batch --dearmor --output "$task_temp/llvm.gpg" "$task_temp/key.asc"
sudo install -m 0644 "$task_temp/llvm.gpg" /usr/share/keyrings/jai-ci-llvm.gpg
printf '%s\n' \
  'deb [arch=amd64 signed-by=/usr/share/keyrings/jai-ci-llvm.gpg] https://apt.llvm.org/noble/ llvm-toolchain-noble-22 main' \
  | sudo tee /etc/apt/sources.list.d/jai-ci-llvm.list > /dev/null
sudo apt-get update
sudo apt-get install --yes --no-install-recommends llvm-22-dev clang-22 libpolly-22-dev
case "$(/usr/lib/llvm-22/bin/llvm-config --version)" in
  22.1.*) ;;
  *) echo 'Expected LLVM 22.1 for llvm-sys 221.' >&2; exit 1 ;;
esac
