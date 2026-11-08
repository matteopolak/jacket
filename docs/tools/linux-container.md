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
