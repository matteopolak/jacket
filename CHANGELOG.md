# Changelog

## [Unreleased]

### Added

- Native Windows executables (x86-64): `jaic build` on Windows writes `.exe`/`.dll`/`.lib` files using the Microsoft x64 calling convention, linked with clang (or `lld-link`/`link.exe`) against the MSVC runtime. From macOS and Linux, `jaic build file.jai -os windows` cross-compiles with MinGW-w64; `-target <triple>` picks any LLVM triple. The Windows archive now includes the LLVM backend. See `docs/native/windows.md`.
- Windows runtime: UTF-8 command-line arguments, a crash handler that prints the exception and a backtrace, and `#program_export` procedures exported from DLLs.
- The interpreter on Windows loads DLLs and calls foreign procedures (`#run`, `jaic run` and metaprograms that use C libraries). Callbacks from C into interpreted code are not supported there yet.
- Native debug information: `jaic build` emits DWARF (a `.dSYM` next to the executable on macOS) with source lines, procedure names and typed locals, parameters and globals, so lldb and gdb can set breakpoints by `file.jai:line`, step and print variables. On by default; `--no-debug-info` or `Build_Options.emit_debug_info = .NONE` turns it off.
- `#asm`: division (`div`/`idiv`, trapping like hardware), widening (`cqo`, `cdqe`, ...), string instructions with `rep_`/`repe_`/`repne_` prefixes and the direction flag, `shld`/`shrd`, `rcl`/`rcr`, BMI1/BMI2, `adcx`/`adox`, `crc32`, the parity flag, `lahf`/`sahf`, `xlat` and `cmpxchg8b`/`cmpxchg16b`.
- `#asm`: AVX-512 op-mask registers (`omr`) with the `k*` instructions, compares into masks, merge/zero masking on every vector instruction and masked stores.
- `#asm`: a much wider SIMD set: saturating and horizontal arithmetic, FMA, shuffles, permutes, blends, unpacks, inserts/extracts, packs, conversions, compress/expand, ternary logic, scatter, AES and `pclmulqdq`. SSE–AVX2 results were checked against x86-64 hardware; AVX-512 has hand-computed tests.
- `#asm` accepts every CPUID feature name as a modifier (`#asm AVX512_VBMI`, `GFNI`, ...).
- `Bindings_Generator` matches the public Jai modules's public API and output: system types are referred to by name, unnamed parameters become `unknownN`, comments are placed before or after a declaration like the reference, `extern "C"` blocks are walked, packed members get `#align`, and macros that reference enum constants are dropped or rewritten. The generators shipped with the public Jai modules (Curl, lz4, stb, POSIX, Socket, nvtt, macho, CoreFoundation) run under `jaic` and their output type checks.
- `Bindings_Generator`: C++ virtual bases (own vtable pointer, base placed at clang's offset, offset checks) and renamed overloads that are equal in Jai, with `/*const reference*/` comments.
- `Bindings_Generator`: `Block_X_literal` constructors turn Jai `#c_call` procedures into Objective-C blocks (`objc_make_block` and `objc_block_user_data` in `Objective_C`).
- `Bindings_Generator`: 16-byte `long double` members keep their layout as `[16] u8`, and functions or Objective-C methods passing one (including the x86-64 `objc_msgSend_fpret` case) are stripped with a log line. Printf wrappers are made for any variadic whose last argument is a `char *`.
- `Toolchains`: Android NDK helpers and the macOS SDK path; `Compiler` gains a default minimum macOS version.
- `jaifmt`, a Jai code formatter written in Jai: the `Jai_Format` stdlib module (`format_source`, `parse_config`; no file access, so it also runs in the browser playground through `tools/jaifmt/playground.jai`) and the `tools/jaifmt` command line: indentation, operator and comma spacing, brace placement, blank lines and trailing whitespace. It keeps comments, string literals, here-strings and `#asm` bodies, never wraps lines, supports `jaifmt.toml` (indent width, brace style, ignore globs) and `// jaifmt: off` regions, and refuses to write output whose token stream differs from the input. `--check` for CI, `--stdin` for editors.
- `jaic run file.jai -- args` passes `args` to the program (`get_command_line_arguments`).

### Fixed

- A struct member's `#align N` now replaces its natural alignment (it could only raise it), so packed C layouts can be reproduced.

## [0.1.0] - 2026-10-05

The first release of `jaic`, an independent Jai compiler written in Rust.

### Downloads

| Archive | Contents |
| --- | --- |
| `jaic-macos-arm64.tar.gz` | `jaic` with the LLVM backend (check, run, build), `jai-lsp`, `stdlib/` |
| `jaic-linux-x64.tar.gz` | `jaic` with the LLVM backend (check, run, build), `jai-lsp`, `stdlib/` |
| `jaic-windows-x64.zip` | `jaic` without a native backend (check, run), `jai-lsp`, `stdlib/` |

Unpack the archive and run `jaic` from it. It finds the `stdlib/` folder next to the executable (or the folder in `JAIC_STDLIB`). LLVM is linked into the macOS and Linux builds, so it does not need to be installed. Native builds call the system linker: Xcode's command line tools on macOS, `cc` on Linux.

### Compiler

- Type-checks, interprets (`jaic run`) and natively builds (`jaic build`) Jai programs.
- Language: polymorphic procedures and structs, baking, `#modify`, macros, `#code`/`#insert`, custom `for_expansion`, `using`, `context`, `Any` and type info.
- Compile-time execution (`#run`) in the IR interpreter, including calls into native libraries.
- Metaprograms: the `Compiler` module's workspaces, message loop and build options.
- Arithmetic overflow and array bounds checks, stack traces.
- `#asm` for common x86-64 and arm64 instructions.
- `Bindings_Generator` for C, C++ and Objective-C headers, through libclang (loaded at run time when present).

### Standard library

An independently written standard library: `Basic`, `String`, `Hash_Table`, `File`, `Process`, `Thread`, `Math`, `Compiler`, `Bindings_Generator`, `Objective_C`, `GetRect`, `Simp`, `Window_Creation` and more. See `docs/stdlib/` in the repository.

### Tooling

- `jai-lsp`: a language server with diagnostics, type-checked hover and completion, and go to definition.
- A browser playground: the same compiler built for WebAssembly.

### Compatibility

These projects check, run or build: Focus (builds and runs on macOS), Jails, jaison, sgpu, the examples from *The Way to Jai*, and the reference `how_to` programs. Vk-Engine checks for Linux.

### Known limitations

- No native executables for Windows targets yet (no Win64 calling convention). Windows programs can be checked (`-os windows`), and the Windows build of `jaic` checks and interprets.
- No native debug information.
- `#asm` rejects string operations, division, x87 and mask registers.
