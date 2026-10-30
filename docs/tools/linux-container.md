# Linux container

Builds, tests and runs `jaic` as a Linux program (x86-64 or arm64) from an Apple Silicon Mac, for checking behaviour that depends on the target (the `#asm` forms, system calls, the Linux standard-library paths, the signal-context code in `interp/guard.rs`).

## How it works

`tools/dev/linux_x64.sh` wraps Apple's `container` tool:

- Image `docker.io/library/rust:1-bookworm`. `LINUX_ARCH=amd64` (the default) is emulated: slow, but exact for the instruction set. `LINUX_ARCH=arm64` runs natively.
- The repository is mounted read-only at `/repo`. Cargo's home, rustup and the target folder live on the scratch volume, so nothing is written to the checkout. Each architecture has its own folder with `target/` and `cargo/` inside: `LINUX_X64_SCRATCH` (default `/Volumes/LodestoneScratch/jai-targets/linux-x64`) and `LINUX_ARM64_SCRATCH` (default `.../linux-arm64`).
- `PATH` is `/cargo/bin` first, then the image's own `/usr/local/cargo/bin`, so the image's toolchain is used while the scratch `cargo/bin` is empty (rustup then fetches the repository's pinned toolchain into `cargo/rustup`).
- The VM gets `LINUX_MEMORY` (default `8G`) and `LINUX_CPUS` (default `4`); rustc is killed in the default, smaller VM.
- Every command runs with `--network none`. `build` and `test` add `--offline`, so the cargo cache in `cargo/` must already hold the crates and the toolchain. The first fill of that cache is the only step that needs the network: `LINUX_X64_NETWORK=1 tools/dev/linux_x64.sh build` (the variable name covers both architectures).
- `build` and `test` pass `--no-default-features`: the container has no LLVM, so `-exe` is unavailable. `-check`, `-interp` and the language-server paths work.
- `container system start` is run when the service is down (once per boot).

```sh
tools/dev/linux_x64.sh build
tools/dev/linux_x64.sh test -p jaic --lib guard
LINUX_ARCH=arm64 tools/dev/linux_x64.sh test -p jaic-cli --test wild_pointers
tools/dev/linux_x64.sh run -m "$PWD/scratch" -- -check /repo/examples/basics/main.jai
tools/dev/linux_x64.sh shell
```

`test` runs `cargo test --offline --no-default-features <args>` (tests that need LLVM are not built). `run` mounts extra host folders read-only with `-m PATH` (at the same path inside) and runs `/target/debug/jaic` with the remaining arguments. `JAIC_STDLIB` is `/repo/stdlib`.

## Using another module set

Set `JAIC_OFFICIAL_MODULES` to a folder of modules you were given, for example `JAIC_OFFICIAL_MODULES=~/projects/jai-reference/<version>/modules`. It is mounted read-only at `/modules` and becomes `JAIC_STDLIB`. This is for local probes only: do not read the folder, do not copy anything from it into the repository, and keep the network off (the script never enables it for `run`). What you may use is `jaic`'s own diagnostics. Never run an official compiler binary this way.

## How to change it

- A different image, scratch folder, VM size or architecture: `LINUX_X64_IMAGE`, `LINUX_X64_SCRATCH` / `LINUX_ARM64_SCRATCH`, `LINUX_MEMORY` / `LINUX_CPUS`, `LINUX_ARCH`.
- A new subcommand goes in the `case` at the end of the script; keep the mounts, `vm` options and `--network none` shared.
- Rebuild after changing the compiler (`build` is incremental through `target/`).

## Dependencies

Apple's `container` CLI, the Rust image, and the scratch volume being mounted. See also [development](development.md) and [LLVM setup](llvm-setup.md).
