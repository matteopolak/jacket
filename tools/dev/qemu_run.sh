#!/usr/bin/env bash
# Runs a Linux program built for i686, s390x, powerpc64 or armv7 under qemu-user in the cross
# container: the `--runner` of `jacket test`, and a way to run one program by hand. See
# docs/tools/linux-container.md.
#
#   JAIC_CROSS_ARCH=s390x jacket test --runner "$PWD/tools/dev/qemu_run.sh" -- -target s390x-unknown-linux-gnu
#   JAIC_CROSS_ARCH=armv7 tools/dev/qemu_run.sh ./prog arg...
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
arch="${JAIC_CROSS_ARCH:-}"
case "$arch" in
  i686) qemu=qemu-i386 sysroot=/usr/i686-linux-gnu ;;
  s390x) qemu=qemu-s390x sysroot=/usr/s390x-linux-gnu ;;
  powerpc64) qemu=qemu-ppc64 sysroot=/usr/powerpc64-linux-gnu ;;
  armv7) qemu=qemu-arm sysroot=/usr/arm-linux-gnueabihf ;;
  *)
    echo "qemu_run.sh: set JAIC_CROSS_ARCH to i686, s390x, powerpc64 or armv7" >&2
    exit 2
    ;;
esac
if ! command -v container >/dev/null 2>&1; then
  echo "qemu_run.sh: Apple's \`container\` is not installed (https://github.com/apple/container); it runs the qemu-user image" >&2
  exit 127
fi
exec "$here/linux_x64.sh" cross-exec -- env "QEMU_LD_PREFIX=$sysroot" "$qemu" -L "$sysroot" "$@"
