#!/usr/bin/env bash
# Builds, tests and runs jaic on Linux (x86-64 or arm64) inside an Apple `container` VM, with no
# network. See docs/tools/linux-container.md.
#
#   tools/dev/linux_x64.sh build [cargo args...]
#   tools/dev/linux_x64.sh test [cargo test args...]
#   tools/dev/linux_x64.sh run [-m HOST_PATH]... [--] JAIC_ARGS...
#   tools/dev/linux_x64.sh shell
#
# Environment:
#   LINUX_ARCH             amd64 (default; also x64, x86_64) or arm64 (aarch64): the VM's architecture
#   LINUX_X64_SCRATCH      folder holding target/ and cargo/ for amd64 (default /Volumes/LodestoneScratch/jai-targets/linux-x64)
#   LINUX_ARM64_SCRATCH    the same for arm64 (default /Volumes/LodestoneScratch/jai-targets/linux-arm64)
#   LINUX_X64_IMAGE        image (default docker.io/library/rust:1-bookworm)
#   LINUX_MEMORY, LINUX_CPUS  VM size (default 8G and 4; rustc is killed in a smaller VM)
#   JAIC_OFFICIAL_MODULES  host folder of a module set used as JAIC_STDLIB (mounted read-only at /modules)
#   LINUX_X64_NETWORK=1    lets `build` and `test` use the network (only to fetch crates the cache lacks)
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
case "${LINUX_ARCH:-amd64}" in
  amd64 | x64 | x86_64 | x86-64)
    arch=amd64
    scratch="${LINUX_X64_SCRATCH:-/Volumes/LodestoneScratch/jai-targets/linux-x64}"
    ;;
  arm64 | aarch64)
    arch=arm64
    scratch="${LINUX_ARM64_SCRATCH:-/Volumes/LodestoneScratch/jai-targets/linux-arm64}"
    ;;
  *)
    echo "LINUX_ARCH must be amd64 or arm64" >&2
    exit 2
    ;;
esac
memory="${LINUX_MEMORY:-8G}"
cpus="${LINUX_CPUS:-4}"
image="${LINUX_X64_IMAGE:-docker.io/library/rust:1-bookworm}"

usage() {
  sed -n '2,19p' "$0" | sed 's/^# \{0,1\}//'
  exit "${1:-2}"
}

[ $# -ge 1 ] || usage
cmd="$1"
shift

if ! container system status >/dev/null 2>&1; then
  container system start
fi
mkdir -p "$scratch/target" "$scratch/cargo"

mounts=(
  --volume "$repo:/repo:ro"
  --volume "$scratch/target:/target"
  --volume "$scratch/cargo:/cargo"
)
env_args=(
  --env CARGO_HOME=/cargo
  --env RUSTUP_HOME=/cargo/rustup
  --env PATH=/cargo/bin:/usr/local/cargo/bin:/usr/local/bin:/usr/bin:/bin
  --env CARGO_TARGET_DIR=/target
)
vm=(--arch "$arch" --memory "$memory" --cpus "$cpus")
network=(--network none)
offline=(--offline)
if [ "${LINUX_X64_NETWORK:-0}" = 1 ]; then
  network=()
  offline=()
fi

case "$cmd" in
  build)
    # No LLVM in the container: -exe is unavailable, -check and -interp work.
    exec container run --rm "${vm[@]}" ${network[@]+"${network[@]}"} "${mounts[@]}" "${env_args[@]}" \
      --workdir /repo "$image" \
      cargo build ${offline[@]+"${offline[@]}"} --no-default-features -p jaic-cli "$@"
    ;;
  test)
    exec container run --rm "${vm[@]}" ${network[@]+"${network[@]}"} "${mounts[@]}" "${env_args[@]}" \
      --workdir /repo "$image" \
      cargo test ${offline[@]+"${offline[@]}"} --no-default-features "$@"
    ;;
  run | shell)
    extra=()
    jaic_env=(--env "JAIC_STDLIB=/repo/stdlib")
    if [ -n "${JAIC_OFFICIAL_MODULES:-}" ]; then
      extra+=(--volume "$JAIC_OFFICIAL_MODULES:/modules:ro")
      jaic_env=(--env "JAIC_STDLIB=/modules")
    fi
    while [ $# -gt 0 ]; do
      case "$1" in
        -m | --mount)
          extra+=(--volume "$2:$2:ro")
          shift 2
          ;;
        --)
          shift
          break
          ;;
        *) break ;;
      esac
    done
    if [ "$cmd" = shell ]; then
      exec container run --rm -it "${vm[@]}" --network none "${mounts[@]}" ${extra[@]+"${extra[@]}"} \
        "${env_args[@]}" "${jaic_env[@]}" --workdir /repo "$image" bash
    fi
    exec container run --rm "${vm[@]}" --network none "${mounts[@]}" ${extra[@]+"${extra[@]}"} \
      "${env_args[@]}" "${jaic_env[@]}" --workdir /repo "$image" /target/debug/jaic "$@"
    ;;
  *) usage ;;
esac
