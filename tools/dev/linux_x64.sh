#!/usr/bin/env bash
# Builds and runs jaic on Linux x86-64 inside an Apple `container` VM, with no network.
# See docs/tools/linux-x64-container.md.
#
#   tools/dev/linux_x64.sh build [cargo args...]
#   tools/dev/linux_x64.sh run [-m HOST_PATH]... [--] JAIC_ARGS...
#   tools/dev/linux_x64.sh shell
#
# Environment:
#   LINUX_X64_SCRATCH      folder holding target/ and cargo/ (default /Volumes/LodestoneScratch/jai-targets/linux-x64)
#   LINUX_X64_IMAGE        image (default docker.io/library/rust:1-bookworm)
#   JAIC_OFFICIAL_MODULES  host folder of a module set used as JAIC_STDLIB (mounted read-only at /modules)
#   LINUX_X64_NETWORK=1    lets `build` use the network (only to fetch crates the cache lacks)
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
scratch="${LINUX_X64_SCRATCH:-/Volumes/LodestoneScratch/jai-targets/linux-x64}"
image="${LINUX_X64_IMAGE:-docker.io/library/rust:1-bookworm}"

usage() {
  sed -n '2,13p' "$0" | sed 's/^# \{0,1\}//'
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
  --env PATH=/cargo/bin:/usr/local/bin:/usr/bin:/bin
  --env CARGO_TARGET_DIR=/target
)
network=(--network none)
offline=(--offline)
if [ "${LINUX_X64_NETWORK:-0}" = 1 ]; then
  network=()
  offline=()
fi

case "$cmd" in
  build)
    # No LLVM in the container: -exe is unavailable, -check and -interp work.
    exec container run --rm --arch amd64 "${network[@]}" "${mounts[@]}" "${env_args[@]}" \
      --workdir /repo "$image" \
      cargo build "${offline[@]}" --no-default-features -p jaic-cli "$@"
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
      exec container run --rm -it --arch amd64 --network none "${mounts[@]}" "${extra[@]}" \
        "${env_args[@]}" "${jaic_env[@]}" --workdir /repo "$image" bash
    fi
    exec container run --rm --arch amd64 --network none "${mounts[@]}" "${extra[@]}" \
      "${env_args[@]}" "${jaic_env[@]}" --workdir /repo "$image" /target/debug/jaic "$@"
    ;;
  *) usage ;;
esac
