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

## Tool target selection

`jaic::target::Selection::build_target` owns command-line native target resolution;
`apply_for_mode` also supplies the effective OS/CPU for checking tools and jacket tests.
A triple takes precedence over OS/CPU flags. Native cross builds without a triple support
Windows and WASI; checking and interpretation permit every supported OS/CPU combination.
Use this policy when adding a consumer rather than deriving defaults from the host again.
The compiler driver exposes argument inspection through `jaic_cli_args::arguments::target_flags`,
which preserves parser boundaries and uses its existing option table.

Wasm uses its own CPU class in checking, interpretation and native builds, including when
`-cpu` is supplied. Native link eligibility is separate from OS/CPU metadata: compile-only
cases can check a cross target that has no configured native linker.
