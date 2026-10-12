# Developer documentation

Start with [compiler architecture](compiler/architecture.md). Real projects that compile are listed in [upstream corpus](tools/upstream-corpus.md#project-status).

- [Features](features.md): what works, performance and what is missing
- [License](license.md): AGPL-3.0-or-later with a runtime library exception for compiled programs

## Language

Jai language behavior as implemented by `jaic`.

- [Casts and conversions](language/casts-and-conversions.md)
- [The implicit context](language/context.md)
- [Control flow: loops, cases and defer](language/control-flow.md)
- [Dead-code elimination: what is checked when unused](language/dead-code-elimination.md)
- [Declarations, constants and globals](language/declarations-and-constants.md)
- [Statement directives, flags and notes](language/directives-and-notes.md)
- [Enums](language/enums.md)
- [External data](language/external-data.md)
- [Compiler intrinsics](language/intrinsics.md)
- [Lambdas and anonymous procedures](language/lambdas.md)
- [Macros and custom iteration](language/macros-and-custom-iteration.md)
- [`#must` results and `#discard` parameters](language/must-and-discard.md)
- [Module parameters](language/module-parameters.md)
- [Modules and imports](language/modules-and-imports.md)
- [Arithmetic overflow checks and `#no_aoc`](language/arithmetic-overflow-checks.md)
- [Integers, floats and numeric literals](language/numbers.md)
- [`Long_Double`: C's `long double` (jaic extension)](language/long-double.md)
- [Operator overloading](language/operator-overloading.md)
- [Operator precedence](language/operators.md)
- [Pointers, arrays and bounds checks](language/pointers-and-arrays.md)
- [Polymorphism and baking](language/polymorphism.md)
- [Procedures and calls](language/procedures.md)
- [Scoping: visibility, using and conditional declarations](language/scoping.md)
- [SIMD and `#asm`](language/simd-asm.md)
- [Strings and literals](language/strings-and-literals.md)
- [Structs and aggregate literals](language/structs.md)
- [Type values, identity and type info](language/type-values-and-info.md)
- [Unions and tagged unions](language/unions.md)
- [using](language/using.md)

## Metaprogramming

Compile-time execution, the `Compiler` module, reflection and code values.

- [Build options and the metaprogram API: what jaic honours](metaprogramming/build-options.md)
- [Code values and #insert](metaprogramming/code-values-and-insertion.md)
- [Compile-time values, globals and runtime info](metaprogramming/compile-time-data-and-state.md)
- [Compile-time execution (`#run`)](metaprogramming/compile-time-execution.md)
- [Compiler module (Jai side)](metaprogramming/compiler-module.md)
- [Compiler records (messages, syntax trees, type descriptors)](metaprogramming/compiler-records.md)
- [Metaprogram plugins (`-plug`)](metaprogramming/metaprogram-plugins.md)
- [Preload and Runtime_Support bootstrap](specs/jai/README.md)
- [Reflection and Type_Info](metaprogramming/reflection-and-type-info.md)
- [Workspaces and metaprograms](metaprogramming/workspaces.md)

## Compiler internals

How `crates/jaic`, the interpreter and the language server work.

- [Compiler architecture](compiler/architecture.md)
- [Diagnostics](compiler/diagnostics.md): error message style guide, rendering, runtime failures, exit statuses
- [Format string check](compiler/format-string-check.md): `print` format strings against their argument count
- [jaic `#asm` blocks](compiler/asm.md)
- [SIMD lowering](compiler/simd-lowering.md): target features, per-ISA backends and how to add an intrinsic or instruction set
- [Threads under `jaic run`](compiler/interpreter-threads.md)
- [jaic interpreter](compiler/interpreter.md)
- [Low-level IR](compiler/ir.md)
- [Compile speed](compiler/compile-speed.md): where an `-O0` build spends its time, what was done about it, current numbers
- [Memory limit (`JAIC_MEMORY_LIMIT`)](compiler/memory-limit.md)
- [Shared Jai language server](compiler/language-server.md) (front page: [jailsp README](../crates/jai-language-server/README.md), with the comparison with Jails): feature list, expansions, inlay hints, format strings, lints and quick fixes, `#import` links
- [Doc comments](compiler/doc-comments.md): the Markdown comment convention (parameters, intra-doc links) and how the server and the VS Code grammar use it
- [Language server refactorings](compiler/language-server-refactorings.md): extract, inline, fill in cases and fields, `ifx` rewrites, call hierarchy, selection ranges
- [Parser](compiler/parser.md)
- [Sema: module loading and top-level expansion](compiler/sema-modules.md)
- [Sema: polymorphism and declarations](compiler/sema-polymorphism-and-declarations.md)
- [Stack traces (`context.stack_trace`)](compiler/stack-traces.md)

## Native builds

The LLVM backend, C ABI and linking.

- [C ABI, callbacks and C++ methods](native/c-abi.md)
- [C ABI coverage matrix](native/c-abi-coverage.md)
- [jaic LLVM backend](native/llvm-backend.md)
- [Native debug information](native/debug-info.md)
- [Native build and linking](native/native-linking.md)
- [Sanitizers for native builds](native/sanitizers.md)
- [Vk-Engine corpus project](native/vk-engine.md)
- [Native Windows executables](native/windows.md)
- [WebAssembly target (wasm64, WASI, `Wasi_Runtime`)](native/wasm-target.md)
- [Threads in WASI builds (green threads)](native/wasm-threads.md)

## Standard library


- [Standard library layout](specs/stdlib/README.md)
- [Basic and collection modules](specs/stdlib/README.md)
- [Basic calendar time, working directory and platform exports](specs/stdlib/README.md)
- [Binary formats and checksums](specs/stdlib/README.md)
- [Bindings_Generator](specs/stdlib/README.md)
- [Stdlib extensions: jaic-only modules in `stdlib/Extensions/`](specs/stdlib/README.md)
- [Command_Line](specs/stdlib/README.md)
- [Args: a compile-time-generated command-line parser (jaic extension)](specs/stdlib/README.md)
- [Regex: regular expressions with compile-time typed patterns (jaic extension)](specs/stdlib/README.md)
- [SIMD: portable fixed-width vectors (jaic extension)](specs/stdlib/README.md)
- [Byte_Search: vectorized byte and substring search (jaic extension)](specs/stdlib/README.md)
- [Compiler API, reflection and metaprogram support](specs/stdlib/README.md)
- [Files, processes and OS services](specs/stdlib/README.md)
- [GetRect](specs/stdlib/README.md)
- [Iprof profiler module](specs/stdlib/README.md)
- [Math, random numbers and color/float helpers](specs/stdlib/README.md)
- [Memory, allocators and hashing](specs/stdlib/README.md)
- [meow_hash: AES-based 128-bit hash for x64](specs/stdlib/README.md)
- [Native and platform bindings](specs/stdlib/README.md)
- [Program_Print](specs/stdlib/README.md)
- [Strings, Unicode and text files](specs/stdlib/README.md)
- [Threads, atomics, sockets and input](specs/stdlib/README.md)
- [Toolchains: macOS SDK and Android NDK helpers](specs/stdlib/README.md)
- [Tooling modules: Debug, MacOS_Bundler, BuildCpp, Autorun, Performance_Report](specs/stdlib/README.md)
- [Drawing, windows and audio](specs/stdlib/README.md)
- [Simp (2D renderer)](specs/stdlib/README.md)
- [Sound_Player (audio mixing and output)](specs/stdlib/README.md)
- [WebGPU (generated bindings, native and in the playground; support matrix, callback modes, tests)](specs/stdlib/README.md)

## Browser

The WebAssembly compiler bundle and its releases. The hosted playground UI lives in the portfolio site (https://matteopolak.com/playground/jai).

- [Browser compiler (WebAssembly bundle)](browser/playground.md)
- [Browser compiler releases](browser/compiler-releases.md)
- [Language tour (playground default workspace)](browser/tour.md)

## Tools and workflow

Scripts, CI and project policies.

- [Benchmarks and profiling](tools/benchmarks.md)
- [Build storage and target directories](tools/build-storage.md)
- [Compile-time benchmark](tools/compile-time-benchmark.md): wall time, peak RSS and phases of real projects, `--timings`
- [Regex benchmark](tools/regex-benchmark.md): `Extensions/Regex` against Rust `regex`, RE2, PCRE2, CTRE and Hyperscan on a generated corpus
- [Edit benchmark](tools/edit-bench.md): how long `jailsp` takes to answer after an edit inside a body, compiles per edit
- [IDE coverage](tools/ide-coverage.md): which identifiers of a project get no hover or go to definition from `jailsp`
- [Language server benchmark](tools/lsp-benchmark.md): latency, CPU time and RSS of `jailsp` (or another server) on real and generated projects and very large files
- [Code formatting](tools/code-formatting.md): rustfmt and jaifmt checks, format-only commits and `.git-blame-ignore-revs`
- [Continuous integration](tools/continuous-integration.md)
- [Dependency policy](tools/dependency-policy.md)
- [Justfile](tools/justfile.md): `just build|fmt|lint|test|check|hooks`, the recipes CI also runs
- [Pre-commit hook](tools/pre-commit-hook.md): `.githooks/pre-commit`, jaifmt, jailint, rustfmt and item spacing on the staged files
- [Benchmark generator](tools/benchmark-generator.md): `benchgen.py` corpus-shaped programs of any size, `jaistats.py` corpus statistics
- [Differential testing](tools/differential-testing.md): interpreter, native, `-O2` and wasm must agree; `jaigen` random programs; `jaic-reduce`
- [GitHub Action](tools/github-action.md): `setup-jai`, installing the released toolchain in CI
- [Fuzzing](tools/fuzzing.md)
- [jaic regression sweep](tools/jaic-sweep.md)
- [Stdlib target check](tools/stdlib-target-check.md): every stdlib module type-checked for every target, unreferenced code included
- [Stdlib runtime tests](tools/stdlib-runtime-tests.md): every stdlib test run in the interpreter, natively and on wasm on every platform; skip list; per-module coverage ratchet
- [Objective-C selector check](tools/objc-selector-check.md): every selector the stdlib sends matches its arguments and exists in the runtime
- [jaifmt (Jai formatter)](tools/jaifmt.md) (front page: [jaifmt README](../jaifmt/README.md)): the `jaifmt/` program, its `build.jai` metaprogram and `jaifmt.wasm`
- [jailint (Jai linter)](tools/jailint.md) (front page: [jailint README](../crates/jailint/README.md)): rules, `jailint.toml`, suppression, adding a rule
- [LLVM setup](tools/llvm-setup.md)
- [LLVM backends and binary size](tools/llvm-backends.md)
- [Nix flake](tools/nix.md)
- [Package managers and installers](tools/package-managers.md): Homebrew tap, `install.sh`/`install.ps1`, the release jobs that publish them and their one-time setup
- [PGO and BOLT](tools/pgo-and-bolt.md): profile-guided and post-link optimisation of the released binaries, `tools/build_pgo.py`
- [Third-party native libraries](tools/native-libs.md)
- [Regex Unicode tables](tools/regex-unicode-tables.md): `tools/gen_regex_unicode.py`, the generated `unicode_data.jai` and the lookup API of `Extensions/Regex`
- [Third-party smoke test](tools/third-party-smoke-test.md): finding public Jai repositories and trying them
- [open-jai expectation harness](tools/openjai-expectations.md)
- [Releases](tools/releases.md)
- [Rule coverage](tools/rule-coverage.md): rule IDs in the language docs and the tests that cite them
- [Upstream corpus](tools/upstream-corpus.md)
- [VS Code extension](tools/vscode-extension.md): `editors/vscode`, grammar, language client, jaifmt formatting, toolchain download, packaging and publishing
- [Neovim plugin](tools/neovim-plugin.md): `editors/nvim`, the `jailsp` config for `vim.lsp.enable`, jaifmt formatting, syntax file, toolchain download, tests and the `jai.nvim` mirror

## Language and library specifications

- [Jai language guide](specs/jai/README.md)
- [Standard library API specification](specs/stdlib/README.md)
