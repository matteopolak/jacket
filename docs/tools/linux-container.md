# Linux container

`tools/dev/linux_x64.sh` builds, tests and runs `jaic` as a Linux program (x86-64 or arm64) on an Apple Silicon Mac, for checking target-dependent behaviour: `#asm` forms, system calls, Linux stdlib paths, the signal-context code in `interp/guard.rs`.

```sh
tools/dev/linux_x64.sh build
tools/dev/linux_x64.sh test -p jaic --lib guard
LINUX_ARCH=arm64 tools/dev/linux_x64.sh test -p jaic-cli --test wild_pointers
tools/dev/linux_x64.sh run -m "$PWD/scratch" -- -check /repo/examples/basics/main.jai
tools/dev/linux_x64.sh shell
```

## How it works

It wraps Apple's `container` tool with the image `docker.io/library/rust:1-bookworm`.

- `LINUX_ARCH=amd64` (default) is emulated: slow, but exact for the instruction set. `arm64` runs natively.
- The repo is mounted read-only at `/repo`. Cargo's home, rustup and `target/` live in a scratch folder (`LINUX_X64_SCRATCH`, `LINUX_ARM64_SCRATCH`; default `target/linux-x64` and `target/linux-arm64` in the repo), so no source file is written.
- Every command runs with `--network none`, and `build`/`test` add `--offline`. The first fill of the cargo cache is the only step that needs the network: `LINUX_X64_NETWORK=1 tools/dev/linux_x64.sh build`.
- `build` and `test` pass `--no-default-features`: there is no LLVM in the container, so `-exe` is unavailable. `-check`, `-interp` and the language server work.
- `run` mounts extra host folders read-only with `-m PATH` and runs `/target/debug/jaic`. `JAIC_STDLIB` is `/repo/stdlib`.
- The VM gets `LINUX_MEMORY` (default `8G`) and `LINUX_CPUS` (default `4`); rustc is killed in a smaller VM.

`JAIC_OFFICIAL_MODULES` mounts a module folder you were given at `/modules` as `JAIC_STDLIB`. It is for local probes only: don't read the folder or copy anything from it into the repository.

## How to change it

- Image, scratch folder, VM size, architecture: `LINUX_X64_IMAGE`, `LINUX_X64_SCRATCH`/`LINUX_ARM64_SCRATCH`, `LINUX_MEMORY`/`LINUX_CPUS`, `LINUX_ARCH`.
- A new subcommand goes in the `case` at the end of the script; keep the mounts, `vm` options and `--network none` shared.

## Cross targets: i686, s390x, powerpc64 and armv7

`jaic` builds for Linux CPUs a Mac cannot run. The cross image adds the glibc sysroots, cross C drivers and `qemu-user`; programs link and run inside it.

```sh
tools/dev/linux_x64.sh sysroots                       # once, with the network: builds jacket-cross:bookworm
export JAIC_CROSS_ARCH=i686                           # i686, s390x, powerpc64 or armv7
JAIC_LINKER=$PWD/tools/dev/cross_link.sh jaic -target i686-unknown-linux-gnu prog.jai
tools/dev/qemu_run.sh ./prog arg...
jacket test --runner "$PWD/tools/dev/qemu_run.sh" -- -target i686-unknown-linux-gnu
```

- `sysroots` is the only step that needs the network. It builds `tools/dev/linux-cross.Dockerfile` (Debian `qemu-user`, `gcc-*-linux-gnu`, `libc6-dev-*-cross`, `clang`, `lld`); everything after it runs with `--network none`.
- `cross-up` starts one long-lived container, `jacket-cross`, that mounts the repo, `CROSS_SCRATCH` (a scratch volume, when set), `$TMPDIR` and `CROSS_MOUNTS` at the same paths as on the Mac, so the paths in a link command or a test mean the same thing inside. `cross-exec -- COMMAND` runs in it (about 0.4 s each); `cross-down` removes it.
- `cross_link.sh` is a `JAIC_LINKER`: it runs the target's cross driver (`i686-linux-gnu-gcc`, `s390x-linux-gnu-gcc`, `arm-linux-gnueabihf-gcc`; clang with lld for powerpc64, which Debian does not package for arm64) on jaic's link command.
- The cross container gets `CROSS_MEMORY` (default `4G`) and `CROSS_CPUS` (default `2`). Keep it at 4G or less on a 16 GiB Mac: qemu plus a cargo build in a larger VM sends the host into swap. `cross-down` then `cross-up` applies a change.
- `qemu_run.sh` caps each program: its address space is limited to `QEMU_MEMORY_KB` (default 4 GiB; 32-bit guests also get `-R 3G`) so a runaway test fails with an allocation error, and `timeout` kills qemu after `QEMU_TIMEOUT` seconds (default 300, exit 124). Run `jacket test` with `-j 1` or `-j 2`, and not while cargo builds.
- `qemu_run.sh` is the runner: `qemu-i386`, `qemu-s390x`, `qemu-ppc64` or `qemu-arm` with the matching glibc sysroot. Without Apple's `container` it fails with an install hint; there is no fallback.
- The image is arm64, so armv7 runs under qemu too; the amd64 `LINUX_ARCH` is not used here.
