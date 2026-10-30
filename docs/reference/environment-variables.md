# Environment variables

Generated from `crates/jacket-config/src/env.rs` by `tools/generators/generate_reference_docs.py`; edit the table there. Every variable the Rust tools read is declared there, and clippy rejects a direct `std::env::var` anywhere else. See [the environment registry](../tools/environment.md).

Path lists use the platform separator (`:` on Linux and macOS, `;` on Windows). An empty value counts as unset wherever a setting says so.

## Settings

| Variable | Meaning | When unset | Read by |
| --- | --- | --- | --- |
| `JACKET_CHECK_TIME_LIMIT` | Seconds one program check may take when the linter or language server runs compile-time code (`0`: no limit). CI sets it to `0`. | 10 seconds | jailint, jailsp |
| `JACKET_OUTPUT_DIR` | Relocates the build output directory (the `native-libs/` lookup and `jacket test --coverage` reports follow it). | `target/` in the project | jaic, jacket |
| `JACKET_TEST_DLLS` | A path list of directories whose `.dll` files `jacket test` links next to each Windows test executable. | unset | jacket |
| `JAIC_AR` | The program that archives object files into a static library. | the system `ar`, or `llvm-ar`, a MinGW `ar` or `lib.exe` where that is needed | jaic |
| `JAIC_CACHE_DIR` | Where `jaic` unpacks the helpers it embeds (the crash runtime). | `$XDG_CACHE_HOME/jaic`, else the platform's user cache (`~/Library/Caches/jaic`, `~/.cache/jaic`) | jaic |
| `JAIC_CROSS_LIBS` | A path list of directories with libraries cross-built for a MinGW target; a cross build links from these instead of the host's native libraries. | unset | jaic |
| `JAIC_DIAGNOSTICS` | The diagnostic layout: `plain`, `ascii` or `unicode`. | by terminal: `unicode` or `ascii` on one, `plain` otherwise | jaic, jailint |
| `JAIC_DSYM` | When set, a macOS debug build writes a `.dSYM` bundle instead of keeping its object files. | unset: macOS debug builds keep their object files for the debugger | jaic |
| `JAIC_HOST_TWIN_DEBUG` | When set, the host twin prints what it generates. | unset | jaic |
| `JAIC_ID_SCRAMBLE` | The same as `JAIC_SYM_SCRAMBLE`, for AST ids. | unset | jaic |
| `JAIC_LINKER` | The program that links executables and libraries. Empty counts as unset. | the linker found for the target | jaic |
| `JAIC_MEMORY_LIMIT` | Stop with exit status 120 once this much memory is allocated: bytes, or a number with a `K`, `M` or `G` suffix. Child processes inherit it. | no limit | jaic |
| `JAIC_NATIVE_LIBS` | A path list of directories holding the prebuilt third-party native libraries (SDL3, FreeType, ...), replacing the default search. | `native-libs/<os>-<cpu>` beside the library, else under `target/` | jaic |
| `JAIC_NO_HOST_TWIN` | Any value but `0` turns the host twin off, which makes compile-time code with foreign calls fail the way it would without one. | unset: compile-time code that needs the host's types gets a host twin | jaic |
| `JAIC_PRELUDE` | The prelude file to load instead of the library's own, as a path (relative paths are taken from the library root). | the library's `prelude/module.jai`, else `Preload.jai` | jaic, jailint, jailsp |
| `JAIC_PROGRESS` | The build status lines: `off`, `plain` or `live`, whatever stderr is (`-quiet` still wins). | by terminal: a live status line on one, nothing otherwise | jaic, jacket |
| `JAIC_SANITIZER_CC` | The C driver that links a build made with a sanitizer. | the `clang` of the LLVM jaic was built with, else `cc` | jaic |
| `JAIC_STDLIB` | The standard library directory. `jacket test` also hands it to every `jaic` it starts. | the `stdlib/` next to the executable, else the checkout's | jaic, jacket, jailint, jailsp, tests |
| `JAIC_SYM_SCRAMBLE` | A seed that spreads the interner's symbol numbers apart, so output that depends on them rather than spellings changes (`tools/differential/determinism.py`). | unset | jaic |
| `JAIC_WASM_LD` | The `wasm-ld` that links WebAssembly output. | `wasm-ld` next to the build's LLVM, an LLD install, then `PATH` | jaic |

## Set by one tool for another

| Variable | Meaning | When unset | Read by |
| --- | --- | --- | --- |
| `JACKET_PROJECT` | The project name `jaic` shows in its `Compiling` line; `jacket` sets it. | the main file's name | jaic |
| `JAIC_COVERAGE_DIR` | The directory a `-interp` run writes its line coverage to, one file per program. `jacket test --coverage` sets it. | the working directory | jaic |
| `JAIC_COVERAGE_FILE` | The file a `-interp` run writes its line coverage to. | unset: the file is named for the program, in `JAIC_COVERAGE_DIR` | jaic |
| `JAIC_REPORT_EXE` | A file `jaic` appends the absolute path of each executable it writes to. `jacket run` and `jacket test` set it, because a metaprogram picks the name. | unset | jaic |

## Tests and fuzzing

| Variable | Meaning | When unset | Read by |
| --- | --- | --- | --- |
| `EXPECTED_PRELUDE` | The prelude the child process of the prelude-selection tests must find, or `missing`. | unset | tests |
| `JACKET_DATABASE_URL` | A PostgreSQL connection URL for the typed-query tests. | unset: the Postgres tests skip | tests |
| `JACKET_STDLIB_TARGETS_PART` | `K/N`: the stdlib-by-target check takes every Nth module and target pair from the Kth, so CI can split it. | `1/1` | tests |
| `JACKET_TEST_CODEGEN_UNITS` | Passes `-codegen_units N` to the native build tests, to run them split into several modules. | unset | tests |
| `JAIC_ABI_CLANG` | The `clang` the C ABI tests compare against. | `clang` | tests |
| `JAIC_ABI_CROSS` | Cross targets the C ABI tests also compile for, as a comma list of specs. | unset | tests |
| `JAIC_CORPUS_ROOT` | The checkout whose `corpus/upstream` and `stdlib` the corpus sweep parses. | the repository | tests |
| `JAIC_CORPUS_STAGE` | How far the corpus sweep takes each file. | `parse` | tests |
| `JAIC_KEEP_TEST_TMP` | When set, test scratch directories are kept after the test. | unset: scratch directories are removed | tests |
| `JAIC_PRELUDE_TEST_CHILD` | Marks the child process of the prelude-selection tests, which run themselves again with `JAIC_PRELUDE` set. | unset | tests |
| `JAIC_REQUIRE_TOOLCHAIN` | `1`, or a comma list of tool names: a missing tool fails the test instead of skipping it. Always on under `CI`. | unset: a test whose tool is missing skips | tests |
| `JAIC_TEST_LIBRARY_MARKER` | A file the native-link test's library writes to, to show it was loaded. | unset | tests |
| `JAIC_TEST_TMPDIR` | Where test scratch directories are made. | the system temporary directory | tests |
| `JAIC_WASM_TEST` | A value the WebAssembly test passes to the program, which prints it back. | unset | tests |
| `JAILSP_ENFORCE_SCALE_BUDGET` | `1`: the language server scale test fails when a case is over its budget. | unset: the budget is only reported | tests |
| `JAI_FUZZ_VERBOSE` | When set, the fuzz harness prints each input it checks. | unset | fuzzing |
| `JAI_IDE_AUDIT_REVISION` | Read at build time: the revision the language server coverage audit stamps into its report. | unset | tests |
| `TYPING_FUZZ_DUMP` | A directory the typing-session rig writes the failing document to. | unset | tests |

## Standard variables

| Variable | Meaning | When unset | Read by |
| --- | --- | --- | --- |
| `CC` | The C compiler the native tests build their helper libraries with. | `cc` | tests |
| `CI` | Set by CI services: the build status lines are plain, and a missing test tool fails instead of skipping. | unset | jaic, jacket, tests |
| `CLICOLOR_FORCE` | The same as `FORCE_COLOR`. | unset | jaic, jailint |
| `COLUMNS` | The width the live status line is fitted to. | the terminal's width | jaic |
| `ConEmuANSI` | `ON` marks a Windows console that takes ANSI control codes. | unset | jaic, jacket, jailint |
| `DEVELOPER_DIR` | The Xcode installation whose SDK and libclang jaic uses. | `xcode-select -p` | jaic |
| `FORCE_COLOR` | Anything but `0` turns colour on even when output is not a terminal. | unset | jaic, jailint |
| `GITHUB_ACTIONS` | Set by GitHub Actions: tests that need a capable filesystem stop skipping. | unset | tests |
| `GITHUB_TOKEN` | Sent with `jacket upgrade`'s release lookups, which raises GitHub's rate limit. | unset | jacket |
| `HOME` | Locates the user cache when neither `JAIC_CACHE_DIR` nor `XDG_CACHE_HOME` is set. | unset | jaic |
| `INSTA_UPDATE` | `always` rewrites the test snapshots instead of failing on a difference. | unset | tests |
| `LANG` | See `LC_ALL`. | unset | jaic, jailint |
| `LC_ALL` | With `LC_CTYPE` and `LANG`: a UTF-8 locale selects the Unicode diagnostic layout. | unset | jaic, jailint |
| `LC_CTYPE` | See `LC_ALL`. | unset | jaic, jailint |
| `LIB` | On Windows, the library directories the MSVC linker searches; set by a developer prompt. | found from the Visual Studio and Windows SDK installs | jaic |
| `LIBRARY_PATH` | Extra library directories compile-time code may load native libraries from on Windows. | unset | jaic |
| `LLVM_SYS_231_PREFIX` | The LLVM install jaic is built against; its `bin/` supplies `clang`, `wasm-ld` and the LLVM tools. | unset | jaic, jacket |
| `MACOSX_DEPLOYMENT_TARGET` | The macOS version native output targets. | the SDK's | jaic |
| `NO_COLOR` | Any value turns colour off. | unset | jaic, jacket, jailint |
| `PATH` | Where tools such as the linker, `clang` and `wasm-ld` are searched for. | the process's | jaic, jacket, jailsp, tests |
| `PATHEXT` | The executable extensions tried when a test looks a tool up on Windows. | the Windows default | tests |
| `ProgramFiles` | Where Visual Studio and the Windows SDK are looked for. | `C:\Program Files` | jaic |
| `ProgramFiles(x86)` | See `ProgramFiles`. | `C:\Program Files (x86)` | jaic |
| `RUST_BACKTRACE` | When set, an internal compiler error shows the Rust backtrace. | unset | jaic |
| `SystemRoot` | Locates the Windows system tools `jacket upgrade` uses. | `C:\Windows` | jacket |
| `TERM` | `dumb` turns colour, layout characters and the live status line off. | unset | jaic, jacket, jailint |
| `WT_SESSION` | Set by Windows Terminal; it gets the Unicode diagnostic layout. | unset | jaic, jailint |
| `XDG_CACHE_HOME` | The user cache root; `jaic` uses its `jaic` folder. | unset | jaic |
