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
# Debian packages no powerpc64 binutils for arm64, and lld refuses ELFv1 objects, which the
# big-endian glibc sysroot is made of: build GNU ld and as for powerpc64-linux-gnu.
RUN apt-get update \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends build-essential texinfo xz-utils curl ca-certificates \
 && cd /tmp && curl -fsSL https://ftp.gnu.org/gnu/binutils/binutils-2.40.tar.xz | tar xJ \
 && cd binutils-2.40 \
 && ./configure --target=powerpc64-linux-gnu --prefix=/usr/local --disable-nls --disable-gdb --disable-gprofng --disable-werror \
 && make -j2 MAKEINFO=true && make install MAKEINFO=true \
 && cd / && rm -rf /tmp/binutils-2.40 \
 && apt-get purge -y texinfo xz-utils && rm -rf /var/lib/apt/lists/*
