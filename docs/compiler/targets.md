# Targets

`jaic::target::Target` (`crates/jaic/src/target.rs`) is the one value for what a program is built for: `os`, `cpu`, `abi` (`Msvc`, `Gnu`, `Musl`, `Apple`, `Wasi`, `Unknown`), plus `memory32` for `wasm32`. `Display` writes the LLVM triple.

```text
x86_64-pc-windows-msvc   aarch64-unknown-linux-gnu   wasm64-unknown-wasi   wasm32-wasi
```

Other spellings (`aarch64-w64-mingw32`, `arm64-apple-macosx11.0`) parse to the same value and print canonically.

## How it works

- `-os`, `-cpu` and `-target` become a `Selection`. `select_os_cpu` holds the one default: host CPU for the host OS, x64 for other OSes, wasm for the wasm OS.
- `-target` and a workspace's `llvm_target_system_triple` go through `Target::parse`, which rejects what jaic can't build for.
- `jaic-cli` resolves the final `Target`: `-os windows` off Windows is MinGW, `-os wasm` is a WASI command, a workspace with only `os_target = .WASM` is a bare module. `None` means the host.
- `jaic-llvm` matches on `os`/`abi`/`cpu` for every linker, archiver and extension decision. How the C runtime is linked: [toolchain](toolchain.md).

## Data layout

`Target::data_layout()` is the only place a `DataLayout` is chosen (`crates/jaic/src/layout.rs`): pointer size and alignment, byte order, the alignment of each scalar, the size of an x87 `long double`, and the C type widths. It is stored in `sema::Options::layout` and copied into `Types`, `Interp` and the LLVM backend. `Ty::size` and `Ty::align` take it, so a pointer is the target's, never 8 bytes.

`crates/jaic-llvm` has a test that builds an LLVM target machine for each supported triple and checks that `DataLayout` agrees with LLVM on `i8` to `i64`, `f32`, `f64`, `ptr` and byte order. `wasm32` pointers are 4 bytes (`Target::memory32`): `string` and `[]` are 12 bytes padded to 16 because `s64` counts stay 8-aligned, `Any` and `Allocator` are 8. `i686` Linux (`TargetCpu::X86`) has 4-byte pointers, `s64` and `float64` aligned to 4 (so `string` is 12 bytes) and a 12-byte `long double`; its ABI is `Arch::X86` (i386 System V: stack arguments, aggregates returned in memory). Other layouts arrive with their targets ([plan](../native/n-bit-and-embedded.md)). Programs for a narrow target run at compile time in the interpreter's [address window](interpreter.md).

Some sema IR is built from layout offsets, not constants: `box_any`, `Type_Info` arrays (`info_view`), relocations (`segments`), the stack-trace node (`trace_layout`) and the atomics. A hard-coded `8` or `16` there is a bug on a narrow target.

## How to change it

- New layout: change `Target::data_layout`; the agreement test says whether LLVM agrees.
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
