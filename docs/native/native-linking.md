# Native build and linking

## What it is

`jaic build` compiles the program to object files with `jaic-llvm` and links them with the system `cc` on macOS and Linux, or a Windows toolchain (see [Windows](windows.md)). It adds the libraries named by `#library` and `#system_library` that foreign symbols actually use.

## How it works

`build` and `LlvmBackend` in `crates/jaic-cli/src/main.rs` pick the output path (`-o`, else the metaprogram's `output_path` plus executable name, else the source file stem), apply `-O0..-O3` over the metaprogram's optimisation setting, and call `jaic_llvm::emit_objects` and `jaic_llvm::link`. A metaprogram can also ask for a shared or static library, an object file, or no output (`jaic::build::OutputType`).

`used_libraries` keeps the libraries referenced by a foreign symbol plus `link_always` ones, including unnamed statements like `#library,system,link_always "libc++";` (registered by `declare_library` in `sema/modules.rs`). `library_args` turns each into linker inputs (`LinkArg`), and `render_link_arg` renders them per linker style. On macOS and Linux:

- `libc` / `c`: nothing; it is implicit.
- libm: always linked (`-lm`), named or not. Builds target the baseline CPU ([LLVM backend](llvm-backend.md)), and on x86-64 without SSE4.1 LLVM turns float rounding (`#asm` `roundps`, `floor`, `trunc`) into calls to libm's `trunc`, `floor`, `ceil` and `round`.
- Non-system library (`#library "native/own"`): looked up relative to the declaring file as `own.a`, `libown.a`, then `own.dylib`/`.so`, `libown.dylib`/`.so`. A static archive wins; a shared library adds `-Wl,-rpath,<dir>`. If the name contains `/` and nothing is found, the error says where it looked.
- Apple framework (`/System/Library/Frameworks/<name>.framework` exists): `-framework <name>`.
- Otherwise a `lib<name>.a` in the native-libs directories is linked by path, else `-l<name>` (a leading `lib` is stripped; Homebrew's library directory, `/opt/homebrew/lib` on Apple silicon or `/usr/local/lib` on Intel, is added on macOS if present).

Identical argument groups are added once. A missing system library fails at link time:

```
error: linking failed: `cc` stopped with exit status: 1
ld: library 'nothere' not found
help: a `#library` or `#system_library` names a library the linker cannot find: check its path, or install the library
```

`link_failure` adds that help when the linker's output says a library or a symbol is missing; `linker_not_run` explains how to install a linker when `cc` (or `JAIC_LINKER`) is not found.

Windows targets (`LinkFlavor::MinGw`, `Msvc`) have their own rules; see [Windows](windows.md). WebAssembly targets link with `wasm-ld` instead of `cc`, and system libraries become import modules; see [wasm target](wasm-target.md).

## How to change it

- Linker rules: `library_args` in `crates/jaic-llvm/src/lib.rs`. Keep its search order in step with how the interpreter loads libraries (`interp/native.rs`). It receives the `LinkFlavor` and whether the build is a cross build, in which case host directories aren't searched.
- Static libraries go through `jaic_llvm::archive` (`ar rcs`, or the Windows archivers).
- Extra linker flags from a metaprogram arrive in the `extra_args` parameter of `link`.

Tests in `crates/jaic-cli/tests/native.rs`: `corpus_runs_natively_like_the_interpreter` builds every `tests/corpus/manifest.json` case that passes under the interpreter and compares exit code and stdout; also `hello_world_builds_and_prints`, `c_structs_by_value`, `c_variadic_calls`. Native fixtures live in `tests/native/`.

## Configuration

- `JAIC_STDLIB`: stdlib directory (default `<repo>/stdlib`).
- `JAIC_NATIVE_LIBS`: path list searched for third-party static archives. Default: `artifacts/native-libs/<os>-<arch>` next to the stdlib (`macos`/`linux`, `arm64`/`x64`), produced by `tools/build_native_libs.py`; see [third-party native libraries](../tools/native-libs.md).
- `JAIC_LINKER`, `JAIC_AR`: override the linker or archiver (see [Windows](windows.md)).
- CLI: `-o`, `-O0..-O3`, `--emit-ir file.ll`, `-sanitize address,undefined` (links with the LLVM install's `clang`; see [sanitizers](sanitizers.md)), `-I dir`, `-os windows`, `-cpu x64|arm64`, `-target triple`.

## Dependencies

The system `cc` (Clang on macOS) as linker driver, and the [LLVM backend](llvm-backend.md).
