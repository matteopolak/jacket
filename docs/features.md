# Features

What jaic, its standard library and tools support, what is missing, and how fast it compiles real projects.

## What works

- **Language.** The whole core language: types, procedures, structs, enums, unions, `using`, `defer`, `context`, `Any`, polymorphism and baking, `#modify`, macros, `#code`/`#insert`, `for_expansion`, `#asm` (SSE through AVX2, FMA, AES, SHA, F16C, GFNI and common AVX-512, run on any CPU), SIMD, and runtime checks for overflow, bounds, narrowing casts and `#complete` switches.
- **Compile-time execution.** `#run` and `jaic -interp` use an IR interpreter that calls C libraries (C can call back into Jai, even from its own threads). Metaprograms get the full `Compiler` module: workspaces, the message loop, build options.
- **Errors.** Code excerpts with labels, notes and suggested fixes ([diagnostics](compiler/diagnostics.md)).
- **Native executables** through LLVM with debug information, for macOS, Linux and Windows on x86-64 and arm64. `jacket build -sanitize address,undefined` adds ASan and bounds checks on macOS and Linux ([sanitizers](native/sanitizers.md)).
- **WebAssembly** (wasm64): `jacket build -os wasm` writes a WASI command that runs under node 24, wasmtime or a Memory64 browser, with cooperative green threads ([wasm target](native/wasm-target.md)).
- **Standard library.** `stdlib/` (Basic, String, Hash_Table, File, Thread, Process, Compiler, Simp, GetRect, Sound_Player, Iprof and more) plus `Bindings_Generator` for C, C++ and Objective-C.
- **Extensions** ([`stdlib/Extensions/`](extensions/README.md)): [`WebGPU`](extensions/webgpu.md), [`Long_Double`](language/long-double.md), [`Args`](extensions/args.md), `Jai_Format` (the formatter as a library), `Wasi_Runtime`, and more.
- **Real projects** such as Focus, Jails, jaison, jai-xml, sgpu and *The Way to Jai* compile and run ([upstream corpus](tools/upstream-corpus.md)).

### Tools

- [`jailsp`](compiler/language-server.md) ([README](../crates/jai-language-server/README.md)): diagnostics, hover, completion with auto-import, definition, references, rename, signature help, semantic tokens, inlay hints, format-string checks, and views of what macros, `#insert`, `#run` and `#if` expanded to.
- [`jaifmt`](tools/jaifmt.md) ([README](../jaifmt/README.md)): canonical, idempotent formatter, native and in the browser.
- [`jailint`](tools/jailint.md) ([README](../crates/jailint/README.md)): rules on the type-checked program for unused code, likely bugs and unidiomatic code. `--fix`, `[lint]` in `jacket.toml`, `// jailint: allow(rule)`.
- A [browser build](browser/playground.md) of the compiler and language server behind the [online playground](https://matteopolak.com/playground/jacket), with [example workspaces](../examples/basics/README.md).

## Performance

Compile times from a release `jaic` on an Apple M5 (LLVM 23, median of warm runs):

| Project | Lines of Jai | Type-check | Debug build | Release build |
| --- | ---: | ---: | ---: | ---: |
| [Focus](https://github.com/focus-editor/focus) | 49k + 79k in modules | 0.49 s | 0.90 s | 4.1 s |
| [Jails](https://github.com/SogoCZE/Jails) | 4.0k | 0.07 s | 0.17 s | 0.54 s |
| [jaison](https://github.com/rluba/jaison) (tests) | 1.7k | 0.05 s | 0.10 s | 0.52 s |

Rerun with `PYTHONPATH=tools python3 tools/bench/main.py compile --categories real` ([benchmarks](tools/benchmarks.md)).

Against the official compiler on the same machine, jaic's wall time divided by its time (below 1 is faster), warm:

| Workload | Type-check | Debug build | Optimized build |
| --- | ---: | ---: | ---: |
| Generated, 10k lines | 0.75 | 0.73 | 0.21 |
| Generated, 100k lines | 0.87 | 0.46 | 0.09 |
| Jails | 0.68 | 0.76 | 0.24 |
| jaison tests | | 0.71 | 0.35 |
| Small benchmark programs | | 0.86-0.92 | 0.82-0.92 |

Process start and linking cost the same in both, so small programs sit near 0.9. Read these as rough; the machine was busy.

## What is missing

- C's 16-byte `long double` exists only through [`Extensions/Long_Double`](language/long-double.md); C variadic calls cannot pass it, and under `-interp` C cannot call back into Jai code taking one.
- WebAssembly builds are wasm64 only, run threads cooperatively, have no processes or sockets, see only pre-opened directories, and call only the common part of libm (`sin`, `pow`, `fmod`; not `sinh` or `cbrt`).

Anything unsupported fails with a compile error rather than being silently accepted.

## How to change it

Update this page when a feature lands or a limitation goes away, and keep the root README to a line that links here. Rerun the [benchmarks](tools/benchmarks.md) before changing the numbers.
