# Native build and linking

## What it is

`jaic build` compiles the program to an object file with `jaic-llvm` and links it with the system `cc`, adding the libraries named by `#library` / `#system_library` declarations that foreign symbols actually use.

## How it works

`crates/jaic-cli/src/main.rs` (`build`, `LlvmBackend`) picks the output path (`-o`, else the metaprogram's `output_path` plus the executable name, else the source file stem), applies `-O0..-O3` over the metaprogram's optimization setting, then calls `jaic_llvm::emit_object` and `jaic_llvm::link` (`crates/jaic-llvm/src/lib.rs`). A metaprogram can also request a shared library or no output (`jaic::build::OutputType`).

Which libraries are linked: `used_libraries` keeps those referenced by a foreign symbol plus `link_always` ones. `library_args` turns each into linker arguments:

- `libc` / `c`: nothing (implicit).
- Non-system library (`#library "native/own"`): looked up relative to the declaring source file as `own.a`, `libown.a`, then `own.dylib`/`.so`, `libown.dylib`/`.so`. A static archive wins; a shared library adds `-Wl,-rpath,<dir>`. If the name contains `/` and nothing is found the error says where it looked.
- Apple framework (`/System/Library/Frameworks/<name>.framework` exists): `-framework <name>`.
- Otherwise a `lib<name>.a` found in the native-libs directories (see Configuration) is linked by path, else `-l<name>` (a leading `lib` is stripped; `/opt/homebrew/lib` is added on macOS if present).

Identical argument groups are added once. A missing system library fails at link time:

```
error: linking failed (exit status: 1):
ld: library 'nothere' not found
```

Verified round trip: a program calling `strlen` and `labs` from `#system_library "libc"` prints the same output under `jaic run` and the binary from `jaic build -O2 --emit-ir out.ll`.

## How to change it

- New linker rule: edit `library_args` in `crates/jaic-llvm/src/lib.rs`; keep its search order in step with how the interpreter loads libraries (`crates/jaic/src/interp/native.rs`).
- Extra linker flags from a metaprogram go through the `extra_args` parameter of `link`.
- Tests: `crates/jaic-cli/tests/native.rs` has `corpus_runs_natively_like_the_interpreter` (every `tests/corpus/manifest.json` case that passes under the interpreter is built and its exit code and stdout compared), `hello_world_builds_and_prints`, `c_structs_by_value` and `c_variadic_calls`. Native fixtures live in `tests/native/`.
- There is no debug info yet: `Inst::Loc` is ignored in `lower.rs`.

## Configuration

- `JAIC_STDLIB`: stdlib directory (default `<repo>/stdlib`).
- `JAIC_NATIVE_LIBS`: path list of directories searched for third-party static archives; default is `artifacts/native-libs/<os>-<arch>` next to the stdlib (`macos`/`linux`, `arm64`/`x64`), produced by `tools/build_native_libs.py`.
- `LLVM_SYS_221_PREFIX`: LLVM 22 install, needed to build `jaic-llvm`.
- CLI: `-o`, `-O0..-O3`, `--emit-ir file.ll`, `-I dir`.

## Dependencies

- System `cc` (Clang on macOS) as the linker driver.
- [LLVM backend](llvm-backend.md) for object emission.
