# Changelog

## [Unreleased]

### New

- `Objective_C/AppKit` adds `NSWindow.close`, `NSCursor`, `NSSavePanel`, `NSOpenPanel` and `NSPasteboardTypeFileURL`.
- The `Codex` module takes the `USAGE_MODE` parameter, `#import "Codex"(USAGE_MODE = .READ)`.
- `jailint` rule levels can be set per path with `[[lint.overrides]]` in `jacket.toml`.
- `jailint` has an opt-in `cognitive_complexity` lint for procedures with too much nested branching, with the limit set by `[lint] max_complexity`.
- `jailint` reports reading a variable declared `= ---` before anything is written to it.
- `jailint` reports changing a variable that bounds a `for` range inside the loop, which does not change how often it runs.
- `jailint` suggests `PI`, `TAU` and the other `Math` constants over hand-typed approximations such as `3.14159`.
- `jailint` reports passing an array's element count to `memcpy`, `memset` or `memcmp`, which take bytes.
- `jailint` reports returning the address of a local variable.
- `jailint` reports returning a value that a `defer` above frees on the way out.
- `jailint` reports integer comparisons that can never pass together or always pass, such as `x > 5 && x < 3` and `x != 1 || x != 2`.
- `jailint` has an opt-in `magic_number` lint for unexplained numbers in arithmetic, comparisons and assignments, tuned in `[lint]`.
- The VS Code extension warns when `PATH` has an older toolchain than it expects, and can switch to its downloaded one.
- `BuildCpp` builds C and C++ on Windows x64 and arm64, with MSVC or `clang-cl`.
- `Autorun` runs workspaces on Windows.
- `Deep_Copy` and `Overwriting_Allocator` work on wasm.
- `Vulkan` works on macOS.
- `Extensions/SIMD` detects CPU features at run time, `#target_features` compiles a procedure for extra ones, and `Extensions/Byte_Search` uses 32-byte versions where available.
- The crash handler reports crashes on Windows arm64.
- The crash handler reports stack overflows, including on threads made with `Thread`.
- `IS_CROSS_COMPILING` and `MACHINE_OPTIONS_SIZE` are available as compiler constants.
- `Module.name` reaches a module's `#scope_module` declarations.
- Polymorphic procedure lenses count instances from every program in the workspace.

### Improved

- `String` searches and `compare` on long text, and `array_find` on integer arrays, are several times faster.
- The `Compiler` module's operations run inside the compiler, so they work with any standard library folder.
- A `#compiler` declaration whose records don't fit its operation is reported where it is declared, and `get_build_options` fills a record of any name with that record's own defaults.
- Settings in `jacket.toml`, `jaifmt.toml` and `jailint.toml` apply when the file is saved.
- Typing in one program no longer makes the language server recheck other open programs that don't use the file.
- Polymorphic procedure lenses update without waiting for unrelated programs to be rechecked.
- Struct literal completion lists fields in declaration order and selects the first one left (#29).

### Changed

- `jailint` rules belong to categories (`correctness`, `suspicious`, `complexity`, `style`, `perf`, `pedantic`, `restriction`) that set their default level and can be set as a group in `[lint.rules]`, with `-A`/`-W`/`-D` and in `allow(...)` comments. `correctness` rules, such as `double_free` and `use_after_free`, are now `deny` by default, so they fail the run.
- The `Fuzz` module moved to `Extensions/Fuzz`.

### Fixed

- A 64-bit signed/unsigned cast evaluated at compile time, `xx` or `cast`, reinterprets the bits instead of failing the range check; the program's own run still checks it.
- `*Node.[first, last]` is an array literal of `*Node`.
- A procedure with a bare polymorphic struct parameter such as `pairs: Table` can still write `Table(string, s64)` in its body.
- A build program with `do_output = false` that reaches `init_plugins` on a branch that does not run no longer fails for the initialization that was never generated.
- A struct with a scalar `#as` member compares as that member when no `operator` takes it, so `a.socket == b.socket` works on a wrapped handle.
- A module's procedures and the same-named procedures of a module it re-exports with `using` form one overload set for its importers.
- Overloads that take constant parameters of different distinct types, such as `($c: Position)` and `($c: Velocity)`, are told apart by the argument's type.
- A global initializer can hold the `data` of a global fixed array, as in `view : [] T = .{0, buffer.data}`.
- `jaifmt` reads the same `jacket.toml` forms as the other tools: single-quoted strings, quoted and dotted keys, and `format = { ... }` inline tables.
- `jacket lint` prints each configuration warning once, not once from `jacket` and again from `jailint`.
- `exclude = [""]` in `[lint]` is reported as an error instead of excluding every file.
- `jailint --fix` keeps a trailing comment when it removes an unused variable or import.
- `jailint`'s `manual_assign_op` no longer turns `a = f() + a` into `a += f()`, which reads `a` before the call instead of after.
- `jailint`'s `needless_bool` no longer rewrites `!(a == b)` to `a != b` for a type that only defines `operator ==`.
- `llvm_options.function_sections` in `Build_Options` is accepted: on Linux targets every function gets its own `.text.<name>` section.
- A module parameter of an `enum_flags` type takes a combination such as `#import "Mod"(MODES = .READ | .WRITE)`.
- Indexing a constant or a returned struct through an `operator *[]` that takes a pointer, as `Enumerated_Array` does, is accepted.
- A file imported while compile-time code is running can declare `#add_context`, instead of failing because the Context was already laid out.
- An overloaded procedure passed to a polymorphic parameter, `$f: (int, int) -> int` or `$F`, is accepted: the member with the declared type is used, or the first member for `$F`.
- An enum literal such as `.A` passed to overloads that include a struct parameter is no longer reported as ambiguous.
- The language server and `jailint --fix` no longer run out of memory on a polymorphic procedure that instantiates itself with ever deeper pointer types.
- An enum value with no name prints as `(enum out of range: N)`, not with the enum's own name.
- `GetRect` and `GetRect_LeftHanded` take `Type_Indicator` as a program parameter, `#import "GetRect"()(Type_Indicator = T)`, shared by every import in the program.
- The `macOS` module declares `OSStatus` and `OSType`.
- `cast(float).5` parses the `.5` as the operand.
- `#overlay(a) b: s64;` works as the single statement of an `#if ... then` in a struct.
- `(#import "Basic").print(...)` works: `#import` can be used inside an expression.
- `#code a.b."c"` accepts a string literal as a member name.
- `#code` ends at a trailing comma, so `.[ #code 2 + 4, ]` parses, and `#code return 5` ends at the call's `)`.
- `using Other;` inside an enum body makes the other enum's names reachable through it, keeping their values, without making them members or shifting the numbering.
- A `return` in code passed to a macro now returns from the function that wrote the code, not from the macro.
- Debug builds check integer casts that change sign, such as `cast(u64)` of a negative number, not only narrowing ones.
- Opening a module file on its own (such as `Extensions/SIMD/module.jai`) no longer reports `#module_parameters` as misplaced.
- `BuildCpp` on Windows accepts `/bigobj` objects and finds Visual Studio's `clang-cl` for the host when targeting another CPU.
- Hovers explain `using,only`, `using,except` and `using,map`.
- `Vector3` and `Vector4` support componentwise `*` and `/`, and `Vector4` gains `multiply`, `distance_squared` and `unit_vector` (#34).
- Copying or changing procedure code keeps the parentheses around casts.
- An equally good fixed-arity overload is chosen over an empty variadic one, and `[]u8` over `[]$T`.
- Nested `using` array fields no longer report a false recursive-size error.
- Completion suggests struct fields inside generic call arguments, such as `array_add(*a, .{})`.
- Code after an empty `case` is no longer shown as unreachable.
- Changing VS Code settings while the language server restarts no longer logs errors.
- Restarting the language server no longer logs "document is not open" errors.
- Compile-time code that runs before the context is set up no longer breaks later compile-time code.
- Unsupported target options set by a build program are reported as errors.
- A `jacket.toml` key written outside its section, such as `import_path` or `build_files`, now says where it belongs.
- The build status line no longer shares a line with output from compile-time code that runs a shell or other native code.
- `#type` aliases can use type variables, as in `#type (value: *$T, body: Code)`.
- `#bake_arguments` keeps `$T` of a baked parameter, including a procedure passed for a parameter that names `T`.
- `#if` accepts float and `Code` constants, including an omitted `$children := #code,null`.
- `for a..b` accepts ends of different integer types.
- `a, b += 3, 4` works on lists of targets.
- Parameters written as a bare polymorphic struct, such as `(a: Table, b: Table)`, each take their own instance.
- `x, y = xx a, xx b` converts each value to its own target's type.
- `a, b:, _ = f()` discards the value of `_`.
- A `#modify` block can assign a `$x := value` parameter.
- `operator* :: #procedure_of_call f(...)` can sit next to ordinary `operator*` overloads.
- A struct can have several `_` members.
- A global declared as `f := (x: int) -> int { ... };` can be called.
- `#load` works in code that a procedure body inserts.
- An anonymous `union { ... }` or `struct { ... }` statement in a procedure brings its members into scope.
- `#bake_arguments f(,, allocator = a)` bakes a context field into the procedure.
- Using a polymorphic `#type` alias such as `#type (value: *$T)` as a value is a compile error, not a crash when the program runs.
- Compile-time code can use a declaration from an `#if` in a file added later with `add_build_file`.

## [0.25.0] - 2026-10-21

### New

- The prelude file can be set with `JAIC_PRELUDE` or the `jai.stdlib.prelude` editor setting.
- A standard library folder whose prelude is a root `Preload.jai` is found without settings.

### Fixed

- `#assert,stallable` parses.
- `#asm` accepts a `?T` size before a memory operand, as in `mov?T [p], v;`.
- `jailint` no longer reports `if flag && DEBUG` and similar target checks as constant conditions.

## [0.24.0] - 2026-10-21

### New

- `Math` has `Quaternion` arithmetic operators (#25).
- Completion suggests the remaining fields inside struct literals, including `x: Foo = { }` (#24).
- `for:` accepts a path to a `for_expansion`, like `Module.name` or `value.Walk` (#23).
- `Bindings_Generator` binds C++ virtual methods on Linux and Windows, so headers like Slang's generate there too.

### Improved

- `jacket.toml` settings are checked: unknown keys are reported and ignored.
- Completions that fit the expected type are listed first.
- An untaken `#if` arm fades together with its header in editors.
- Metaprograms that build large programs use much less time and memory.

### Fixed

- `Bindings_Generator` works from release installs, including on Windows, reads C++ headers as C++, and calls C++ methods from Windows DLLs (#26, #27).
- Go to definition no longer causes "document is not open" errors in folders reached through a symlink.
- Opening a module's `module.jai` in the editor no longer reports `#module_parameters` errors.
- A closed window no longer breaks buffer swaps or GL context selection.

## [0.23.0] - 2026-10-20

### New

- `Extensions/Hash_Map`: a faster hash map that never stores a key twice (#17).
- Five new lints: `dropped_result`, `use_after_free`, `double_free`, `unpaired_lock` and `leaked_allocation` (off by default).
- Many invalid programs are now compile errors, including:
  - ambiguous overloaded calls;
  - writing to a by-value aggregate argument or loop variable;
  - duplicate or non-constant `case` values;
  - `#complete` on a non-enum;
  - loading one file twice;
  - name clashes involving `using`.

### Improved

- Untyped values (`.{...}`, `.NAME`, `xx`, untyped `ifx` arms) take their type from the operand, branch or overload next to them (#20, #21).
- Overload choice picks the closest match and reports ties as errors (#22).
- The jailint declaration notes have clearer names: `@FreesArgument`, `@RemovesFromArray`, `@ReturnsSmaller`, `@ReturnsLarger`.
- Much faster float parsing and printing, with identical results.
- Faster integer printing, string hashing, UTF-8 decoding and hash table inserts.

### Fixed

- `remove` in a `for` over a hash table drops the entry and keeps the count right (#18).
- OpenGL windows on Windows pick a usable pixel format (#19), and the GL renderer no longer aborts on GL errors.
- A core dump when compile-time code aborts the compiler.
- Bit-pattern constants and scalar elements in by-value `for` loops get the right values.
- TLS reads from a fiber, and native cursor creation failures.
- `Random`'s float helpers return correctly scaled values, and a reversed range returns its lower bound.
- A breakpoint on the last statement of a `for` loop stops before that statement runs.
- Several editor fixes.

## [0.22.1] - 2026-10-20

### Fixed

- Installing with Homebrew on macOS works again.
- Editor hovers on polymorphic procedures that are never called and take a baked polymorphic parameter.

## [0.22.0] - 2026-10-20

### New

- `jaic` and `jacket build` show a live build status on the terminal, with `Compiling` and `Finished` lines.

### Fixed

- The browser compiler is about 15 MB smaller and fits static hosts' 25 MiB file limit again.
- `compiler_get_code` prints a `case` that ends in `#through` as code that parses again.
- Reading the macOS clipboard no longer fails when another program changes it during the read.
- Several editor fixes: hovers on polymorphic procedures and their instances, constants whose `#run` failed, the branches of an `if` whose condition has an error, and braces around a broken statement.

## [0.21.1] - 2026-10-20

### New

- `jacket upgrade` updates a script or PowerShell install to the latest release.
- `-exe name` with no folder writes the executable to the current directory.

### Fixed

- Browser and wasm builds have Basic's clock, calendar, `sleep_milliseconds` and `exit`.
- `log` and `log_error` end each message with a newline, as in the official compiler.
- `#module_parameters` in a program file is now an error, as in the official compiler.
- C callbacks that take stack arguments work on Windows arm64.
- Several editor fixes: renames, procedure-type results, statements before a switch's first `case`, constants whose `#run` failed, and hovers while a procedure is being edited.

## [0.21.0] - 2026-10-19

### New

- Two new lints: `constant_condition` (a condition that is always true or false) and `string_literal_write` (writing into a string literal).
- Inlay hints for `it` and `it_index` in `for` loops.
- Code after a call that never returns is faded as unreachable.
- `jaifmt --version` and `jaifmt --is-ignored`.
- `-timings` shows memory allocated per phase.

### Improved

- Rename in the editor is more complete (overloads, polymorphic parameters, loop variables and more) and refuses names it can't find.
- Hover and go-to-definition work in more places, including code not checked yet and module aliases.

### Fixed

- Release archives were missing native libraries, most visibly on Windows (#13).
- Writing into a string literal at compile time is now an error instead of silently changing the literal everywhere (#14).
- Multisampled OpenGL windows on Windows (#15).
- `String.tprint` and other Basic procedures through `String` (#16).
- `jacket run` runs the executable the build actually produced.
- Several error messages point at the right place, and a few invalid programs the reference compiler rejects are now rejected too.
- An editor hang on structs that refer to `#this`.

## [0.20.0] - 2026-10-19

Adds `jacket`, one front door and one config file for the toolchain, along with editor completion and dead-code features and a faster `jaic`. Checking a 60k-line program is about 1.5x faster than in 0.19.1, and building it about 1.4x faster.

### Highlights

- **`jacket`, one command for the toolchain.** `jacket build`, `check`, `run`, `lint`, `fmt` and `lsp` are drop-ins for `jaic`, `jailint`, `jaifmt` and `jailsp`: every argument, single-dash Jai flags included, reaches the tool exactly as typed. What `jacket` adds is the project: with no `.jai` file named it builds the configured or inferred entry file. `jacket new`, `init`, `config`, `clean` and `version` are its own. It ships in the release archives, the installers and the Homebrew formula. See `docs/tools/jacket.md`.
- **One config file, `jacket.toml`.** Its `[project]`, `[toolchain]`, `[lint]`, `[format]` and `[experimental]` tables replace `jai.toml`, `jailint.toml` and `jaifmt.toml`, and `jaic`, `jacket`, `jailsp`, `jailint` and `jaifmt` read the same file. The old files still work; the first `jacket` command next to them writes `jacket.toml` (never under `CI`, never deleting anything), and `jacket config migrate [--dry-run]` does it on demand.
- Entry files are inferred the same way everywhere: the configured entry, then `build.jai` / `first.jai`, then `main.jai` / `src/main.jai`, then `module.jai`.

### Added

- `jailsp` completes a procedure as a call when the client takes snippets: `name($0)` leaves the cursor inside the parentheses and opens parameter hints, `name()$0` for a procedure without parameters. Parentheses are left out where the procedure is a value: before an existing `(`, after `*` or `#procedure_of_call`, inside `type_of(...)`, in a `::` or global `:=` initialiser, and in type positions. Setting `jai.completion.callSnippets` (`completion.callSnippets` in the server's options): `addParentheses` (default), `fillArguments` (a placeholder per required parameter) or `none`. Also in the Neovim plugin as `completion.call_snippets`.
- `jailsp` offers postfix completions after a value: `flag.if`, `.while`, `.for`, `.return` (at the start of a statement), `.not` and `.ref`, which rewrite the whole expression (`if flag { }`, `!flag`, `*flag`).
- `jailsp` shows `// name` as an inlay hint after the closing brace of procedures, structs and enums of 25 lines or more. Setting `jai.inlayHints.closingBraceMinLines` (`0` turns it off), also `inlay_hints.closing_brace_min_lines` in the Neovim plugin.
- `jailsp` completes `if`, `for` and `while` at the start of a statement as a skeleton with the cursor in the body, and `case` inside the braces of `if x == {`.
- `jailsp` fades code that can never run: statements after an exit, branches and loops a condition rules out (also by integer ranges and narrowing), `case`s a value cannot select, and `#if`/`ifx` arms that were not taken. They arrive as `unreachable` hints tagged `Unnecessary`, with no squiggle.
- The browser formatter reads `jacket.toml`: the playground driver finds the nearest `jacket.toml` or `jaifmt.toml`, and `jaifmt.wasm --config-name jacket.toml` takes `jacket.toml` text.
- A misplaced `remove` says why it cannot apply: the loop is over a fixed-size array (with the `[..]` and `[]` types that would work), a range of numbers or is a `while` loop, or there is no enclosing loop. It used to say only that `remove` is valid inside a for loop over an array.

### Performance

- Source files are lexed and parsed on worker threads ahead of type checking. Workers crawl the `#load` and `#import` graph from the main file, and type checking takes each finished file from a shared cache. Output is identical to a serial parse. `-serial_parse` turns the workers off, and `-parse_threads N` sets their number (default: cores - 1, at most 4).
- Name lookup no longer copies `using` and import lists on every lookup, skips exported-`using` probing for modules that have none, and remembers scopes whose imports hold nothing under a name.
- On macOS, LLVM's own C++ allocations also go through mimalloc.
- Finding procedures that need no stack-trace instrumentation uses one parallel scan and a worklist instead of repeated passes.

### Fixes

- In a polymorphic procedure that nothing calls, a declaration initialised by a call to another polymorphic procedure (`found, index := array_find(array.*, item)`) got the type hint `: ?` and a `?` hover, because the editor refused the call. The call now takes the callee's signature with the caller's own `$T`, so the hints read `: bool`, `: s64`, `: T`, `: [] T`; a hint is never `?`.
- `jailint --allow-incomplete` prints how many programs did not compile completely on the target instead of one line for each; `-v` lists them.
- A baked `$xs: [$N] T` parameter takes `$N` from the element count of a constant array view (`xs :: #run make()`), where it used to say "array kind mismatch".
- Assigning to an indexed expression (`=` and the compound assignments) on a type with `operator *[]` calls it and writes through the returned pointer, where it used to say "cannot assign to a value of type"; reads keep using `operator []`.
- A `[$N] T` parameter takes `$N` from the element count of an untyped `.[...]` argument or default (`f(.["a", "b"])` makes `N` 2), where it used to say it could not infer `$N`. A default `.[...]` whose length differs from the inferred count is an error at the default; `[$N] $T` still cannot get `$T` from an untyped literal.
- Vectors, quaternions and square matrices of the `Math` module compare with `==` and `!=`, where it used to say no `operator==` matched.
- `a op= b` is `a = a op b` when no `operator op=` accepts the operands (`v /= 2.0` with only `operator /= (*Vector2, Vector2)` and a `Vector2 / float`), where it used to be a no-overload error.
- A `for_expansion` taking `*T` iterates a `T` value through its address when other `for_expansion` overloads in scope have polymorphic first parameters that do not match, where it used to say none matched.
- A `[$N] T` module parameter with a `.[...]` default has the default's count when the import gives no value, and can be forwarded to another module's parameter.
- An overload that is not polymorphic is preferred to `$T` when the argument needs only a mild conversion (a wider integer, a pointer to the struct), as in the reference compiler; an integer is still not made a float for it.
- An untyped `.{...}` argument no longer matches a `string` or other built-in aggregate parameter whose members its values cannot fill (an array literal for `data`).
- `value[xx x]` on a struct with `operator []` casts to the parameter of the overload picked for the struct, where it used to cast to an integer.
- In `jailsp`, a statement missing only its `;` at the end of a line keeps working: goto-definition, hover and highlighting work on it, and the error is shown on its last token instead of covering the statement's first name.

### Breaking changes

| Before | Now |
| --- | --- |
| `jai.toml`, `jailint.toml`, `jaifmt.toml` | `jacket.toml` (the old files are still read, with a hint to migrate; a `jacket.toml` in the same directory wins) |
| `jai.toml` `build_files` | `[project] entry` |
| `JAIC_PARSE_THREADS=N`, `JAIC_SERIAL_PARSE=1` | `-parse_threads N`, `-serial_parse`; `[experimental]` for projects |
| `JAIC_CODEGEN_UNITS=N` | `-codegen_units N` (`-no_split` for 1); `[experimental] codegen_units` |
| `JAIC_PROFILE=1`, `JAIC_PROFILE=ops` | `-profile`, `-profile_ops` |
| `JAIC_COVERAGE=FILE` | `-coverage_file FILE` |
| `JAIC_VERIFY_IR`, `JAIC_SPLIT_UNITS`, `JAIC_NO_SINK`, `JAIC_NO_FLOW`, `JAIC_DUMP_OPS`, `JAIC_BODY_PROBE`, `JAIC_BODY_ORDER`, `JAIC_O0_PASSES` | `-dev_verify_ir`, `-dev_split_units`, `-dev_no_sink`, `-dev_no_flow`, `-dev_dump_ops`, `-dev_body_probe`, `-dev_body_fifo`, `-dev_o0_passes` (for working on the compiler; may change) |
| `JAIC_CODEGEN_UNITS=4` for CI's native tests | `JACKET_TEST_CODEGEN_UNITS=4` (the test helpers pass `-codegen_units 4`) |

Kept: `JAIC_STDLIB`, `JAIC_NATIVE_LIBS`, `JAIC_LINKER`, `JAIC_AR`, `JAIC_CACHE_DIR`, `JAIC_MEMORY_LIMIT`, `JAIC_DIAGNOSTICS`, `JAIC_DSYM`, `JAIC_COVERAGE_DIR`/`JAIC_COVERAGE_FILE` (read by the instrumented program), `JAIC_SANITIZER_CC`, `JAIC_WASM_LD` and the installer variables. They locate things on the machine or are read where a flag cannot reach, rather than tuning one project.

## [0.19.1] - 2026-10-18

Restores stdlib compatibility and makes `jaic` faster. Several stdlib changes in 0.19.0 made observable behaviour differ from the official standard library; they are reverted, and UTF-8 decoding of malformed input now matches it too.

### Performance

- `jaic` uses mimalloc instead of the system allocator, which makes checks and builds about 7–13% faster (a 60k-line check takes 137 ms instead of 154 ms).

### Fixes

- `split_from_right(s, byte)` returns `(false, "", s)` again when the byte is absent.
- `scan` checks the `%` that `%%` matches but leaves it unconsumed again.
- `Sort.quick_sort` is no longer deprecated, and `Sort` no longer exports `intro_sort` or `heap_sort`. `quick_sort` keeps the faster introsort from 0.19.0, and its key form still computes each key once. `IntroSort.intro_sort` is unchanged.
- Strict `character_utf8_to_utf32` again reports only the bytes before a bad continuation byte as consumed, which is 0 when the first continuation is bad.
- `character_utf32_to_utf8` encodes surrogates (U+D800 to U+DFFF) as themselves again.
- `find_index_from_left(s, substring, start_index)` asserts on a negative `start_index` instead of treating it as 0.
- `Window_Creation` builds for Android again instead of failing with an `#assert`.
- `character_utf8_to_utf32` decodes malformed input to the payload bits of its bytes: a continuation byte without its `10` marker contributes its low six bits, and a stray continuation byte decodes as 0 in one byte, also in strict mode.

## [0.19.0] - 2026-10-18

Faster default builds and compile-time string code, a much faster `jailsp` on large workspaces, stdlib fixes found by fuzzing and review, and a new `excessive_nesting` lint. This release includes the changes planned for 0.18.1, which was not published separately. The breaking changes are small stdlib behaviour fixes, listed below.

### Highlights

- **`jailsp` keeps up with typing.** In this repository a keystroke is re-analysed in about 60 ms (it was about 700 ms when editing a stdlib file), and the first hover or completion answers in under 10 ms instead of about half a second. The server keeps its `#load` graph and per-module indexes across edits instead of rescanning the stdlib, and workspace symbols no longer resolve every file path. On very large single programs the remaining cost is the full type check.
- **Faster default builds.** Library files skip the `#if OS`/`CPU` branches the target rules out while they are lexed and parsed, which removes most of an empty program's front-end time. Default builds are about 10% faster, and small leaf procedures no longer get a stack-trace node, so debug executables run up to 13% faster.
- **Faster compile-time string code.** `String_Builder` appends, integer printing and float formatting take far fewer steps, so string-heavy `#run` code runs about twice as fast.
- **Stdlib and `#load`ed files analyse correctly.** A file inside a standard library tree is checked against that library, so its `internal/...` imports are allowed. A file that another module `#load`s is checked as part of that module.
- **New lint: `excessive_nesting`.** `jailint` reports blocks nested deeper than `max_nesting` (also `--max-nesting`).
- **Coverage builds.** `jaic -coverage` builds a program that records which code ran; `tools/coverage/report.py` turns that into line and function coverage. See [coverage](https://github.com/matteopolak/jacket/blob/main/docs/tools/coverage.md).
- **Fuzzing builds.** `-sanitize fuzzer` (with `address`) instruments a program for libFuzzer and links its engine. See [sanitizers](https://github.com/matteopolak/jacket/blob/main/docs/native/sanitizers.md).

### Breaking changes

- `split_from_right(s, byte)` returns `(false, s, "")` when the byte is absent, as the `string` overload and `split_from_left` already did. It returned `(false, "", s)`.
- `scan` consumes the `%` that `%%` matches, as `scan2` does. `scan("%%", "%")` now succeeds; it used to fail on the unconsumed byte.
- `Sort.quick_sort` is deprecated. The comparator form was a heap sort and the key form an insertion sort. Use `intro_sort` (a real introsort, also in `Sort`; the key form computes each key once and keeps equal keys in order) or `heap_sort`. `quick_sort` forwards to `intro_sort` for one release.
- Strict `character_utf8_to_utf32` reports at least the lead byte as consumed when a continuation byte is invalid (it could report 0). The consumed count for such input is now 1 or more.
- `Window_Creation` fails with a clear `#assert` on Android instead of leaving its procedures undefined.

### Fixes

- A decode loop over invalid UTF-8 no longer stalls on a bad first continuation byte.
- `character_utf32_to_utf8` encodes surrogates (U+D800 to U+DFFF) as U+FFFD.
- `find_index_from_left(s, substring, start_index)` treats a negative `start_index` as 0 instead of reading before the string.
- `table_find_multiple` probes from the key's slot instead of scanning every entry, and a `REFILL_REMOVED = false` table no longer doubles under remove/add churn.
- Substring search skips to the first byte of the substring, about ten times faster on large text.
- A declaration whose initializer has an error still declares its name, so one mistake no longer causes a cascade of "unknown identifier" errors.
- Completions rank named arguments first, then locals, the file's own names, the rest, internal names and auto-imports.
- In auto-import, a workspace's own stdlib is offered only through its public modules.
- `jailint` and `jaic` render diagnostics in linear time, so a file with thousands of findings no longer takes seconds to report. `jailint` also caches neighbouring files between runs.
- The interpreter runs compile-time code faster: dead local setup is removed, jumps are straightened, and copies, fills and tests use dedicated operations.
- On Windows, a compile-time call no longer loses the program's last error while the interpreter sets up a frame, and a failed check is reported at its own line.
- A compile-time call into a foreign library that is missing names the library.
- Path completion works inside a closed `#import "..."` and with `,file` and `,dir`; a completion after `.` with no expected type is about ten times faster.
- `Machine_X64` can be imported on every target, with feature checks that report no features and no-op hints off x64.
- A missing `#import,file` prints only its path in the diagnostic.
- `#run` with no result as a constant's value now says that it has no result.
- `Input` keeps its per-frame arrays on the default allocator when a program sets the context allocator to temp.
- `Simp` resolves its GL procedures once per context instead of on every draw.

### For tool authors

- `jaic` exposes more of what the editor tools need, and `jailsp` and `jailint` now use it instead of their own copies: `jaic::project` (settings, directive scan, load closure, root resolution, loader graph), `jaic::paths` (normalisation, canonical caches, stdlib discovery), `jaic::source::LineIndex`, `jaic::lexer::lex_with_trivia` (comments), `#asm` block lookup, and `ide_facts::cursor_context`.
- The message callback receives a `ReportSeverity` (`ReportSink`).

## [0.18.0] - 2026-10-17

The project has a new name: **Jacket** ("Jai kit"). This release also lets editors open code you don't trust safely, and makes `jailint` faster and easier to extend.

### Highlights

- **New name, same tools.** The repository is now [matteopolak/jacket](https://github.com/matteopolak/jacket). `jaic`, `jailsp`, `jaifmt` and `jailint` keep their names, and so do the crates.
- **Workspace trust.** In an untrusted folder, `jailsp` checks your code without running any of its compile-time code (`#run`, `#insert`, build metaprograms). VS Code follows its own workspace trust setting; in Neovim, pass `trusted = false` to `setup()`. See [workspace trust](https://github.com/matteopolak/jacket/blob/main/docs/tools/workspace-trust.md).
- **`jailint` is several times faster on large trees.** It lints each file once and compiles programs in parallel (`-j`). `--allow-incomplete` lets you lint a tree where some modules only build on another OS.
- **Better switch completion.** The "add missing cases" action now works on every kind of `if x == { case ...; }`, and its error covers the switch's header instead of the whole block.
- **Native console colours on Windows.** `Print_Color` sets console colours through the Windows API, so they also work in consoles that don't understand ANSI escapes.

### Breaking changes

| What | Was | Now |
| --- | --- | --- |
| VS Code extension | `matteopolak.jai-toolchain` (and `matteopolak.jai` before that) | `matteopolak.jacket`. It offers to uninstall the old one, and your `jai.*` settings carry over. |
| Neovim plugin | `matteopolak/jai.nvim` | [`matteopolak/jacket.nvim`](https://github.com/matteopolak/jacket.nvim). The `jai` Lua module and the `:Jai` command are unchanged. |
| Homebrew | `matteopolak/tap/jai` | `matteopolak/tap/jacket`. `brew upgrade` moves you over. |
| Release archives | `jai-<platform>.tar.gz` | `jacket-<platform>.tar.gz` (`.zip` on Windows), unpacking to `jacket-<platform>/`. |
| How `jailint` recognises procedures | by name (`trim`, `min`, `free`, ...) | by a note on the declaration: `@UseResult`, `@Minimum`, `@Maximum`, `@RemovesElements`, `@Releases`. The standard library already has them; add them to your own procedures if you want the rules to apply. See [declaration notes](https://github.com/matteopolak/jacket/blob/main/docs/tools/jailint.md#declaration-notes). |
| ImGui and Curl printf-style procedures (`Text`, `TreeNode`, `curl_msnprintf`, ...) | a Jai `string` format, forwarded through a libffi bridge that releases never shipped | the C functions themselves: the format is a C string and the arguments are C varargs. Pass text arguments as `*u8` (`cast(*u8) "ok"`, `name.data`), and `TextColored` takes `*ImVec4`. The `_CFormat` names are gone. |

Earlier releases have been removed. Install 0.18.0 or later.

### Fixes

- A `*void` converts to a procedure type without a cast, as it should.
- `#no_reset` is accepted alongside other declaration modifiers.
- A variable is never treated as a constant, even when its initializer is one.
- The interpreter runs `memcpy`, `memset` and `memcmp` without extra system calls, so memory-heavy programs under `-interp` run faster, especially on Linux.
- On Linux, a shared library marked `link_always` is no longer dropped by `--as-needed`.
- On X11, an empty clipboard reads as empty text instead of failing, and fullscreen works without a window manager.

## [0.17.0] - 2026-10-13

One `jaic` command line instead of the `run`, `check` and `build` subcommands, with more options for the program being built.

### Breaking changes

`jaic` has one command line, `jaic [options] file.jai [options] [+Plugin ...] [- metaprogram args] [-- program args]`, and no subcommands. A build is the default. See [the command line](https://github.com/matteopolak/jacket/blob/main/docs/tools/cli.md).

| Was | Now |
|---|---|
| `jaic run file.jai [-- args]` | `jaic file.jai -interp [-- args]` |
| `jaic check file.jai` | `jaic file.jai -check` |
| `jaic build file.jai` | `jaic file.jai` |
| `-o path` (the output file) | `-exe name` and `-output_path dir`; `-exe` may carry a directory and extension. `-o` now means an optimized build |
| `--emit-ir file.ll` | `-emit_ir file.ll` |
| `--no-debug-info` | `-no_debug_info` |
| `--timings` | `-timings` (read by `tools/bench`) |
| `--sanitize`, `--target` | `-sanitize`, `-target` |
| `--color never`, `-color` | `-no_color`; `FORCE_COLOR=1` forces colour on |
| `-plug Name`, `-plugin Name` | `+Name`; every argument after it, up to the next `+Name`, `-` or `--`, is that plugin's own option |
| `jaic help`, `jaic version` | `-help`, `-version` |

The old forms are errors that name their replacement, including `jaic run|check|build file.jai`, which prints the equivalent new command. Plugins now receive only their own options (`Metaprogram_Plugins.build_with_plugin_groups`; `build_with_plugins` keeps its shared list). `--` with a build or `-check` is an error.

### Added

- `-add "code"` and `-run "code"` add code to the target program and run code at compile time; `-exe` and `-output_path` name the executable.
- `-o`/`-optimized`, `-od`/`-optimized_debug` and `-very_debug` pick a build flavor; `-no_inline`, `-no_split` and `-no_backtrace_on_crash` switch single behaviors off.
- `-no_color`, `-msvc_format` (`path(line,col): error: message` lines), `-quiet`, `-verbose`, `-no_check` and `-no_check_bindings`.

## [0.16.1] - 2026-10-13

Optimized builds of large programs, faster TLS and JSON encoding, and fixes in `Io`, `Http` and `Input`.

### Highlights

- `-O2` build time now grows linearly with program size: a 250,000-line program builds in 17 s instead of 301 s on 4 cores. See [compile speed](https://github.com/matteopolak/jacket/blob/main/docs/compiler/compile-speed.md#fourth-pass--o2-time-grows-linearly).
- Typed `to_json` is 1.2x to 1.5x faster, with byte-identical output.
- TLS streams batch records into fewer system calls, about 10% more throughput per stream.
- `Http/bench/server_bench.jai` reports p50 to p99.9 latency and can run at a fixed request rate.
- New: `Tls.cipher_suite(stream)` and `Tls.ended_without_close_notify(reader)`.

### Fixes

- Windows `Input`: scrolling down, or a cursor or window left of or above the primary monitor, no longer crashes with a cast overflow ([#1](https://github.com/matteopolak/jacket/issues/1)).
- `Http` client: an https body read until the connection closes ends normally when the server skips close_notify.
- io_uring: a burst of over 128 connections could leave queued connections unaccepted.
- epoll: timers no longer fire up to a millisecond late.

## [0.16.0] - 2026-10-13

HTTPS in the async IO stack, and a round of hardening and speed for `Extensions/Io`, `Http` and `Json`.

### Highlights

- `Extensions/Tls`: TLS 1.2 and 1.3 on a bundled mbedTLS, with the system's trust store on Linux, macOS and Windows. `https://` works in the `Http` client, and `app.config.tls` serves HTTPS. See [tls](https://github.com/matteopolak/jacket/blob/main/docs/extensions/tls.md).
- An idle keep-alive connection costs about 16 KB instead of 35 KB: it holds no read buffer while it waits (`Io.wait_readable`).
- epoll serves 10% to 17% more HTTP requests per second (edge-triggered, one registration per socket); io_uring accepts with one multishot request per listener.
- `Json` parses 1.3x to 1.7x faster and reads floats up to 6x faster (Eisel-Lemire), with float64 printed by Schubfach; results are unchanged.
- Name lookups for the same host share one system call, and lookup threads grow on demand.

### Breaking changes

Names are now consistent across Io, Tls, Http and Json (see [conventions](https://github.com/matteopolak/jacket/blob/main/docs/extensions/extensions-conventions.md)), and internals are no longer exported.

| Was | Now |
| --- | --- |
| `Json.json_error_text(error)` | `error_text(error)` |
| `Client_Error_Kind.TIMEOUT` | `Client_Error_Kind.TIMED_OUT` |
| `Client_Error_Kind.UNSUPPORTED` (for `https://`) | gone: https works |
| `Http.Error_Kind` (the parser's) | `Parse_Error_Kind` |
| `Io.Fiber_Context`, `fiber_init`, `fiber_switch`, `fiber_free`, `Timer`, `Scheduler`, `Pipe_Link` | private to `Io` |
| `Http.header_table`, `header_hash`, `make_header_table`, `HEADER_TABLE_SIZE`, `DAY_NAMES`, `MONTH_NAMES`, `Connection` | private to `Http` |
| `Json.decode_document`, `pair_kind`, `Pair_Kind` | private to `Json` |

Programs that import `Http` now link mbedTLS (about 1 MB).

### Fixes

- `Io`: a lookup answered before its fiber parked, a pipe read whose deadline fired, and an epoll registration the kernel refused no longer wake a fiber twice (`woke a fiber that is not blocked`).
- `Io`: io_uring completions are safe if a completion handler submits more work.
- `Http` client: `close(*response, now = true)` closes the connection instead of draining the body.
- `Tls_Error` text no longer points into the error value, so a copied error keeps its message.
- `Server_Config` documents what 0 and negative mean for each timeout.

## [0.15.0] - 2026-10-12

### Added

- `Atomics`: ordered operations with a `Memory_Order` (`RELAXED`, `ACQUIRE`, `RELEASE`, `ACQ_REL`, `SEQ_CST`): `atomic_load`, `atomic_store`, `atomic_exchange`, `atomic_compare_exchange` and `atomic_fetch_add/sub/and/or/xor`. They compile to single LLVM atomic instructions with the order you name (native and wasm) and run as sequentially consistent in the interpreter.
- `Extensions/Io_Uring` (Linux only): the io_uring kernel interface. Constants, enums and structs are generated from `linux/io_uring.h` by `tools/generators/gen_io_uring.py` with layout `#assert`s; `io_uring_setup/enter/register`; and a `Ring` with `ring_init`, `get_sqe`, `submit`, `peek_cqe`, `wait_cqe`, `cqe_seen` and `probe_ops`. 128-byte entries and 32-byte completions are supported.
- `Extensions/Io`: cooperative fibers on a per-thread scheduler (`io_run`, `spawn`, `yield`, `sleep`, `cancel`) with a bounded `Channel(T)`, `Wait_Group` and `Fiber_Mutex`. Fibers get guard-paged, pooled stacks and their own copy of the context and temporary storage. Not available on wasm.
- The `fiber_init`, `fiber_switch` and `fiber_free` intrinsics switch stacks in native builds (x64 SysV and Win64, arm64) and under `jaic run`; wasm builds refuse them.
- `Extensions/Json`: `to_json` and `from_json` with encoders and decoders generated from your types at compile time, `@json(...)` attributes, exact integers, shortest round-trip floats, strict RFC 8259 parsing checked against JSONTestSuite, and SIMD string scanning. `from_json(Json_Value, text)` gives a dynamic tree, one `free_json` overload set frees decoded values, trees and errors, and `from_json(T, Json_Source)` decodes from a `fill` callback. `from_query` and `from_param` decode URL queries, forms and path text into typed values.
- `Extensions/Http`: a sans-IO HTTP/1.0 and 1.1 codec. Incremental request and response parsers for input in pieces of any size (Content-Length, chunked with trailers, read-until-close, HEAD/1xx/204/304 rules, `Expect: 100-continue`, keep-alive and pipelining) with strict RFC 9112 framing against request smuggling and limits that map to 400/413/414/431/501/505, writers for heads and chunked bodies that choose the framing, a `Date` cache, and SIMD line and field scanning. Checked against llhttp's fixtures, a smuggling table and chunking tests (`tools/stdlib/http_conformance.py`).
- `Extensions/Io` IO: a completion engine with io_uring (Linux 6.1+, with graceful fallback), epoll and kqueue backends (`JAI_IO_BACKEND` forces one), and fiber `tcp_listen`/`accept`/`connect`/`read`/`write`/`close`, file open/read/write, `resolve` on helper threads, `with_deadline`, IO-aware `cancel`, a portable `Io_Error` with `last_platform_code()`, and `io_run_threads` with `SO_REUSEPORT` listeners.
- `Extensions/Io` on Windows (x64 and arm64): I/O completion ports for sockets (AcceptEx, ConnectEx, WSARecv, WSASend, CancelIoEx) and files, and an IoRing for files on Windows 11 22H2 and newer, loaded from KernelBase.dll at run time so older Windows keeps using completion ports. `JAI_IO_BACKEND=iocp` or `ioring` forces one; `backend_reason()` says why a fallback happened. Name lookups work too. The Io tests, which printed a skip on Windows, now run there.
- `Extensions/Io` file thread pool for epoll and kqueue: reads and writes longer than `JAI_IO_FILE_INLINE` bytes (default 32768) run on `JAI_IO_FILE_THREADS` worker threads (default 4, 0 for none) instead of blocking the scheduler's thread, and wake it through an eventfd or an `EVFILT_USER` event. `Backend` gains `.IOCP` and `.IORING`.
- `Extensions/Http` server: `get(*app, "/users/{id}", handler)` registers a plain function; its parameter types (`Path`, `Params`, `Rest`, `Query`, `Json`, `Form`, `Header`, `Header_Optional`, `Bytes`, `Text`, `*Request`, `*State(T)`) say what it needs and are checked against the route at compile time, with errors that point at the route. A radix router with 405/`Allow`, automatic HEAD and OPTIONS and startup conflict detection; return values map to JSON, text, bytes or `Response`; `Http_Error` helpers send RFC 9457 problem documents; middleware and nested groups; `serve` with one scheduler per core, limits and timeouts, keep-alive, pipelining, `Expect: 100-continue` and graceful shutdown on SIGINT/SIGTERM.
- `Extensions/Io`: scheduler-local storage and an interrupt flag (`catch_interrupts`, `interrupted`); `make_pipe`, in-memory duplex streams that honour deadlines and cancellation; `in_scheduler()`. `Reader` and `Writer` are small handles (a data pointer and procedures) over a file, a TCP connection, a pipe or a source of your own (`reader_of`, `writer_of`), and every source has the same operations: `read`, `read_full`, `write`, `write_some`, `read_all` and `close(x, now = false)`. `with_deadline(ms, #code { ... })` is the deadline primitive. `Io.read_json(reader, T)` decodes JSON straight from a stream. See `docs/extensions/io-streams.md`.
- `Extensions/Http` client: a `Client` value (`base_url`, default headers with `set_header`, `timeout`, `max_redirects`, a pluggable `transport`; a zeroed `api: Client;` is ready to use), `send` and the verbs `get`, `post`, `put`, `patch`, `delete`, `head` and `options`, a per-scheduler connection pool, redirects and DNS through Io. Plain http only. A request returns a response with the status and headers, and the body is read from it afterwards with `read_text`, `read_bytes`, `read_json(*response, T)` or, in pieces, `read_some`. A connection goes back to the pool when its body has been read to the end; `close(*response)` releases one that was not. `error_for_status` turns a non-2xx response into an error. Errors are a `Client_Error` (`TRANSPORT`, `TIMEOUT`, `INVALID_URL`, `UNSUPPORTED`, `REDIRECTS`, `DECODE` with the `Json_Error`, `BODY`, `TOO_LARGE`, `STATUS`). Calls made outside `Io.io_run` run a scheduler for that call.
- `Extensions/Http` testing: `test_client(*app)` is the ordinary client wired to an `App` through an in-memory connection, so end-to-end tests need no socket and no `io_run` while the whole stack (codec, keep-alive, middleware, routing, extractors, error mapping) still runs, deterministically. `expect_status`, `expect_json`, `expect_problem`, `expect_header`, `expect_text`, `expect_redirect`, `expect` and `expect_equal` print the request, the response with its body and the caller's location when they fail, and `run_tests` runs named tests with a summary and a non-zero exit code. `test_client(*app, .{ over_tcp = true })` runs the same tests over a loopback socket. `examples/http/crud_test.jai` tests the notes API, now split into `notes.jai` and `crud.jai`.
- `tools/stdlib/http_conformance.py` also builds an echo server and probes it with smuggling shapes, limits and a SIGINT shutdown. `stdlib/Extensions/Http/bench/server_bench.jai` measures requests per second.

### Fixes

- Windows: `Extensions/Io` creates its IoRing with a version that accepts file writes (`IORING_VERSION_3`, 300, when the system has it) instead of always version 2, which refused them, and treats `ERROR_TIMEOUT` from `GetQueuedCompletionStatusEx` as a timeout.
- Windows: `CreateEventW` under `jaic run` now makes a real kernel event, so it can be an `OVERLAPPED`'s `hEvent`; the interpreter's scheduler still waits on it and sees the kernel signal it.
- Windows: the main workspace's default `output_path` uses `/` like `#file`, and `Bindings_Generator` types enums without negative values `u32` for non-Windows targets when run on Windows.
- Windows: `find_visual_studio_in_a_ridiculous_garbage_way` returns the folder of `link.exe` rather than the file, and `find_windows_kit_root` returns the versioned `Lib` folder of the SDK, which is how upstream projects (Focus) build their link commands from them; `find_windows_kit_install_root` is the registry's root.
- Windows (MSVC): the `system_libraries` of a custom link command end with the C runtime libraries jaic's own link names (`msvcrt.lib`, `vcruntime.lib`, `ucrt.lib`, `oldnames.lib`, `legacy_stdio_definitions.lib`), so a command with `/NODEFAULTLIB` such as Focus's links; the third-party smoke test now builds Focus on Windows.
- MinGW cross builds no longer fail in `ld.lld` on globals whose names contain spaces: COFF symbols are not given hidden visibility, which LLVM turned into unquoted `-exclude-symbols:` directives.
- A baked `$loc := #caller_location` default is the location of the call, not of the declaration.
- `string_to_float64`, `string_to_float` and the Json float decoder are correctly rounded: about 1 in 600 random doubles used to parse one ulp off, and float32 results are now rounded once from the digits instead of through a float64. Json shares Basic's conversion; its own copy is gone.
- `from_json` frees everything it allocated when decoding fails (and returns a default value), including the error message of an indexed attempt that was parsed again; a repeated key in a `Hash_Table` object no longer leaks the earlier value. New `free_json(*value)` releases a decoded value.

## [0.14.1] - 2026-10-12

### Added

- `FormatStruct.recursive_long_form_depth`: nested structs inside a long-form struct print in long form too, for that many levels (`-1` for all).

### Fixes

- `Bindings_Generator` enums match the official generator's: a shared camel-case prefix is stripped (with the C names kept as aliases), member comments are left out, empty structs print as `struct {}`, Windows targets get `s32` for plain C enums, and negative values in unsigned enums print as hex.

## [0.14.0] - 2026-10-12

### Breaking changes

- `Regex.Static` `Groups` enums name their members in upper snake case, like other Jai enums: `cat|dog` gives `.CAT` and `.DOG`, `hotDog` gives `.HOT_DOG`. Words that spell the same member (`dog|DOG`) fold into one.

### Fixes

- Regex patterns long enough to overflow the wasm call stack no longer crash when compiled.
- Case-insensitive word groups (`(?i)(?<answer>yes|no)`) become enums too.

## [0.13.0] - 2026-10-12

Three new stdlib extensions: regular expressions that match Rust's `regex` crate and run about as fast, portable SIMD vectors, and vectorized byte search.

### Highlights

- `Extensions/Regex`: Rust `regex` syntax, full Unicode, linear time. `Regex.Static` compiles patterns with the program and fills typed fields from named groups. See [Regex](https://github.com/matteopolak/jacket/blob/main/docs/extensions/regex.md).
- `Extensions/SIMD`: fixed-width vectors (`u8x16`, `f32x4`, ...) with the same results on every target and real vector instructions in native builds.
- `Extensions/Byte_Search`: byte, byte set and substring search at memory bandwidth, on par with `memchr`.
- Native builds honour `inline` and `no_inline`, and leave stack trace nodes out of more procedures.
- New intrinsics `byte_lookup` and `sign_mask` lower to vector instructions; `#asm` `pshufb` and `pmovmskb` use them.

### Breaking changes

- winget is no longer offered; install on Windows with `install.ps1` or the release zip.

### Fixes

- `for` over a range starting with a directive (`for #char "a"..#char "e"`) parses correctly.
- Constants an `#insert` declares in a struct body are visible to the members after it.
- An `#expand` macro can backtick-declare a name equal to one of its parameters.
- The address of a `#run` struct constant is static, so returning it no longer dangles.
- `type_info` of a generic struct without arguments works, for `#modify` comparisons.
- A `#modify` can name a value polymorph inferred from an argument type (`$N` in `Vec($T, $N)`).
- `size_of` of a non-constant `Type` is a compile error pointing to `runtime_size`.

## [0.12.0] - 2026-10-11

Projects built by a metaprogram, like Focus, now check, hover and go to definition through it, and the language server answers edits faster with much less memory.

### Highlights

- Projects whose `first.jai` builds a workspace are checked through that metaprogram, so their files hover and go to definition.
- Code under another target's `#if OS == ...` or `#if CPU == ...` branch hovers too: the server compiles for that target when asked.
- Unused polymorphic procedures and macros, named results, `#foreign` parameters and library names now hover.
- Faster and leaner on large projects: an edit in Focus answers in about 0.7 s instead of 1.3-1.7 s, with less memory.
- Compile-time code runs faster, and metaprograms get their messages and syntax trees several times faster.

### Fixes

- `MeowHash` works on x64 and matches the official Meow hash 0.5 digests.
- `syscall` works in `#asm` blocks on x86-64.
- `*Name` of a polymorphic struct without arguments is a pointer to it, not an error.
- A loop value named `it_index` no longer causes an internal compiler error.
- Hovering a word right after a non-ASCII character works.
- Modules in the project's own `modules/` folder are checked through the importing program.
- `#scope_module` names are reachable from importers as `Module.name`.
- `#bake_constants` fixing only some polymorphic types keeps the procedure polymorphic in the rest.
- `a, b :: 1, 2;` gives each name its own value.
- A missing `#import,dir` or `#import,file` path is reported as written, with help.
- Freezing uninitialized memory at compile time no longer aborts the compiler.

## [0.11.0] - 2026-10-11

Hover and go to definition now reach almost every name in a program, and every directive explains itself.

### Highlights

- Hover and go to definition on more names: fields reached through a `using` parameter, named arguments, struct literal fields, `,,` context arguments, a polymorph's `$T` and its uses, `for name:` over a `for_expansion`, `break`/`continue` labels, `a=,` targets, procedure type parameters, tagged union tags and `#place`/`#overlay` targets. Enum names go to their declaration.
- Hover on every directive (a one-line description) and on cast flags such as `trunc` and `no_check`; a polymorphic struct's fields hover with their declaration.
- `examples/ide_coverage.rs` in the language server crate lists the names of a project that get no hover or go to definition.

### Breaking changes

- The language server's compile errors come from `jaic` with codes `check`, `parser`, `lexer`, `source` and `limit` (they were `jai` with `jai-check` and so on), so editors show `jaic(check)`. Filters on the old source or codes need the new names.

### Fixes

- An incomplete `#complete` switch keeps hover and inlay hints inside its cases, and its error has an "Add missing cases" quick fix.
- Hover, signature help and completion show a `#bake_arguments` procedure under its own name, without the baked parameters.

## [0.10.0] - 2026-10-11

Jai now has a Neovim plugin, and doc comments behave in every popup while you type.

### Highlights

- **Neovim plugin**: [jai.nvim](https://github.com/matteopolak/jai.nvim) sets up `jailsp`, `jaifmt` formatting, `jailint` and syntax highlighting, and can download the toolchain for you (`:Jai install`). Install it with `vim.pack.add({ 'https://github.com/matteopolak/jai.nvim' })` or lazy.nvim. See [the Neovim plugin](https://github.com/matteopolak/jacket/blob/main/docs/tools/neovim-plugin.md).
- Jai code in doc comment fences is highlighted by the language server, so every editor colours it.
- Doc comment links reach the fields and parameters of polymorphic structs (`[Ring.items]`).
- The playground tour shows off doc comments in its basics and polymorphism stops.

### Fixes

- Completion shows a declaration's whole doc comment after edits above it, instead of cutting it short.
- Signature help shows docs, with working links, while the call is still unclosed.

## [0.9.0] - 2026-10-10

Doc comments are now real documentation: the editor renders them as Markdown and follows the links in them. `jailint` gets two new rules and fewer false positives.

### Highlights

- Doc comments show as Markdown in hover, completion and signature help, with per-parameter docs. See [doc comments](https://github.com/matteopolak/jacket/blob/main/docs/compiler/doc-comments.md).
- Links like `[print]`, `[Basic.print]` and `[Point.LIMIT]` in doc comments resolve: hover, go to definition and clickable links.
- VS Code highlights Markdown in comments, and Jai inside fenced code blocks.
- New `jailint` rules: `broken_doc_link` and `redundant_alias` (`a := f(); b := a;`).
- `index_only_loop` and `manual_index_counter` now cover loops over strings.

### Fixes

- `jailint --fix` no longer rewrites nested index loops into `it - it`.
- Fewer `jailint` false positives in `defer_in_loop`, `unused_parameter` and `self_assignment`.
- The language server no longer crashes showing a completion's docs after an edit.
- Parallel macOS builds into one directory no longer fail writing `.build/` objects.
- A metaprogram re-offering the same `provide_import` no longer hangs the compile.
- `jaifmt` keeps notes with punctuation (`@a!=b`) as one token.

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

![Release archive sizes, 0.6.2 against 0.7.0: every platform is smaller](https://raw.githubusercontent.com/matteopolak/jacket/main/docs/images/releases/0.7.0-download-size.svg)

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

![Language server peak memory, 0.6.0 against 0.6.1: down by a factor of two to five](https://raw.githubusercontent.com/matteopolak/jacket/main/docs/images/releases/0.6.1-language-server-memory.svg)

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

Language server timings can be measured with [the benchmark script](https://github.com/matteopolak/jacket/blob/main/docs/tools/lsp-benchmark.md).

## [0.6.0] - 2026-10-09

Builds with optimizations on compile up to 2.7 times faster, and unoptimized builds of large programs about 1.3 times faster.

![Optimized build times, 0.5.1 against 0.6.0: Focus 12.7 to 7.8 seconds, chess-jai 4.5 to 1.7 seconds](https://raw.githubusercontent.com/matteopolak/jacket/main/docs/images/releases/0.6.0-build-time.svg)

### Highlights

- Faster builds: see the chart, and [compile speed](https://github.com/matteopolak/jacket/blob/main/docs/compiler/compile-speed.md) for how.
- `jailsp` completes inside range expressions such as `for i: 0..table.co`.
- `jailint`'s `index_only_loop` also catches loops that fill elements from their index, and offers a fix.
- `self_assignment` and `absurd_comparison` get editor quick fixes.
- Compiler messages `PERFORMANCE_REPORT` and `DEBUG_DUMP` are sent, so the `Performance_Report` plugin works. See [build options](https://github.com/matteopolak/jacket/blob/main/docs/metaprogramming/build-options.md).
- The playground tour runs every stop in order, without waiting for input.

### Breaking changes

| Was | Now |
| --- | --- |
| `enable_split_modules = false` built an optimized program as one module | Still split into parts; set `JAIC_CODEGEN_UNITS=1` for whole-program optimization |
| `File_Change.time_of_last_change` was a `float32` | A `float64` |
| `index_only_loop` skipped loops that write elements from the index | Reports them, so a clean lint run may now warn |
| The playground tour took `--stop`, `--all` and `--help` | It always runs every stop |

### Fixes

- The GPU stop of the tour draws in its own window instead of aborting; without a display it says it skipped. See [WebGPU](https://github.com/matteopolak/jacket/blob/main/docs/extensions/webgpu.md).
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

See [compile speed](https://github.com/matteopolak/jacket/blob/main/docs/compiler/compile-speed.md) for what changed.

## [0.5.0] - 2026-10-09

Programs get checked more thoroughly: `print` format strings, null pointers, notes and clashing names. There is also a command-line parser for your own tools and a much bigger set of editor refactorings.

![jaic warns that a print call has too few arguments for its format string](https://raw.githubusercontent.com/matteopolak/jacket/main/docs/images/releases/0.5.0-print-check.svg)

### Highlights

- `print`-style format strings are checked at compile time. See [format string check](https://github.com/matteopolak/jacket/blob/main/docs/compiler/format-string-check.md).
- Built programs stop with `null pointer dereference` at the offending line instead of crashing.
- New [`Extensions/Args`](https://github.com/matteopolak/jacket/blob/main/docs/extensions/args.md): declare a struct and get a command-line parser with help, suggestions and shell completions. `jaifmt` uses it.
- Editor refactorings: extract into variable or procedure, inline variable, add missing cases or fields, convert `ifx`.
- Call hierarchy, expand selection, pull diagnostics, and rename of struct fields and enum members. See [refactorings](https://github.com/matteopolak/jacket/blob/main/docs/compiler/language-server-refactorings.md).
- A procedure that can end without returning a value warns. See [diagnostics](https://github.com/matteopolak/jacket/blob/main/docs/compiler/diagnostics.md#compile-warnings-compilerwarnings).
- New `just` commands for building, formatting, linting and testing the repository. See [Justfile](https://github.com/matteopolak/jacket/blob/main/docs/tools/development.md).

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
- C structs now pass correctly in more cases, including packed structs and structs through `...`. See [C ABI](https://github.com/matteopolak/jacket/blob/main/docs/native/c-abi.md).
- New `File.file_read_line` and `File.read_stdin_line`; `exit(n)` works in the playground.
- More `Compiler` module procedures work, and more directives reach metaprograms as their own node kinds. See [the Compiler module](https://github.com/matteopolak/jacket/blob/main/docs/metaprogramming/compiler-module.md).
- More `#asm` instructions, and `offset_of` and `is_value_type` are implemented.
- Windows: `#file` and `get_absolute_path` use `/`, and a metaprogram's custom link command handles five or more libraries.
- `rpmalloc` no longer crashes when a new thread allocates.

## [0.4.3] - 2026-10-08

Numbers and `print` now behave as Jai programs expect, mistakes that used to compile are errors, and the language server can add imports for you.

### Highlights

- Auto-import: type part of a name you haven't imported and accept it; the `#import` or `#load` is added. See [auto-import](https://github.com/matteopolak/jacket/blob/main/docs/compiler/language-server.md#auto-import-completion).
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

See [build options](https://github.com/matteopolak/jacket/blob/main/docs/metaprogramming/build-options.md) for what is supported.

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

- New `Extensions/WebGPU` module: one program draws in a window on macOS, Linux and Windows, and in the playground's new **Render** pane. See [WebGPU](https://github.com/matteopolak/jacket/blob/main/docs/extensions/webgpu.md).
- Release archives ship the WebGPU library; nothing else to install.
- New examples: a triangle, a ray marcher and a compute shader.
- `jaic build -os wasm` programs can use threads. See [threads in WASI builds](https://github.com/matteopolak/jacket/blob/main/docs/native/wasm-threads.md).
- The playground tour has four new stops, and VS Code highlights `#string WGSL`, `GLSL`, `SQL` and `JSON` bodies.
- The extension is listed as **Jai Toolchain** on the VS Code Marketplace.

### Breaking changes

| Was | Now |
| --- | --- |
| `#import "Jaic_Extensions";`, `"Jai_Format"`, `"Wasi_Runtime"` | `#import "Extensions/Long_Double";`, `"Extensions/Jai_Format"`, `"Extensions/Wasi_Runtime"` |
| `jaic-<platform>` archives | `jai-<platform>` |
| `JAIC_VERSION`, `JAIC_INSTALL_DIR`, `JAIC_BIN_DIR` | `JAI_VERSION`, `JAI_INSTALL_DIR`, `JAI_BIN_DIR` (the old names still work) |

An import of a moved module fails with an error naming the import to write. See [stdlib extensions](https://github.com/matteopolak/jacket/blob/main/docs/extensions/extensions.md).

### Fixes

- The compiler no longer crashes on a bad address from a `#run` or metaprogram.
- Metaprograms that add code every round no longer slow down quadratically.
- `print("%", p.*)` with a null `p` stops with a clear error.
- `jailsp` no longer crashes on builtin constants such as `OS`.

## [0.4.0] - 2026-10-07

A VS Code extension, one-command installs and error messages that say what to change.

![jaic reports a type mismatch with the expected type and a suggested cast](https://raw.githubusercontent.com/matteopolak/jacket/main/docs/images/releases/0.4.0-error.svg)

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
| macOS, Linux | `brew install matteopolak/tap/jacket` |
| macOS, Linux | `curl -fsSL https://raw.githubusercontent.com/matteopolak/jacket/main/install.sh \| sh` |
| Windows | `irm https://raw.githubusercontent.com/matteopolak/jacket/main/install.ps1 \| iex` |
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

![jailint reports a loop that only uses its index, with the suggested rewrite](https://raw.githubusercontent.com/matteopolak/jacket/main/docs/images/releases/0.3.0-jailint.svg)

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

![jaifmt turns messy code into formatted code](https://raw.githubusercontent.com/matteopolak/jacket/main/docs/images/releases/0.2.0-jaifmt.svg)

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
- The standalone playground UI is gone; use the hosted playground at https://matteopolak.com/playground/jacket.

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
