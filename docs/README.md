# Documentation

Jacket is a toolchain for the Jai language. `jaic` is the compiler (lexer, parser, type checker, interpreter, LLVM backend for native code and WebAssembly, metaprogramming API). `jailsp`, `jaifmt` and `jailint` share its front end, and `jacket` runs them all from one `jacket.toml`. The standard library in `stdlib/` is written in Jai. The API reference is at https://jacket.matteopolak.com.

Start with [features](features.md), the [command line](tools/cli.md) and [install channels](tools/releases.md#install-channels). Contributors: [CONTRIBUTING.md](../CONTRIBUTING.md) and [development](tools/development.md).

## Guides

- [Testing](guides/testing.md): `@test` procedures, golden files, snapshots and doc tests.
- [Benchmarking](guides/benchmarking.md): `@bench` procedures and baselines.
- [Documenting a library](guides/documenting.md): doc comments, runnable examples and a docs site.
- [Memory safety](guides/memory-safety.md): what `jacket lint` reports about freed, reset and moved memory.
- [Dependencies](guides/dependencies.md): git dependencies with `jacket dep`.
- [Other targets](guides/other-targets.md): WebAssembly, 32-bit and big-endian Linux, boards.
- [Toolchains](guides/toolchains.md): linkers, sysroots and builtins for cross builds.

## Reference

Generated from the code by `tools/generators/generate_reference_docs.py`: [tools](reference/tools.md) (every flag, and the environment variables declared in [the registry](tools/environment.md)) and [lint rules](reference/lint-rules.md). The docs site's [front page](reference/home.md).

## Tools and editors

- [jacket](tools/jacket.md), [jaic command line](tools/cli.md), [jacket test](tools/jacket-test.md), [jacket bench](tools/jacket-bench.md), [jacket doc](tools/jacket-doc.md) ([site assets](tools/doc-site-assets.md)), [jacket migrate](tools/jacket-migrate.md), [dependencies](tools/dependencies.md).
- [jaifmt](tools/jaifmt.md) ([rules](tools/jaifmt-rules.md)), [jailint](tools/jailint.md), [argdef](tools/argdef.md) (option parsing, help and completions).
- Editors: [VS Code](tools/vscode-extension.md), [Neovim](tools/neovim-plugin.md), [workspace trust](tools/workspace-trust.md).
- [Environment variables](tools/environment.md), [prerequisites](tools/prerequisites.md), [community chat](tools/community-chat.md).

## Language

One page per topic in [language/](language/), describing what jaic does: [modules](language/modules-and-imports.md), [module parameters](language/module-parameters.md), [scoping](language/scoping.md), [`using`](language/using.md), [declarations](language/declarations-and-constants.md), [procedures](language/procedures.md), [polymorphism](language/polymorphism.md), [macros](language/macros-and-custom-iteration.md), [inline holes](language/interpolates.md), [numbers](language/numbers.md), [strings](language/strings-and-literals.md), [pointers and arrays](language/pointers-and-arrays.md), [casts](language/casts-and-conversions.md), [operators](language/operators.md), [operator overloading](language/operator-overloading.md), [enums](language/enums.md), [structs](language/structs.md), [unions](language/unions.md), [control flow](language/control-flow.md), [directives and notes](language/directives-and-notes.md), [intrinsics](language/intrinsics.md), [SIMD and `#asm`](language/simd-asm.md), [`long double`](language/long-double.md), [context](language/context.md), [dead-code elimination](language/dead-code-elimination.md), and [leniencies](language/leniencies.md) (what jaic accepts that the official compiler rejects).

## Metaprogramming

- [Compile-time execution, `#code` and `#insert`](metaprogramming/compile-time-execution.md)
- [Compiler module and checked AST types](metaprogramming/compiler-module.md)
- [Workspaces and plugins](metaprogramming/workspaces.md)
- [Build options](metaprogramming/build-options.md)
- [Reflection and type info](metaprogramming/reflection-and-type-info.md)

## Compiler

- Overview: [architecture and crates](compiler/architecture.md), [lazy code generation backend](compiler/lazy-backend.md), [project structure](compiler/project.md), [lexicon](compiler/lexicon.md), [prelude selection](compiler/prelude-selection.md), [command-line driver](compiler/command-line-driver.md).
- Front end: [parser](compiler/parser.md), [sema](compiler/sema.md), [parallel type checking](compiler/parallel-sema.md), [polymorph reporting](compiler/polymorph-reporting.md), [file identity](compiler/file-identity.md), [source positions](compiler/source-positions.md).
- Back end: [IR](compiler/ir.md), [interpreter](compiler/interpreter.md), [interpreter memory model](compiler/interpreter-memory-model.md), [fibers](compiler/fibers.md), [`#asm`](compiler/asm.md), [integer arithmetic](compiler/integer-arithmetic.md), [compile-time limits](compiler/compile-time-limits.md), [compile-time code on the host](compiler/compile-time-host-execution.md), [targets](compiler/targets.md), [toolchain](compiler/toolchain.md), [stack traces](compiler/stack-traces.md), [compile speed](compiler/compile-speed.md).
- Diagnostics and editors: [diagnostics](compiler/diagnostics.md), [doc comments](compiler/doc-comments.md), [language server](compiler/language-server.md), [IDE facts](compiler/ide-facts.md), [note readers](compiler/note-readers.md), [saved configuration](compiler/saved-configuration.md).
- [Compiler and stdlib contract](compiler/stdlib-contract.md).

## Native and browser

- [LLVM backend](native/llvm-backend.md), [C ABI](native/c-abi.md), [C++ vtables](native/cpp-vtables.md), [linking](native/native-linking.md), [debug info](native/debug-info.md), [sanitizers](native/sanitizers.md), [crash runtime build](native/crash-runtime-build.md).
- [Windows](native/windows.md), [Windows console colors](native/windows-console-colors.md).
- [WebAssembly](native/wasm-target.md), [per-CPU Linux POSIX bindings](native/linux-cpu-bindings.md), [embedded and freestanding targets](native/embedded.md), [bindings for another target](native/bindings-targets.md), [toolchain discovery](native/toolchain-discovery.md).
- [Browser build and playground](browser/playground.md).

## Standard library

- [Extensions](extensions/README.md): one page per module that only Jacket has (Http, Json, Toml, Time, Postgres, UI and the rest).
- Postgres: [overview](postgres/design.md), [typed queries](postgres/typed-queries.md), [ORM and COPY](postgres/orm.md), [schema and diff](postgres/schema.md), [migrations](postgres/migrations.md), [managed server](postgres/managed.md), [conformance and load test](postgres/conformance.md).
- Internals: [default allocator](stdlib/default-allocator.md), [memory debugger](stdlib/memory-debugger.md), [native layout](stdlib/native-layout.md), [float conversion](stdlib/float-conversion.md), [crypto helpers](stdlib/crypto.md), [MeowHash input safety](stdlib/meow-hash-safety.md), [ICO files](stdlib/ico-files.md), [GetRect default font](stdlib/getrect-default-font.md).
- [Math](specs/stdlib/modules/math.md), [third-party native libraries](tools/native-libs.md), [Unicode tables](tools/regex-unicode-tables.md).

## Building and testing

- Workflow: [development](tools/development.md), [testing](tools/testing.md), [test harness](tools/test-harness.md), [tools layout](tools/tools-layout.md), [tools scripts](tools/tools-scripts.md), [CI job selection and wall time](tools/ci-job-selection.md), [action pins](tools/action-pins.md), [pin updates](tools/pin-updates.md), [LLVM setup](tools/llvm-setup.md), [Linux container](tools/linux-container.md), [generated images](tools/generated-images.md).
- Quality: [diagnostic coverage](tools/diagnostic-coverage.md), [stdlib doc gaps](tools/stdlib-doc-gaps.md), [determinism](tools/determinism.md), [coverage](tools/coverage.md), [Rust coverage](tools/rust-coverage.md), [sanitizers and Miri](tools/sanitizers.md), [mutation testing](tools/mutation-testing.md).
- Language server: [project indexing](tools/lsp-project-indexing.md), [inclusion-graph audit](tools/lsp-graph-audit.md), [architecture plan](tools/lsp-architecture.md), [typing sessions](tools/lsp-fuzzing.md), [oracle](tools/lsp-oracle.md).
- Fuzzing: [stdlib fuzzing](tools/stdlib-fuzzing.md).
- Performance: [benchmarks and profiling](tools/benchmarks.md), [stdlib benchmarks](tools/stdlib-benchmarks.md), [library comparisons](tools/library-comparisons.md), [performance gate](tools/perf-gate.md).
- Conformance: [image codecs](tools/image-conformance.md), [gRPC interoperability](tools/grpc-interop.md).
- Corpus and releases: [upstream corpus](tools/upstream-corpus.md), [bindings corpus](tools/bindings-corpus.md), [dogfood applications](tools/dogfood.md), [releases](tools/releases.md), [release smoke tests](tools/release-smoke.md), [distro smoke tests](tools/distro-smoke.md), [Windows probe](tools/windows-probe.md), [license](license.md).

## Specifications

[specs/jai/](specs/jai/) and [specs/stdlib/](specs/stdlib/) are what the compiler and standard library are implemented from. They are public domain ([license](specs/LICENSE.md)).
