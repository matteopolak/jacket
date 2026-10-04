# Jai, in Rust

[![Compiler checks](https://github.com/matteopolak/jai/actions/workflows/ci.yml/badge.svg)](https://github.com/matteopolak/jai/actions/workflows/ci.yml)

An independent Jai compiler written in Rust. The goal is to compile existing Jai programs and libraries, with a clean implementation that is easy to test, understand, and improve.

**In development:** the compiler runs a growing set of Jai programs, including records, generics and compile-time code. Full standard-library and larger-project builds are still pending.

```jai
sum_to :: (n: int) -> int {
    total := 0;
    while n > 0 {
        total += n;
        n -= 1;
    }
    return total;
}

main :: () -> int {
    return sum_to(9); // Exit status: 45
}
```

## What works today

| Area | Status |
| --- | --- |
| Numbers and Booleans | Integer widths, floating point, casts and comparisons tested |
| Strings, arrays and pointers | VM and native tests; descriptor copies, iteration and pointer operations |
| Records, unions and enums | Nominal types, defaults, copies, packed layouts and static pointer data tested |
| Procedures | Named/default arguments, multiple results, callbacks and recursion tested |
| Generics and overloads | Procedure and record specialization tested; remaining forms need coverage |
| Context and `Any` | Context overrides, variadic forwarding and value boxing tested |
| Control flow | Conditions, cases, loops, named exits and lexical `defer` tested |
| Modules | Loads, scoped imports, visibility and scalar/enum/type parameters tested; advanced parameters in progress |
| Compile-time code | Source `#run`, reflection and code insertion tested; broader metaprogramming in progress |
| Compiler APIs | Workspace scheduling and selected CLI artifacts tested; broader API integration in progress |
| Foreign calls and native builds | LLVM library backend; host ABI tests and selected system-library calls |
| Independent standard library | Authored modules in `stdlib/`, regression-tested by `tests/stdlib/` |
| Reference examples | Source checks in progress; complete coverage pending |
| Focus, Jails, jaison, sgpu, Vk-Engine | Build natively or check; see `HANDOFF.md` for per-project status |
| Browser editor | Real Wasm runs, file tree, source editor and shared source LSP tested; host services and full standard library pending |
| Windows and mobile | C ABI and object tests; runtime compatibility unverified |

The compatibility corpus is the set of real-world Jai projects pinned in `corpus/upstreams.json` (see [upstream corpus](docs/tools/upstream-corpus.md)); `HANDOFF.md` records which of them currently check and build.

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
