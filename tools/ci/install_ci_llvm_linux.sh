#!/usr/bin/env bash
# Install the signed apt.llvm.org packages of the LLVM major version .github/pins.json pins on the
# clean Ubuntu 24.04 hosted CI runner. Exports (to $GITHUB_ENV and $GITHUB_PATH, when set) the
# llvm-sys prefix variable, LLVM_PREFIX and the toolchain's bin directory.
set -euo pipefail

# shellcheck source=/dev/null
source /etc/os-release
if [[ "$ID" != ubuntu || "$VERSION_ID" != 24.04 ]]; then
  echo 'This setup supports only Ubuntu 24.04.' >&2
  exit 1
fi
case "$(uname -m)" in
  x86_64) package_arch=amd64 ;;
  aarch64) package_arch=arm64 ;;
  *) echo 'This setup supports only x86_64 and ARM64 hosts.' >&2; exit 1 ;;
esac

# shellcheck source=tools/ci/llvm_env.sh
source "$(dirname "${BASH_SOURCE[0]}")/llvm_env.sh"
llvm_prefix="/usr/lib/llvm-$LLVM_MAJOR"
# shellcheck source=tools/ci/apt_lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/apt_lib.sh"
apt_configure /etc/apt

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
  "deb [arch=$package_arch signed-by=/usr/share/keyrings/jai-ci-llvm.gpg] https://apt.llvm.org/noble/ llvm-toolchain-noble-$LLVM_MAJOR main" \
  | sudo tee /etc/apt/jai-ci-sources.list.d/jai-ci-llvm.list > /dev/null
apt_update
# libclang-rt-N-dev: the sanitizer runtimes `jaic -sanitize` links; lld-N: wasm-ld, for
# `jaic -os wasm`.
m=$LLVM_MAJOR
packages=("llvm-$m" "llvm-$m-tools" "llvm-$m-dev" "clang-$m" "libclang-$m-dev" "libpolly-$m-dev" "libclang-rt-$m-dev" "lld-$m")
apt_prefetch "${packages[@]}"
apt_get install --yes --no-install-recommends "${packages[@]}"
# The release line llvm-sys was built for (the pin's major and minor).
case "$("$llvm_prefix/bin/llvm-config" --version)" in
  "${LLVM_VERSION%.*}".*) ;;
  *) echo "Expected LLVM ${LLVM_VERSION%.*} for $LLVM_SYS_PREFIX_VAR." >&2; exit 1 ;;
esac

# Cross-resource fixtures use the actual versioned tools, not optional PATH aliases.
for tool in llvm-rc llvm-readobj lld-link; do
  test -x "$llvm_prefix/bin/$tool"
done

if [[ -n "${GITHUB_ENV:-}" ]]; then
  echo "$LLVM_SYS_PREFIX_VAR=$llvm_prefix" >> "$GITHUB_ENV"
  echo "LLVM_PREFIX=$llvm_prefix" >> "$GITHUB_ENV"
  echo "$llvm_prefix/bin" >> "$GITHUB_PATH"
fi
