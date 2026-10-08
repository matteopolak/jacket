# Changelog

## [Unreleased]

### Fixed

- `Hash_Table.init` rounds an explicit size up to a power of two without raising it to the default minimum (`init(*t, 5)` allocates 8 slots).
- `Random`'s float32 draws use the low 24 bits of the next value, and `random_get_within_range` returns `min + (max - min) * fraction`.

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
