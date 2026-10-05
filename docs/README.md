# Developer documentation

Start with [compiler architecture](compiler/architecture.md). The [README](../README.md) has the compatibility table; per-project notes are in [upstream corpus](tools/upstream-corpus.md#project-status).

## Language

Jai language behavior as implemented by `jaic`.

- [Casts and conversions](language/casts-and-conversions.md)
- [The implicit context](language/context.md)
- [Control flow: loops, cases and defer](language/control-flow.md)
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
- [Operator overloading](language/operator-overloading.md)
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

- [Code values and #insert](metaprogramming/code-values-and-insertion.md)
- [Compile-time values, globals and runtime info](metaprogramming/compile-time-data-and-state.md)
- [Compile-time execution (`#run`)](metaprogramming/compile-time-execution.md)
- [Compiler module (Jai side)](metaprogramming/compiler-module.md)
- [Compiler records (messages, syntax trees, type descriptors)](metaprogramming/compiler-records.md)
- [Preload and Runtime_Support bootstrap](specs/jai/README.md)
- [Reflection and Type_Info](metaprogramming/reflection-and-type-info.md)
- [Workspaces and metaprograms](metaprogramming/workspaces.md)

## Compiler internals

How `crates/jaic`, the interpreter and the language server work.

- [Compiler architecture](compiler/architecture.md)
- [jaic `#asm` blocks](compiler/asm.md)
- [Threads under `jaic run`](compiler/interpreter-threads.md)
- [jaic interpreter](compiler/interpreter.md)
- [Low-level IR](compiler/ir.md)
- [Shared Jai language server](compiler/language-server.md)
- [Parser](compiler/parser.md)
- [Sema: module loading and top-level expansion](compiler/sema-modules.md)
- [Sema: polymorphism and declarations](compiler/sema-polymorphism-and-declarations.md)
- [Stack traces (`context.stack_trace`)](compiler/stack-traces.md)

## Native builds

The LLVM backend, C ABI and linking.

- [C ABI, callbacks and C++ methods](native/c-abi.md)
- [jaic LLVM backend](native/llvm-backend.md)
- [Native debug information](native/debug-info.md)
- [Native build and linking](native/native-linking.md)
- [Vk-Engine corpus project](native/vk-engine.md)

## Standard library

The independently written `stdlib/`.

- [Standard library layout](specs/stdlib/README.md)
- [Basic and collection modules](specs/stdlib/README.md)
- [Basic calendar time, working directory and platform exports](specs/stdlib/README.md)
- [Binary formats and checksums](specs/stdlib/README.md)
- [Bindings_Generator](specs/stdlib/README.md)
- [Command_Line](specs/stdlib/README.md)
- [Compiler API, reflection and metaprogram support](specs/stdlib/README.md)
- [Files, processes and OS services](specs/stdlib/README.md)
- [GetRect](specs/stdlib/README.md)
- [Iprof profiler module](specs/stdlib/README.md)
- [Math, random numbers and color/float helpers](specs/stdlib/README.md)
- [Memory, allocators and hashing](specs/stdlib/README.md)
- [Native and platform bindings](specs/stdlib/README.md)
- [Program_Print](specs/stdlib/README.md)
- [Strings, Unicode and text files](specs/stdlib/README.md)
- [Threads, atomics, sockets and input](specs/stdlib/README.md)
- [Toolchains: macOS SDK and Android NDK helpers](specs/stdlib/README.md)
- [Tooling modules: Debug, MacOS_Bundler, BuildCpp, Autorun, Performance_Report](specs/stdlib/README.md)
- [Drawing, windows and audio](specs/stdlib/README.md)
- [Simp (2D renderer)](specs/stdlib/README.md)

## Browser

The WebAssembly playground, editor and release bundle.

- [Browser compiler releases](browser/compiler-releases.md)
- [Browser editor](browser/editor.md)
- [Browser playground (jaic backend)](browser/playground.md)

## Tools and workflow

Scripts, CI and project policies.

- [Benchmarks and profiling](tools/benchmarks.md)
- [Build storage and target directories](tools/build-storage.md)
- [Code formatting](tools/code-formatting.md)
- [Continuous integration](tools/continuous-integration.md)
- [Dependency policy](tools/dependency-policy.md)
- [jaic regression sweep](tools/jaic-sweep.md)
- [LLVM setup](tools/llvm-setup.md)
- [Third-party native libraries](tools/native-libs.md)
- [open-jai expectation harness](tools/openjai-expectations.md)
- [Releases](tools/releases.md)
- [Upstream corpus](tools/upstream-corpus.md)

## Language and library specifications

- [Jai language guide](specs/jai/README.md)
- [Standard library API specification](specs/stdlib/README.md)
