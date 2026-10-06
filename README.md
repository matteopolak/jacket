# `jai{c,lsp,fmt,lint}`

[![Compiler checks](https://github.com/matteopolak/jai/actions/workflows/ci.yml/badge.svg)](https://github.com/matteopolak/jai/actions/workflows/ci.yml)

An independent toolchain for the [Jai](https://en.wikipedia.org/wiki/Jai_(programming_language)) programming language:

- `jaic`, a compiler that type-checks, interprets and natively builds real Jai programs and libraries, with its own standard library;
- `jailsp`, a language server;
- `jaifmt`, a formatter;
- `jailint`, a linter.

The compiler, language server and formatter also run in the browser through WebAssembly, and the language server shows jailint's findings there too.

**[Try it in the browser →](https://matteopolak.com/playground/jai)**

- [What works](#what-works)
- [Performance](#performance)
- [What is missing](#what-is-missing)
- [Install](#install)
- [Usage](#usage)
- [Language server](docs/compiler/language-server.md) · [Formatter (jaifmt)](docs/tools/jaifmt.md) · [Linter (jailint)](docs/tools/jailint.md) · [Browser build](docs/browser/playground.md)
- [Compatibility with real projects](docs/tools/upstream-corpus.md#project-status)
- [Contributing](#contributing) · [Development](#development) · [Developer docs](docs/README.md)
- [License](#license)

## What works

**The language.** The whole core language: types, procedures, structs, enums, unions, `using`, `defer`, `context`, `Any`, polymorphism and baking, `#modify`, macros, `#code`/`#insert`, custom `for_expansion`, `#asm` (SSE through AVX2, FMA, AES, SHA, F16C, GFNI and the common AVX-512 instructions with mask registers, run on any CPU), SIMD, and arithmetic overflow and bounds checks. All 56 of the reference `how_to` programs run.

**Compile-time execution.** `#run` runs in an IR interpreter that can call into C libraries, and metaprograms get the full `Compiler` module: workspaces, the message loop and build options.

**Native executables** through LLVM, with debug information (DWARF on macOS and Linux):

| | x86-64 | arm64 |
| --- | :---: | :---: |
| macOS | ✅ | ✅ |
| Linux | ✅ | ✅ |
| Windows | ✅ | ✅ |

**WebAssembly modules** (wasm64): `jaic build -os wasm` writes a WASI command that runs under node 24, wasmtime or a Memory64 browser, and a metaprogram can target it with `os_target = .WASM` ([wasm target](docs/native/wasm-target.md)).

On macOS and Linux, `jaic build -sanitize address,undefined` adds AddressSanitizer and LLVM's bounds checks ([sanitizers](docs/native/sanitizers.md)); CI runs the test programs that way.

**Standard library.** An independently written `stdlib/` covering the modules real programs use (Basic, String, Hash_Table, File, Thread, Process, Compiler, Simp, GetRect, Sound_Player, Iprof and more), plus `Bindings_Generator` for C, C++ (including virtual bases) and Objective-C (including block literals).

**Tools.**
- A [language server](docs/compiler/language-server.md) (`jailsp`) with diagnostics, type-checked hover and completion, go to definition (including `#import`/`#load` targets), find references and rename, signature help, semantic tokens, inlay hints, format-string checks, and hovers and documents showing what macros, `#insert`, `#run` and `#if` expanded to ([feature list](docs/compiler/language-server.md#feature-list)).
- A [formatter](docs/tools/jaifmt.md) (`jaifmt`) that runs natively and in the browser, either interpreted or compiled to a 214 KB `jaifmt.wasm`. Its output is canonical and idempotent: formatting twice changes nothing.
- A [linter](docs/tools/jailint.md) (`jailint`) with rules that run on the type-checked program: unused variables, parameters and imports, loops that only index (`for i: 0..xs.count-1`), hand-kept counters, redundant casts, `x == true`, format strings with the wrong number of arguments, a shadowed `it`, a `defer` in a loop, likely bugs (`u >= 0` on an unsigned value, `1 << n - 1`, `for i: 0..xs.count` indexing `xs[i]`, identical branches or operands, a `while` whose condition nothing changes, `trim(s);` with the result dropped) and unidiomatic code (`x = x + 1`, `if c return true; else return false;`), and opt-in checks for exact float comparison and narrowing `xx`. It applies fixes with `--fix`, reads `jailint.toml`, honours `// jailint: allow(rule)`, and its findings and fixes also show up in editors through `jailsp`.
- A [browser build](docs/browser/playground.md) of the compiler and language server, used by the [online playground](https://matteopolak.com/playground/jai), which opens with a [multi-file tour of the language](examples/tour/tour.md) (`examples/tour`).

**Real projects** such as the Focus editor, the Jails language server, jaison, sgpu and the programs from *The Way to Jai* compile and run; see [the full list](docs/tools/upstream-corpus.md#project-status).

## Performance

Compile times for real projects, from a release `jaic` on an Apple M5 (LLVM 23, median of warm runs):

| Project | Lines of Jai | Type-check | Debug build | Release build |
| --- | ---: | ---: | ---: | ---: |
| [Focus](https://github.com/focus-editor/focus) (text editor) | 49k + 79k in modules | 1.7 s | 2.3 s | 15 s |
| [chess-jai](https://github.com/danieltan1517/chess-jai) | 10k | 0.66 s | 2.0 s | 4.9 s |
| [Jails](https://github.com/SogoCZE/Jails) (language server) | 5.6k | 0.22 s | 0.43 s | 2.2 s |
| [jaison](https://github.com/rluba/jaison) (tests) | 2.2k | 0.04 s | 0.17 s | 0.81 s |

Release builds spend most of their time in LLVM's optimiser. The formatter compiled to WebAssembly formats a 330-line file in about 1 ms. Method and full results: [compile-time benchmark](docs/tools/compile-time-benchmark.md), [latest numbers](benchmarks/results/compile-time-apple-m5.md).

## What is missing

- C's 16-byte `long double` (x86-64, arm64 Linux) is available only through jaic's non-standard [`Long_Double` extension](docs/language/jaic-extensions.md); C variadic calls cannot pass it, and in `jaic run` C cannot call back into Jai code that takes one.
- WebAssembly builds are wasm64 only, cannot call libm (`sin`, `pow`, float `%`) or use `Long_Double`, and have no threads or files beyond stdin, stdout and stderr.

Anything unsupported fails with a compile error rather than being silently accepted.

## Install

Prebuilt archives for macOS (Apple silicon), Linux (x86-64) and Windows (x86-64 and arm64), with `jaic`, `jailsp`, `jailint`, `jaifmt` and the standard library, are on the [releases page](https://github.com/matteopolak/jai/releases); see the [changelog](CHANGELOG.md). Unpack one anywhere (say `/opt/jaic`) and put `jaic` on your `PATH`, directly or through a symlink; it finds the `stdlib` folder next to the real file.

With Nix (see [Nix](docs/tools/nix.md)):

```sh
nix run github:matteopolak/jai -- run hello.jai
nix profile install github:matteopolak/jai
```

To build from source you need [Rustup](https://rustup.rs/) (it picks up the pinned toolchain) and, for native builds, LLVM 23 with Clang ([setup guide](docs/tools/llvm-setup.md)):

```sh
git clone https://github.com/matteopolak/jai.git
cd jai
cargo build -p jaic-cli --release --locked
cargo build -p jai-language-server -p jailint --release --locked   # jailsp and jailint
```

## Usage

```jai
Node :: struct(T: Type) {
    next: *Node(T);
    value: T = 19;
}

answer :: () -> int { return 23; }

main :: () -> int {
    node: Node(int);
    return node.value + #run answer(); // Exit status: 42
}
```

```sh
jaic check examples/compile-time-record.jai        # type-check only
jaic run examples/compile-time-record.jai          # run in the interpreter
jaic build examples/compile-time-record.jai -O2    # native executable (-os windows to cross-build)
jaic run examples/tour/main.jai                    # the language tour the playground opens with
jailint src/                                       # lint every .jai file under src/
jailint src/ --fix                                 # apply the safe fixes
```

## Contributing

Pull requests are not accepted; they are closed without review. Bug reports and feature requests are welcome as [issues](https://github.com/matteopolak/jai/issues/new/choose). See [CONTRIBUTING.md](CONTRIBUTING.md).

## Development

```sh
cargo test --workspace --locked --no-fail-fast
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
python3 tools/jaic-sweep.py corpus negative stdlib modules upstream howto --timeout 900   # everything should pass
target/jaifmt --check prelude stdlib tests benchmarks tools jaifmt examples   # build: jaic build jaifmt/build.jai
cargo run -p jailint -- -D warnings -j 2 prelude stdlib examples tools jaifmt tests benchmarks
```

The workspace is `crates/jaic` (lexer, parser, semantic analysis, IR, interpreter), `crates/jaic-cli` (the `jaic` binary), `crates/jaic-llvm` (native backend), `crates/jai-language-server`, `crates/jailint` (the linter) and `crates/jai-wasm` (browser build). Start with the [developer docs](docs/README.md) and the [compiler architecture](docs/compiler/architecture.md).

This project is a clean-room implementation, written without reading the source of an official Jai distribution: its modules, `how_to` programs, compiler or any other part of it. It is built from public documentation, third-party Jai code and the behaviour of programs in this repository's tests.

## License

jaic, its standard library and tools are licensed under the [GNU Affero General Public License v3.0 or later](LICENSE): you may use, study and change them, including commercially, but anything you distribute or offer over a network that is based on this code must be released under the same license, with its source and the original copyright notices.

Programs you compile are yours: a [runtime library exception](LICENSE-EXCEPTION) lets you ship programs that include code from `stdlib/` and `prelude/` under any license. See [docs/license.md](docs/license.md).
