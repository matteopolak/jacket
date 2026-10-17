# Native Windows executables

## What it is

`jaic` writes Windows executables, DLLs and static libraries for x86-64 and arm64: natively on a Windows host with the Microsoft toolchain, or cross-compiled from macOS and Linux with MinGW-w64 (`jaic main.jai -os windows`, plus `-cpu arm64` for Windows on Arm). C calls use the Microsoft x64 convention, or AAPCS64 with Windows' variadic rule on arm64.

## How it works

Target selection (`Cli::target_triple` in `crates/jaic-cli/src/main.rs`):

- No flags, or ones naming the host: the host triple.
- `-os windows` on a non-Windows host: `x86_64-pc-windows-gnu` (MinGW-w64), or `aarch64-pc-windows-gnu` with `-cpu arm64`. x64 stays the default even on an arm64 Mac, since most Windows machines are x64.
- `-cpu x64|arm64` on a Windows host: the other CPU's MSVC triple (needs that CPU's MSVC libraries). Other cross `-os` values are refused for builds.
- `-target <triple>`: any triple; `OS` and `CPU` are derived from it (`os_and_cpu`).

`jaic::abi::Arch::from_triple` maps a Windows x86-64 triple to `Arch::Win64` and aarch64 to `Arch::Win64Arm` ([C ABI](c-abi.md)). Lowering differences are small: `#program_export` definitions get `dllexport`, `CompilerWrite` calls the CRT's `_write`, and on arm64 the cycle counter reads `cntvct_el0` and `pause` is `yield`. `#asm` is lowered to portable IR, so x64 `#asm` works in arm64 builds ([#asm](../compiler/asm.md)). `#bytes` is never emitted as machine code; only debug-trap encodings are recognized.

Linking (`link` and `LinkFlavor` in `crates/jaic-llvm/src/lib.rs`):

| Flavor | Triple | Linker | C runtime |
|---|---|---|---|
| `MinGw` | `*-windows-gnu` | `<cpu>-w64-mingw32-gcc` or `-clang` (`mingw_cpu`); `gcc`/`clang` on Windows | the toolchain's (UCRT) |
| `Msvc` | `*-windows-msvc` | `clang --target=...` on `PATH`; else `lld-link` or `link.exe` on `PATH`; else Clang or Visual Studio's `link.exe` (`msvc_linker_command`) | dynamic CRT for the target CPU |

GCC's MinGW-w64 only targets x64. For arm64 use [llvm-mingw](https://github.com/mstorsjo/llvm-mingw): unpack a release tarball anywhere and put its `bin/` on `PATH` (append, to keep GCC's MinGW-w64 as the x64 linker).

Clang's driver finds the MSVC and SDK libraries itself. `lld-link`/`link.exe` need `LIB`; without a developer prompt `crates/jaic-llvm/src/msvc.rs` works it out (newest MSVC tool set with `lib\<cpu>\msvcrt.lib`, newest Windows SDK with `ucrt` and `um`) and also finds `link.exe` and `lib.exe`. A `link` on `PATH` is used only if `link /?` prints Microsoft's banner, because Git for Windows and MSYS2 ship a coreutils `link`. Executables reserve an 8 MiB main stack. Outputs without an extension get `.exe`, `.dll` or `.lib`.

Libraries: `#system_library "kernel32"` becomes `-lkernel32` / `kernel32.lib` (names passed in lowercase, since MinGW toolchains on Linux/macOS ship only lowercase `lib<name>.a`). CRT and POSIX names (`libc`, `msvcrt`, `ucrt`, `m`, `pthread`, `dl`, ...) are dropped. A `#library "foo"` next to the source is looked up as `foo.lib`, `libfoo.lib` (MSVC) or also `foo.a`, `libfoo.a`, `foo.dll.a`, `foo.dll` (MinGW). A DLL must be next to the executable or on `PATH` at run time. When cross-compiling, host Homebrew, frameworks and `target/native-libs` are not searched. Static libraries use `*-w64-mingw32-ar`, `llvm-ar`, `llvm-lib` or `lib` (`jaic_llvm::archive`).

A member's `#align N` sets its alignment even below natural, as `#pragma pack` does (`Windows.jai`'s `FILETIME` depends on it, or `WIN32_FIND_DATAW.cFileName` lands at the wrong offset).

Runtime (`stdlib/Runtime_Support/module.jai`): the CRT start-up calls the exported `main`; `__jai_runtime_init` replaces ANSI `argv` with UTF-8 copies from `CommandLineToArgvW`. Output uses `GetStdHandle` + `WriteFile`. `Runtime_Support_Crash_Handler` installs `SetUnhandledExceptionFilter` printing the exception code and addresses. `debug_break` is `llvm.debugtrap`, so a failed assertion ends with `0x80000003` on both CPUs. `Windows.jai` declares `CONTEXT` per CPU.

## How to change it

- ABI rules: `abi.rs` (unit tests compare with Clang's IR). Linker discovery and library naming: `LinkFlavor`, `linker_command`, `library_args` (unit tests at the bottom), `msvc.rs`. Start-up: the Windows block at the end of `Runtime_Support/module.jai`.
- Gotchas:
  - Foreign *data* (`x: T #elsewhere lib`) of a `#system_library` is `dllimport`, loaded through `__imp_x`; data of a `#library` and of the C runtime (`is_windows_c_runtime`) is referenced directly.
  - `jaic` hands programs a plain working directory and `#file` (`C:\dir\main.jai`), not the `\\?\` form `canonicalize` returns (`jaic::canonicalize`), which broke drive-rooted paths.
  - `File.file_open(for_writing = true)` also asks `CreateFileW` for read access, as the POSIX `wb+` allows (`Zip_File_Directory` reads through such a handle).
  - Cross builds still run `#run` blocks on the build host with `OS == .WINDOWS`.
  - On arm64 Windows, C++ `#cpp_return_type_is_non_pod` results use x0 for the hidden pointer, not x8 (`result_in_x0` in `call_with`, `inreg sret` in the backend).
  - Debug info: MinGW targets get DWARF in the executable; MSVC targets get CodeView in objects and a `.pdb` next to the output (`pdb_args`). `-no_debug_info` writes none.
- The interpreter on a Windows host (`crates/jaic/src/interp/native/windows.rs`) loads DLLs with `LoadLibraryW`; library-less symbols resolve in `msvcrt.dll`, `ucrtbase.dll`, `kernel32.dll`, `ntdll.dll` and DLLs opened so far. On x64 it calls with the Microsoft convention (`windows::call`, a variadic prototype so the first four arguments reach both integer and XMM registers, up to 24 arguments) and callbacks go through assembly stubs ([interpreter](../compiler/interpreter.md#native-foreign-calls)). On arm64 it uses the AAPCS64 trampolines plus the variadic rule. `long double` math resolves to the `double` functions (`LONG_DOUBLE_MATH`). The `printf` family is inline in the UCRT headers and missing in msvcrt.dll, so `lookup` returns stand-ins calling the UCRT's `__stdio_common_vsprintf`/`vfprintf` (`stdio` in `native/windows.rs`); native builds link `legacy_stdio_definitions`. Threads use the cooperative scheduler ([interpreter](../compiler/interpreter.md#threads)). `Bindings_Generator` loads `libclang.dll` (`JAI_LIBCLANG`, `C:\Program Files\LLVM\bin`, then the DLL search path).
- Third-party libraries: `tools/build_native_libs.py` builds FreeType, Dear ImGui, MojoShader, the `stb_*`, lz4, meshoptimizer and pl_mpeg libraries and SDL2's import library on Windows into `target/native-libs/windows-<cpu>/` (`<name>.lib` linked by `jaic`, `<name>.dll` loaded by the interpreter); MinGW cross builds get static `lib<name>.a` archives found through `JAIC_CROSS_LIBS` ([third-party native libraries](../tools/native-libs.md)).

## Configuration

- `-os windows`, `-cpu x64|arm64`, `-target <triple>`.
- `JAIC_LINKER`: linker to use instead of the search above. A program named `link` or `lld-link` gets `link.exe`-style arguments (and `LIB` filled in); `clang-cl` uses its `/link` boundary; others get C-driver arguments. `JAIC_AR`: archiver.
- Cross builds need MinGW-w64 for x64 (`brew install mingw-w64`, `apt install gcc-mingw-w64-x86-64 g++-mingw-w64-x86-64`) and llvm-mingw for arm64.
- On Windows: LLVM (official `clang+llvm-*` archive, optional) and the Visual Studio build tools or Windows SDK (ARM64 tools on Windows on Arm). With only Visual Studio installed, `jaic` finds `link.exe` and the libraries without a developer prompt.
- Building `jaic` itself on Windows against the official LLVM archive: link the static C runtime (`CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS=-C target-feature=+crt-static`, or the AARCH64 variable), because that LLVM is built with `/MT` and mixing runtimes crashes `jaic`; and run `tools/windows-llvm/prepare.sh` on the unpacked archive ([LLVM setup](../tools/llvm-setup.md#windows-msvc-builds)).

## Dependencies

`jaic::abi`, `jaic-llvm` (`lib.rs`, `lower.rs`, `msvc.rs`), `stdlib/Runtime_Support`, and the `OS == .WINDOWS` paths in `Basic`, `File`, `Thread`, `Process`, `Default_Allocator`. Tests: `tests/native/windows/runtime.jai`, `tests/native/c-structs-by-value`, `windows_runtime_program` in `crates/jaic-cli/tests/native.rs` (cross-builds when a MinGW compiler is installed and checks the PE machine), `tools/windows_cross.py`. CI: `.github/workflows/windows-native.yml` cross-builds on Linux and runs on `windows-2025` and `windows-11-arm`, and separately builds `jaic` on those runners.
