# Changelog

## [Unreleased]

- `jailsp` offers completions inside range expressions again: `for i: 0..ta`, `0 .. ta`, `lo..hi.co` and `for 0..table.co` complete names and members like anywhere else (the second `.` of `..` was taken for member access on a number). The playground shares the fix.
- `jailint`'s `index_only_loop` also reports loops that fill elements from their index (`for i: 0..t.count-1 { t[i] = sin(cast(float) i * step); }`), rewriting them to `for *t { it.* = sin(cast(float) it_index * step); }`. `self_assignment` and `absurd_comparison` now offer quick fixes in the editor (remove the statement; replace the comparison by `true`/`false`); `--fix` does not apply them.
- The tour's GPU stop (`examples/tour/gpu/raymarch_gpu.jai`) draws natively again, in a window of its own, instead of aborting the process with wgpu-native's `invalid surface` panic; without a display or GPU adapter it prints a skip line. `webgpu_create_surface` now returns null for a null window and `webgpu_surface_format` returns `.Undefined` for a null surface, rather than handing null to wgpu-native, which aborts. See [WebGPU](docs/stdlib/webgpu.md).
- The playground tour runs every stop in order again. The `--stop`/`--all`/`--help` command line and the standard-input menu are gone: a first run no longer waits for input in the terminal before showing anything.
- `PERFORMANCE_REPORT` and `DEBUG_DUMP` compiler messages are now sent. A metaprogram that passes `DO_PERFORMANCE_REPORT_RUNS` and/or `DO_PERFORMANCE_REPORT_POLYMORPHS` to `compiler_begin_intercept` (the `Performance_Report` plugin does) gets one `PERFORMANCE_REPORT` right before `COMPLETE`, with timings of compile-time runs and procedure lowering, IR instruction and call counts, each `#run` with its callee, stalls and time, and each polymorphic procedure with its call sites and instances; the flags no longer warn. Every intercepting metaprogram gets a `DEBUG_DUMP` with a listing of jaic's IR for each lowered procedure declared `#dump`. Fields jaic has no counterpart for (bytecode inlining) are `0`; see [build options](docs/metaprogramming/build-options.md).
- `File_Watcher` no longer holds a change back by a rounding error: `File_Change.time_of_last_change` is now a `float64` (it was a `float32` that could round past the present, so a zero merge window sometimes reported nothing yet and `wait_seconds` could exceed the window).
- On Windows, a `link.exe` or `lld-link.exe` named by `JAIC_LINKER` now finds the Windows SDK and C runtime libraries outside a developer prompt instead of failing with `LNK1181: cannot open input file 'user32.lib'`.
- Optimized builds of large programs compile 2 to 3 times faster in the code generator: the program is divided by call graph into up to 8 parts that are optimized and written on separate threads, with small callees copied between parts so they can still be inlined (Focus `-O2` codegen 4.75 s to 1.9 s, Chess engine `-O2` 4.5 s to 1.3 s, Jails 2.4 s to 0.7 s; the Chess engine runs up to 3% more instructions, other programs measured the same). `enable_split_modules = false` no longer stops this, and `JAIC_CODEGEN_UNITS=1` asks for whole-program optimization. Embedded data (a global's bytes) is now emitted as one constant array instead of one constant per byte, which halves `-O0` codegen for Focus. See [compile speed](docs/compiler/compile-speed.md).
- Builds compile faster again: a 240k-line `-O0` build 1.50 s to 1.15 s (peak memory 1.58 to 1.31 GiB), Focus `-O2` 12.7 s to 11.0 s, chess-jai `-O2` 4.5 s to 3.9 s, Jails `-O2` 2.2 s to 1.6 s. Null-pointer checks are one compare and branch to a shared helper and are skipped for a pointer already checked in the same block; LLVM value names are discarded; `jaic build` no longer frees the compiler and LLVM state just before exiting; and the split after the optimizer now also applies when `enable_split_modules` is `false`. See [compile speed](docs/compiler/compile-speed.md).

## [0.5.1] - 2026-10-09

Unoptimized builds compile about twice as fast, the compile-time interpreter runs about twice as fast, and release archives carry FreeType, Dear ImGui, MojoShader and (on Windows) SDL2.

- Unoptimized builds compile 1.7 to 2.4 times faster on Apple silicon (a 240k-line program: 4.1 s to 1.7 s; `jaifmt`: 0.19 s to 0.12 s): `-O0` code is selected with FastISel instead of GlobalISel on AArch64, each codegen unit declares only the symbols it uses, units are 5,000 IR instructions instead of 20,000, and a release `jaic` no longer runs the LLVM verifier (`JAIC_VERIFY_IR=1` brings it back). See [compile speed](docs/compiler/compile-speed.md).
- On macOS an unoptimized `jaic build` no longer runs `dsymutil`: it keeps its objects in `.build/` beside the program (stale ones from earlier builds are removed), and lldb reads the debug info from them; add `.build/` to your `.gitignore`. `-O1` and up still write `<output>.dSYM`; `JAIC_DSYM=1` writes it for `-O0` too.
- Release archives (and `tools/build_native_libs.py`) now carry every third-party library the standard library binds that a system does not provide, so an installed toolchain runs and builds programs that use them with no setup: FreeType on macOS and Linux (it was the system's there; the shipped copy is found first and linked statically, so built programs no longer need Homebrew's or the distribution's FreeType), Dear ImGui (`ImGui`, built from the 1.89.6 source the bindings were written against; no C++ runtime library is needed) and MojoShader (`MojoShader`, with its Lemon-generated HLSL parser) on macOS, Linux and Windows, and SDL2 on Windows x64 (the official prebuilt import library and DLL, hash-checked; macOS and Linux still use the installed SDL2, and a program that uses SDL2 on Windows needs `SDL2.dll` beside it). The MinGW cross builds get FreeType, MojoShader and, on x64, SDL2 as well; ImGui is not built for MinGW, because its Windows bindings name MSVC-decorated symbols. See [native libraries](docs/tools/native-libs.md).
- The libraries in a release archive come with their licences in `artifacts/native-libs/<platform>/licenses/<library>/` (stb, lz4, meshoptimizer, pl_mpeg, rpmalloc, FreeType, ImGui, MojoShader, SDL2 and wgpu-native), and the build stops if a library or licence it should have produced is missing.
- `-sanitize` reports on Intel Macs show `file.jai:line` for every frame again: a sanitized build is now always one codegen unit and writes a `.dSYM`, since the Intel linker kept only the first unit's debug info.
- Calling a C function that takes or returns `Vector2` (or any struct with members placed over each other) by value passes it as the two floats C sees, in floating-point registers; before, it was treated as four floats and the value arrived as zeros or garbage (`ImGui.GetItemRectMin()` and every other ImGui function returning an `ImVec2`).
- The compile-time interpreter is about 2 times faster on loops, strings and calls, and 3 times on calls into C (since 0.5.0: `jaic run` loops 0.20 s to 0.09 s, strings 1.20 s to 0.51 s, recursive `fib` 0.33 s to 0.17 s, 60 million `labs` calls 7.9 s to 2.6 s; Focus `check` 2.1 s to 1.7 s). Registers live on the interpreter's own stack so a call allocates nothing, each procedure runs as one flat op array with block endings as ops, scalar locals are kept in registers, foreign calls are classified once instead of by name on every call, and memory checks cache the system's mappings. See [compile speed](docs/compiler/compile-speed.md#compile-time-interpreter) and [interpreter](docs/compiler/interpreter.md).

## [0.5.0] - 2026-10-09

Jai's own command-line parser arrives as `Extensions/Args` (a typed, compile-time clap-style parser that `jaifmt` now uses), the language server gains refactorings, call hierarchy, member rename and pull diagnostics, and built executables stop on a null pointer dereference instead of crashing. `print` format strings are checked at compile time, and more invalid programs (notes, `using` clashes, identical overloads, procedures that fall off their end) are now reported.

### Heads-up

| Before | Now |
| --- | --- |
| `print("% %\n", a)` and `print("done\n", a)` ran | compile warnings, as in Jai (``incorrect number of arguments supplied to `print`: the format string requires 2 arguments, but 1 argument is given``); the build goes on |
| jailint rule `format_arg_count` | removed, since jaic reports it; delete it from any `jailint.toml`, which now fails with `unknown rule` |
| `jaifmt --help` printed a hand-written text; a bad option printed ``help: `jaifmt --help` lists the options`` | The help is generated by `Extensions/Args` (new layout, `--no-` forms listed); a bad option prints `help: did you mean ...?`, a `usage:` line and a `--help` pointer |
| `jaifmt --stdin paths...` said ``error: `--stdin` takes no paths`` | ``error: `<PATHS>` cannot be used with `--stdin` `` |
| `jaifmt --color` accepted only lowercase values | Any case; an invalid value is a normal usage error listing the choices |
| `jailsp` pushed `publishDiagnostics` to every client | A client that advertises `textDocument.diagnostic` pulls instead and is no longer pushed to |
| `jailsp` compiled a document from itself when no open file loaded it | It compiles from the project's entry file (`jai.toml` or an inferred `main.jai`), so names of unopened files are visible |
| Built executables and a null pointer load or store | stop with `null pointer dereference` (on by default); `Build_Options.null_pointer_check = .OFF` removes the check |
| On Windows, `File.handle` was an `s64` | `File.handle` is a `HANDLE`, as in Jai; write `File.{ GetStdHandle(STD_INPUT_HANDLE) }` |
| `Bindings_Generator` C++ output marked later base classes that repeat a member name with `using` | no `using` for them |
| `a: int @tag;` (a note before the semicolon), `@note("with spaces")`, clashing `using` members, identical overloads | compile errors (a note ends at the next whitespace and goes after the `;`: `a: int; @tag`) |

- `print`, `sprint`, `tprint`, `log`, `assert` messages and user wrappers that forward a `string` and `..Any` to one of them are checked at compile time when the format is a literal; a wrong argument count is a warning at the call, which the language server shows too. `%N`, `%00` and `\%` are read as `print` reads them; spread (`..args`) calls are skipped. See [format string check](docs/compiler/format-string-check.md).
- Notes are lexed like Jai: `@` and everything up to the next whitespace. `@"two words"` stays one note. Two `using` members that bring in the same field name (field names only) and two procedures in one scope with identical parameter types are errors; Jai rejects them too.
- A procedure with results whose body can reach its end without returning gets the warning `not all control paths return a value`. `jaic check`, `run` and `build` print compile warnings before any error. See [diagnostics](docs/compiler/diagnostics.md#compile-warnings-compilerwarnings).

### Args and jaifmt

- New extension module [`Extensions/Args`](docs/stdlib/args.md): a typed command-line parser generated at compile time, in the spirit of Rust's clap derive. Declare a struct, annotate fields with notes (`@positional`, `@short=c`, `@long=`, `@env=`, `@range(a,b)`, `@value=`, `@count`, `@required`, `@conflicts=`, `@requires=`, `@hidden`, `@global`, `@subcommand`), and `Args.parse(Cli, HELP)` returns the filled struct plus an `Is_Set` record. It supports bundled shorts, `--no-<flag>`, `--`, `@env` fallbacks, enums, repeatable options, nested subcommands, generated help, "did you mean" suggestions, colour that follows `NO_COLOR`/`FORCE_COLOR`, and shell completions for bash, zsh, fish and PowerShell. Bad declarations are compile errors that name the field. Help shows `[default: ...]` only for a non-zero initial value.
- `jaifmt` parses its command line with Args: generated `--help`, a `--no-` form for every flag (`--no-check`), a mistyped option prints `help: did you mean ...?`, and a value may be given as `--config=file`. Exit codes (0, 1 for `--check`, 2 for errors) and the other options are unchanged.
- `jaifmt.wasm` and `jaifmt/build.jai` parse their options with Args too: both take `--help`, and a bad `jaifmt.wasm` command line now exits 2 (config and format errors still exit 1).
- The language tour takes a command line (`run --stop enums`, `run --all`, `run --help`); with no arguments it shows a numbered menu and reads your choice from standard input.

### Language server

- Refactorings in the light-bulb menu: *Extract into variable*, *Extract into procedure* (parameters and results worked out; declined when the selection returns, breaks out, or changes a variable declared outside it), *Inline variable*, *Add missing cases* for an `if x == {` on an enum, *Add missing fields* for a `Type.{...}` literal, and *Convert ifx to if/else* and back. Every edit is already in [jaifmt](docs/tools/jaifmt.md) style. See [refactorings](docs/compiler/language-server-refactorings.md).
- *Show Call Hierarchy* lists the callers and callees of a procedure, overloads and modules included, and *Expand Selection* grows from a token through expressions, statements and blocks to the file.
- Find references, highlights and rename work on struct fields (declarations, `a.b` through pointers and `using`, `T.{ b = 1 }`) and enum members (`Enum.B`, `.B`, `case .B`), told apart by the struct that declares them. A member declared outside the project (stdlib) can be found but not renamed.
- The type checker's errors are shown for every independent declaration, struct and procedure body, not just the first (`jaic` itself still stops at the first, as real Jai does for bodies).
- `textDocument/diagnostic` and `workspace/diagnostic` for clients that support pull diagnostics.

### Native builds and C interop

- Built executables stop with `null pointer dereference: read through a null pointer` (and the write, copy and `just past null` variants), at the line, instead of crashing at `-O0` or doing undefined things at `-O2`. On by default at every optimization level, like array bounds checks; `Build_Options.null_pointer_check = .OFF` (what the shipping presets set) or `#no_abc` removes it. See [pointers and arrays](docs/language/pointers-and-arrays.md#null-pointer-checks-in-built-executables).
- C aggregate ABI gaps closed (new fixtures and a [coverage matrix](docs/native/c-abi-coverage.md), see [C ABI](docs/native/c-abi.md)): packed structs on x86-64 System V go in memory, a struct that needs more registers than are left goes whole to the stack, a struct passed through `...` travels by value, a `#c_call` procedure returning a struct through a hidden result pointer can be handed to C on AArch64, a `#cpp_return_type_is_non_pod` callback runs on arm64 in the interpreter, and foreign calls in the interpreter return structs over 512 bytes.
- `tools/build_native_libs.py` also builds lz4, meshoptimizer and pl_mpeg on every platform, and the MinGW cross builds get stb_image, stb_image_write, stb_image_resize, stb_vorbis, FreeType and those libraries as static archives (`build_native_libs.py --platform windows-<cpu>-mingw`; `jaic build -os windows` finds them through the new `JAIC_CROSS_LIBS` path list).

<details>
<summary>All changes (21)</summary>

- A `Justfile` organizes the dev commands: `just build`, `fmt`, `lint`, `test`, `check`, `hooks`, `wasm`, `vscode` and `fetch-upstreams`, with options such as `just fmt --check --lang jai` and `just lint --staged`. CI's format and lint steps run the same recipes. See [Justfile](docs/tools/justfile.md). `tools/staged-files.sh` lists the staged Jai or Rust files.
- A tracked pre-commit hook checks the staged files with `jaifmt --check`, `jailint -D warnings`, `rustfmt --check` and `tools/rust_item_spacing.py --check`. Enable it with `git config core.hooksPath .githooks`; see [pre-commit hook](docs/tools/pre-commit-hook.md).
- VS Code extension build tooling: Rolldown (`rolldown.config.ts`) replaces esbuild, the lint config is `oxlint.config.ts`, and `scripts/*.mjs` became TypeScript run with `node scripts/x.ts`, type-checked by `tsc`. The bundle is `dist/extension.cjs` (`package.json` is now `"type": "module"`) and is about 1% smaller. See [VS Code extension](docs/tools/vscode-extension.md).
- `jailint` given only a file that a nearby program `#load`s (in its directory or up to two above) now compiles that program and reports the listed file, instead of linting it as a standalone program with bogus errors. See [jailint](docs/tools/jailint.md).
- CI builds open-source Jai projects with their own entry points and runs them on Linux, macOS and Windows. See [third-party smoke test](docs/tools/third-party-smoke-test.md).
- A metaprogram's custom link command got its libraries in the wrong lists, and stopped on an index error with five or more libraries; forbear's `build.jai` now builds from scratch.
- `using Module.global;` at file scope brings in the members of another module's global variable (Simp's `using GL.gl_procs;`).
- On Windows, `#file`, `#filepath` and `get_absolute_path` use `/`, so paths can go into string literals and `#load`.
- New `File.file_read_line(file)` and `File.read_stdin_line()` read a line from a file or from standard input (the newline and a carriage return before it are dropped). They work natively, on Windows and in the browser playground.
- Fix: `exit(n)` in a playground program trapped in the wasm-hosted interpreter; it now ends the program with exit code `n`.
- Fix: an Args baked argument (`$help: Help(T)`) whose declared type names an earlier `$T`, or that is a member of a constant (`HELP.build`), was rejected.
- `offset_of(T, "member")` / `offset_of(T.member)` and `is_value_type(T)` are implemented. See [structs](docs/language/structs.md). `#overlay` outside a struct body now says so instead of "not supported"; inside struct bodies it is tested in `#if`, `#insert`, union and nested forms.
- The `Compiler` module procedures that stopped with "not supported yet" now work or say why they do nothing: `remap_import`, `provide_import`, `compiler_make_procedure_live`, `get_root_type`, `compiler_get_base_path`, `compiler_add_library_search_directory`, `add_data_segment`; `compiler_report_errors_for_unresolved_identifiers`, `compiler_report_errors_for_untyped_declarations_with_these_notes` and `compiler_set_memory_breakpoint` are accepted and ignored. See [the Compiler module](docs/metaprogramming/compiler-module.md). `Default_Metaprogram` reads `-os`, `-cpu`, `-exe`, `-no_dce`, `-no_color`, `-msvc_format`, `-natvis`, `-quiet` and `-version`; an unknown option is named in the error.
- Intercepting metaprograms get `FAILED_IMPORT` (module not found) and `ERROR` messages when a workspace fails to compile.
- `#this`, `#compile_time`, `#bytes`, `#procedure_name`, `#bake_arguments`/`#bake_constants`, `#asm` (opaque), `#load`, `#place` and `#overlay` reach metaprograms as their own `Code_Node` kinds instead of `.PLACEHOLDER`, and `Program_Print` prints them (not `#asm`).
- `Code_Node` export models `#caller_code`, `$T`/`$$x`/`$T/Restriction`, `#add_context`, `#module_parameters`, and (with the compiler's sources) `#file`, `#filepath` and `#line`. New jaic fields: `Code_Ident.polymorph_restriction`, the `BAKED_POLYMORPH_VARIABLE`/`INTERFACE_RESTRICTED` flags and `Code_Directive_Location.is_caller_code`. See [build options](docs/metaprogramming/build-options.md).
- A wide `Long_Double` (x87, binary128) can be passed to a C variadic procedure such as `printf("%Lf", x)` in both backends, and a C variadic procedure that returns a struct or `Long_Double` gets its result pointer before the variadic arguments. See [long double](docs/language/long-double.md).
- `#asm`: `pcmpistri`, `pcmpestri`, `pcmpistrm`, `pcmpestrm`, `mpsadbw` and `pmadd52luq`/`pmadd52huq` are implemented; `movd`/`movq` accept a float variable; integer instructions given a float variable or a `.x/.y/.z` size say what to use instead. The remaining unsupported instructions are explained in [jaic `#asm` blocks](docs/compiler/asm.md).
- `type_of` of a polymorphic struct is `Type`, and printing an uninstantiated polymorphic struct prints its name; `type_of` of a polymorphic procedure keeps its parameters and results, with the parts that depend on a type variable shown as `$` (`procedure ($, s64) -> $`).
- Standard library: `Pool(USE_UNMAPPING_ALLOCATOR=true)` is accepted; `Unmapping_Allocator` and `Overwriting_Allocator` answer `IS_THIS_YOURS` and `CREATE_HEAP`/`DESTROY_HEAP`; `Simp` font effects `SMALLCAPS`, `LINING_FIGURES` and `LEFT_JUSTIFIED` work; `Debug` and `Runtime_Support_Crash_Handler` build for wasm; `Icon` (`-icon FILE`) lays out `<name>.app` on macOS and writes `<name>.desktop` on Linux.
- Fix: `rpmalloc` crashed when a thread that never used it allocated; the thread is now initialised on first allocation.

</details>

## [0.4.3] - 2026-10-08

Closer to Jai's behaviour: `print` formats numbers, floats and enums the same way, constants must fit their types, and a set of invalid programs jaic used to accept are now compile errors. The language server also gains auto-import completion, and formatting works again with the renamed VS Code extension.

### Heads-up

| Before | Now |
| --- | --- |
| `x: u32 = -1;`, `x: s16 = 40000;`, `cast(u8) 300` on a constant | compile errors; use `cast,trunc(T)` for the low bits |
| `16777216.0` was `float32` | 8+ significant digits (or outside `float32`'s range) makes a float literal `float64` |
| `true + 1`, `a, b := f(1)`, `*5`, `defer;`, repeated member names | compile errors |

- Integer constants must fit their type ([numbers](docs/language/numbers.md)). A negative constant no longer converts to an unsigned type (`x: u32 = -1;`), a decimal constant must fit a signed type's range (`x: s16 = 40000;`), and a cast of a constant checks its range at compile time (`cast(u8) 300`, `y: u8 = xx -1`); write `cast,trunc(T)` for the low bits. Hex, binary, `~` and named constants may still fill a signed type's bits (`x: s8 = 0xff` is `-1`). A literal that does not fit the other operand makes the operation `s64` (`small_u8 == -1` is now `false` for 255).
- A float literal with 8 or more significant digits is `float64`, even when `float32` holds it exactly (`16777216.0`), and so is one outside `float32`'s normal range (`1.0e39`), which no longer converts to `float32` without a cast. Integer literals must fit in 64 bits.

### Auto-import

Typing part of a name you haven't imported offers it from the standard library, your own modules and your project's other files; accepting it adds the `#import` or `#load` too, in VS Code and the playground. A new `jai.toml` names the project's `build_files` and extra `import_path` folders when jailsp can't infer them. Turn it off with `jai.completion.autoImport`. See [auto-import](docs/compiler/language-server.md#auto-import-completion).

### Fixes

- VS Code: Jai files format with the extension again. Its default formatter setting still named the old ID `matteopolak.jai`, so VS Code said the formatter "isn't available"; *Open Settings* in the missing-toolchain warning also filtered by the old ID.
- `print`: negatives in other bases are two's complement, ties round away from zero, `Inf`/`-Inf`/`NaN`, `(enum out of range: N)` for unnamed enum values, and `[1, 2...]` for cut-short arrays.
- `Hash_Table`, `Random` and `make_look_at_matrix` match Jai (details below).
- Simp programs run and build from an installed toolchain: the release archives now include the stb_image, stb_image_write, stb_image_resize and stb_vorbis libraries (plus rpmalloc, and FreeType on Windows). On Windows, `jaic build` without Clang also links with Visual Studio instead of the `link` that Git for Windows puts on `PATH`.

<details>
<summary>All changes (10)</summary>

- Auto-import completion in jailsp: typing part of a name the file cannot see offers it from standard-library modules, the project's modules and its other files, and accepting it also adds the `#import "Basic";` or `#load "util/strings.jai";` it needs (after the existing ones). It needs two typed characters, offers at most 50 names, sorts them after names in scope, and leaves out modules already imported and modules for other target OSes. `jai.toml` at a project root names its entry files (`build_files`) and module folders (`import_path`); without one they are inferred (`build.jai`, `first.jai`, `main.jai`, `src/main.jai`). VS Code: `jai.completion.autoImport` (on by default) and a schema for `jai.toml`. The playground applies the added import too ([language server](docs/compiler/language-server.md#auto-import-completion)).
- `Hash_Table.init` rounds an explicit size up to a power of two without raising it to the default minimum (`init(*t, 5)` allocates 8 slots).
- `Random`'s float32 draws use the low 24 bits of the next value, and `random_get_within_range` returns `min + (max - min) * fraction`.
- `make_look_at_matrix(..., x_is_forward = false)` builds a right-handed view: the camera's right is `+x` (it was `-x`).
- `print` handles several edge cases as Jai does: a negative integer in another base is its two's complement (`formatInt(-1, base = 16)` on an `s8` is `ff`); fixed-precision floats round ties away from zero (`2.5` with no decimals is `3`); trailing zeros removed by zero removal keep their width as spaces; infinity and NaN print as `Inf`, `-Inf` and `NaN`; an enum value without a name prints as `(enum out of range: 5)`, and leftover `enum_flags` bits print in hex after the names; an array cut short ends `[1, 2...]`; an `Any` with no type prints `Any.{}`; strings inside structs escape only newlines; type names print `(anonymous enum)`, `#c_call` and `#compiler`.
- Type info gives an anonymous enum an empty name, as for anonymous structs.
- Two typed integer constants of different types meet in the wider one (`cast(s16) 1 + cast(s32) 2` is `s32`), as variables do.
- Compile errors for programs jaic accepted: arithmetic on `bool`, more names than a call returns (`a, b := f(1)` with one result), the address of a number constant (`*5`), `#char` of an empty or multi-character string, `using` on a variable that is not a struct or enum, a bare `defer;`, an enum value outside its type, a repeated enum or struct member name, and assigning into a constant array (`ARR[0] = 1` changed a copy).
- Release archives ship the stdlib's C libraries (`stb_image`, `stb_image_write`, `stb_image_resize`, `stb_vorbis`, plus `rpmalloc` on macOS and Linux and FreeType on Windows) for every platform, so Simp programs run and build from an installed toolchain. Before, `jaic run` stopped with ``foreign procedure `stbi_load` is not available here`` and `jaic build` could not find `stb_image`.
- Windows: `jaic build` without Clang on `PATH` no longer runs the coreutils `link` that Git for Windows puts there (``link: extra operand 'build\\game.exe.o'``). It uses Microsoft's `link.exe`, finding Visual Studio and the Windows SDK libraries itself when no developer prompt is open, or a Clang installed outside `PATH`.

</details>

## [0.4.2] - 2026-10-08

Builds for Intel Macs and arm64 Linux, programs that run on any CPU of their architecture, and every `Build_Options` field either honoured or reported. Several real-world projects (toml-jai, jai-format, jai-protobuf, jaison, Vk-Engine) now build and pass their tests.

### Heads-up

| What | Change |
| --- | --- |
| VS Code extension | New ID `matteopolak.jai-toolchain` on the Marketplace and Open VSX. Install it and uninstall `matteopolak.jai` (it offers to); settings carry over. |
| Built programs | Target the baseline CPU (x86-64, Apple M1, generic arm64) instead of the build machine's. `llvm_options.target_system_cpu = "native"` restores the old behaviour. |
| `Build_Options` | Fields jaic can't act on warn when changed; impossible ones (an unsupported `os_target`, `add_build_string` with a `code` scope) are errors. `ERROR_CONTINUABLE` reports now fail the build. |
| Workspaces | `set_build_options` workspaces get the code generation their options describe (frame pointers, inlining, crash handler, …); `set_optimization` sets them per flavor. |

See [build options](docs/metaprogramming/build-options.md) for the full support table.

### New platforms

| Archive | Notes |
| --- | --- |
| `jai-macos-x64.tar.gz` | Intel Macs; static LLVM from conda-forge |
| `jai-linux-arm64.tar.gz` | arm64 Linux; PGO-optimised like the others |

`install.sh`, the Homebrew formula and the VS Code extension's toolchain download all use them. Both include wgpu-native for WebGPU.

### Fixes

- The 0.4.1 Linux `jaifmt` (and any program built on a newer CPU) crashed with an illegal instruction on machines without AVX-512: `jaic build` targeted the build machine's CPU.
- A struct pointer passes where its pointer-typed `#as` member is expected (`*Physical_Device` for `VkPhysicalDevice`).
- A by-value `for` loop's `it` is the array element itself, so `*it` points into the array.
- A struct's `type_info` lists its constants, so member indexes match the official layout.
- `push_context` without a context is accepted again and pushes a copy of the current one.
- Passing a null `p.*` as an `Any` no longer stops the program; `print` stops when it reads it (0.4.1 stopped too early and broke jaison).

<details>
<summary>Other changes (9)</summary>

- Metaprograms can link a workspace themselves with `use_custom_link_command` (`READY_FOR_CUSTOM_LINK_COMMAND`, `compiler_custom_link_command_is_complete`); `POST_WRITE_EXECUTABLE` fills `executable_write_failed` and `linker_exit_code`.
- Newly honoured `Build_Options`: `intermediate_path`, `entry_point_name`, `append_executable_filename_extension`, `minimum_os_version`, `runtime_support_definitions`, `backtrace_on_crash`, `enable_frame_pointers`, `disable_redzone`, most `llvm_options` (including real `.OS`/`.OZ` size pipelines), `SKIP_*` intercept flags and `compiler_destroy_workspace`.
- Metaprograms: calls to polymorphic procedures resolve to their instance, and static `#if` reports the branch taken.
- A `#!` first line is skipped by jaic and kept by jaifmt.
- `File.handle` is the C stream (`*FILE`) on macOS and Linux, so `File.{ stdin }` works.
- Built programs always link libm on macOS and Linux, for the float rounding calls baseline x86-64 needs.
- CI presents WebGPU frames to real windows on Linux, macOS and Windows.
- `WEBGPU_SURFACE_OCCLUDED` in `Extensions/WebGPU` for hidden windows on macOS; the triangle example waits it out.
- The release smoke test feeds jaifmt input without a trailing newline, checks its exit code, and rejects AVX code in the Linux build.

</details>

## [0.4.1] - 2026-10-07

WebGPU graphics on every platform and in the playground, threads in WASI builds, and a round of fuzzer-found crash fixes. The jaic-only modules move to `stdlib/Extensions/`, and the VS Code extension reaches the Marketplace as **Jai Toolchain**.

### Heads-up

| Before | Now |
| --- | --- |
| `#import "Jaic_Extensions";` | `#import "Extensions/Long_Double";` |
| `#import "Jai_Format";`, `"Wasi_Runtime"` | `#import "Extensions/Jai_Format";`, `"Extensions/Wasi_Runtime"` |
| `jaic-<platform>` release archives | `jai-<platform>` (installers and the extension pick the right name for older versions) |
| `JAIC_VERSION`, `JAIC_INSTALL_DIR`, `JAIC_BIN_DIR` | `JAI_VERSION`, `JAI_INSTALL_DIR`, `JAI_BIN_DIR` (old names still work) |

A bare import of a moved module fails with an error naming the import to write. See [stdlib extensions](docs/stdlib/extensions.md).

### WebGPU

- New `Extensions/WebGPU` module: the whole `webgpu.h` API, generated from webgpu-headers, plus helpers for windows, adapters, surface formats, buffer mapping and error scopes.
- One program draws in a window on macOS (Metal), Linux (Vulkan) and Windows (D3D12), and in the playground's new **Render** pane (Chromium browsers).
- Release archives ship the pinned wgpu-native, so `jaic run` and `jaic build` need nothing else.
- Examples: `examples/webgpu/triangle.jai`, `raymarch.jai`, `compute.jai`, and a GPU ray marcher at the end of the tour. See [WebGPU](docs/stdlib/webgpu.md).

### Playground and editor

| Where | Change |
| --- | --- |
| Tour | Four new stops: runtime safety checks, threads, files, and `#asm` with `Long_Double` |
| VS Code | A here-string body is highlighted in the language its terminator names: `#string WGSL`, `GLSL`, `SQL`, `JSON`, `JAI` and more ([embedded languages](docs/tools/vscode-extension.md#embedded-languages)) |
| Playground | Highlights `#string WGSL` and `#string JAI` bodies |
| Marketplace | The extension is listed as **Jai Toolchain** (ID still `matteopolak.jai`) |

### Threads in WASI builds

`jaic build -os wasm` programs that start threads used to fail or hang under WASI, which has no threads. They now run as green threads that take turns on one host thread: mutexes, condition variables, semaphores, joins, sleeps and `Thread_Group` behave as natively. A busy wait that never blocks still keeps the turn. See [threads in WASI builds](docs/native/wasm-threads.md).

### Crash fixes

The nightly fuzzer's findings are fixed by class rather than one by one:

- The compiler checks every address a program hands it (`#run` results, metaprogram messages, `compiler_*` out-parameters) instead of crashing on a bad pointer.
- Metaprograms that add code every round no longer slow down quadratically.
- jailsp no longer crashes on builtin constants such as `OS` or on huge `%N` argument numbers.
- `print("%", p.*)` with a null `p` now stops with a null-dereference error instead of printing `null`.

<details>
<summary>Other changes (4)</summary>

- Browser build: a foreign procedure the sandbox lacks is offered to the page by name (`createEngine(bytes, { host })`), with every pointer checked; a promise result suspends the program through JSPI while other Jai threads keep running.
- CI runs a headless WebGPU test on Linux, macOS and Windows x64 and arm64, and tests the browser host against a mock WebGPU.
- jailsp: a `cast,trunc(T) x` or `xx,no_check x` argument of `print` counts as one argument.
- `Bindings_Generator` output imports `Extensions/Long_Double`.

</details>

## [0.4.0] - 2026-10-07

A VS Code extension, one-command installs on every platform, and error messages that say what to change. Built programs now check narrowing casts and report every failed runtime check with its location, and the language server understands `#asm`, memory layouts and missing imports.

### Install

| Platform | Command |
| --- | --- |
| macOS (Apple silicon), Linux x86-64 | `brew install matteopolak/tap/jai` |
| macOS (Apple silicon), Linux x86-64 | `curl -fsSL https://raw.githubusercontent.com/matteopolak/jai/main/install.sh \| sh` |
| Windows x64, arm64 | `irm https://raw.githubusercontent.com/matteopolak/jai/main/install.ps1 \| iex` |
| Windows x64, arm64 | `winget install matteopolak.jai` (once accepted into winget-pkgs) |

The scripts verify the download against the release's `SHA256SUMS` and upgrade in place; `JAIC_VERSION` picks a release and `JAIC_INSTALL_DIR` the folder.

### VS Code extension

<table>
<tr>
<td width="50%"><img src="https://raw.githubusercontent.com/matteopolak/jai/main/editors/vscode/images/hover.png" width="400"><br>Hover shows a type's size, alignment and padding</td>
<td width="50%"><img src="https://raw.githubusercontent.com/matteopolak/jai/main/editors/vscode/images/asm-completion.png" width="400"><br>Completion and docs for every <code>#asm</code> instruction</td>
</tr>
<tr>
<td><img src="https://raw.githubusercontent.com/matteopolak/jai/main/editors/vscode/images/error.png" width="400"><br>Compiler errors as you type</td>
<td><img src="https://raw.githubusercontent.com/matteopolak/jai/main/editors/vscode/images/quick-fix.png" width="400"><br>jailint findings with quick fixes</td>
</tr>
</table>

- New extension `matteopolak.jai`: highlighting, the `jailsp` language server, jailint fixes (and `source.fixAll.jailint`), `jaifmt` formatting, *Jai: Run/Build/Check File*, snippets and `jailint.toml`/`jaifmt.toml` schemas.
- With no toolchain on `PATH`, it offers to download the matching release, checksum-verified.
- Get it from [Open VSX](https://open-vsx.org/extension/matteopolak/jai) or as `jai-vscode-0.4.0.vsix` attached to this release. The VS Code Marketplace listing, named **Jai Toolchain**, comes with the next release.

### Language server

- `#asm` blocks: completion, hover and signature help for every instruction, with operand forms and the CPU feature each needs. Also in the browser playground.
- Hover shows memory layout: size, alignment and padding of types, and each field's offset.
- An unknown name a standard module declares gets an *Add `#import "Basic";`* quick fix.

### Clearer errors

- Errors say what is wrong and what to change, with labels, `help:` lines and fix previews, in colour on a terminal (`--color`, `NO_COLOR`).
- An unknown identifier names the module that declares it: ``help: `print` is declared in the `Basic` module: add `#import "Basic";` ``.
- Type and call mismatches underline the argument, show the declaration and suggest the conversion (`cast`, `.data`, `tprint`, `.*`).
- Runtime errors name the failed check, point at your line and print a tidy call stack; a failed `assert` shows its condition.
- All four tools take `--help`, suggest the closest option for a typo, and exit with 2 on a command-line mistake.

### Runtime safety

- Narrowing casts (`cast(u8)`, `xx`) check that the value fits: ``runtime error: cast of 300 to `u8` overflows``. Opt out per cast with `cast,trunc` or `cast,no_check`, or globally with `Build_Options.cast_bounds_check`.
- A `#complete` switch stops the program when no case matches instead of running none.
- Built executables print `path:line: error: ...` for an out-of-range index, an overflowing cast, division by zero or a missing `return`, instead of trapping silently.
- A crash inside a C library called from `jaic run` or `#run` names the foreign procedure and the Jai line that called it, instead of killing jaic.

### Formatter and linter

| Tool | Change |
| --- | --- |
| jaifmt | A blank line between multi-line top-level items and after a group of imports. Turn off with `blank_lines_between_items = false`. |
| jailint | New rule `wrapping_constant`: a constant that silently wraps to the other operand's type (`h < 0x8000_0000` with `h: s32`). |

### Other highlights

- **WASI builds** (`jaic build -os wasm`) get files, directories and an accurate libm, so `File`, `File_Utilities` and friends work under node; `Long_Double` works there too.
- **Threads in `jaic run`** keep running while another is inside a long C call, and C threads (an audio callback) can lock Jai mutexes and wait on Jai semaphores.
- **Windows**: Simp text, GetRect and Sound_Player work in native builds (prebuilt FreeType and stb libraries for x64 and arm64).

### Heads-up

- A procedure that changes a struct or string parameter now changes its own copy, not the caller's variable.
- `jaic build` of a program without `main` exits with status 3 (was 1).
- `Bindings_Generator`: `Enum.enumerates` holds declarations, and `Library_Info.name` is the library's file name (its Jai name is `identifier`). Older generators need updating.

<details>
<summary>Other changes (50)</summary>

**Language and compiler**

- Mixed declaration lists can assign to members, indexes and dereferences: `ok:, toki.str = f();`.
- Struct literals without a dot in arguments: `f({1})`, `f({.A, 1})`.
- A one-byte string constant works as a `u8` in arithmetic, arguments and casts (`c - "0"`).
- A mismatched `return` value points at the declared return type.
- `Type_Info_Struct_Member.Flags.OVERLAY` marks members declared after `#overlay`.
- A `$$` parameter whose argument is omitted bakes its default.
- A `#modify` block where it can never apply is an error instead of being ignored.
- `push_context,defer_pop;` without a context is an error.
- A 16-digit `0h` literal is `float64`.
- Baked variadic parameters (`$args: ..Code`) get one instance per argument list; structs with a variadic parameter can be instantiated, so `Tagged_Union` and `Print_Vars` work.
- Enum members written `A : :5` parse; `using g;` of a global pointer no longer reports a struct containing itself.
- Fixed: a crash on unknown escapes of multibyte characters, exponential parse time on nested `ifx`, and a stack corruption with structs smaller than a pointer.
- `#asm`: a general-purpose destination declared in place (`pmovmskb.x found:, v;`); invalid `punpck*` spellings are rejected.

**`jaic run` and `#run`**

- Windows: `snprintf` and the `printf` family resolve, up to 24 foreign arguments, `long double` math such as `sqrtl`, and programs see their own command-line arguments.
- macOS arm64: foreign calls and callbacks with small arguments on the stack.
- A `#c_call` procedure stored where C reads it (a struct field, a global) is a real C function pointer, callable from C threads.
- Linux finds libraries installed only by versioned name (`libatomic.so.1`); Intel Macs find Homebrew's `/usr/local/lib`.
- macOS no longer wakes every 5 ms while waiting for the program.
- Compile-time code that exports typed trees no longer crashes the compiler or language server, and `code_to_string(compiler_get_code(...))` no longer loops.
- The playground's, jailsp's and jailint's time budget now covers workspaces a metaprogram compiles.

**Standard library**

- `Basic.string_to_float64_new` returns success first; `string_to_float` rounds correctly.
- Math: correctly rounded `sqrt`, exact `pow` for whole exponents, fixed `round` and float64 constants.
- `String.parse_int` fails on out-of-range text; `path_decomp` keeps `..`.
- A game that resets temporary storage every frame no longer corrupts Input, Simp, GetRect, Sound_Player or Window_Creation.
- `Tagged_Union.isa` returns a pointer into the caller's union again.
- `Clipboard`: setting bitmaps works on every platform; Windows `get_text` returns a string the caller owns.
- `Input`: `set_custom_cursor_handling` works; Linux reports a new window's size; Windows reports resizes and the close button.
- `Simp`: `deinit_fonts` with several fonts, `set_window_dimensions` before a shader is bound, and windows in macOS virtual machines.
- GetRect: X11 cursors on Linux, and the color animation editor with keyframes.
- `Process.run_command` closes the child's input, so programs reading stdin finish.
- `File` on macOS and Linux: write handles can read back, append mode honours seeks, and opening a directory fails.
- `Unmapping_Allocator` on Windows.
- Windows: `Debug` backtraces and signal handlers, `Thread.Semaphore` timeouts, BuildCpp with MSVC, `#elsewhere` variables from DLLs, MinGW system libraries named with capitals, and plain (not `\\?\`) working directories.
- Linux: GL 1.1 procedures load, stb_image and stb_vorbis link libm, and `get_number_of_processors(.ALL_PHYSICAL)` is sane in containers.
- Fixed: `File_Async` compiles, `Remap_Context` works, `Print_Vars` compiles, `Zip_File_Directory` of a missing archive, `compare_and_swap` on floats, `FormatFloat` of distinct types, and Jai_Lexer tokens.
- Atomics, Relative_Pointers and Socket wrap on purpose where they narrow, so the new cast checks do not stop them.

**Bindings_Generator**

- `Library_Info.identifier` names the bindings' library; `will_print_bindings` may change it.
- Replaced enum value expressions are printed from the replacement.
- Windows: C++ functions bind to their Microsoft-mangled names, and drive-rooted header paths are found.

**WASI and the browser**

- Only `_start` is exported, so a hello world imports four WASI calls instead of all of them.
- Calendar, sleeping, the working directory and single-threaded `pthread` calls link.
- Threads no longer hang or end in a false deadlock while waiting for work.
- The sandbox keeps `errno`, implements `strerror` and gives files distinct inodes and advancing modification times.
- The playground can render errors in colour (`jai_play_set_styled(1)`) and its diagnostics carry a stable `code`.

**Tools and packaging**

- The macOS `jaic` no longer needs Homebrew's zstd.
- `jaifmt --check` prints paths relative to the current directory.
- `jailint` given a file that a module loads checks it as part of that module.
- jailsp: go to definition on a builtin constant such as `OS` no longer crashes.
- Built executables' error reports use paths relative to the build directory.
- Stdlib tests now run in the interpreter, natively, in the browser and as WASI builds on every platform, with coverage tracked in CI.

</details>

## [0.3.0] - 2026-10-06

Introduces **jailint**, a linter that runs on the type-checked program, with 31 rules, fixes and editor integration. Also new: WebAssembly (WASI) output, faster release binaries built with PGO, and a toolchain archive that includes `jaifmt`.

### jailint

<table>
<tr>
<td width="50%"><img src="https://raw.githubusercontent.com/matteopolak/jai/main/editors/vscode/images/lint.png" width="400"><br>A finding, linked to its rule's docs</td>
<td width="50%"><img src="https://raw.githubusercontent.com/matteopolak/jai/main/editors/vscode/images/quick-fix.png" width="400"><br>Its quick fix</td>
</tr>
</table>

- `jailint src/` checks files, `--fix` applies safe fixes, `-A`/`-W`/`-D` set levels; settings live in `jailint.toml`.
- Silence a finding with `// jailint: allow(rule)`, `// jailint: allow-file(rule)` or `@jailint_allow(rule)`.
- `jailsp` shows findings as diagnostics with quick fixes, and `source.fixAll.jailint` applies them all.
- Workspaces a `build.jai` metaprogram creates are linted too.

| Kind | Rules |
| --- | --- |
| Likely bugs | `absurd_comparison`, `almost_swapped`, `bitwise_precedence`, `defer_in_loop`, `duplicate_condition`, `erasing_op`, `format_arg_count`, `identical_branches`, `identical_operands`, `infinite_loop`, `integer_division_in_float`, `min_max`, `no_effect`, `range_past_count`, `remove_in_for`, `reversed_range`, `self_assignment`, `shadowed_it`, `unused_result` |
| Unused code | `unused_variable`, `unused_parameter`, `unused_import` |
| Style | `bool_comparison`, `identity_op`, `index_only_loop`, `manual_assign_op`, `manual_index_counter`, `needless_bool`, `redundant_cast` |
| Off by default | `float_equality`, `lossy_xx` |

### WebAssembly

- `jaic build -os wasm` writes a wasm64 WASI module; `-target wasm64-unknown-unknown` leaves out the runtime for your own host.
- `#foreign` procedures become imports and `#program_export` procedures exports.
- Needs `wasm-ld` (from LLVM, an `lld` package or `JAIC_WASM_LD`).

### Compiler

- Your own files are type-checked even where nothing uses them, as in Jai; `-no_dce` and `Build_Options.dead_code_elimination` control it.
- `JAIC_MEMORY_LIMIT=2G` caps what `jaic` allocates and exits with status 120 past it.
- `#asm` adds F16C, SHA and GFNI instructions.
- `#intrinsic "llvm.<name>"` calls an LLVM intrinsic in native builds.
- Windows: threaded programs and C callbacks into `#c_call` procedures run in the interpreter; MSVC builds write a `.pdb`.

### Heads-up

- Casts no longer push their type into the operand: `cast(float32) (0 - w)` with `w: u16` now subtracts in `u16`. Write `cast(float32) 0 - w` for the old result.
- Building from source needs LLVM 23 and `LLVM_SYS_231_PREFIX`.

<details>
<summary>Other changes (19)</summary>

**Added**

- Release archives include `jaifmt`, and release binaries are built with profile-guided optimisation (plus BOLT on Linux).
- `jaifmt` builds to WebAssembly, about 35 times faster than the interpreted formatter in the playground.
- `jailsp` reports the type checker's first error and checks metaprograms that create workspaces.
- The `jaifmt` sources moved to a top-level `jaifmt/` folder: `jaic build jaifmt/main.jai -O2 -o target/jaifmt`.

**Fixed**

- A release `jaic` started through a symlink finds its standard library, and a missing one is reported clearly.
- `jaic run` writes the executables a metaprogram's workspaces ask for, as `jaic build` does (`-no_workspace_output` skips them); `jaic check` warns instead.
- Under `jaic run`, `get_path_of_running_executable` returns the path `jaic build` would write, so relative asset paths work.
- `jaic run` on a file with only `#program_export` procedures runs its compile-time code and exits 0.
- `jaic check -os wasm` and `jaic run -os wasm` set `CPU` to `.CUSTOM`.
- An omitted argument binds a polymorphic type from its default (`$T = 0`).
- An enum member's value may name its own enum, and pointer constants with the top bit set keep their value.
- Struct constants inside nested structs no longer report a circular dependency.
- macOS `Input`: clicks no longer quit, closing the window does, and windows appear.
- macOS: several Objective-C selectors sent wrongly are fixed.
- `Process.create_process` says why a program could not start.
- `File.read_entire_file` takes `zero_terminated` as its second argument.
- Windows: write handles can read, so `Zip_File_Directory` works.
- Type errors in rarely compiled stdlib code (other platforms, unused procedures) are fixed, and CI now checks every module for every target.
- `Objective_C`, `macos` and `Metal` fail with a clear `#assert` off Apple platforms.

</details>

## [0.2.0] - 2026-10-06

Native Windows executables (x64 and arm64), native debug information, a code formatter (**jaifmt**), and a much richer language server, now called `jailsp`.

### Highlights

<table>
<tr>
<td width="50%"><img src="https://raw.githubusercontent.com/matteopolak/jai/main/editors/vscode/images/formatting.png" width="400"><br><code>jaifmt</code> before and after</td>
<td width="50%"><img src="https://raw.githubusercontent.com/matteopolak/jai/main/editors/vscode/images/completion.png" width="400"><br>Type-checked completion from <code>jailsp</code></td>
</tr>
</table>

| Area | What's new |
| --- | --- |
| Windows | `jaic build` writes native `.exe`/`.dll`/`.lib` for x64 and arm64; cross-compile with `-os windows` (and `-cpu arm64`). New `windows-arm64` archive. |
| Debugging | DWARF debug info (a `.dSYM` on macOS): breakpoints by `file.jai:line` and typed variables in lldb and gdb. Off with `--no-debug-info`. |
| Formatter | `jaifmt` formats like rustfmt: canonical spacing, one statement per line, comments kept. `--check` for CI, `--stdin` for editors, `jaifmt.toml` and `// jaifmt: off`. |
| Language server | Expansions of macros, `#insert`, `#run` and `#if`; inlay hints; references; rename; signature help; folding; workspace symbols. |
| `#asm` | Division, string instructions, BMI, AVX-512 mask registers and a much wider SIMD set (FMA, shuffles, AES...). |
| `Long_Double` | C's `long double` at full precision, in the new `Jaic_Extensions` module. |

### Heads-up

- The language server binary is renamed `jailsp` (was `jai-lsp`).
- Operator precedence follows Jai, not C: `1 << 2 + 3` is 7 and `10 % 3 * 2` is 4.
- `cast(T)` and `xx` take a following chain of bitwise operators as their operand: `cast(u32) byte << 16` shifts before widening.

<details>
<summary>Other changes (22)</summary>

**Added**

- `jaic run file.jai -- args` passes arguments to the program.
- Metaprogram plugins: `jaic build file.jai -plug Tracy`.
- `jaic --timings` prints time per compile phase.
- The interpreter on Windows calls into DLLs; the Windows runtime gets UTF-8 arguments, a crash handler with a backtrace and DLL exports.
- `Bindings_Generator` matches the official module's API and output, and handles C++ virtual bases, Objective-C blocks and `long double`.
- The browser playground opens a language tour by default.
- `Toolchains`: Android NDK helpers and the macOS SDK path.
- A module parameter without a type stays an untyped constant.

**Fixed**

- `#complete` switches report a missing enum case.
- A struct member with an undefined type is an error instead of an empty struct.
- Linux arm64 system bindings had x86-64 layouts.
- `push_allocator(proc, data)` takes effect.
- Crashes and hangs found by fuzzing (deep nesting, recursion in polymorphs and `#insert`, oversized types).
- `#placeholder` declarations filled by a metaprogram are inserted reliably.
- `pointer & int` and friends keep the pointer type.
- `cast(bool)` of a number tests for non-zero.
- Arithmetic on an untyped struct literal argument takes the parameter's type.
- A `name:` marker in a multi-declaration declares every name.
- `#align N` on a member replaces its natural alignment; `#align 64` heap and stack data is aligned.
- `continue` and `break` in a `for_expansion` body run its defers.
- `#add_context` constants, unnamed `#library,link_always`, `SIG_IGN`-style constants, `Process` pipes and `String_Builder` resets.

**Removed**

- The standalone playground UI; the hosted playground is at https://matteopolak.com/playground/jai.

</details>

## [0.1.0] - 2026-10-05

The first release of `jaic`, an independent Jai compiler written in Rust, with its own standard library and a language server.

### Downloads

| Archive | Contents |
| --- | --- |
| `jaic-macos-arm64.tar.gz` | `jaic` (check, run, build), `jai-lsp`, `stdlib/` |
| `jaic-linux-x64.tar.gz` | `jaic` (check, run, build), `jai-lsp`, `stdlib/` |
| `jaic-windows-x64.zip` | `jaic` (check, run; no native builds yet), `jai-lsp`, `stdlib/` |

Unpack and run `jaic`; it finds `stdlib/` beside itself (or in `JAIC_STDLIB`). LLVM is built in. Native builds use the system linker (Xcode's command line tools on macOS, `cc` on Linux).

### What works

| Area | Support |
| --- | --- |
| Compiler | Type-checks, interprets (`jaic run`) and builds native executables (`jaic build`). |
| Language | Polymorphic procedures and structs, baking, `#modify`, macros, `#code`/`#insert`, `for_expansion`, `using`, `context`, `Any` and type info. |
| Compile time | `#run` in an interpreter that can call native libraries; metaprograms with the `Compiler` module. |
| Safety | Overflow and bounds checks, stack traces. |
| `#asm` | Common x86-64 and arm64 instructions. |
| Bindings | `Bindings_Generator` for C, C++ and Objective-C headers (needs libclang). |
| Standard library | `Basic`, `String`, `Hash_Table`, `File`, `Process`, `Thread`, `Math`, `Compiler`, `Simp`, `GetRect`, `Window_Creation` and more. |
| Tools | `jai-lsp` (diagnostics, hover, completion, go to definition) and a browser playground. |

Projects that check, run or build: Focus, Jails, jaison, sgpu, the examples from *The Way to Jai*, and the `how_to` programs. Vk-Engine checks for Linux.

### Known limitations

- No native Windows executables yet; the Windows `jaic` checks and interprets.
- No native debug information.
- `#asm` lacks string operations, division, x87 and mask registers.
