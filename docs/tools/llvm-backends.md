# LLVM backends and binary size

## What it is

`jaic` registers only the three LLVM backends it can emit code for: x86-64, AArch64 and WebAssembly. The release archives link LLVM statically, and every backend that is registered is linked in, so registering all twenty made the 0.6.2 macOS arm64 `jaic` 136 MiB.

## How it works

`crates/jaic-llvm/src/lib.rs` (`target_machine`) calls `Target::initialize_x86`, `initialize_aarch64` and `initialize_webassembly` once, with inkwell's default `InitializationConfig` minus the disassembler (nothing reads machine code back; the asm parser stays because inline asm is assembled by the backend). It no longer calls `Target::initialize_all`, which expands to `LLVM_InitializeAll*` and references every backend. The root `Cargo.toml` enables inkwell's `target-x86`, `target-aarch64` and `target-webassembly` features, which are what compile those `initialize_*` functions in.

A triple for any other architecture is rejected before LLVM sees it: `jaic::abi::Arch::from_triple` returns `None` and `target_machine` reports `unsupported target architecture in '<triple>'`. Windows, Linux and macOS are all variants of the x86-64 and AArch64 backends; `-os wasm` and `wasm64-*` triples use the WebAssembly one (jaic builds wasm64 only).

Nothing references the other backends, so a static link only takes the members it needs from each LLVM archive, and the linker's dead-code removal (`-dead_strip` on macOS, `--gc-sections` on Linux, `/OPT:REF` on MSVC; rustc passes these for release builds) drops the rest. `llvm-sys` still lists every `libLLVM*.a` from `llvm-config --libs`; that is harmless, an unreferenced archive member is never loaded.

### Symbols

The release workflow strips the packaged `jaic`, `jailsp` and `jailint` after the PGO and BOLT build (both need symbols): `strip -x` on macOS (local symbols only; then `codesign --force -s -`, because stripping invalidates the ad-hoc signature that arm64 macOS requires) and `strip --strip-unneeded` on Linux. Windows has nothing to strip: its symbols are in the `.pdb`, which the archive does not ship. A future Developer ID signature goes on after this step, so it does not conflict.

Cost: `RUST_BACKTRACE=1` frames in a stripped `jaic` have no function names. The internal-compiler-error report itself is unaffected, since its panic message and `file:line` come from the panic location, not the symbol table. To get a symbolised backtrace, build from source (`cargo build --release -p jaic-cli`), which is not stripped.

## How to change it

- A new target architecture: add its `Arch` variant in `crates/jaic/src/abi.rs`, add the `target-<name>` feature to inkwell in the root `Cargo.toml`, and call its `Target::initialize_<name>` in `target_machine`. Expect the static release binary to grow by that backend's size.
- To check the effect without CI, compare the size of a static `jaic` built before and after, or `nm`/`size` it for `LLVMInitialize<Target>Target` symbols; the release workflow lists the packaged binaries (`ls -l`) before archiving.
- `llvm-sys` has a `disable-alltargets-init` feature that skips its `LLVM_InitializeAll*` wrapper object. It is not enabled: inkwell's `initialize_all` still names those wrappers, and an unverified link failure on one of six release platforms costs more than it saves.

## Configuration

None at run time. Build time: inkwell's `target-*` features (root `Cargo.toml`), the `strip` step in `.github/workflows/release.yml`.

## Dependencies

inkwell, `llvm-sys`, the platform's `strip` and `codesign`. See [LLVM setup](llvm-setup.md) and [releases](releases.md).
