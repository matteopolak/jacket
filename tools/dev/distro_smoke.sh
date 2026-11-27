#!/usr/bin/env bash
# Installs a jacket release in a clean container per Linux distribution and runs the everyday
# commands (run, build, new, test, fmt, lint, toolchain), with Apple `container`. See
# docs/tools/distro-smoke.md.
#
#   tools/dev/distro_smoke.sh [DISTRO...]          every distro, or the ones named
#   tools/dev/distro_smoke.sh --list               the distro names
#
# Environment:
#   JAI_VERSION      the release to install (default: the latest)
#   JAI_BASE_URL     where install.sh fetches it from, as <base>/v<version>/<asset>; a file:// URL
#                    of a host folder is mounted into the container (needs JAI_VERSION)
#   SMOKE_ARCH       arm64 (default) or amd64; amd64 runs under Rosetta, whose process spawning
#                    differs from a real x86-64 machine
#   SMOKE_MEMORY, SMOKE_CPUS   container size (default 4G and 4)
#   SMOKE_KEEP=1     keep each container's log in target/distro-smoke
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# name|image|command that installs a C compiler, git and curl
distros=(
  "ubuntu|docker.io/library/ubuntu:24.04|apt-get update -qq && apt-get install -y -qq gcc git curl ca-certificates"
  "debian|docker.io/library/debian:stable|apt-get update -qq && apt-get install -y -qq gcc git curl ca-certificates"
  "fedora|registry.fedoraproject.org/fedora:latest|dnf -y -q install gcc git"
  "opensuse|registry.opensuse.org/opensuse/tumbleweed:latest|zypper -n -q install gcc git curl"
  "arch|ARCH_IMAGE|pacman -Sy --noconfirm --needed --disable-sandbox gcc git"
)

arch="${SMOKE_ARCH:-arm64}"
case "$arch" in
  arm64 | aarch64) arch=arm64; arch_image=docker.io/menci/archlinuxarm:latest ;;
  amd64 | x64 | x86_64) arch=amd64; arch_image=docker.io/library/archlinux:latest ;;
  *) echo "SMOKE_ARCH must be arm64 or amd64" >&2; exit 2 ;;
esac

if [ "${1:-}" = --list ]; then
  for entry in "${distros[@]}"; do echo "${entry%%|*}"; done
  exit 0
fi

command -v container >/dev/null || {
  echo "distro_smoke: needs Apple's container tool" >&2
  echo "note: install it with \`brew install container\`, then run \`container system start\`" >&2
  exit 2
}

wanted=("$@")
mounts=(--volume "$repo/install.sh:/smoke/install.sh:ro" --volume "$repo/tools/dev/distro_smoke_steps.sh:/smoke/steps.sh:ro")
env_args=(--env "JAI_VERSION=${JAI_VERSION:-}")
base="${JAI_BASE_URL:-}"
if [[ "$base" == file://* ]]; then
  [ -n "${JAI_VERSION:-}" ] || { echo "distro_smoke: a file:// JAI_BASE_URL needs JAI_VERSION" >&2; exit 2; }
  mounts+=(--volume "${base#file://}:/smoke/releases:ro")
  base=file:///smoke/releases
fi
[ -z "$base" ] || env_args+=(--env "JAI_BASE_URL=$base")

logs="$repo/target/distro-smoke"
mkdir -p "$logs"
failed=0
for entry in "${distros[@]}"; do
  IFS='|' read -r name image setup <<<"$entry"
  [ "$image" = ARCH_IMAGE ] && image="$arch_image"
  if [ ${#wanted[@]} -gt 0 ] && [[ ! " ${wanted[*]} " == *" $name "* ]]; then continue; fi
  log="$logs/$name.log"
  printf '%-10s ' "$name"
  if container run --rm --arch "$arch" -m "${SMOKE_MEMORY:-4G}" -c "${SMOKE_CPUS:-4}" \
    "${mounts[@]}" "${env_args[@]}" --env "SMOKE_SETUP=$setup" "$image" \
    sh /smoke/steps.sh >"$log" 2>&1; then
    echo "ok"
    [ "${SMOKE_KEEP:-}" = 1 ] || rm -f "$log"
  else
    failed=1
    echo "FAILED: $(grep '^smoke: FAILED' "$log" | tail -1 | cut -c15-)"
    echo "           log: $log"
  fi
done
exit "$failed"
