# Jai, in Rust

[![Compiler checks](https://github.com/matteopolak/jai/actions/workflows/ci.yml/badge.svg)](https://github.com/matteopolak/jai/actions/workflows/ci.yml)

An independent Jai compiler written in Rust. The goal is to compile existing Jai programs and libraries, with a clean implementation that is easy to test, understand, and improve.

It type-checks, interprets and natively builds real Jai projects, including the Focus editor, the Jails language server and the examples from *The Way to Jai*. It also runs in the browser through WebAssembly.

## Compatibility

✅ supported · ⚠️ partial · ❌ not supported yet

### Language and compiler

| Feature | Status | Notes |
| --- | :---: | --- |
| Core language (types, procedures, structs, enums, unions, control flow, `defer`, `context`, `Any`) | ✅ | |
| Polymorphism, baking, `#modify`, `#type_info_*` | ✅ | |
| Macros, `#code`, `#insert`, custom `for_expansion` | ✅ | |
| Compile-time execution (`#run`) | ✅ | Runs in the IR interpreter, including foreign calls |
| Metaprograms (`Compiler` module: workspaces, message loop, build options) | ✅ | |
| Arithmetic overflow and bounds checks | ✅ | |
| `#asm` and SIMD | ⚠️ | Common instructions; string ops, division, x87 and mask registers are rejected |
| Reference `how_to` programs | ✅ | 56 of 56 run |

### Targets and tooling

| Area | Status | Notes |
| --- | :---: | --- |
| Interpreter (`jaic run`) | ✅ | |
| Native executables on macOS (arm64, x86-64) | ✅ | LLVM backend |
| Native executables on Linux (arm64, x86-64) | ✅ | LLVM backend |
| Checking for Windows (`-os windows`) | ✅ | |
| Native executables for Windows | ❌ | No Win64 calling convention yet |
| Native debug information | ✅ | DWARF on macOS (`.dSYM`) and Linux: lines, backtraces, typed locals and globals; Windows CodeView not yet |
| Browser playground (WebAssembly) | ✅ | 168 of 186 stdlib tests run; the rest need native processes or libraries |
| Language server (`jai-lsp`) | ✅ | Diagnostics, type-checked hover and completion, go to definition |
| `Bindings_Generator` | ⚠️ | C, C++ and Objective-C; no virtual bases, `objc_msgSend_fpret` or block literals |

### Projects

| Project | Status | Notes |
| --- | :---: | --- |
| [Focus](https://github.com/focus-editor/focus) | ✅ | Builds and runs natively on macOS |
| [Jails](https://github.com/SogoCZE/Jails) | ✅ | Builds a native language server |
| [jaison](https://github.com/rluba/jaison) | ✅ | Tests and examples run, also natively |
| [sgpu](https://github.com/roeyb1/sgpu) | ✅ | All examples build on macOS; mesh shaders need a driver MoltenVK lacks |
| [The Way to Jai](https://github.com/Ivo-Balbaert/The_Way_to_Jai) | ✅ | 316 of 343 programs run; the rest check (windowed, interactive, Windows-only or deliberately failing) |
| [Vk-Engine](https://github.com/ostef/Vk-Engine) | ⚠️ | Checks for Linux; its Vulkan, ImGui and Jolt modules have no macOS support |

The exact revisions are pinned in `corpus/upstreams.json`; per-project build notes are in [upstream corpus](docs/tools/upstream-corpus.md#project-status).

For example, record specialization and compile-time execution can work together:

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

This example is included as [compile-time-record.jai](examples/compile-time-record.jai).

An independently authored standard library is included in [`stdlib/`](stdlib/). See [the stdlib docs](docs/stdlib/architecture.md).

## Try it

Prebuilt archives for macOS (Apple silicon), Linux (x86-64) and Windows (x86-64) are on the [releases page](https://github.com/matteopolak/jai/releases); see the [changelog](CHANGELOG.md). To build from source:

You need [Rustup](https://rustup.rs/) (it picks up this repository's pinned toolchain) and, for native builds, an independently installed LLVM 22 with Clang; see the [LLVM setup guide](docs/tools/llvm-setup.md).

```sh
git clone https://github.com/matteopolak/jai.git
cd jai
cargo build -p jaic-cli --locked
```

The example above is included as `examples/sum.jai`. `jaic run` executes it in the interpreter, `check` only type-checks it:

```sh
cargo run -p jaic-cli -- check examples/sum.jai
cargo run -p jaic-cli -- run examples/sum.jai
```

To build and serve the standalone browser playground, follow the [browser playground docs](docs/browser/playground.md). Its compiler and language service run locally in WebAssembly.

The same lexer and parser power the `jai-lsp` language server (`cargo run -p jai-language-server --bin jai-lsp`); see [language-server.md](docs/compiler/language-server.md).

## Compatibility and performance

The local Jai distribution helps establish language behavior. Newer, maintained Jai projects guide compatibility when they differ from that older reference. The [upstream corpus](docs/tools/upstream-corpus.md) records the projects and exact revisions used.

Tests cover rejected programs and the behavior of newly compiled programs. Unsupported features produce errors instead of counting as successful builds.

The compiler loads an independently authored prelude from `prelude/` and standard library from `stdlib/`; supplied source distributions remain external compatibility inputs.

## Working on the compiler


```sh
cargo test --workspace --locked --no-fail-fast
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
python3 tools/jaic-sweep.py corpus negative stdlib modules upstream howto --timeout 900   # expect only the negative control to fail
```

The workspace is `crates/jaic` (lexer, parser, semantic analysis, IR, interpreter), `crates/jaic-cli` (the `jaic` binary), `crates/jaic-llvm` (native backend), `crates/jai-language-server` and `crates/jai-wasm` (browser build). Start with the [developer guide](docs/README.md) and [compiler architecture](docs/compiler/architecture.md). Rustfmt keeps code formatting consistent, and Cargo enforces a minimum dependency release age of 14 days.
