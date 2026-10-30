# Linux x86-64 container

Builds and runs `jaic` as a Linux x86-64 program from an Apple Silicon Mac, for checking behaviour that depends on the target (the `#asm` forms, system calls, the Linux standard-library paths).

## How it works

`tools/dev/linux_x64.sh` wraps Apple's `container` tool:

- Image `docker.io/library/rust:1-bookworm` with `--arch amd64` (emulated; slow, but exact for the instruction set).
- The repository is mounted read-only at `/repo`. Cargo's home, rustup and the target folder live on the scratch volume (`LINUX_X64_SCRATCH`, default `/Volumes/LodestoneScratch/jai-targets/linux-x64`, with `target/` and `cargo/` inside), so nothing is written to the checkout.
- Every command runs with `--network none`. `build` adds `--offline`, so the cargo cache in `cargo/` must already hold the crates. The first fill of that cache is the only step that needs the network: `LINUX_X64_NETWORK=1 tools/dev/linux_x64.sh build`.
- `build` passes `--no-default-features -p jaic-cli`: the container has no LLVM, so `-exe` is unavailable. `-check`, `-interp` and the language-server paths work.
- `container system start` is run when the service is down (once per boot).

```sh
tools/dev/linux_x64.sh build
tools/dev/linux_x64.sh run -m "$PWD/scratch" -- -check /repo/examples/basics/main.jai
tools/dev/linux_x64.sh shell
```

`run` mounts extra host folders read-only with `-m PATH` (at the same path inside) and runs `/target/debug/jaic` with the remaining arguments. `JAIC_STDLIB` is `/repo/stdlib`.

## Using another module set

Set `JAIC_OFFICIAL_MODULES` to a folder of modules you were given, for example `JAIC_OFFICIAL_MODULES=~/projects/jai-reference/<version>/modules`. It is mounted read-only at `/modules` and becomes `JAIC_STDLIB`. This is for local probes only: do not read the folder, do not copy anything from it into the repository, and keep the network off (the script never enables it for `run`). What you may use is `jaic`'s own diagnostics. Never run an official compiler binary this way.

## How to change it

- A different image or scratch folder: `LINUX_X64_IMAGE`, `LINUX_X64_SCRATCH`.
- A new subcommand goes in the `case` at the end of the script; keep the mounts and `--network none` shared.
- Rebuild after changing the compiler (`build` is incremental through `target/`).

## Dependencies

Apple's `container` CLI, the Rust image, and the scratch volume being mounted. See also [development](development.md) and [LLVM setup](llvm-setup.md).
