# Targets

`jaic::target::Target` (`crates/jaic/src/target.rs`) is the one value for what a program is built for: `os`, `cpu`, `abi` (`Msvc`, `Gnu`, `Musl`, `Apple`, `Wasi`, `Unknown`), plus `memory32` for `wasm32`. `Display` writes the LLVM triple.

```text
x86_64-pc-windows-msvc   aarch64-unknown-linux-gnu   wasm64-unknown-wasi
```

Other spellings (`aarch64-w64-mingw32`, `arm64-apple-macosx11.0`) parse to the same value and print canonically.

## How it works

- `-os`, `-cpu` and `-target` become a `Selection`. `select_os_cpu` holds the one default: host CPU for the host OS, x64 for other OSes, wasm for the wasm OS.
- `-target` and a workspace's `llvm_target_system_triple` go through `Target::parse`, which rejects what jaic can't build for.
- `jaic-cli` resolves the final `Target`: `-os windows` off Windows is MinGW, `-os wasm` is a WASI command, a workspace with only `os_target = .WASM` is a bare module. `None` means the host.
- `jaic-llvm` matches on `os`/`abi`/`cpu` for every linker, archiver and extension decision. How the C runtime is linked: [toolchain](toolchain.md).

## How to change it

- New OS or CPU: add the variant, its names, and `parse`/`Display` arms, then follow the compiler errors to each `match`. There are no string comparisons on target names.
- New C environment: a `TargetAbi` variant plus the matches (`Toolchain::for_target`, `DebugFormat`).

Options: `-os`, `-cpu`, `-target`, `os_target`, `cpu_target`, `llvm_options.target_system_triple`. `JAIC_LINKER` and `JAIC_AR` pick tools, not targets.
