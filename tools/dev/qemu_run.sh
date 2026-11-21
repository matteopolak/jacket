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
guest=()
case "$arch" in
  i686) qemu=qemu-i386 sysroot=/usr/i686-linux-gnu guest=(-R 3G) ;;
  s390x) qemu=qemu-s390x sysroot=/usr/s390x-linux-gnu ;;
  powerpc64) qemu=qemu-ppc64 sysroot=/usr/powerpc64-linux-gnu ;;
  armv7) qemu=qemu-arm sysroot=/usr/arm-linux-gnueabihf guest=(-R 3G) ;;
  *)
    echo "qemu_run.sh: set JAIC_CROSS_ARCH to i686, s390x, powerpc64 or armv7" >&2
    exit 2
    ;;
esac
if ! command -v container >/dev/null 2>&1; then
  echo "qemu_run.sh: Apple's \`container\` is not installed (https://github.com/apple/container); it runs the qemu-user image" >&2
  exit 127
fi
# One runaway program must fail, not take the VM down: its address space is capped
# (QEMU_MEMORY_KB, default 4 GiB; 32-bit guests also get a 3 GiB guest space) and it is killed after QEMU_TIMEOUT seconds (default 300),
# qemu included, whether it ends or not.
limit="${QEMU_MEMORY_KB:-4194304}"
seconds="${QEMU_TIMEOUT:-300}"
exec "$here/linux_x64.sh" cross-exec -- sh -c "ulimit -v $limit; exec timeout -k 5 $seconds \"\$@\"" sh \
  env "QEMU_LD_PREFIX=$sysroot" "$qemu" ${guest[@]+"${guest[@]}"} -L "$sysroot" "$@"
