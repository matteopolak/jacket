# Documentation

Jacket is a compiler and toolchain for the Jai language. `jaic` is the compiler (lexer, parser, type checker, interpreter, LLVM backend for native code and WebAssembly, metaprogramming API). `jailsp`, `jaifmt` and `jailint` share its front end, and `jacket` runs them all from one `jacket.toml`. The standard library in `stdlib/` is written in Jai.

New here: [features](features.md), [the command line](tools/cli.md), [install channels](tools/releases.md). Contributing: [CONTRIBUTING.md](../CONTRIBUTING.md) and [development](tools/development.md).

## Language

One page per topic in [language/](language/): [modules](language/modules-and-imports.md), [procedures](language/procedures.md), [polymorphism](language/polymorphism.md), [macros](language/macros-and-custom-iteration.md), [inline holes](language/interpolates.md), [numbers](language/numbers.md), [strings](language/strings-and-literals.md), [operator overloading](language/operator-overloading.md), [enums](language/enums.md), [unions](language/unions.md), [control flow](language/control-flow.md), [context](language/context.md), and [leniencies](language/leniencies.md) (what jaic accepts that the official compiler rejects).

## Metaprogramming

- [Compile-time execution, `#code` and `#insert`](metaprogramming/compile-time-execution.md)
- [Compiler module](metaprogramming/compiler-module.md)
- [Workspaces and plugins](metaprogramming/workspaces.md)
- [Build options](metaprogramming/build-options.md)
- [Reflection and type info](metaprogramming/reflection-and-type-info.md)

## Compiler

- Overview: [architecture](compiler/architecture.md), [lazy code generation backend](compiler/lazy-backend.md), [project structure](compiler/project.md), [lexicon](compiler/lexicon.md), [prelude selection](compiler/prelude-selection.md).
- Front end: [parser](compiler/parser.md), [sema](compiler/sema.md), [variables vs constants](compiler/variable-constness.md), [parallel type checking](compiler/parallel-sema.md), [polymorph reporting](compiler/polymorph-reporting.md), [file identity](compiler/file-identity.md), [source positions](compiler/source-positions.md).
- Back end: [IR](compiler/ir.md), [interpreter](compiler/interpreter.md), [fibers](compiler/fibers.md), [`#asm`](compiler/asm.md), [integer arithmetic](compiler/integer-arithmetic.md), [compile-time limits](compiler/compile-time-limits.md), [compile-time code on the host](compiler/compile-time-host-execution.md), [builtins](compiler/compiler-builtins.md), [targets](compiler/targets.md), [toolchain](compiler/toolchain.md), [platform differences](compiler/platform.md), [stack traces](compiler/stack-traces.md), [compile speed](compiler/compile-speed.md).
- Diagnostics and editors: [diagnostics](compiler/diagnostics.md), [doc comments](compiler/doc-comments.md), [language server](compiler/language-server.md), [IDE facts](compiler/ide-facts.md), [saved configuration](compiler/saved-configuration.md).
- Standard library contract: [index](compiler/stdlib-index.md), [compiler and stdlib contract](compiler/stdlib-contract.md).

## Native and browser

- [LLVM backend](native/llvm-backend.md), [C ABI](native/c-abi.md), [C++ vtables](native/cpp-vtables.md), [linking](native/native-linking.md), [debug info](native/debug-info.md), [sanitizers](native/sanitizers.md), [crash runtime build](native/crash-runtime-build.md)
- [Windows](native/windows.md), [Windows console colors](native/windows-console-colors.md)
- [WebAssembly](native/wasm-target.md)
- [Browser build and playground](browser/playground.md)

## Tools and editors

[jacket](tools/jacket.md), [jacket test](tools/jacket-test.md), [jacket doc](tools/jacket-doc.md), [jacket bench](tools/jacket-bench.md), [jaifmt](tools/jaifmt.md), [jailint](tools/jailint.md), [VS Code](tools/vscode-extension.md), [Neovim](tools/neovim-plugin.md), [workspace trust](tools/workspace-trust.md), [environment variables](tools/environment.md).

## Standard library

- [Math](specs/stdlib/modules/math.md), [Extensions](extensions/README.md) (Io, Http, Tls, Json, [UI](extensions/ui.md) and the rest), [crypto helpers](stdlib/crypto.md)
- [Native layout](stdlib-native-layout.md), [float conversion](stdlib-float-conversion.md), [third-party native libraries](tools/native-libs.md), [Unicode tables](tools/regex-unicode-tables.md), [benchmarks](tools/stdlib-benchmarks.md)
- Postgres: [overview](postgres/design.md), [typed queries](postgres/typed-queries.md), [ORM](postgres/orm.md), [schema](postgres/schema.md), [migrations](postgres/migrations.md), [schema diff](postgres/diff.md), [COPY](postgres/copy.md), [managed server](postgres/managed.md), [conformance](postgres/conformance.md), [load test](postgres/benchmark.md)

## Building and testing

- Workflow: [development](tools/development.md), [testing](tools/testing.md), [test harness](tools/test-harness.md), [tools layout](tools/tools-layout.md), [CI job selection](tools/ci-job-selection.md), [LLVM setup](tools/llvm-setup.md), [Linux container](tools/linux-container.md).
- Quality: [diagnostic coverage](tools/diagnostic-coverage.md), [stdlib doc gaps](tools/stdlib-doc-gaps.md), [determinism](tools/determinism.md), [coverage](tools/coverage.md), [Rust coverage](tools/rust-coverage.md), [sanitizers and Miri](tools/sanitizers.md), [mutation testing](tools/mutation-testing.md).
- Fuzzing: [stdlib fuzzing](tools/stdlib-fuzzing.md), [language server typing sessions](tools/lsp-fuzzing.md), [language server oracle](tools/lsp-oracle.md).
- Performance: [benchmarks and profiling](tools/benchmarks.md), [stdlib benchmarks](tools/stdlib-benchmarks.md), [library comparisons](tools/library-comparisons.md), [performance gate](tools/perf-gate.md).
- Corpus and releases: [upstream corpus](tools/upstream-corpus.md), [dogfood applications](tools/dogfood.md), [releases](tools/releases.md), [release smoke tests](tools/release-smoke.md), [Windows probe](tools/windows-probe.md), [license](license.md).

## Reference

Generated from the code by `tools/generators/generate_reference_docs.py`: [command line](reference/command-line.md), [environment variables](reference/environment-variables.md) (declared in [the registry](tools/environment.md)), [lint rules](reference/lint-rules.md).

## Specifications

[specs/jai/](specs/jai/) and [specs/stdlib/](specs/stdlib/) are what the compiler and standard library are implemented from.
