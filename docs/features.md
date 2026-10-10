# Features

## What it is

What jaic, its standard library and tools support today, what is missing, and how fast it compiles real projects. The root README stays short and links here.

## What works

### Language

The whole core language: types, procedures, structs, enums, unions, `using`, `defer`, `context`, `Any`, polymorphism and baking, `#modify`, macros, `#code`/`#insert`, custom `for_expansion`, `#asm` (SSE through AVX2, FMA, AES, SHA, F16C, GFNI and the common AVX-512 instructions with mask registers, run on any CPU), SIMD, and runtime checks for arithmetic overflow, bounds, narrowing casts and `#complete` switches. All 56 of the reference `how_to` programs run.

### Compile-time execution

`#run` and `jaic run` use an IR interpreter that calls into C libraries, and C can call back into Jai through procedure pointers passed as arguments or stored in memory, even from threads C starts itself. A crash inside C code names the foreign procedure and the Jai call stack. Metaprograms get the full `Compiler` module: workspaces, the message loop and build options.

### Errors

Errors show the code around the problem with labels, notes and suggested fixes, in colour with box drawing when the terminal supports it ([diagnostics](compiler/diagnostics.md)).

### Native executables

Native executables through LLVM, with debug information (DWARF on macOS and Linux):

| | x86-64 | arm64 |
| --- | :---: | :---: |
| macOS | ✅ | ✅ |
| Linux | ✅ | ✅ |
| Windows | ✅ | ✅ |

### WebAssembly

WebAssembly modules (wasm64): `jaic build -os wasm` writes a WASI command that runs under node 24, wasmtime or a Memory64 browser, with threads run as cooperative green threads ([threads in WASI builds](native/wasm-threads.md)), and a metaprogram can target it with `os_target = .WASM` ([wasm target](native/wasm-target.md)).

On macOS and Linux, `jaic build -sanitize address,undefined` adds AddressSanitizer and LLVM's bounds checks ([sanitizers](native/sanitizers.md)); CI runs the test programs that way.

### Standard library

An independently written `stdlib/` covering the modules real programs use (Basic, String, Hash_Table, File, Thread, Process, Compiler, Simp, GetRect, Sound_Player, Iprof and more), plus `Bindings_Generator` for C, C++ (including virtual bases) and Objective-C (including block literals).

### Extensions

Modules only jaic has live in [`stdlib/Extensions/`](specs/stdlib/README.md) and are imported by that path:

| Module | What it is |
| --- | --- |
| [`Extensions/WebGPU`](specs/stdlib/README.md) | The whole `webgpu.h` API: one program draws in a window on macOS, Linux and Windows (wgpu-native ships in the release archives) and in the playground's Render tab (browsers with WebGPU and JSPI) |
| [`Extensions/Long_Double`](language/long-double.md) | C's 16-byte `long double` |
| [`Extensions/Args`](specs/stdlib/README.md) | A clap-style command-line parser generated at compile time: option notes, subcommands, help, completions |
| `Extensions/Jai_Format` | The formatter `jaifmt` is built on, as a library |
| `Extensions/Wasi_Runtime` | The runtime of `-os wasm` builds, imported automatically |

### Tools

- A [language server](compiler/language-server.md) (`jailsp`, [README](../crates/jai-language-server/README.md)) with diagnostics, type-checked hover and completion (with auto-import of names from modules and project files), go to definition (including `#import`/`#load` targets), find references and rename, signature help, semantic tokens, inlay hints, format-string checks, and hovers and documents showing what macros, `#insert`, `#run` and `#if` expanded to ([feature list](compiler/language-server.md#feature-list)).
- A [formatter](tools/jaifmt.md) (`jaifmt`, [README](../jaifmt/README.md)) that runs natively and in the browser, either interpreted or compiled to a 283 KB `jaifmt.wasm`. Its output is canonical and idempotent: formatting twice changes nothing.
- A [linter](tools/jailint.md) (`jailint`, [README](../crates/jailint/README.md)) with rules that run on the type-checked program: unused variables, parameters and imports, loops that only index (`for i: 0..xs.count-1`), hand-kept counters, redundant casts, `x == true`, format strings with the wrong number of arguments, a shadowed `it`, a `defer` in a loop, likely bugs (`u >= 0` on an unsigned value, `1 << n - 1`, `for i: 0..xs.count` indexing `xs[i]`, identical branches or operands, a `while` whose condition nothing changes, `trim(s);` with the result dropped) and unidiomatic code (`x = x + 1`, `if c return true; else return false;`), and constants that silently wrap to a narrower operand type, and opt-in checks for exact float comparison and narrowing `xx`. It applies fixes with `--fix`, reads `jailint.toml`, honours `// jailint: allow(rule)`, and its findings and fixes also show up in editors through `jailsp`.
- A [browser build](browser/playground.md) of the compiler and language server, used by the [online playground](https://matteopolak.com/playground/jai), which opens with a [multi-file tour of the language](../examples/tour/tour.md) (`examples/tour`).

### Real projects

Real projects such as the Focus editor, the Jails language server, jaison, jai-xml, sgpu and the programs from *The Way to Jai* compile and run; see [the full list](tools/upstream-corpus.md#project-status).

## Performance

Compile times for real projects, from a release `jaic` on an Apple M5 (LLVM 23, median of warm runs):

| Project | Lines of Jai | Type-check | Debug build | Release build |
| --- | ---: | ---: | ---: | ---: |
| [Focus](https://github.com/focus-editor/focus) (text editor) | 49k + 79k in modules | 1.2 s | 1.6 s | 7.8 s |
| [chess-jai](https://github.com/danieltan1517/chess-jai) | 10k | 0.40 s | 0.70 s | 1.7 s |
| [Jails](https://github.com/SogoCZE/Jails) (language server) | 5.6k | 0.18 s | 0.27 s | 0.81 s |
| [jaison](https://github.com/rluba/jaison) (tests) | 2.2k | 0.03 s | 0.09 s | 0.41 s |

About 4 s of Focus's release build is its build script packaging the app (`dsymutil`, `hdiutil`); release builds optimize up to 8 parts of the program in parallel. The formatter compiled to WebAssembly formats a 330-line file in about 1 ms. Method and full results: [compile-time benchmark](tools/compile-time-benchmark.md), [latest numbers](../benchmarks/results/compile-time-apple-m5.md).

## What is missing

- C's 16-byte `long double` (x86-64, arm64 Linux) is available only through jaic's non-standard [`Extensions/Long_Double`](language/long-double.md) module; C variadic calls cannot pass it, and in `jaic run` C cannot call back into Jai code that takes one.
- WebAssembly builds are wasm64 only, run threads cooperatively (a busy wait that never blocks keeps the turn), have no processes or sockets, see only the directories their runtime pre-opens, and can call only the common part of libm (`sin`, `pow`, `fmod` and the like, not `sinh` or `cbrt`).

Anything unsupported fails with a compile error rather than being silently accepted.

## How to change it

Update this page when a feature lands or a limitation goes away, and keep the root README to a line that links here. The performance table comes from the [compile-time benchmark](tools/compile-time-benchmark.md); rerun it before changing the numbers.
