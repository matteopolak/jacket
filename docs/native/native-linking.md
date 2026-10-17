# Native build and linking

## What it is

`jaic` compiles the program to object files with `jaic-llvm` and links them with the system `cc` on macOS and Linux, or a Windows toolchain ([Windows](windows.md)). It adds the libraries named by `#library` and `#system_library` that foreign symbols actually use.

## How it works

`build` and `LlvmBackend` in `crates/jaic-cli/src/main.rs` pick the output path (`-output_path` joined with `-exe`, else the metaprogram's `output_path`, else the source file stem), apply `-O0..-O3` over the metaprogram's optimisation setting, and call `jaic_llvm::emit_objects` and `link`. A metaprogram can also ask for a shared or static library, an object file, or no output (`jaic::build::OutputType`).

`used_libraries` keeps libraries referenced by a foreign symbol plus `link_always` ones (registered by `declare_library` in `sema/modules.rs`). `library_args` turns each into linker inputs, `render_link_arg` renders them per linker style. On macOS and Linux:

- `libc` / `c`: nothing, implicit.
- libm: always `-lm`. Builds target the baseline CPU, and on x86-64 without SSE4.1 LLVM turns float rounding into calls to libm.
- Non-system library (`#library "native/own"`): looked up relative to the declaring file as `own.a`, `libown.a`, then `own.dylib`/`.so`, `libown.dylib`/`.so`. A static archive wins; a shared library adds `-Wl,-rpath,<dir>`.
- Apple framework (`/System/Library/Frameworks/<name>.framework` exists): `-framework <name>`.
- Otherwise `lib<name>.a` in the native-libs directories by path, else `-l<name>` (Homebrew's library directory is added on macOS if present).

A missing library fails at link time; `link_failure` adds a help when the linker output says a library or symbol is missing, and `linker_not_run` explains how to install a linker when `cc` (or `JAIC_LINKER`) is not found. Windows targets have their own rules ([Windows](windows.md)); WebAssembly links with `wasm-ld` ([wasm target](wasm-target.md)).

## How to change it

- Linker rules: `library_args` in `crates/jaic-llvm/src/lib.rs`. Keep its search order in step with how the interpreter loads libraries (`interp/native.rs`). It receives the `LinkFlavor` and whether the build is a cross build (host directories are not searched then).
- Static libraries go through `jaic_llvm::archive` (`ar rcs`, or the Windows archivers). Extra linker flags from a metaprogram arrive in the `extra_args` parameter of `link`.
- Tests in `crates/jaic-cli/tests/native.rs`: `corpus_runs_natively_like_the_interpreter` builds every passing `tests/corpus/manifest.json` case and compares exit code and stdout. Fixtures live in `tests/native/`.

## Configuration

- `JAIC_STDLIB`: stdlib directory (default `<repo>/stdlib`).
- `JAIC_NATIVE_LIBS`: path list searched for third-party static archives; default `native-libs/<os>-<arch>` (release package) or `target/native-libs/<os>-<arch>` (checkout), next to the stdlib, produced by `tools/build_native_libs.py` ([third-party native libraries](../tools/native-libs.md)).
- `JAIC_LINKER`, `JAIC_AR`: override the linker or archiver.
- CLI: `-exe name`, `-output_path dir`, `-o`, `-od`, `-very_debug`, `-O0..-O3`, `-no_inline`, `-no_split`, `-emit_ir file.ll`, `-sanitize address,undefined` ([sanitizers](sanitizers.md)), `-I dir`, `-os windows`, `-cpu x64|arm64`, `-target triple`.

## Dependencies

The system `cc` (Clang on macOS) as linker driver, and the [LLVM backend](llvm-backend.md).
