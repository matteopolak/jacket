# Changelog

## [Unreleased]

### Added

- Native Windows executables (x86-64): `jaic build` on Windows writes `.exe`/`.dll`/`.lib` files using the Microsoft x64 calling convention, linked with clang (or `lld-link`/`link.exe`) against the MSVC runtime. From macOS and Linux, `jaic build file.jai -os windows` cross-compiles with MinGW-w64; `-target <triple>` picks any LLVM triple. The Windows archive now includes the LLVM backend. See `docs/native/windows.md`.
- Windows on arm64: `jaic build` on an arm64 Windows host writes arm64 executables, DLLs and libraries (MSVC runtime, via clang or `lld-link`), and `jaic build file.jai -os windows -cpu arm64` cross-compiles from macOS and Linux with llvm-mingw (`-target aarch64-pc-windows-msvc` or `aarch64-w64-mingw32` also work). C calls follow AAPCS64 with the Windows variadic rule (every argument of a variadic call in general registers, no float-aggregate treatment). The interpreter there calls C libraries, including variadic ones, and C can call back into interpreted `#c_call` procedures. `Windows.jai` declares the arm64 `CONTEXT`. Releases include a `windows-arm64` archive. `-cpu x64|arm64` is new; `-os windows` alone still means x64 when cross-compiling.
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
- `Bindings_Generator`: with `use_jaic_long_double = false`, 16-byte `long double` members keep their layout as `[16] u8` and functions passing one are stripped with a log line; Objective-C methods passing one (including the x86-64 `objc_msgSend_fpret` case) are always stripped. Printf wrappers are made for any variadic whose last argument is a `char *`.
- `Jaic_Extensions`, a module for jaic-only, non-standard features. Its first is `Long_Double` (`#jaic_type long_double`): C's `long double` with the target's format (x87 80-bit on x86-64 SysV and MinGW, binary128 on arm64 Linux and wasm32, `float64` on Apple arm64 and MSVC). Arithmetic, comparisons and casts run at full precision (native LLVM types in builds, a soft-float in the interpreter that matches x87 hardware bit for bit), and it is passed by the C ABI through `#foreign` and `#c_call`. `print` shows it through `float64`. See `docs/language/jaic-extensions.md`.
- `Bindings_Generator`: `use_jaic_long_double` (default on) binds functions using a 16-byte `long double` with `Long_Double` instead of stripping them.
- `Toolchains`: Android NDK helpers and the macOS SDK path; `Compiler` gains a default minimum macOS version.
- `jaifmt`, a Jai code formatter written in Jai: the `Jai_Format` stdlib module (`format_source`, `parse_config`; no file access, so it also runs in the browser playground through `tools/jaifmt/playground.jai`) and the `tools/jaifmt` command line: indentation, operator and comma spacing, brace placement, blank lines and trailing whitespace. It keeps comments, string literals, here-strings and `#asm` bodies, never wraps lines, supports `jaifmt.toml` (indent width, brace style, ignore globs) and `// jaifmt: off` regions, and refuses to write output whose token stream differs from the input. `--check` for CI, `--stdin` for editors.
- `jaic run file.jai -- args` passes `args` to the program (`get_command_line_arguments`).
- Metaprogram plugins: `jaic check|build file.jai -plug Name [plugin options]` (Tracy, Iprof, ...). `Metaprogram_Plugin` gains the newer `init` hook.
- The upstream corpus pins rluba's libraries (jai-tracy, jai-redis, uniform, cluster, jai-csv, jai-postgres, stubborn, hyperserve, wait_group, jai-date) with 15 sweep cases.

### Changed

- `jaifmt` output is canonical, like rustfmt: exactly zero or one space between tokens (binary operators always spaced, no alignment runs, one space before trailing comments, `.{ a, b }` literals), one statement per line, non-empty `{ }` bodies expanded onto their own lines, and a `{` or `else` on its own line always joined to its header (including `if s == "a"`, `if x ==` switches and blank lines in between). Output is idempotent; `tests/native/debug-info` is excluded because the debugger test pins line numbers.

### Fixed

- Uncalled procedures with notes in imported modules are no longer type checked for intercepting metaprograms (only the program's own files are), so Vk-Engine's `Common` module, which has a stale `@PrintLike` procedure, compiles again.
- A struct member's `#align N` now replaces its natural alignment (it could only raise it), so packed C layouts can be reproduced.
- `New` of an `#align 64` type (and any default-allocator or `rpmalloc` block of 64 bytes or more, including `realloc` and array growth) is now 64-byte aligned; it was only 16-aligned, so such objects were misaligned intermittently. Larger alignments are not guaranteed for heap blocks.
- `#align 64` (and larger) locals are aligned in the interpreter too; they were only aligned relative to their stack frame.
- `continue`/`break` in a `for` body inserted by a `for_expansion` run the expansion's loop defers (`defer i += 1;` before `#insert body;` no longer loops forever).
- `null` binds a type variable before a defaulted baked parameter that names it (`is(null)` with `$cmp: (T, T) -> bool = null`).
- `#add_context name :: value;` is a Context constant (`#Context.name`), not a field.
- Unnamed `#library,link_always "x";` statements are linked.
- Integer sentinels cast to procedure types (`SIG_IGN`) are valid constants; the macOS `SIG_*` dispositions have the handler type, as on Linux.
- `Process`: a captured child's stdin is a socket, and `read_pipe` no longer closes a pipe at end of input.
- `String_Builder`: space ensured in the first buffer survives a reset.

### Removed

- The standalone browser playground UI (`web/scripting-runtime`, the CodeMirror editor and its npm dependencies). The hosted playground is at https://matteopolak.com/playground/jai. The browser bundle now holds `jai_wasm.wasm`, `engine.mjs` (moved to `crates/jai-wasm/js/`), `jaifmt-playground.jai`, `build-metadata.json` and a README, and its manifest is schema v2.

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
