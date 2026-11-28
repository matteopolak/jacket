# Features

What jaic, its standard library and tools support, and what is missing.

## What works

- **Language.** The whole core language: types, procedures, structs, enums, unions, `using`, `defer`, `context`, `Any`, polymorphism and baking, `#modify`, macros, `#code`/`#insert`, `for_expansion`, `#asm` (SSE through AVX2, FMA, AES, SHA, F16C, GFNI and common AVX-512, run on any CPU), SIMD, and runtime checks for overflow, bounds, narrowing casts and `#complete` switches.
- **Compile-time execution.** `#run` and `jaic -interp` use an IR interpreter that calls C libraries (C can call back into Jai, even from its own threads). Metaprograms get the full `Compiler` module: workspaces, the message loop, build options.
- **Errors.** Code excerpts with labels, notes and suggested fixes ([diagnostics](compiler/diagnostics.md)).
- **Native executables** through LLVM with debug information, for macOS, Linux and Windows on x86-64 and arm64. `jacket build -sanitize address,undefined` adds ASan and bounds checks on macOS and Linux ([sanitizers](native/sanitizers.md)).
- **WebAssembly** (wasm64 and wasm32): `jacket build -os wasm` writes a WASI command that runs under node, wasmtime or a Memory64 browser, with cooperative green threads ([wasm target](native/wasm-target.md)).
- **Other targets** (experimental): 32-bit x86 and ARM Linux, big-endian s390x and powerpc64 Linux, and freestanding Cortex-M, AArch64 and AVR boards ([other targets](guides/other-targets.md)).
- **Standard library.** `stdlib/` (Basic, String, Hash_Table, File, Thread, Process, Compiler, Simp, GetRect, Sound_Player, Iprof and more) plus `Bindings_Generator` for C, C++ and Objective-C.
- **Extensions** ([`stdlib/Extensions/`](extensions/README.md)): [`WebGPU`](extensions/webgpu.md), [`Long_Double`](language/long-double.md), [`Args`](extensions/args.md), `Jai_Format` (the formatter as a library), `Wasi_Runtime`, and more.
- **Real projects** such as Focus, Jails, jaison, jai-xml, sgpu and *The Way to Jai* compile and run ([upstream corpus](tools/upstream-corpus.md)).

### Tools

- [`jailsp`](compiler/language-server.md) ([README](../crates/jai-language-server/README.md)): diagnostics, hover, completion with auto-import, definition, references, rename, signature help, semantic tokens, inlay hints, format-string checks, and views of what macros, `#insert`, `#run` and `#if` expanded to.
- [`jaifmt`](tools/jaifmt.md) ([README](../jaifmt/README.md)): canonical, idempotent formatter, native and in the browser.
- [`jailint`](tools/jailint.md) ([README](../crates/jailint/README.md)): rules on the type-checked program for unused code, likely bugs and unidiomatic code. `--fix`, `[lint]` in `jacket.toml`, `// jailint: allow(rule)`.
- A [browser build](browser/playground.md) of the compiler and language server behind the [online playground](https://jacket.matteopolak.com/playground), with [example workspaces](../examples/basics/README.md).

## What is missing

- C's 16-byte `long double` exists only through [`Extensions/Long_Double`](language/long-double.md); C variadic calls cannot pass it, and under `-interp` C cannot call back into Jai code taking one.
- WebAssembly builds run threads cooperatively, have no processes, use only pre-opened sockets and directories, and call only the common part of libm (`sin`, `pow`, `fmod`; not `sinh` or `cbrt`).

Anything unsupported fails with a compile error rather than being silently accepted.

## How to change it

Update this page when a feature lands or a limitation goes away. The root README lists only the standout features of each tool and links here for the rest.
