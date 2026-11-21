# The image `tools/dev/linux_x64.sh sysroots` builds: rust:1-bookworm plus the cross C
# environments and qemu-user that run i686, s390x, powerpc64 and armv7 Linux programs.
# See docs/tools/linux-container.md.
FROM docker.io/library/rust:1-bookworm
RUN apt-get update \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    qemu-user \
    gcc-i686-linux-gnu libc6-dev-i386-cross \
    gcc-s390x-linux-gnu libc6-dev-s390x-cross \
    libc6-dev-ppc64-cross \
    gcc-arm-linux-gnueabihf libc6-dev-armhf-cross \
    binutils-i686-linux-gnu binutils-s390x-linux-gnu binutils-arm-linux-gnueabihf \
    clang lld \
 && rm -rf /var/lib/apt/lists/*
