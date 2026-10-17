# Documentation

`jaic` is a compiler for the Jai language: lexer, parser, type checker, interpreter (`#run`, `-interp`), an LLVM backend for native code and WebAssembly, and a metaprogramming API. `jailsp` (language server), `jaifmt` (formatter, written in Jai) and `jailint` (linter) share its front end. The standard library in `stdlib/` is written in Jai.

Start with [features](features.md), [the command line](tools/cli.md) and [install channels](tools/releases.md). Contributors: [CONTRIBUTING.md](../CONTRIBUTING.md) and [development](tools/development.md).

## Language

[language/](language/) has one page per topic (for example [modules](language/modules-and-imports.md), [procedures](language/procedures.md), [polymorphism](language/polymorphism.md), [macros](language/macros-and-custom-iteration.md), [numbers](language/numbers.md)). Compile-time execution and the `Compiler` module are in [metaprogramming/](metaprogramming/).

## Compiler

[Architecture](compiler/architecture.md), [parser](compiler/parser.md), [sema](compiler/sema.md), [variables vs constants](compiler/variable-constness.md), [IR](compiler/ir.md), [interpreter](compiler/interpreter.md), [diagnostics](compiler/diagnostics.md), [compile speed](compiler/compile-speed.md), [`#asm`](compiler/asm.md), [stack traces](compiler/stack-traces.md), [doc comments](compiler/doc-comments.md), [language server](compiler/language-server.md).

## Native and browser

[LLVM backend](native/llvm-backend.md), [C ABI](native/c-abi.md), [Windows](native/windows.md), [WebAssembly](native/wasm-target.md), [debug info](native/debug-info.md), [linking](native/native-linking.md), [crash runtime build](native/crash-runtime-build.md), [sanitizers](native/sanitizers.md), [browser build and playground](browser/playground.md).

## Tools and editors

[jaifmt](tools/jaifmt.md), [jailint](tools/jailint.md), [VS Code](tools/vscode-extension.md), [Neovim](tools/neovim-plugin.md), [workspace trust](tools/workspace-trust.md).

## Standard library

[Extensions](extensions/README.md) (Io, Http, Tls, Json and the rest), [native layout](stdlib-native-layout.md), [third-party native libraries](tools/native-libs.md), [Unicode tables](tools/regex-unicode-tables.md).

## Building and testing

[Development](tools/development.md) (Justfile, CI, hooks, dependency policy), [testing](tools/testing.md), [upstream corpus](tools/upstream-corpus.md), [benchmarks](tools/benchmarks.md), [LLVM setup](tools/llvm-setup.md), [releases](tools/releases.md), [license](license.md).

## Specifications

[specs/jai/](specs/jai/) and [specs/stdlib/](specs/stdlib/) are what the compiler and standard library are implemented from. Unresolved points go in [specs/questions/](specs/questions/).
