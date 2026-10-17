# Targets

## What it is

`jaic::target::Target` (`crates/jaic/src/target.rs`) is the one value for what a program is built for: `os` (`TargetOs`), `cpu` (`TargetCpu`) and `abi` (`TargetAbi`: `Msvc`, `Gnu`, `Musl`, `Apple`, `Wasi`, `Unknown`), plus `memory32` for a `wasm32` triple. `Display` writes the LLVM triple.

## How it works

It is parsed once, then passed on:

- `-os`/`-cpu` give `TargetOs`/`TargetCpu` (`parse_os`, `parse_cpu`; bad values keep the same error text). `-target <triple>` and a workspace's `llvm_target_system_triple` go through `Target::parse`, which rejects what jaic cannot build for (`unsupported target architecture in ...`).
- `os_target`/`cpu_target` of a workspace arrive as typed numbers ([compiler builtins](compiler-builtins.md)) and decode to `TargetOs`/`TargetCpu`; `BuildSettings.llvm_target` holds an explicit triple.
- `jaic-cli` resolves a `Target` (`Cli::build_target`, `LlvmBackend::workspace_target`): `-os windows` off Windows is MinGW (`Target::windows`), `-os wasm` is a WASI command (`Target::wasi`), a workspace with only `os_target = .WASM` is a bare module (`Target::wasm_module`). `None` means the host (`Target::host`).
- `jaic-llvm` takes `Options.target: Option<Target>` and every linker, archiver and extension decision matches on `os`/`abi`/`cpu` (`LinkFlavor::for_target`, `output_extension`, `baseline_cpu`, `DebugFormat::for_target`). `Target::arch` gives the calling-convention family (`abi::Arch`), `msvc::arch_dir` the SDK folder.

Canonical triples: `x86_64-pc-windows-msvc`, `aarch64-pc-windows-gnu` (MinGW), `x86_64-apple-darwin`, `aarch64-unknown-linux-gnu`, `wasm64-unknown-wasi`, `wasm64-unknown-unknown`. Other spellings parse to the same value (`aarch64-w64-mingw32`, `arm64-apple-macosx11.0`) and print canonically.

## How to change it

A new OS or CPU: add the `TargetOs`/`TargetCpu` variant, its names in `target.rs`, a `Target::parse`/`Display` arm, and let the compiler point at every `match` (there are no string comparisons on target names to find). A new C environment: a `TargetAbi` variant and the places that match on it (`LinkFlavor`, `DebugFormat`).

## Configuration

`-os`, `-cpu`, `-target`, and the `os_target`, `cpu_target` and `llvm_options.target_system_triple` build options. `JAIC_LINKER`, `JAIC_AR` pick tools, not targets.

## Dependencies

`jaic::sema` (`TargetOs`, `TargetCpu`, `Options::host`), `jaic::abi`, `jaic::msvc`; used by `jaic-cli`, `jaic-llvm`, `jailint`.
