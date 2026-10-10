# Changelog

## [Unreleased]

## [0.8.0] - 2026-10-10

Switches on constants are now checked like any other switch, so mistakes in them are caught and the editor understands every case.

### Highlights

- Every case of an `if x == {` switch is type-checked even when `x` is a constant, so the language server shows hover, go to definition and highlighting in all of them.

### Breaking changes

| Was | Now |
| --- | --- |
| A switch on a compile-time constant checked only the matching case, and skipped `#complete` | Every case is checked and `#complete` applies; use `#if` to skip code |

## [0.7.1] - 2026-10-10

Faster debug builds: programs built without optimization run up to 1.7x faster, and they compile a little faster too.

### Highlights

- Call-heavy code built at `-O0` runs about 1.6x faster (e.g. a recursive `fib(40)` takes 1.0 s instead of 1.6 s), and code that returns structs by value about 1.7x faster.
- Stack traces cost less: procedures that call nothing no longer record a frame.
- Small struct copies, and division or shifts by a constant, compile to shorter code.

## [0.7.0] - 2026-10-09

Smaller downloads, more checks that catch mistakes, and more of your program visible to compile-time code.

![Release archive sizes, 0.6.2 against 0.7.0: every platform is smaller](https://raw.githubusercontent.com/matteopolak/jai/main/docs/images/releases/0.7.0-download-size.svg)

### Highlights

- Release archives are 4 to 18 MiB smaller on every platform.
- New warnings for code after `return`, `break` or `continue`, and for procedures that can end without returning a value.
- `get_type_table()` works in compile-time code and lists every declared type.
- Type info marks local structs and lists every procedure constant of a struct.
- `get_build_options()` reports real output paths before a metaprogram changes anything.
- Compiler messages now include procedures nothing calls, and file-level declarations say so.

### Breaking changes

| Was | Now |
| --- | --- |
| A name declared twice in one scope was accepted | An error (procedures still overload) |
| An unknown or mistyped module import argument was ignored | An error |
| `#import "M"` after `#import "M"(FLAG = true)` shared the configured instance | A separate instance with the default parameters |
| `if true return 1;` or `while true {}` as the last statement counted as returning | Warns `not all control paths return a value`; end such loops with a `return` |
| Release binaries kept function names in backtraces | Stripped; build from source for full backtraces |

## [0.6.2] - 2026-10-09

Programs built from source behave better, and type info reports more.

### Fixes

- Focus built from source takes keyboard input, and its open-file dialog no longer aborts.
- A GetRect subwindow's close button closes it.
- `array_view` with a count past the end returns what exists instead of failing.
- Type info reports `#no_padding` structs, structs whose members are all uninitialized, and notes on struct constants.
- A context hook added with `#add_context` shows up in the context's type info before anything uses it.

## [0.6.1] - 2026-10-09

The language server now copes with real projects and very large files, and uses a fraction of the memory.

![Language server peak memory, 0.6.0 against 0.6.1: down by a factor of two to five](https://raw.githubusercontent.com/matteopolak/jai/main/docs/images/releases/0.6.1-language-server-memory.svg)

### Highlights

- Memory use on large files is down by a factor of two to five.
- Typing stays responsive: diagnostics wait until you pause, and completion no longer recompiles.
- Files up to 32 MiB open; before, large files lost hover, symbols and diagnostics.
- Go to definition reaches struct fields, enum members and the one overload a call uses.
- `.` completes enum members wherever the type is known.
- Projects without a `jai.toml` find their `modules/` folders.
- A large real project (Focus) now gets its first diagnostics in about 0.3 seconds. It never finished before.

### Fixes

- Signature help works for calls through procedure-typed variables and members.
- Workspace symbols list files you haven't opened.
- A `jai.toml` or `jailint.toml` that doesn't parse is reported instead of ignored silently.
- A crash while answering one request no longer ends the server.
- Sanitized builds on Intel Macs keep `file:line` for every frame.

Language server timings can be measured with [the benchmark script](https://github.com/matteopolak/jai/blob/main/docs/tools/lsp-benchmark.md).

## [0.6.0] - 2026-10-09

Builds with optimizations on compile up to 2.7 times faster, and unoptimized builds of large programs about 1.3 times faster.

![Optimized build times, 0.5.1 against 0.6.0: Focus 12.7 to 7.8 seconds, chess-jai 4.5 to 1.7 seconds](https://raw.githubusercontent.com/matteopolak/jai/main/docs/images/releases/0.6.0-build-time.svg)

### Highlights

- Faster builds: see the chart, and [compile speed](https://github.com/matteopolak/jai/blob/main/docs/compiler/compile-speed.md) for how.
- `jailsp` completes inside range expressions such as `for i: 0..table.co`.
- `jailint`'s `index_only_loop` also catches loops that fill elements from their index, and offers a fix.
- `self_assignment` and `absurd_comparison` get editor quick fixes.
- Compiler messages `PERFORMANCE_REPORT` and `DEBUG_DUMP` are sent, so the `Performance_Report` plugin works. See [build options](https://github.com/matteopolak/jai/blob/main/docs/metaprogramming/build-options.md).
- The playground tour runs every stop in order, without waiting for input.

### Breaking changes

| Was | Now |
| --- | --- |
| `enable_split_modules = false` built an optimized program as one module | Still split into parts; set `JAIC_CODEGEN_UNITS=1` for whole-program optimization |
| `File_Change.time_of_last_change` was a `float32` | A `float64` |
| `index_only_loop` skipped loops that write elements from the index | Reports them, so a clean lint run may now warn |
| The playground tour took `--stop`, `--all` and `--help` | It always runs every stop |

### Fixes

- The GPU stop of the tour draws in its own window instead of aborting; without a display it says it skipped. See [WebGPU](https://github.com/matteopolak/jai/blob/main/docs/stdlib/webgpu.md).
- `File_Watcher` no longer misses a change because of float rounding.
- Windows: `JAIC_LINKER` finds the SDK and C runtime libraries outside a developer prompt.

## [0.5.1] - 2026-10-09

Unoptimized builds and compile-time code both run about twice as fast, and release archives carry more libraries.

### Highlights

- Unoptimized builds compile 1.7 to 2.4 times faster on Apple silicon.
- Compile-time code (`#run`, `jaic run`) is about twice as fast, and calls into C three times.
- Release archives include FreeType, Dear ImGui and MojoShader on every platform, and SDL2 on Windows x64, so programs using them need no setup.
- Library licences ship in each archive.
- On macOS, unoptimized builds no longer run `dsymutil`; objects stay in `.build/` beside the program, so add it to your `.gitignore`.

### Fixes

- C functions that take or return a `Vector2` by value now get the right values (every Dear ImGui function returning an `ImVec2`).
- `-sanitize` reports on Intel Macs show `file.jai:line` again.

See [compile speed](https://github.com/matteopolak/jai/blob/main/docs/compiler/compile-speed.md) for what changed.

## [0.5.0] - 2026-10-09

Programs get checked more thoroughly: `print` format strings, null pointers, notes and clashing names. There is also a command-line parser for your own tools and a much bigger set of editor refactorings.

![jaic warns that a print call has too few arguments for its format string](https://raw.githubusercontent.com/matteopolak/jai/main/docs/images/releases/0.5.0-print-check.svg)

### Highlights

- `print`-style format strings are checked at compile time. See [format string check](https://github.com/matteopolak/jai/blob/main/docs/compiler/format-string-check.md).
- Built programs stop with `null pointer dereference` at the offending line instead of crashing.
- New [`Extensions/Args`](https://github.com/matteopolak/jai/blob/main/docs/stdlib/args.md): declare a struct and get a command-line parser with help, suggestions and shell completions. `jaifmt` uses it.
- Editor refactorings: extract into variable or procedure, inline variable, add missing cases or fields, convert `ifx`.
- Call hierarchy, expand selection, pull diagnostics, and rename of struct fields and enum members. See [refactorings](https://github.com/matteopolak/jai/blob/main/docs/compiler/language-server-refactorings.md).
- A procedure that can end without returning a value warns. See [diagnostics](https://github.com/matteopolak/jai/blob/main/docs/compiler/diagnostics.md#compile-warnings-compilerwarnings).
- New `just` commands for building, formatting, linting and testing the repository. See [Justfile](https://github.com/matteopolak/jai/blob/main/docs/tools/justfile.md).

### Breaking changes

| Was | Now |
| --- | --- |
| `print("% %\n", a)` ran | A warning; the build goes on |
| `jailint` rule `format_arg_count` | Removed, since `jaic` reports it; delete it from `jailint.toml` |
| `a: int @tag;`, clashing `using` members, identical overloads | Errors; write notes after the semicolon: `a: int; @tag` |
| `jailsp` pushed diagnostics to every client | Clients that ask for pull diagnostics are no longer pushed to |
| Windows `File.handle` was an `s64` | A `HANDLE` |
| Null reads and writes in built programs were not checked | Checked; `Build_Options.null_pointer_check = .OFF` removes the check |

### Fixes

- `jaifmt` help and error messages are clearer, and every flag has a `--no-` form.
- `jailsp` shows type errors for every declaration, not just the first.
- Names from files you have not opened are visible to the language server.
- `jailint` given a file that another program `#load`s now lints it as part of that program.
- C structs now pass correctly in more cases, including packed structs and structs through `...`. See [C ABI](https://github.com/matteopolak/jai/blob/main/docs/native/c-abi.md).
- New `File.file_read_line` and `File.read_stdin_line`; `exit(n)` works in the playground.
- More `Compiler` module procedures work, and more directives reach metaprograms as their own node kinds. See [the Compiler module](https://github.com/matteopolak/jai/blob/main/docs/metaprogramming/compiler-module.md).
- More `#asm` instructions, and `offset_of` and `is_value_type` are implemented.
- Windows: `#file` and `get_absolute_path` use `/`, and a metaprogram's custom link command handles five or more libraries.
- `rpmalloc` no longer crashes when a new thread allocates.

## [0.4.3] - 2026-10-08

Numbers and `print` now behave as Jai programs expect, mistakes that used to compile are errors, and the language server can add imports for you.

### Highlights

- Auto-import: type part of a name you haven't imported and accept it; the `#import` or `#load` is added. See [auto-import](https://github.com/matteopolak/jai/blob/main/docs/compiler/language-server.md#auto-import-completion).
- A new `jai.toml` names a project's build files and module folders when they can't be inferred.
- Number constants must fit their type: `x: u32 = -1;` and `cast(u8) 300` are errors.
- `print` handles negative numbers in other bases, rounding, `Inf`/`NaN` and unnamed enum values correctly.
- Programs using Simp run from an installed toolchain; the archives now include the image and sound libraries.
- VS Code formats Jai files again after the extension rename.

### Breaking changes

| Was | Now |
| --- | --- |
| `x: u32 = -1;`, `x: s16 = 40000;`, `cast(u8) 300` | Errors; use `cast,trunc(T)` for the low bits |
| A float literal such as `16777216.0` was `float32` | 8 or more significant digits make it `float64` |
| `true + 1`, `a, b := f(1)`, `*5`, a bare `defer;`, repeated member names | Errors |
| `make_look_at_matrix(..., x_is_forward = false)` pointed the camera's right at `-x` | `+x` |

### Fixes

- `Hash_Table.init` and `Random` float draws behave as expected.
- Windows: `jaic build` without Clang uses Visual Studio's linker, not the `link` that Git for Windows installs.
- More invalid programs are reported: bad `using`, enum values out of range, assigning into a constant array.

## [0.4.2] - 2026-10-08

Intel Mac and arm64 Linux builds, programs that run on any CPU of their family, and build options that are either honoured or reported.

### Highlights

- New archives for Intel Macs (`jai-macos-x64.tar.gz`) and arm64 Linux (`jai-linux-arm64.tar.gz`).
- Built programs target a baseline CPU, so they no longer crash with an illegal instruction on older machines. Set `llvm_options.target_system_cpu = "native"` to tune for your own.
- Build options `jaic` can't act on now warn, and impossible ones are errors.
- Metaprograms can link a workspace themselves with `use_custom_link_command`.
- Several open-source Jai projects build and pass their tests.

### Breaking changes

| Was | Now |
| --- | --- |
| VS Code extension `matteopolak.jai` | `matteopolak.jai-toolchain`; install it and uninstall the old one (it offers to) |
| `ERROR_CONTINUABLE` reports let the build pass | They fail the build |
| Built programs used the build machine's CPU | The baseline CPU |

See [build options](https://github.com/matteopolak/jai/blob/main/docs/metaprogramming/build-options.md) for what is supported.

### Fixes

- The 0.4.1 Linux `jaifmt` crashed on machines without AVX-512.
- A struct pointer passes where its `#as` pointer member is expected.
- A by-value `for` loop's `it` is the element itself.
- `push_context` without a context works again.
- Passing a null `p.*` as an `Any` no longer stops the program early.
- `File.{ stdin }` works on macOS and Linux, and a `#!` first line is skipped.

## [0.4.1] - 2026-10-07

WebGPU graphics on every platform and in the playground, threads in WebAssembly builds, and many crash fixes.

### Highlights

- New `Extensions/WebGPU` module: one program draws in a window on macOS, Linux and Windows, and in the playground's new **Render** pane. See [WebGPU](https://github.com/matteopolak/jai/blob/main/docs/stdlib/webgpu.md).
- Release archives ship the WebGPU library; nothing else to install.
- New examples: a triangle, a ray marcher and a compute shader.
- `jaic build -os wasm` programs can use threads. See [threads in WASI builds](https://github.com/matteopolak/jai/blob/main/docs/native/wasm-threads.md).
- The playground tour has four new stops, and VS Code highlights `#string WGSL`, `GLSL`, `SQL` and `JSON` bodies.
- The extension is listed as **Jai Toolchain** on the VS Code Marketplace.

### Breaking changes

| Was | Now |
| --- | --- |
| `#import "Jaic_Extensions";`, `"Jai_Format"`, `"Wasi_Runtime"` | `#import "Extensions/Long_Double";`, `"Extensions/Jai_Format"`, `"Extensions/Wasi_Runtime"` |
| `jaic-<platform>` archives | `jai-<platform>` |
| `JAIC_VERSION`, `JAIC_INSTALL_DIR`, `JAIC_BIN_DIR` | `JAI_VERSION`, `JAI_INSTALL_DIR`, `JAI_BIN_DIR` (the old names still work) |

An import of a moved module fails with an error naming the import to write. See [stdlib extensions](https://github.com/matteopolak/jai/blob/main/docs/stdlib/extensions.md).

### Fixes

- The compiler no longer crashes on a bad address from a `#run` or metaprogram.
- Metaprograms that add code every round no longer slow down quadratically.
- `print("%", p.*)` with a null `p` stops with a clear error.
- `jailsp` no longer crashes on builtin constants such as `OS`.

## [0.4.0] - 2026-10-07

A VS Code extension, one-command installs and error messages that say what to change.

![jaic reports a type mismatch with the expected type and a suggested cast](https://raw.githubusercontent.com/matteopolak/jai/main/docs/images/releases/0.4.0-error.svg)

### Highlights

- New VS Code extension: highlighting, language server, jailint fixes, `jaifmt` formatting and run/build/check commands. It offers to download the toolchain if there is none.
- Install with one command, listed below.
- Errors say what is wrong and what to change, with labels, `help:` lines and colour.
- Built programs check narrowing casts and report failed runtime checks with file and line.
- The language server understands `#asm` and shows size, alignment and padding on hover.
- Unknown names get an *Add `#import`* quick fix.

### Install

| Platform | Command |
| --- | --- |
| macOS, Linux | `brew install matteopolak/tap/jai` |
| macOS, Linux | `curl -fsSL https://raw.githubusercontent.com/matteopolak/jai/main/install.sh \| sh` |
| Windows | `irm https://raw.githubusercontent.com/matteopolak/jai/main/install.ps1 \| iex` |
| Windows | `winget install matteopolak.jai` (once accepted into winget-pkgs) |

The VS Code extension is on [Open VSX](https://open-vsx.org/extension/matteopolak/jai) and attached to this release as `jai-vscode-0.4.0.vsix`.

### Breaking changes

- A procedure that changes a struct or string parameter now changes its own copy, not the caller's variable.
- `jaic build` of a program without `main` exits with status 3 (was 1).
- Out-of-range casts stop the program; use `cast,trunc` for the old behaviour.
- `Bindings_Generator`: `Enum.enumerates` holds declarations, and `Library_Info.name` is the library's file name. Update older generators.

### Fixes

- `jailint` has a new rule, `wrapping_constant`, and `jaifmt` puts a blank line between multi-line top-level items.
- A `#complete` switch stops the program when no case matches.
- A crash inside a C library names the foreign procedure and the Jai line.
- WebAssembly builds get files and directories; Windows native builds get Simp text, GetRect and Sound_Player.
- All four tools take `--help` and suggest the closest option for a typo.
- Many standard library and platform fixes: math, `File`, `Input`, `Clipboard`, Windows and Linux details.

## [0.3.0] - 2026-10-06

The linter arrives: `jailint`, with 31 rules, automatic fixes and editor integration. Also new: WebAssembly output and faster release binaries.

![jailint reports a loop that only uses its index, with the suggested rewrite](https://raw.githubusercontent.com/matteopolak/jai/main/docs/images/releases/0.3.0-jailint.svg)

### Highlights

- `jailint src/` checks your code; `--fix` applies the safe fixes. Settings go in `jailint.toml`.
- Silence a finding with `// jailint: allow(rule)`.
- Findings appear in the editor with quick fixes, and `source.fixAll.jailint` applies them all.
- `jaic build -os wasm` writes a WASI WebAssembly module.
- Release binaries are built with profile-guided optimisation and run faster; archives include `jaifmt`.
- Your own files are type-checked even where nothing uses them.

### Breaking changes

- Casts no longer push their type into the operand: `cast(float32) (0 - w)` with `w: u16` now subtracts in `u16`. Write `cast(float32) 0 - w` for the old result.
- Building from source needs LLVM 23 and `LLVM_SYS_231_PREFIX`.

### Fixes

- `JAIC_MEMORY_LIMIT=2G` caps what `jaic` allocates.
- `jaic run` writes the executables a metaprogram asks for, like `jaic build`.
- A release `jaic` started through a symlink finds its standard library.
- More `#asm` instructions, and `#intrinsic "llvm.<name>"` for LLVM intrinsics.
- macOS `Input` windows appear, clicks no longer quit and closing the window does.
- Windows: threads and C callbacks work in the interpreter, and MSVC builds write a `.pdb`.
- `jaifmt` builds to WebAssembly, about 35 times faster in the playground.

## [0.2.0] - 2026-10-06

Native Windows executables, debug information, a code formatter and a much richer language server.

![jaifmt turns messy code into formatted code](https://raw.githubusercontent.com/matteopolak/jai/main/docs/images/releases/0.2.0-jaifmt.svg)

### Highlights

- `jaic build` writes native Windows `.exe`, `.dll` and `.lib` files for x64 and arm64.
- Debug information: set breakpoints by `file.jai:line` and see typed variables in lldb and gdb.
- New formatter `jaifmt`, with `--check` for CI, `--stdin` for editors and `jaifmt.toml`.
- The language server (now `jailsp`) adds inlay hints, references, rename, signature help, folding, workspace symbols and macro expansions.
- `Long_Double` gives C's `long double` at full precision.
- Many more `#asm` instructions.

### Breaking changes

- The language server binary is renamed `jailsp` (was `jai-lsp`).
- Operator precedence follows Jai, not C: `1 << 2 + 3` is 7.
- `cast(T)` and `xx` take a following chain of bitwise operators as their operand: `cast(u32) byte << 16` shifts before widening.

### Fixes

- `#complete` switches report a missing enum case.
- A struct member with an undefined type is an error.
- Fuzzing found crashes and hangs (deep nesting, runaway recursion); they are fixed.
- Alignment, `cast(bool)`, `push_allocator` and pointer arithmetic types are corrected.
- `jaic run file.jai -- args` passes arguments to the program.
- `jaic --timings` prints time per compile phase, and `-plug Tracy` loads a metaprogram plugin.
- The standalone playground UI is gone; use the hosted playground at https://matteopolak.com/playground/jai.

## [0.1.0] - 2026-10-05

The first release of `jaic`, an independent Jai compiler written in Rust, with its own standard library and a language server.

### Highlights

- `jaic` type-checks, runs and builds native executables.
- Compile-time `#run` and metaprograms work, including calls into native libraries.
- A standard library covering `Basic`, `String`, `File`, `Thread`, `Math`, `Simp`, `GetRect` and more.
- A language server with diagnostics, hover, completion and go to definition, and a browser playground.
- Real projects check, run or build, including Focus and jaison.

### Downloads

| Archive | Platform |
| --- | --- |
| `jaic-macos-arm64.tar.gz` | macOS (Apple silicon) |
| `jaic-linux-x64.tar.gz` | Linux x86-64 |
| `jaic-windows-x64.zip` | Windows x64 (no native builds yet) |

Unpack and run `jaic`; it finds `stdlib/` beside itself. Native builds use the system linker (Xcode's command line tools on macOS, `cc` on Linux).

### Known limitations

- No native Windows executables yet.
- No native debug information.
- `#asm` lacks string operations, division, x87 and mask registers.
