# Native Windows executables

## What it is

`jaic build` writes x86-64 Windows executables (`.exe`), DLLs and static libraries: natively on a Windows host with the Microsoft toolchain, or cross-compiled from macOS and Linux with MinGW-w64 (`jaic build main.jai -os windows`). The program uses the Microsoft x64 calling convention for C calls and the Windows code paths of the stdlib (`OS == .WINDOWS`).

```
$ jaic build hello.jai -os windows -o hello      # on macOS/Linux, writes hello.exe
$ jaic build hello.jai                           # on Windows, writes hello.exe
$ jaic build hello.jai -target x86_64-pc-windows-msvc -o hello   # explicit triple
```

## How it works

Target selection (`crates/jaic-cli/src/main.rs`, `Cli::target_triple`):

- No `-os`/`-target`: the host (LLVM's default triple; `x86_64-pc-windows-msvc` on Windows).
- `-os windows` on a non-Windows host: `x86_64-pc-windows-gnu` (MinGW-w64). Other cross `-os` values are refused for `build` (they still work for `check`/`run`).
- `-target <triple>`: any triple; `OS` and `CPU` are derived from it (`os_and_cpu`), so `CPU == .X64` when cross-building from an arm64 Mac.

The triple goes to `jaic_llvm::Options::target`; `jaic::abi::Arch::from_triple` maps any `windows`/`mingw` x86-64 triple to `Arch::Win64` (see [C ABI](c-abi.md)). Lowering differences for `Win64` in `lower.rs` are small: `#program_export` definitions get `dllexport`, and the `CompilerWrite` intrinsic calls the CRT's `_write`.

Linking (`crates/jaic-llvm/src/lib.rs`, `link` and `LinkFlavor`):

| Flavor | Triple | Linker | C runtime |
|---|---|---|---|
| `MinGw` | `*-windows-gnu`, `*-mingw*` | `x86_64-w64-mingw32-gcc` (or `-clang`; `gcc`/`clang` on a Windows host) | the toolchain's (UCRT with current mingw-w64) |
| `Msvc` | `*-windows-msvc` | `clang --target=...` if on `PATH`, else `lld-link`/`link.exe` | dynamic CRT (`msvcrt.lib`, `oldnames`, `legacy_stdio_definitions`) |

Clang's driver locates the MSVC and Windows SDK libraries itself; `lld-link`/`link.exe` need a developer prompt (`LIB` set). Executables reserve an 8 MiB main stack (`/STACK`, `--stack`) like macOS and Linux. Outputs without an extension get `.exe`, `.dll` or `.lib`.

Libraries: `#system_library "kernel32"` becomes `-lkernel32` / `kernel32.lib`. CRT and POSIX names (`libc`, `c`, `msvcrt`, `ucrt`, `m`, `pthread`, `dl`, `rt`...) are dropped because the toolchain provides the C runtime. A `#library "foo"` next to the source is looked up as `foo.lib`, `libfoo.lib` (MSVC) or `foo.lib`, `foo.a`, `libfoo.a`, `foo.dll.a`, `libfoo.dll.a`, `foo.dll` (MinGW, whose `ld` can link a DLL directly). A DLL must be next to the executable (or on `PATH`) at run time; nothing is copied. When cross-compiling, the host's Homebrew, frameworks and `artifacts/native-libs` directories are not searched. Static libraries are archived with `x86_64-w64-mingw32-ar`, `llvm-ar`, `llvm-lib` or `lib` (`jaic_llvm::archive`).

Struct layout: a member's `#align N` sets its alignment even below the natural one, as `#pragma pack` does in C. `Windows.jai`'s `FILETIME` (`QuadPart: u64 #align 4`) depends on it; without it `WIN32_FIND_DATAW.cFileName` was at byte 48 instead of 44 and directory listings returned garbage names (`tests/stdlib/member-align-lowers-alignment.jai`).

Runtime (`stdlib/Runtime_Support.jai`):

- The C runtime's start-up calls the exported `main(argc, argv)`. `__jai_runtime_init` then replaces the ANSI `argv` with UTF-8 copies from `GetCommandLineW` + `CommandLineToArgvW` (`windows_utf8_arguments`; links `shell32`).
- Output goes through `GetStdHandle` + `WriteFile`; the default allocator, threads (`CreateThread`, critical sections, condition variables), files, time and `Process` use their existing `OS == .WINDOWS` branches.
- `Runtime_Support_Crash_Handler` installs `SetUnhandledExceptionFilter`: it prints the exception code and `RtlCaptureStackBackTrace` addresses, then exits with the exception code. `debug_break` is `int3`, so a failed assertion ends the process with `0x80000003`.

## How to change it

- ABI rules: `crates/jaic/src/abi.rs` (`Arch::Win64` arm, unit tests there).
- Linker discovery, CRT choice, library naming: `LinkFlavor`, `linker_command`, `library_args` in `crates/jaic-llvm/src/lib.rs` (unit tests at the bottom).
- Start-up and console output: `stdlib/Runtime_Support.jai`, Windows block at the end.
- Gotchas:
  - Foreign *data* (`#foreign` variables) imported from a DLL is not marked `dllimport`. MinGW's linker fixes such references up (auto-import); MSVC's does not, so it only works there for statically linked data.
  - The interpreter (`jaic run`, `#run`, metaprograms) on a Windows host loads DLLs with `LoadLibraryW` and calls foreign procedures with the Microsoft x64 convention (`crates/jaic/src/interp/native/windows.rs`): every argument goes through a C-variadic prototype, so the first four reach both their integer and XMM registers. Up to 20 arguments. Library-less symbols and `libc`/`msvcrt` resolve in `msvcrt.dll`, `ucrtbase.dll`, `kernel32.dll`, `ntdll.dll` and the DLLs opened so far. C calling back into interpreted `#c_call` procedures is not implemented there (`callbacks.rs` returns an error).
  - Code compiled for Windows still runs its `#run` blocks on the build host when cross-compiling, with `OS == .WINDOWS`.
  - `#asm` and `#bytes` follow the target CPU; cross builds from arm64 hosts assemble x86-64.

## Configuration

- `-os windows`, `-target <triple>` on `jaic build`.
- `JAIC_LINKER`: linker program to use instead of the search above. A program named `link` or `lld-link` gets `link.exe`-style arguments, anything else C-driver arguments (plus `--target` for MSVC).
- `JAIC_AR`: archiver for static libraries.
- Cross builds need MinGW-w64 (`brew install mingw-w64`, `apt install gcc-mingw-w64-x86-64`).
- On Windows: LLVM (for `clang`; the official `clang+llvm-*-x86_64-pc-windows-msvc` archive) and the Visual Studio build tools or Windows SDK for the libraries.
- Building `jaic` itself on Windows against the official LLVM archive: link the static C runtime (`CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS=-C target-feature=+crt-static`), because that LLVM is built with `/MT` and mixing runtimes crashes `jaic build` at once; and give the linker an `xml2s.lib` (llvm-config names it, the archive does not ship it, nothing jaic uses needs it, so an empty library works). Both workflows do this.

## Dependencies

- `jaic::abi`, `jaic-llvm` (`lib.rs`, `lower.rs`), `stdlib/Runtime_Support.jai`, `stdlib/Runtime_Support_Crash_Handler.jai`, the `OS == .WINDOWS` paths in `Basic`, `File`, `Thread`, `Process`, `Default_Allocator`.
- Tests: `tests/native/windows/runtime.jai` (Basic, String, Hash_Table, File, Thread, small-struct `#c_call`), `tests/stdlib/c-variadic-foreign-calls.jai`, the C struct fixture `tests/native/c-structs-by-value`. `crates/jaic-cli/tests/native.rs` (`windows_runtime_program`) builds the runtime program natively and, when MinGW-w64 is installed, cross-builds it and checks the PE header. `tools/windows_cross.py` builds the corpus runtime cases, those programs and the C fixture (`build`, with `--host` on Windows) and runs them on Windows (`run`).
- CI: `.github/workflows/windows-native.yml` cross-builds on Linux and runs the executables on `windows-2025`, and separately builds `jaic` with LLVM on Windows and runs the same programs plus `cargo test --test native` there.
