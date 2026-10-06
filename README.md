# Jai, in Rust

[![Compiler checks](https://github.com/matteopolak/jai/actions/workflows/ci.yml/badge.svg)](https://github.com/matteopolak/jai/actions/workflows/ci.yml)

An independent compiler for the [Jai](https://en.wikipedia.org/wiki/Jai_(programming_language)) programming language, written in Rust, with its own standard library. It type-checks, interprets and natively builds real Jai programs and libraries, and it runs in the browser through WebAssembly.

**[Try it in the browser →](https://matteopolak.com/playground/jai)**

- [What works](#what-works)
- [What is missing](#what-is-missing)
- [Install](#install)
- [Usage](#usage)
- [Language server](docs/compiler/language-server.md) · [Formatter (jaifmt)](docs/tools/jaifmt.md) · [Browser build](docs/browser/playground.md)
- [Compatibility with real projects](docs/tools/upstream-corpus.md#project-status)
- [Contributing](#contributing) · [Developer docs](docs/README.md)
- [License](#license)

## What works

**The language.** The whole core language: types, procedures, structs, enums, unions, `using`, `defer`, `context`, `Any`, polymorphism and baking, `#modify`, macros, `#code`/`#insert`, custom `for_expansion`, `#asm` (SSE through AVX2, FMA, AES and the common AVX-512 instructions with mask registers, run on any CPU), SIMD, and arithmetic overflow and bounds checks. All 56 of the reference `how_to` programs run.

**Compile-time execution.** `#run` runs in an IR interpreter that can call into C libraries, and metaprograms get the full `Compiler` module: workspaces, the message loop and build options.

**Native executables** through LLVM, with debug information (DWARF on macOS and Linux):

| | x86-64 | arm64 |
| --- | :---: | :---: |
| macOS | ✅ | ✅ |
| Linux | ✅ | ✅ |
| Windows | ✅ (MSVC, or cross-built with MinGW-w64 via `-os windows`) | ✅ (MSVC, or cross-built with llvm-mingw via `-os windows -cpu arm64`) |

**Standard library.** An independently written `stdlib/` covering the modules real programs use (Basic, String, Hash_Table, File, Thread, Process, Compiler, Simp, GetRect, Sound_Player, Iprof and more), plus `Bindings_Generator` for C, C++ (including virtual bases) and Objective-C (including block literals).

**Tools.**
- A [language server](docs/compiler/language-server.md) (`jai-lsp`) with diagnostics, type-checked hover and completion, go to definition (including `#import`/`#load` targets), find references and rename, signature help, semantic tokens, inlay hints, format-string checks, and hovers and documents showing what macros, `#insert`, `#run` and `#if` expanded to ([feature list](docs/compiler/language-server.md#feature-list)).
- A [formatter](docs/tools/jaifmt.md) (`jaifmt`), written in Jai, that runs natively and in the browser. Its output is canonical and idempotent, like rustfmt.
- A [browser build](docs/browser/playground.md) of the compiler and language server, used by the [online playground](https://matteopolak.com/playground/jai), which opens with a [multi-file tour of the language](examples/tour/tour.md) (`examples/tour`).

**Real projects** such as the Focus editor, the Jails language server, jaison, sgpu and the programs from *The Way to Jai* compile and run; see [the full list](docs/tools/upstream-corpus.md#project-status).

## What is missing

- On Windows x64, C code cannot yet call back into procedures that run in the compile-time interpreter (it can on Windows arm64). On Windows, programs that start threads cannot run in the interpreter, and MSVC builds produce no PDB debug file.
- `Bindings_Generator` drops functions that use the 16-byte `long double`.
- `#asm` rejects the F16C, SHA and GFNI extensions.

Anything unsupported fails with a compile error rather than being silently accepted.

## Install

Prebuilt archives for macOS (Apple silicon), Linux (x86-64) and Windows (x86-64 and arm64) are on the [releases page](https://github.com/matteopolak/jai/releases); see the [changelog](CHANGELOG.md).

With Nix (see [Nix](docs/tools/nix.md)):

```sh
nix run github:matteopolak/jai -- run hello.jai
nix profile install github:matteopolak/jai
```

To build from source you need [Rustup](https://rustup.rs/) (it picks up the pinned toolchain) and, for native builds, LLVM 22 with Clang ([setup guide](docs/tools/llvm-setup.md)):

```sh
git clone https://github.com/matteopolak/jai.git
cd jai
cargo build -p jaic-cli --release --locked
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
```

## Contributing

```sh
cargo test --workspace --locked --no-fail-fast
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
python3 tools/jaic-sweep.py corpus negative stdlib modules upstream howto --timeout 900   # expect only the negative control to fail
```

The workspace is `crates/jaic` (lexer, parser, semantic analysis, IR, interpreter), `crates/jaic-cli` (the `jaic` binary), `crates/jaic-llvm` (native backend), `crates/jai-language-server` and `crates/jai-wasm` (browser build). Start with the [developer docs](docs/README.md) and the [compiler architecture](docs/compiler/architecture.md).


## License

jaic, its standard library and tools are licensed under the [GNU Affero General Public License v3.0 or later](LICENSE): you may use, study and change them, including commercially, but anything you distribute or offer over a network that is based on this code must be released under the same license, with its source and the original copyright notices.

Programs you compile are yours: a [runtime library exception](LICENSE-EXCEPTION) lets you ship programs that include code from `stdlib/` and `prelude/` under any license. See [docs/license.md](docs/license.md).
