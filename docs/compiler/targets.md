# Targets

`jaic::target::Target` (`crates/jaic/src/target.rs`) is the one value for what a program is built for: `os`, `cpu`, `abi` (`Msvc`, `Gnu`, `Musl`, `Apple`, `Wasi`, `Unknown`), plus `memory32` for `wasm32`. `Display` writes the LLVM triple.

The compiler itself also builds and runs on 32-bit hosts (`i686-unknown-linux-gnu`): `compiler_host()` maps x86, arm, s390x and powerpc64 hosts to their `TargetCpu`, and CI runs the `jaic-syntax` and `jaic` tests on i686 in the full tier. A 32-bit host cannot run programs with 8-byte pointers at compile time, since the interpreter window needs a 64-bit address space.

The 32-bit (`wasm32`, i686, armv7, Cortex-M, AVR), big-endian (s390x, powerpc64) and freestanding targets are experimental: their behaviour and APIs may change in any release.

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

`crates/jaic-llvm` has a test that builds an LLVM target machine for each supported triple and checks that `DataLayout` agrees with LLVM on `i8` to `i64`, `f32`, `f64`, `ptr` and byte order. `wasm32` pointers are 4 bytes (`Target::memory32`): `string` and `[]` are 12 bytes padded to 16 because `s64` counts stay 8-aligned, `Any` and `Allocator` are 8. `i686` Linux (`TargetCpu::X86`) has 4-byte pointers, `s64` and `float64` aligned to 4 (so `string` is 12 bytes) and a 12-byte `long double`; its ABI is `Arch::X86` (i386 System V: stack arguments, aggregates returned in memory). `CPU` is `.CUSTOM` for every CPU the spec's `CPU_Tag` lacks, so `#if` code that must tell them apart (bindings with per-CPU layouts) tests `ARCHITECTURE`, an `Architecture` enum declared in `stdlib/prelude/platform.jai` (`.X86`, `.WASM32`, ...). `s390x` Linux (`TargetCpu::S390x`, `-target s390x-unknown-linux-gnu`) is 64-bit big-endian with a binary128 `long double`; its ABI is `Arch::S390x` (z/Architecture ELF: 1-, 2-, 4- and 8-byte aggregates travel as exact-size integers, larger ones by reference, every aggregate result through a hidden pointer). Compile-time code runs in the interpreter, whose loads and stores swap bytes when the program's order differs from the host's (`Interp::swap_bytes`) and which uses the emulated C library, since native functions would read the wrong bytes (`DataLayout::is_foreign_memory`). `powerpc64` Linux (`TargetCpu::PowerPc64`, `-target powerpc64-unknown-linux-gnu`) is 64-bit big-endian (ELFv1, as Debian's glibc is); its ABI is `Arch::PowerPc64` (every aggregate argument by value, every aggregate result through a hidden pointer in `r3`). lld cannot link ELFv1, so the cross container carries GNU `ld` built for `powerpc64-linux-gnu`; `long double` is a plain `double` (IBM double-double is not modelled). `armv7` Linux (`TargetCpu::Arm`, `-target armv7-unknown-linux-gnueabihf`) has 4-byte pointers, `long` and `size_t`, and 8-aligned `s64`; its ABI is `Arch::Arm32` (hard-float AAPCS: aggregates up to 4 bytes are one integer, float aggregates travel in VFP registers, other results over 4 bytes use a hidden pointer, variadic calls use core registers). `qemu-arm` reserves the whole 4 GiB guest space, so `tools/dev/qemu_run.sh` gives it a 5 GiB virtual limit and no `-R`. `avr-none` (`TargetCpu::Avr`, `TargetOs::None`) has 2-byte pointers, all alignments 1, a 2-byte `int`, 4-byte `long` and a 10-byte `string`; its ABI is `Arch::Avr` (18 register bytes, each argument rounded to an even size, an aggregate wholly in registers or wholly on the stack, results over 8 bytes through a hidden pointer). Its interpreter window is 48 KiB with 4 KiB stacks; compile-time buffers shrink to fit it ([AVR](../native/embedded.md#avr-arduino-uno)). Programs for a narrow target run at compile time in the interpreter's [address window](interpreter.md). The interpreter's libc emulation writes the i686 Linux `stat` record itself (`crates/jaic/src/interp/sandbox`); other 32-bit record layouts must be added there when a target needs them. Cross builds have no memory debugger (`Basic.Preamble` returns early unless the target is the build machine), so tests that read its state guard on `OS == BUILD_OS && CPU == BUILD_CPU`.

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
