# Native Windows executables

`jaic` writes Windows executables, DLLs and static libraries for x86-64 and arm64: natively on a Windows host with the Microsoft toolchain, or cross-compiled from macOS and Linux with MinGW-w64.

```
jacket build main.jai -os windows              # x64
jacket build main.jai -os windows -cpu arm64   # Windows on Arm
```

## How it works

**Target.** `Cli::target_triple` (`crates/jaic-cli/src/main.rs`): no flags means the host triple. `-os windows` off Windows gives `x86_64-pc-windows-gnu` (MinGW-w64), or `aarch64-pc-windows-gnu` with `-cpu arm64`; x64 is the default even on an arm64 Mac. `-cpu` on a Windows host picks the other CPU's MSVC triple. `-target <triple>` is taken as given.

`Target::arch` maps x86-64 to `Arch::Win64` and aarch64 to `Arch::Win64Arm` ([C ABI](c-abi.md)). Lowering differences are small: `#program_export` is `dllexport`, `CompilerWrite` calls `_write`, and `#asm` is lowered to portable IR so x64 `#asm` works on arm64 ([#asm](../compiler/asm.md)).

**Linking** (`link` in `crates/jaic-native/src/link.rs`, `Toolchain` in `crates/jaic/src/toolchain/`):

| Linker | Triple | Tool |
|---|---|---|
| `MinGw` | `*-windows-gnu` | `<cpu>-w64-mingw32-gcc` or `-clang`; `gcc`/`clang` on Windows |
| `Msvc` | `*-windows-msvc` | Visual Studio's `link.exe`, else Microsoft's on `PATH`, else `clang`/`lld-link` (`msvc_linker_command`) |

GCC's MinGW-w64 only targets x64. For arm64 use [llvm-mingw](https://github.com/mstorsjo/llvm-mingw) and append its `bin/` to `PATH`.

- Clang's driver finds the MSVC and SDK libraries itself. `link.exe` and `lld-link` need `LIB`; `toolchain/discover.rs` gets it via the `find-msvc-tools` crate when there is no developer prompt. A `link` on `PATH` counts only if it prints Microsoft's banner (Git for Windows ships a coreutils `link`).
- MSVC links use the DLL CRT, so `/MD` libraries link as they are. The Clang path also passes `/NODEFAULTLIB:libcmt`. Executables reserve an 8 MiB stack. Outputs get `.exe`, `.dll` or `.lib`.
- `#system_library "kernel32"` becomes `-lkernel32`/`kernel32.lib`; CRT and POSIX names (`libc`, `msvcrt`, `m`, `pthread`, ...) are dropped. `#library "foo"` beside the source is `foo.lib`/`libfoo.lib`, and for MinGW also `foo.a`, `libfoo.a`, `foo.dll.a`, `foo.dll`. A DLL must be next to the executable or on `PATH`.
- A member's `#align N` sets alignment even below natural, like `#pragma pack`.

**Runtime** (`stdlib/Runtime_Support/module.jai`): `__jai_runtime_init` replaces ANSI `argv` with UTF-8 copies. Output uses `GetStdHandle` and `WriteFile`. A program with no console (linked with `/SUBSYSTEM:WINDOWS`) has null standard handles: `print` drops its text, as it does on any stream that can't be written, instead of stopping the program. `Runtime_Support_Crash_Handler` installs an unhandled exception filter. `debug_break` is `llvm.debugtrap`, so a failed assertion ends with `0x80000003`.

**Interpreter on Windows** (`interp/native/windows.rs`): loads DLLs with `LoadLibraryW`; symbols with no library resolve in `msvcrt.dll`, `ucrtbase.dll`, `kernel32.dll` and `ntdll.dll`. The `printf` family is inline in UCRT headers, so `lookup` returns stand-ins over `__stdio_common_vsprintf`/`vfprintf`; native builds link `legacy_stdio_definitions`. `Bindings_Generator` loads `libclang.dll` (`JAI_LIBCLANG`, then `C:\Program Files\LLVM\bin`).

### Crash diagnostics and build helpers

`Runtime_Support_Crash_Handler` supports both CPUs: x64 uses `StackWalk64`, arm64 uses `RtlVirtualUnwind2` (Windows 11 / Server 2022 or later). The crash runtime is C built from source, never stored: `tools/native/windows_cross.py build --stdlib` builds the DLL and import library into `stdlib/internal/runtime/crash-native/` before cross-compiling and copies the DLL beside the executables. [BuildCpp](../specs/stdlib/modules/buildcpp.md) and [Autorun](../specs/stdlib/modules/autorun.md) need a Windows host.

## How to change it

- ABI rules: `abi.rs` (tests compare with Clang's IR). Linker discovery and library naming: `linker_command`, `library_args`, `discover.rs`. Start-up: the Windows block at the end of `Runtime_Support/module.jai`.
- Foreign data (`x: T #elsewhere lib`) of a `#system_library` is `dllimport`. C runtime data is `dllimport` on MSVC only when the library is named `ucrt` or `ucrtbase`; `msvcrt.lib` defines some data statically and `dllimport` of it fails to link. MinGW links its runtime statically, so no import.
- `jaic` hands programs a plain working directory and `#file` (`C:\dir\main.jai`), not the `\\?\` form.
- `File.file_open(for_writing = true)` also asks for read access, like POSIX `wb+`.
- Cross builds still run `#run` on the build host with `OS == .WINDOWS`.
- On arm64 Windows, `#cpp_return_type_is_non_pod` results use x0 for the hidden pointer.
- Debug info: MinGW gets DWARF; MSVC gets CodeView and a `.pdb` next to the output (`pdb_args`).
- Third-party libraries: `tools/native/build_native_libs.py` builds them into `target/native-libs/windows-<cpu>/`; MinGW cross builds use static archives found through `JAIC_CROSS_LIBS` ([native libraries](../tools/native-libs.md)). The [Windows probe](../tools/windows-probe.md) reproduces a report on a real runner.

## Configuration

- `-os windows`, `-cpu x64|arm64`, `-target <triple>`.
- `JAIC_LINKER`: a program named `link` or `lld-link` gets `link.exe` arguments; `clang-cl` uses its `/link` boundary; others get C-driver arguments. `JAIC_AR`: archiver.
- Cross builds need MinGW-w64 for x64 (`brew install mingw-w64`, `apt install gcc-mingw-w64-x86-64 g++-mingw-w64-x86-64`) and llvm-mingw for arm64. On Windows you need the Visual Studio build tools or the Windows SDK.
- Building `jaic` itself on Windows against the official LLVM archive: link the static CRT (`CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS=-C target-feature=+crt-static`, or the AARCH64 variable) and run `tools/ci/windows-llvm/prepare.sh` on the archive ([LLVM setup](../tools/llvm-setup.md#windows-msvc-builds)).

## Dependencies

`jaic::abi`, `jaic::toolchain`, `jaic-llvm`, `stdlib/Runtime_Support`. Tests: `tests/compiler/native/windows/runtime.jai`, `windows_runtime_program` in `crates/jaic-cli/tests/native.rs`, `tools/native/windows_cross.py`. CI: the `windows-cross`, `windows-run-cross` and `windows-native` jobs.
