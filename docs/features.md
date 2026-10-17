# Features

## What it is

What jaic, its standard library and tools support, what is missing, and how fast it compiles real projects. The root README stays short and links here.

## What works

- **Language.** The whole core language: types, procedures, structs, enums, unions, `using`, `defer`, `context`, `Any`, polymorphism and baking, `#modify`, macros, `#code`/`#insert`, `for_expansion`, `#asm` (SSE through AVX2, FMA, AES, SHA, F16C, GFNI and common AVX-512, run on any CPU), SIMD, and runtime checks for overflow, bounds, narrowing casts and `#complete` switches. All 56 reference `how_to` programs run.
- **Compile-time execution.** `#run` and `jaic -interp` use an IR interpreter that calls C libraries (C can call back into Jai, even from its own threads). Metaprograms get the full `Compiler` module: workspaces, the message loop, build options.
- **Errors.** Code excerpts with labels, notes and suggested fixes ([diagnostics](compiler/diagnostics.md)).
- **Native executables** through LLVM with debug information, for macOS, Linux and Windows on x86-64 and arm64. `jaic -sanitize address,undefined` adds ASan and bounds checks on macOS and Linux ([sanitizers](native/sanitizers.md)).
- **WebAssembly** (wasm64): `jaic -os wasm` writes a WASI command that runs under node 24, wasmtime or a Memory64 browser, with cooperative green threads ([wasm target](native/wasm-target.md), [threads](native/wasm-target.md)).
- **Standard library.** An independently written `stdlib/` (Basic, String, Hash_Table, File, Thread, Process, Compiler, Simp, GetRect, Sound_Player, Iprof and more) plus `Bindings_Generator` for C, C++ and Objective-C.
- **Extensions** (jaic-only modules in [`stdlib/Extensions/`](extensions/README.md)): [`WebGPU`](extensions/webgpu.md) (the whole `webgpu.h` API, native and in the playground), [`Long_Double`](language/long-double.md), [`Args`](extensions/args.md) (compile-time command-line parser), `Jai_Format` (the formatter as a library), `Wasi_Runtime`.
- **Real projects** such as Focus, Jails, jaison, jai-xml, sgpu and *The Way to Jai* compile and run ([upstream corpus](tools/upstream-corpus.md)).

### Tools

- [`jailsp`](compiler/language-server.md) ([README](../crates/jai-language-server/README.md)): diagnostics, type-checked hover and completion with auto-import, definition, references, rename, signature help, semantic tokens, inlay hints, format-string checks, and views of what macros, `#insert`, `#run` and `#if` expanded to.
- [`jaifmt`](tools/jaifmt.md) ([README](../jaifmt/README.md)): canonical, idempotent formatter that runs natively and in the browser (as a 283 KB `jaifmt.wasm`).
- [`jailint`](tools/jailint.md) ([README](../crates/jailint/README.md)): rules that run on the type-checked program, covering unused code, index-only loops, likely bugs (`u >= 0` on unsigned, `1 << n - 1`, identical branches, dropped results, wrapping constants) and unidiomatic code. `--fix`, `jailint.toml`, `// jailint: allow(rule)`, and the same findings in editors.
- A [browser build](browser/playground.md) of the compiler and language server behind the [online playground](https://matteopolak.com/playground/jacket), which opens with a [tour of the language](../examples/tour/tour.md).

## Performance

Compile times from a release `jaic` on an Apple M5 (LLVM 23, median of warm runs):

| Project | Lines of Jai | Type-check | Debug build | Release build |
| --- | ---: | ---: | ---: | ---: |
| [Focus](https://github.com/focus-editor/focus) | 49k + 79k in modules | 1.2 s | 1.6 s | 7.8 s |
| [chess-jai](https://github.com/danieltan1517/chess-jai) | 10k | 0.40 s | 0.70 s | 1.7 s |
| [Jails](https://github.com/SogoCZE/Jails) | 5.6k | 0.18 s | 0.27 s | 0.81 s |
| [jaison](https://github.com/rluba/jaison) (tests) | 2.2k | 0.03 s | 0.09 s | 0.41 s |

About 4 s of Focus's release build is its build script packaging the app. Method and results: [benchmarks](tools/benchmarks.md), [latest numbers](../benchmarks/results/compile-time-apple-m5.md).

## What is missing

- C's 16-byte `long double` exists only through the non-standard [`Extensions/Long_Double`](language/long-double.md); C variadic calls cannot pass it, and under `-interp` C cannot call back into Jai code taking one.
- WebAssembly builds are wasm64 only, run threads cooperatively, have no processes or sockets, see only pre-opened directories, and call only the common part of libm (`sin`, `pow`, `fmod`; not `sinh` or `cbrt`).

Anything unsupported fails with a compile error rather than being silently accepted.

## How to change it

Update this page when a feature lands or a limitation goes away, and keep the root README to a line that links here. Rerun the [benchmarks](tools/benchmarks.md) before changing the performance numbers.
