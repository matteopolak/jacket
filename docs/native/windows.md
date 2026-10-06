# Native Windows executables

## What it is

`jaic build` writes Windows executables (`.exe`), DLLs and static libraries for x86-64 and arm64: natively on a Windows host with the Microsoft toolchain, or cross-compiled from macOS and Linux with MinGW-w64 (`jaic build main.jai -os windows`, plus `-cpu arm64` for Windows on Arm). Programs use the Microsoft x64 calling convention, or on arm64 AAPCS64 with Windows' variadic rule, for C calls, and the Windows code paths of the stdlib (`OS == .WINDOWS`).

```
$ jaic build hello.jai -os windows -o hello              # on macOS/Linux: x64 hello.exe
$ jaic build hello.jai -os windows -cpu arm64 -o hello   # on macOS/Linux: arm64 hello.exe (llvm-mingw)
$ jaic build hello.jai                                   # on Windows: for the host CPU
$ jaic build hello.jai -target aarch64-pc-windows-msvc -o hello   # explicit triple
```

## How it works

Target selection (`crates/jaic-cli/src/main.rs`, `Cli::target_triple`, unit tests at the bottom):

- No `-os`/`-cpu`/`-target`, or ones that name the host: the host (LLVM's default triple; `x86_64-pc-windows-msvc` or `aarch64-pc-windows-msvc` on Windows, depending on which `jaic.exe` runs).
- `-os windows` on a non-Windows host: `x86_64-pc-windows-gnu` (MinGW-w64), or `aarch64-pc-windows-gnu` with `-cpu arm64`. x64 stays the default even on an arm64 Mac: most Windows machines are x64, and Windows on Arm runs x64 programs under emulation.
- `-cpu x64|arm64` on a Windows host: the other CPU's MSVC triple (`aarch64-pc-windows-msvc` from an x64 host and vice versa); it needs that CPU's MSVC libraries installed.
- Other cross `-os` values are refused for `build` (they still work for `check`/`run`, where `-cpu` only sets `CPU`).
- `-target <triple>`: any triple; `OS` and `CPU` are derived from it (`os_and_cpu`), so `CPU == .X64` when cross-building from an arm64 Mac.

The triple goes to `jaic_llvm::Options::target`; `jaic::abi::Arch::from_triple` maps a `windows`/`mingw` x86-64 triple to `Arch::Win64` and an aarch64 one to `Arch::Win64Arm` (see [C ABI](c-abi.md)). LLVM derives the data layout from the triple (`e-m:w-p270:32:32-...` for arm64 Windows, as Clang's). Lowering differences for Windows in `lower.rs` are small: `#program_export` definitions get `dllexport`, the `CompilerWrite` intrinsic calls the CRT's `_write`, and on arm64 the cycle counter reads `cntvct_el0` and `pause` is `yield`, as on other arm64 targets. `#asm` is lowered to portable IR on every CPU (see [#asm](../compiler/asm.md)), so x64 `#asm` blocks still work in arm64 builds.

Linking (`crates/jaic-llvm/src/lib.rs`, `link` and `LinkFlavor`):

| Flavor | Triple | Linker | C runtime |
|---|---|---|---|
| `MinGw` | `*-windows-gnu`, `*-mingw*` | `<cpu>-w64-mingw32-gcc` or `-clang` with `<cpu>` `x86_64` or `aarch64` (`mingw_cpu`); `gcc`/`clang` on a Windows host | the toolchain's (UCRT with current mingw-w64 and llvm-mingw) |
| `Msvc` | `*-windows-msvc` | `clang --target=...` if on `PATH`, else `lld-link`/`link.exe` | dynamic CRT (`msvcrt.lib`, `oldnames`, `legacy_stdio_definitions`) for the target CPU |

GCC's MinGW-w64 only targets x64. For arm64, [llvm-mingw](https://github.com/mstorsjo/llvm-mingw) provides `aarch64-w64-mingw32-clang` (and `-gcc`, `-ar`); it needs no installation step beyond unpacking a release tarball anywhere (no `sudo`) and putting its `bin/` on `PATH`:

```
$ curl -LO https://github.com/mstorsjo/llvm-mingw/releases/download/20260922/llvm-mingw-20260922-ucrt-macos-universal.tar.xz
$ tar -xJf llvm-mingw-20260922-ucrt-macos-universal.tar.xz -C ~/.local/opt
$ export PATH="$PATH:$HOME/.local/opt/llvm-mingw-20260922-ucrt-macos-universal/bin"
```

(Linux: the `ucrt-ubuntu-22.04-x86_64` or `-aarch64` tarball.) Append it to `PATH` rather than prepend it if GCC's MinGW-w64 should stay the x64 linker; llvm-mingw also ships `x86_64-w64-mingw32-*` and either works.

Clang's driver locates the MSVC and Windows SDK libraries itself; `lld-link`/`link.exe` need a developer prompt (`LIB` set). Executables reserve an 8 MiB main stack (`/STACK`, `--stack`) like macOS and Linux. Outputs without an extension get `.exe`, `.dll` or `.lib`.

Libraries: `#system_library "kernel32"` becomes `-lkernel32` / `kernel32.lib`. CRT and POSIX names (`libc`, `c`, `msvcrt`, `ucrt`, `m`, `pthread`, `dl`, `rt`...) are dropped because the toolchain provides the C runtime. A `#library "foo"` next to the source is looked up as `foo.lib`, `libfoo.lib` (MSVC) or `foo.lib`, `foo.a`, `libfoo.a`, `foo.dll.a`, `libfoo.dll.a`, `foo.dll` (MinGW, whose `ld` can link a DLL directly). A DLL must be next to the executable (or on `PATH`) at run time; nothing is copied. When cross-compiling, the host's Homebrew, frameworks and `artifacts/native-libs` directories are not searched. Static libraries are archived with `x86_64-w64-mingw32-ar` (`aarch64-w64-mingw32-ar` for arm64), `llvm-ar`, `llvm-lib` or `lib` (`jaic_llvm::archive`).

Struct layout: a member's `#align N` sets its alignment even below the natural one, as `#pragma pack` does in C. `Windows.jai`'s `FILETIME` (`QuadPart: u64 #align 4`) depends on it: otherwise `WIN32_FIND_DATAW.cFileName` lands at byte 48 instead of 44 and directory listings return garbage names (`tests/stdlib/member-align-lowers-alignment.jai`).

Runtime (`stdlib/Runtime_Support.jai`):

- The C runtime's start-up calls the exported `main(argc, argv)`. `__jai_runtime_init` then replaces the ANSI `argv` with UTF-8 copies from `GetCommandLineW` + `CommandLineToArgvW` (`windows_utf8_arguments`; links `shell32`).
- Output goes through `GetStdHandle` + `WriteFile`; the default allocator, threads (`CreateThread`, critical sections, condition variables), files, time and `Process` use their existing `OS == .WINDOWS` branches.
- `Runtime_Support_Crash_Handler` installs `SetUnhandledExceptionFilter`: it prints the exception code and `RtlCaptureStackBackTrace` addresses, then exits with the exception code. It only reads the exception code, so it is the same on both CPUs. `debug_break` becomes LLVM's `llvm.debugtrap` (`int3` on x64, `brk #0xf000` on arm64 Windows), so a failed assertion ends the process with `0x80000003` on both.
- `Windows.jai` declares `CONTEXT` (and the `CONTEXT_*` flags) per CPU: the x64 layout (1232 bytes, `Rip`, `Rsp`...) or the arm64 one (912 bytes, `X[29]`, `Fp`, `Lr`, `Sp`, `Pc`, `V[32]`...), for `GetThreadContext`, `RtlCaptureContext` and `EXCEPTION_POINTERS.ContextRecord`.

## How to change it

- ABI rules: `crates/jaic/src/abi.rs` (`Arch::Win64` and `Arch::Win64Arm`, `classify_vararg`; unit tests there compare with Clang's IR).
- Linker discovery, CRT choice, library naming: `LinkFlavor`, `linker_command`, `library_args` in `crates/jaic-llvm/src/lib.rs` (unit tests at the bottom).
- Start-up and console output: `stdlib/Runtime_Support.jai`, Windows block at the end.
- Gotchas:
  - Foreign *data* (`x: T #elsewhere lib`) of a `#system_library` is declared `dllimport`, so the code loads it through the DLL's `__imp_x` pointer: import libraries define no `x` for data, and MSVC's linker does not fix such references up as MinGW's does. Data of a `#library` (a static library next to the source) and of the C runtime (`libc`, `msvcrt`...; `is_windows_c_runtime`) is referenced directly. `tests/stdlib/rules-external-data.jai` reads the C runtime's `_HUGE`.
  - System library names are passed to the linker in lowercase (`#system_library "Gdi32"` links `gdi32`): Windows ignores case, but MinGW toolchains on Linux and macOS ship only lowercase `lib<name>.a` files.
  - `jaic run` and `jaic build` hand programs a plain working directory and `#file` (`C:\dir\main.jai`), not the `\\?\` form `std::fs::canonicalize` returns (`jaic::canonicalize`): a drive-rooted `/tmp/x` resolved against a verbatim directory names no folder, so writes there failed under `jaic run`.
  - The interpreter (`jaic run`, `#run`, metaprograms) on a Windows host loads DLLs with `LoadLibraryW` (`crates/jaic/src/interp/native/windows.rs`). Library-less symbols and `libc`/`msvcrt` resolve in `msvcrt.dll`, `ucrtbase.dll`, `kernel32.dll`, `ntdll.dll` and the DLLs opened so far.
    - On x64 it calls foreign procedures with the Microsoft x64 convention (`windows::call`): every argument goes through a C-variadic prototype, so the first four reach both their integer and XMM registers. Up to 20 arguments. C calls back into interpreted `#c_call` procedures through assembly stubs that read the four argument registers by position (`callbacks/win64.rs`; see [the interpreter](../compiler/interpreter.md#callbacks-from-c)).
    - On arm64 it uses the AAPCS64 trampolines that macOS and Linux arm64 use (`call_with` in `native.rs`), plus the Windows variadic rule: when the callee is variadic, every argument (floats as their bits, aggregates as integer pieces or a pointer to a copy) goes to x0-x7 and then the stack. Callbacks from C into interpreted `#c_call` procedures work there as on other arm64 hosts, since non-variadic Windows arm64 calls are plain AAPCS64.
    - `long double` math (`sqrtl`, `fmaxl`...) resolves to the `double` functions: `long double` is `double` on Windows and the C runtime exports no `l` names (`LONG_DOUBLE_MATH` in `native/windows.rs`). `snprintf` and the other C99 `printf` family are header-inline in the UCRT, so `jaic run` cannot call them (native builds link them).
    - Threads: `CreateThread`, critical sections, condition variables, `WaitForSingleObject` and the rest of what `Thread` uses run on the interpreter's cooperative scheduler (see [interpreter threads](../compiler/interpreter-threads.md#win32-windows-hosts)), so programs that start threads run under `jaic run`/`#run` on both CPUs.
  - `Bindings_Generator` on Windows loads `libclang.dll` (`JAI_LIBCLANG`, `C:\Program Files\LLVM\bin`, then the DLL search path) with `LoadLibraryA` (`crates/jaic/src/clang.rs`).
  - Debug information: MinGW targets get DWARF in the executable. MSVC targets get CodeView in the objects, and the linker collects it into `<output stem>.pdb` next to the executable or DLL (`/DEBUG /PDB:... /INCREMENTAL:NO /OPT:REF`, from `pdb_args` in `crates/jaic-llvm/src/lib.rs`; through Clang's driver as `-Wl,`). `--no-debug-info` writes none. No `.dSYM` is written for non-macOS targets.
  - `File.file_open(for_writing = true)` asks `CreateFileW` for read access too, as the POSIX side's `wb+`/`rb+` allow reading: `Zip_File_Directory` reads the archive through such a handle, and a write-only handle made it fail on Windows (`tests/stdlib/file-io-roundtrip.jai` checks the read).
  - Code compiled for Windows still runs its `#run` blocks on the build host when cross-compiling, with `OS == .WINDOWS`.
  - `#bytes` is never emitted as machine code: only the debug-trap encodings (`0xCC`, arm64 `brk #1`) are recognized, as `llvm.debugtrap`, so the stdlib's `debug_break` works on every CPU. `#asm` is portable too (lowered to IR).
  - C++ `#cpp_return_type_is_non_pod` results on arm64 Windows: MSVC passes that hidden result pointer in x0 rather than x8 (Clang's `inreg sret`). The interpreter does so (`result_in_x0` in `call_with`) and the LLVM backend marks that parameter `inreg sret` (see [C ABI](c-abi.md)).

## Configuration

- `-os windows`, `-cpu x64|arm64`, `-target <triple>` on `jaic build`.
- `JAIC_LINKER`: linker program to use instead of the search above. A program named `link` or `lld-link` gets `link.exe`-style arguments, anything else C-driver arguments (plus `--target` for MSVC).
- `JAIC_AR`: archiver for static libraries.
- Cross builds need MinGW-w64 (`brew install mingw-w64`, `apt install gcc-mingw-w64-x86-64`) for x64, and llvm-mingw (above) for arm64.
- On Windows: LLVM (for `clang`; the official `clang+llvm-*-x86_64-pc-windows-msvc` or `clang+llvm-*-aarch64-pc-windows-msvc` archive) and the Visual Studio build tools or Windows SDK for the libraries (the ARM64 build tools on Windows on Arm).
- Building `jaic` itself on Windows against the official LLVM archive: link the static C runtime (`CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS=-C target-feature=+crt-static`, or `CARGO_TARGET_AARCH64_PC_WINDOWS_MSVC_RUSTFLAGS` on arm64), because that LLVM is built with `/MT` and mixing runtimes crashes `jaic build` at once; and run `tools/windows-llvm/prepare.sh` on the unpacked archive, which supplies the `xml2s.lib`, `zs.lib` and `zstd_static.lib` that llvm-config names but the archive lacks, and an `llvm-config` front that turns zstd's absolute build-machine path into a name llvm-sys can pass to rustc ([LLVM setup](../tools/llvm-setup.md#windows-msvc-builds)). Both workflows do this.

## Dependencies

- `jaic::abi`, `jaic-llvm` (`lib.rs`, `lower.rs`), `stdlib/Runtime_Support.jai`, `stdlib/Runtime_Support_Crash_Handler.jai`, the `OS == .WINDOWS` paths in `Basic`, `File`, `Thread`, `Process`, `Default_Allocator`.
- Tests: `tests/native/windows/runtime.jai` (Basic, String, Hash_Table, File, Thread, small-struct `#c_call`), `tests/stdlib/c-variadic-foreign-calls.jai`, the C struct fixture `tests/native/c-structs-by-value`. `crates/jaic-cli/tests/native.rs` (`windows_runtime_program`) builds the runtime program natively and, when `x86_64-w64-mingw32-gcc` or `aarch64-w64-mingw32-clang` is installed, cross-builds it for that CPU and checks the PE header's machine. `tools/windows_cross.py` builds the corpus runtime cases, those programs and the C fixture (`build`, with `--host` on Windows and `--cpu arm64` for llvm-mingw cross builds) and runs them on Windows (`run`).
- CI: `.github/workflows/windows-native.yml`, for each CPU: cross-builds on Linux and runs the executables on Windows (`windows-2025`, `windows-11-arm`), and separately builds `jaic` with LLVM on that Windows runner and runs the same programs plus `cargo test --test native` there.
