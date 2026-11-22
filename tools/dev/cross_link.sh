#!/usr/bin/env bash
# The linker for `JAIC_LINKER` when jaic builds a Linux program for i686, s390x, powerpc64 or
# armv7 on a Mac: runs the target's cross C driver in the cross container, with the paths
# jaic gives it (the container mounts the same folders). See docs/tools/linux-container.md.
#
#   JAIC_CROSS_ARCH=i686 JAIC_LINKER=tools/dev/cross_link.sh jaic -target i686-unknown-linux-gnu prog.jai
#
# JAIC_CROSS_ARCH is i686, s390x, powerpc64 or armv7; without it the target is read from the
# `--target=` or `-m32` option jaic passes, when there is one.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
arch="${JAIC_CROSS_ARCH:-}"
if [ -z "$arch" ]; then
  echo "cross_link.sh: set JAIC_CROSS_ARCH to i686, s390x, powerpc64 or armv7" >&2
  exit 2
fi
case "$arch" in
  i686) driver=(i686-linux-gnu-gcc) ;;
  s390x) driver=(s390x-linux-gnu-gcc) ;;
  armv7) driver=(arm-linux-gnueabihf-gcc) ;;
  # Debian has no powerpc64 gcc for arm64 hosts: clang and GNU ld (built into the image; lld
  # refuses ELFv1) against the glibc sysroot.
  powerpc64)
    sysroot=/usr/powerpc64-linux-gnu
    driver=(clang --target=powerpc64-linux-gnu -isystem "$sysroot/include" -fuse-ld=/usr/local/bin/powerpc64-linux-gnu-ld -nostdlib
      "$sysroot/lib/crt1.o" "$sysroot/lib/crti.o")
    # No `--sysroot`: it would prefix the absolute paths in libc.so's linker script.
    # What gcc's driver adds after the objects: libc, the ELFv1 loader and the end crt.
    tail=("-L$sysroot/lib" -lc "-Wl,--dynamic-linker=/lib64/ld64.so.1" "$sysroot/lib/crtn.o")
    ;;
  *)
    echo "cross_link.sh: unknown JAIC_CROSS_ARCH \`$arch\`" >&2
    exit 2
    ;;
esac
exec "$here/linux_x64.sh" cross-exec -- "${driver[@]}" "$@" ${tail[@]+"${tail[@]}"}
