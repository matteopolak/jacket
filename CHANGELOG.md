# Changelog

## [Unreleased]

### Added

- `JAIC_MEMORY_LIMIT=<bytes|nK|nM|nG>`: an exact cap on what `jaic` allocates (compiler, interpreter and the program's C `malloc` calls). Crossing it prints `error: memory limit of N MiB exceeded` and exits with status 120. The corpus sweep uses it for `--memory-limit` instead of sampling resident memory.
- `jailint`, a linter for Jai whose rules run on the type-checked program. The rules are `index_only_loop`, `manual_index_counter`, `unused_variable`, `unused_parameter`, `unused_import`, `redundant_cast`, `bool_comparison`, `format_arg_count`, `shadowed_it`, `defer_in_loop`, `float_equality` (off by default) and `lossy_xx` (off by default).
  - Diagnostics are printed in rustc's layout. `--fix` applies the safe fixes, `-A`/`-W`/`-D` set levels, and the exit status is non-zero when a `deny` rule fires.
  - Settings come from `jailint.toml` (levels and excluded paths). Findings can be suppressed with `// jailint: allow(rule)`, `// jailint: allow-file(rule)` or `@jailint_allow(rule)`.
  - Workspaces a `build.jai` metaprogram creates are linted too.
  - Release archives and the Nix package include `jailint`, and CI lints the repository's Jai code with it. See `docs/tools/jailint.md`.
- jailint rules for likely bugs: `absurd_comparison` (`u >= 0` on an unsigned value), `almost_swapped`, `bitwise_precedence` (operators that group unlike C, `1 << n - 1`), `duplicate_condition`, `erasing_op`, `identical_branches`, `identical_operands`, `infinite_loop` (a `while` whose condition nothing changes), `integer_division_in_float`, `min_max`, `no_effect`, `range_past_count` (`for i: 0..xs.count` indexing `xs[i]`), `remove_in_for`, `reversed_range`, `self_assignment` and `unused_result` (`trim(line);`); and for unidiomatic code: `identity_op`, `manual_assign_op` and `needless_bool`. All `warn` by default. Each rule has its own section in `docs/tools/jailint.md`.
- `jailsp`: lint fixes are preferred quick fixes that carry their diagnostic and the rule (`data.rule`), with sentence-case titles; `source.fixAll.jailint` applies every safe fix at once; diagnostics link to their rule's docs (`codeDescription`); code action requests honour `context.only` and `context.diagnostics`; an open `jailint.toml` (sent with `didOpen`, as the browser build does) configures the lints of the documents below it.
- `jailsp` publishes jailint's findings as diagnostics (the rule as the code, `jailint` as the source) and offers their fixes as quick-fix code actions. It reuses the compile that hover and inlay hints already use.
- `jaic::build::WorkspaceObserver` lets embedders see each workspace's compiler.
- Release binaries (`jaic`, `jailsp`, `jailint`) are built with profile-guided optimisation on every platform, and the Linux ones are further optimised with BOLT. `tools/build_pgo.py` builds them the same way locally. See `docs/tools/pgo-and-bolt.md`.
- `#asm` accepts the F16C, SHA and GFNI extensions: `vcvtph2ps`/`vcvtps2ph` (all imm8 rounding modes, NaNs, subnormals and overflow as on hardware; MXCSR rounding is round-to-nearest), `sha1rnds4`, `sha1nexte`, `sha1msg1`, `sha1msg2`, `sha256rnds2` (`xmm0` as an explicit last operand), `sha256msg1`, `sha256msg2`, and `gf2p8mulb`, `gf2p8affineqb`, `gf2p8affineinvqb` in legacy, VEX and EVEX (masked, broadcast) forms. Like the rest of `#asm` they run on any host CPU.
- Windows x64: C code can call back into `#c_call` procedures running in the interpreter (`jaic run`, `#run`), with the Microsoft x64 convention: arguments by position in RCX/RDX/R8/R9 or XMM0-XMM3, the stack after them, aggregates that are not 1, 2, 4 or 8 bytes by pointer, and large results through the hidden pointer.
- Windows: programs that start threads run in the interpreter. `CreateThread`, waits on thread, semaphore and event handles, critical sections, SRW locks, condition variables, `Sleep` and `SwitchToThread` go through the cooperative thread scheduler.
- MSVC builds write a PDB next to the executable or DLL (`<name>.pdb`) when debug information is on.

- `jaic build -os wasm` compiles to a wasm64 (Memory64) WASI module linked with `wasm-ld`; `-target wasm64-unknown-unknown` leaves out the runtime so the host supplies the imports. See `docs/native/wasm-target.md`.
  - Metaprograms target it with `os_target = .WASM`, `cpu_target = .CUSTOM` and `llvm_options.target_system_triple`/`_cpu`/`_features`; `additional_linker_arguments` go to `wasm-ld`.
  - `#foreign` procedures and `#elsewhere` globals become wasm imports (module `env` without a library, else the library's name), and `#program_export` procedures are exported.
  - `stdlib/Wasi_Runtime`, written in Jai, provides `_start`, the heap, stdio, the environment and the clock on WASI preview 1; jaic adds it when the triple names WASI.
  - `wasm-ld` comes from `JAIC_WASM_LD`, the LLVM install or an LLD package (`lld`, `lld-23`). wasm32 is refused.
- `#intrinsic "llvm.<name>"` on a bodiless procedure calls that LLVM intrinsic in native builds.
- `jaifmt.wasm`: jaifmt compiled to WebAssembly (`jaifmt/wasm.jai`): source on stdin, `--config <toml>` or `JAIFMT_CONFIG`, result on stdout. Browser bundles include it (`build_scripting_wasm.py --jaic`, required by `package_browser_release.py`). It formats about 35 times faster than the interpreted playground driver, and CI checks its output against native jaifmt on every golden case.
- `tools/jaic-diff.py` has a `wasm-native` backend, and `tools/wasi_run.mjs` runs WASI modules under node.

### Changed

- The `jaifmt` program moved from `tools/jaifmt/` to a top-level `jaifmt/` directory, since `tools/` holds repository-maintenance scripts: build it with `jaic build jaifmt/main.jai -O2 -o target/jaifmt`. The formatter library is still `stdlib/Jai_Format`, and the browser bundle's `jaifmt-playground.jai` and `jaifmt.wasm` keep their names.
- Casts no longer push their type into the operand: `cast(float32) (0 - w)` with `w: u16 = 15` subtracts in `u16` (`65521`) and then converts, where it used to compute `-15` in `float32`. Likewise an untyped literal operand takes the other operand's type rather than a declaration's or parameter's (`f: float32 = 0 - w;`). Code that relied on the old float arithmetic needs the cast on the operand: `cast(float32) 0 - w`.
- LLVM 23: the native backend is built against LLVM 23.1 (llvm-sys 231, Inkwell from a pinned commit of its main branch until a crates.io release supports LLVM 23). Building from source needs LLVM 23 and `LLVM_SYS_231_PREFIX` instead of `LLVM_SYS_221_PREFIX`; release archives, CI and the Nix flake use LLVM 23. The `-unroll-add-parallel-reductions=false` workaround for LLVM 22's miscompiled `sub` reductions is gone, since 23.1.0 fixes the unroller.
- `tools/check_dependency_age.py` accepts git dependencies pinned to a full commit hash on GitHub that is at least 14 days old.
- Every `tests/stdlib` program must pass in the browser engine; CI and the browser release fail otherwise. Tests skip what the WASM target lacks (processes, native libraries, windows) themselves, and `tools/playground_stdlib_expected.json` is gone.

### Fixed

- The browser engine's sandbox implements `chmod`/`fchmod`, so `MacOS_Bundler` runs there.
- Standard library: unused variables and imports removed, a shadowed `it` in `Compiler` named, and index loops in `Basic` and `Math` turned into element loops (found by jailint).

## [0.2.0] - 2026-10-06

### Added

- `jailsp`: macro, `#insert`, `#run` and `#if` expansions on hover and as documents, inlay hints (inferred types, parameter names where parameters share a type, `#run` values), format-string checks and hovers, `#import`/`#load` links, references, rename, signature help, folding, code lenses for polymorph instances, workspace symbols and type definitions.
- A language tour (`examples/tour/`) that the browser playground opens by default.
- Fuzzing: cargo-fuzz targets for the lexer, parser, checker, interpreter, generated programs and the language server, with nightly CI and a regression replay on every push.
- A C ABI check compares the stdlib's hand-written system bindings with the real C headers on every CI host (sizes, alignment, field offsets, constants).
- `jaic --timings` prints wall time per compile phase; `tools/compile_bench.py` benchmarks compile times on real projects.
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

- The language server binary is now `jailsp` (was `jai-lsp`); update editor configurations that launch it.
- Binary operator precedence follows Jai instead of C: the bitwise and shift operators (`& | ^ << >> <<< >>>`) share one level that binds tighter than `*`, and `%` binds looser than `*` and `/` but tighter than `+` and `-`. `1 << 2 + 3` is now 7, `1 | 2 & 4` is 0, `10 % 3 * 2` is 4 and `b | a << 8` is `(b | a) << 8`. Code written for C's grouping changes meaning silently; the stdlib and tests were parenthesized. See `docs/language/operators.md`.
- A module parameter without a written type (`FRAMES := 3`) stays an untyped constant when the importer passes an untyped number, so it converts to any integer type its value fits, like `FRAMES :: 3`.
- A prefix `cast(T)` / `xx` now takes a following chain of bitwise and shift operators (`& | ^ << >> <<< >>>`) as part of its value, matching Jai: `cast(float) (hex >> 16) & 0xFF` masks before converting, and `cast(u32) byte << 16` shifts the `u8` before widening. Arithmetic, comparisons and logical operators still apply to the cast's result. Code that relied on the old grouping needs `(cast(T) x) << n`; the stdlib and tests were updated.
- `jaifmt` output is canonical, like rustfmt: exactly zero or one space between tokens (binary operators always spaced, no alignment runs, one space before trailing comments, `.{ a, b }` literals), one statement per line, non-empty `{ }` bodies expanded onto their own lines, and a `{` or `else` on its own line always joined to its header (including `if s == "a"`, `if x ==` switches and blank lines in between). Output is idempotent; `tests/native/debug-info` is excluded because the debugger test pins line numbers.

### Fixed

- `#complete` switches report a missing enum case.
- A struct member with an undefined type is an error; the struct silently came out empty. Every declared struct is now laid out, as in Jai.
- Linux arm64 and other system bindings had x86-64 layouts (`stat`, pthread types, `ipc_perm`, signal contexts, `epoll_event` and more), found by the C ABI check.
- `push_allocator(proc, data)` takes effect.
- Crashes and hangs found by fuzzing: deep nesting, nested `#assert(`, oversized types and allocations, constant-folding overflow, `using` pointer cycles, unbounded polymorphic recursion and `#insert` recursion, pointer cycles in `#run` values.
- A top-level `#insert` that builds declarations from a metaprogram-filled `#placeholder` runs once the placeholder is defined (it could be dropped after a failed retry), and a placeholder reached through an import gives way to its definition. Vk-Engine's Editor module checks again.
- `pointer & int`, `pointer | int` and `pointer ^ int` are defined and keep the pointer type (`cast(u64) p & MASK`).
- `cast(bool)` of a number, enum or pointer tests for non-zero; it kept the low byte, so `!cast(bool) 2` was true.
- Arithmetic on an untyped struct literal passed as an argument takes the parameter's type (`f(.{300, -1} * scale)`).
- A `name:` marker in a declaration statement (`ok, shader:, time := f()`) declares every name; it treated the unmarked names as existing variables.
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
