# Native build and linking

`jaic` compiles to object files with `jaic-llvm` and links them with the system `cc` on macOS and Linux, or a Windows toolchain ([Windows](windows.md)). It adds the libraries named by `#library` and `#system_library` that foreign symbols actually use.

## How it works

`build` and `LlvmBackend` in `crates/jaic-cli/src/main.rs` pick the output path (`-output_path` plus `-exe`, else the metaprogram's `output_path`, else the source stem), apply `-O0..-O3` over the metaprogram's setting, and call `jaic_llvm::emit_objects` and `link`. A metaprogram can also ask for a shared or static library, an object file, or no output (`jaic::build::OutputType`).

`used_libraries` keeps libraries a foreign symbol references plus `link_always` ones. `library_args` turns each into linker inputs. On macOS and Linux:

- `libc`/`c`: nothing. libm: always `-lm` (without SSE4.1, LLVM turns float rounding into libm calls).
- Your own library (`#library "native/own"`): looked up beside the declaring file as `own.a`, `libown.a`, then `own.dylib`/`.so`, `libown.dylib`/`.so`. A static archive wins; a shared library adds `-Wl,-rpath,<dir>`.
- Apple framework: `-framework <name>`.
- Otherwise `lib<name>.a` in the native-libs directories, else `-l<name>`.

On a Mac, a plain executable link for the host skips the `cc` driver and runs the selected Xcode's `ld` itself (`crates/jaic-native/src/apple_ld.rs`): `cc` is a shim that starts `clang`, which starts `ld` with the SDK, the platform versions (the running macOS and the SDK's `SDKSettings.json`), `-lSystem` and `libclang_rt.osx.a` filled in, and that is about 8 ms of a 50 ms hello world. The command is `cc`'s own, argument for argument. It is not used when `JAIC_LINKER`, `SDKROOT` or `MACOSX_DEPLOYMENT_TARGET` is set, for a shared library, a sanitized or cross build, or when the link has any argument other than `-o`, `-l`, `-L`, `-framework` and `-Wl,`. When the direct `ld` is missing or fails, the link is run again through `cc`, so errors read as they always did.

A missing library fails at link time with a hint (`link_failure`, `linker_not_run`). WebAssembly links with `wasm-ld` ([wasm target](wasm-target.md)).

## How to change it

- Linker rules: `library_args` in `crates/jaic-llvm/src/lib.rs`. Keep its search order in step with the interpreter's library loading (`interp/native.rs`). Cross builds do not search host directories.
- Static libraries go through `jaic_llvm::archive`. Extra linker flags from a metaprogram arrive as `extra_args` to `link`.
- Test: `corpus_runs_natively_like_the_interpreter` in `crates/jaic-cli/tests/native.rs` builds every passing corpus case and compares exit code and stdout. Fixtures are in `tests/native/`.

## Configuration

- `JAIC_STDLIB`: stdlib directory.
- `JAIC_NATIVE_LIBS`: where third-party static archives are searched; default `native-libs/<os>-<arch>` (release) or `target/native-libs/<os>-<arch>` ([native libraries](../tools/native-libs.md)).
- `JAIC_LINKER`, `JAIC_AR`: override the linker or archiver.
- `DEVELOPER_DIR`, `SDKROOT`, `MACOSX_DEPLOYMENT_TARGET`: Xcode selection for the direct `ld` (the last two turn it off).
- Flags: [command line](../reference/command-line.md).

## Dependencies

The system `cc` as linker driver (on a Mac, Xcode's `ld` directly), and the [LLVM backend](llvm-backend.md).
