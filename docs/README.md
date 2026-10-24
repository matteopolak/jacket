# Documentation

`jaic` is a compiler for the Jai language: lexer, parser, type checker, interpreter (`#run`, `-interp`), an LLVM backend for native code and WebAssembly, and a metaprogramming API. `jailsp` (language server), `jaifmt` (formatter, written in Jai) and `jailint` (linter) share its front end, and `jacket` runs them all and reads their shared `jacket.toml`. The standard library in `stdlib/` is written in Jai.

Start with [features](features.md), [the command line](tools/cli.md) and [install channels](tools/releases.md). Contributors: [CONTRIBUTING.md](../CONTRIBUTING.md) and [development](tools/development.md).

## Language

[language/](language/) has one page per topic (for example [modules](language/modules-and-imports.md), [procedures](language/procedures.md), [polymorphism](language/polymorphism.md), [macros](language/macros-and-custom-iteration.md), [inline holes](language/interpolates.md), [numbers](language/numbers.md)). Compile-time execution and the `Compiler` module are in [metaprogramming/](metaprogramming/).

## Compiler

[Prelude selection](compiler/prelude-selection.md), [Architecture](compiler/architecture.md), [parser](compiler/parser.md), [sema](compiler/sema.md), [variables vs constants](compiler/variable-constness.md), [IR](compiler/ir.md), [interpreter](compiler/interpreter.md), [compile-time code on the host](compiler/compile-time-host-execution.md), [diagnostics](compiler/diagnostics.md), [source positions](compiler/source-positions.md), [compile speed](compiler/compile-speed.md), [parallel type checking](compiler/parallel-sema.md), [`#asm`](compiler/asm.md), [compiler builtins](compiler/compiler-builtins.md), [targets](compiler/targets.md), [stack traces](compiler/stack-traces.md), [doc comments](compiler/doc-comments.md), [language server](compiler/language-server.md), [polymorph reporting](compiler/polymorph-reporting.md), [saved editor configuration](compiler/saved-configuration.md), [lexicon](compiler/lexicon.md), [file identity](compiler/file-identity.md), [project structure](compiler/project.md).

## Native and browser

[LLVM backend](native/llvm-backend.md), [C ABI](native/c-abi.md), [C++ vtables](native/cpp-vtables.md), [Windows](native/windows.md) (native build helpers and x64/arm64 crash diagnostics), [WebAssembly](native/wasm-target.md), [Windows console colors](native/windows-console-colors.md), [debug info](native/debug-info.md), [linking](native/native-linking.md), [crash runtime build](native/crash-runtime-build.md), [sanitizers](native/sanitizers.md), [browser build and playground](browser/playground.md).

## Tools and editors

[jacket](tools/jacket.md), [jaifmt](tools/jaifmt.md), [jailint](tools/jailint.md), [VS Code](tools/vscode-extension.md), [Neovim](tools/neovim-plugin.md), [workspace trust](tools/workspace-trust.md).

## Standard library

[Math](specs/stdlib/modules/math.md) (vectors, matrices and scalar helpers), [Extensions](extensions/README.md) (Io, Http, Tls, Json, Postgres and the rest), [crypto helpers](stdlib/crypto.md), [native layout](stdlib-native-layout.md), [float conversion](stdlib-float-conversion.md), [third-party native libraries](tools/native-libs.md), [Unicode tables](tools/regex-unicode-tables.md), [benchmarks](tools/stdlib-benchmarks.md). Planned: [Postgres client and compile-time ORM design](postgres/design.md); [Postgres bulk loading](postgres/copy.md); [Postgres load test](postgres/benchmark.md); [Postgres conformance suites](postgres/conformance.md); [Postgres managed binaries](postgres/managed.md); [Postgres migrations](postgres/migrations.md); [Postgres ORM](postgres/orm.md); [Postgres schema generation](postgres/schema.md); [Postgres schema diff](postgres/diff.md); [Postgres typed queries](postgres/typed-queries.md).

## Building and testing

[Development](tools/development.md) (Justfile, CI, hooks, dependency policy), [testing](tools/testing.md), [diagnostic coverage](tools/diagnostic-coverage.md), [determinism checks](tools/determinism.md), [coverage](tools/coverage.md), [Rust code coverage](tools/rust-coverage.md), [language server typing sessions](tools/lsp-fuzzing.md), [language server oracle checks](tools/lsp-oracle.md), [stdlib fuzzing](tools/stdlib-fuzzing.md), [sanitizers and Miri](tools/sanitizers.md), [mutation testing](tools/mutation-testing.md), [tools/ layout](tools/tools-layout.md), [upstream corpus](tools/upstream-corpus.md), [benchmarks and profiling](tools/benchmarks.md), [stdlib benchmarks](tools/stdlib-benchmarks.md), [performance gate](tools/perf-gate.md), [LLVM setup](tools/llvm-setup.md), [releases](tools/releases.md), [release smoke tests](tools/release-smoke.md), [license](license.md).

## Specifications

[specs/jai/](specs/jai/) and [specs/stdlib/](specs/stdlib/) are what the compiler and standard library are implemented from.
