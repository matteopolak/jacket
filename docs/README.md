# Documentation

`jaic` is a compiler for the Jai language: lexer, parser, type checker, interpreter (`#run`, `-interp`), an LLVM backend for native code and WebAssembly, and a metaprogramming API. `jailsp` (language server), `jaifmt` (formatter, written in Jai) and `jailint` (linter) share its front end. The standard library in `stdlib/` is written in Jai.

New here: [Features](features.md) says what works, then [the command line](tools/cli.md) and the install steps in [Package managers](tools/package-managers.md). Contributors read [CONTRIBUTING.md](../CONTRIBUTING.md) and [Justfile](tools/justfile.md).

## Language

Reference for what the compiler accepts, one topic per page: [language/](language/), covering [modules and imports](language/modules-and-imports.md), [procedures](language/procedures.md), [structs](language/structs.md), [polymorphism](language/polymorphism.md), [macros](language/macros-and-custom-iteration.md), [the context](language/context.md), [numbers](language/numbers.md) and the rest. Compile-time execution and the `Compiler` module API are in [metaprogramming/](metaprogramming/).

## Toolchain

- [Compiler architecture](compiler/architecture.md), then [parser](compiler/parser.md), [semantic analysis](compiler/sema-modules.md), [IR](compiler/ir.md), [interpreter](compiler/interpreter.md), [LLVM backend](native/llvm-backend.md), [diagnostics](compiler/diagnostics.md) and [compile speed](compiler/compile-speed.md).
- Native targets: [C ABI](native/c-abi.md), [Windows](native/windows.md), [WebAssembly](native/wasm-target.md), [debug info](native/debug-info.md), [native linking](native/native-linking.md), [sanitizers](native/sanitizers.md).
- Tools: [jaifmt](tools/jaifmt.md), [jailint](tools/jailint.md), [language server](compiler/language-server.md), [browser build and playground](browser/playground.md).

## Editors

[VS Code](tools/vscode-extension.md), [Neovim](tools/neovim-plugin.md), or any editor that can run `jailsp` over stdio ([language server](compiler/language-server.md)).

## Standard library

[Extensions overview](extensions/README.md) and [conventions](extensions/extensions-conventions.md); how the native-heavy modules are laid out is in [stdlib native layout](stdlib-native-layout.md). The behaviour of the standard library modules is specified in [specs/stdlib/](specs/stdlib/); open questions are in [specs/questions/](specs/questions/).

## Specifications

[specs/jai/](specs/jai/) and [specs/stdlib/](specs/stdlib/) are what the compiler and standard library are implemented from. They are the source of truth for behaviour; unresolved points go in [specs/questions/](specs/questions/) rather than being guessed.

## Building, testing and releasing

[Justfile](tools/justfile.md), [continuous integration](tools/continuous-integration.md), [stdlib tests](tools/stdlib-runtime-tests.md), [upstream corpus](tools/upstream-corpus.md), [differential testing](tools/differential-testing.md), [fuzzing](tools/fuzzing.md), [benchmarks](tools/benchmarks.md), [native libraries](tools/native-libs.md), [LLVM setup](tools/llvm-setup.md), [Nix](tools/nix.md), [releases](tools/releases.md), [dependency policy](tools/dependency-policy.md), [license](license.md).
