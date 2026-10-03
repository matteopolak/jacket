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
| Debugging | Line tables, locals, records and recursive pointers tested |
| Compiler bootstrap | Independently authored source prelude type-checks; runtime integration in progress |
| Independent standard library | Authored modules included; full compilation and behavior testing pending |
| Reference examples | Source checks in progress; complete coverage pending |
| Focus, Jails, jaison, and other recent projects | Source checks only; full builds pending |
| Windows, mobile, WebAssembly | C ABI and object tests; runtime compatibility unverified |

The compatibility corpus contains **702 local reference files** and **1,440 files from seven recent upstream projects**. The latest recorded compiler parses **1,784 files** and checks **101 support files** with the included Preload. A separate feature snapshot passes all **36 positive contracts** through native execution, with **10 expected rejections** also passing. Full library and project builds remain pending. The table describes tested features in the working tree; [compatibility coverage](docs/reference-compatibility.md) explains the remaining work.

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

An independently authored standard library is included in [`stdlib/`](stdlib/). Its [coverage report](docs/stdlib/api-coverage.md) records implemented APIs, source checks, and the remaining work.

## Try it

You need [Rustup](https://rustup.rs/), Python 3.11 or newer, and an independently installed LLVM 22 with Clang. Rustup uses this repository's pinned toolchain automatically.

On macOS, install LLVM with Homebrew and set `LLVM_SYS_221_PREFIX` to its installation directory. See the [LLVM setup guide](docs/llvm-backend.md) for exact commands and troubleshooting.

```sh
git clone https://github.com/matteopolak/jai.git
cd jai

# Check dependency release dates before building.
python3 tools/check_dependency_age.py
cargo build -p jai-cli --locked
```

The example above is included as `examples/sum.jai`:

```sh
cargo run -p jai-cli -- check examples/sum.jai
cargo run -p jai-cli -- build examples/sum.jai sum
./sum
echo $? # 45
```

You can inspect tokens with `lex`, check one file's syntax with `parse`, or save its LLVM output with `emit-llvm`:

```sh
cargo run -p jai-cli -- emit-llvm examples/sum.jai sum.ll
```

## Compatibility and performance

The local Jai distribution helps establish language behavior. Newer, maintained Jai projects guide compatibility when they differ from that older reference. The [upstream corpus](docs/upstream-corpus.md) records the projects and exact revisions used.

Tests cover rejected programs and the behavior of newly compiled programs. Benchmarks cover compiler stages, module discovery, interpreter execution, and memory use; broader native workloads are being verified. Unsupported features produce errors instead of counting as successful builds.


## Working on the compiler

For a public checkout:

```sh
cargo test --workspace --locked -- \
  --skip lex_entire_reference_without_executing_it
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo bench -p jai-bench --bench compiler --locked -- --test --skip reference_lex
cargo bench -p jai-bench --bench vm --locked -- --test
```

The skipped lexer test requires the separately supplied reference files. Native-path rejection tests run in public checkouts too. Local reference checks run without the lexer filter.

Start with the [developer guide](docs/README.md) for architecture, language coverage, benchmarks, and dependency policy. Rustfmt keeps code formatting consistent, and Cargo enforces a minimum dependency release age of 14 days.
