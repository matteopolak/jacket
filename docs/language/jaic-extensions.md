# jaic extensions (`Jaic_Extensions`)

## What it is

`stdlib/Jaic_Extensions/module.jai` holds features only `jaic` has. They are not official Jai, no other compiler knows them, and they may change between jaic releases. A program sees none of them unless it writes `#import "Jaic_Extensions";` {#ext.1}, so code that doesn't opt in stays portable.

The one feature so far is `Long_Double`: C's `long double` in the target's format, so jaic programs and `Bindings_Generator` output can call C functions that take or return one {#ext.2}.

```jai
#import "Basic";
#import "Jaic_Extensions";

libm :: #system_library "libm";
sqrtl :: (x: Long_Double) -> Long_Double #foreign libm;

main :: () {
    one: Long_Double = 1;
    x := one / 3;                                    // full precision on x86-64 / arm64 Linux
    print("% %\n", sqrtl(x), LONG_DOUBLE_IS_WIDE);  // printed through float64
}
```

## How it works

### The opt-in mechanism

The module names compiler builtins with the `#jaic_type name` directive (`Long_Double :: #jaic_type long_double;`). `parser/directive.rs` parses it like any directive with an identifier operand, and `jaic_type` in `sema/expr.rs` resolves it; unknown names fail with `unknown jaic extension type 'x'` {#ext.3}. Without the import, `Long_Double` is just an unknown identifier. New extensions should follow the same pattern: reachable only through this module, and documented on this page.

### `Long_Double` per target

Chosen by `long_double_for(os, cpu, windows_gnu)` in `sema/mod.rs` and stored in `Options.long_double` (the CLI sets it from `-target`/`-os`):

| Target | C `long double` | `Long_Double` |
|---|---|---|
| x86-64 Linux, macOS (Intel), MinGW | x87 80-bit extended, 16 bytes, 16-aligned | wide, `x86_fp80` in LLVM |
| arm64 Linux, wasm32 | IEEE binary128, 16 bytes, 16-aligned | wide, `fp128` in LLVM |
| Apple arm64, Windows x64 MSVC, Windows arm64 (MSVC and MinGW) | same as `double` | `float64` itself |

The table holds for each target {#ext.4}.

`LONG_DOUBLE_IS_WIDE` is `Long_Double != float64`. On the last row `Long_Double` is literally `float64` {#ext.5}, so everything below about the wide type does not apply.

The wide type is `TypeKind::WideFloat(WideFloat::{X87, Binary128})` (`types.rs`), named `Long_Double`. Its `type_info` is `Type_Info_Float` with `runtime_size` 16, so `Any`, `print`, `type_of` and reflection see a float {#ext.6}.

### Semantics of the wide type

- Operators: `+ - * /`, unary `-`, and `== != < <= > >=` (IEEE: NaN is unordered) at full precision {#ext.7}. `%`, bit operations and math intrinsics are errors; `Math` procedures take `float64`, so cast first {#ext.8}.
- Conversions: explicit casts to and from every integer and float type (integers go through `s64`/`u64`). Implicit conversions follow float64's rules so code stays portable to targets where it *is* float64: `float32`/`float64`, integers up to 32 bits and untyped literals convert implicitly, `s64`/`u64` and narrowing to `float64` need a cast {#ext.9}.
- Mixed arithmetic: `Long_Double + float64` is a `Long_Double` {#ext.10}.
- Out-of-range conversion to `s64` (undefined in C): x87 gives `-9223372036854775808` like the `fistp` instruction, binary128 saturates. Both match native builds on that target {#ext.11}.
- Literals: a decimal literal is a `float64` value first, so `cast(Long_Double) 0.1` holds float64's 0.1. Build exact values with arithmetic: `cast(Long_Double) 1 / 10` folds at full precision {#ext.12}.
- Constant folding (`wide_fold`, `sema/wide.rs`) uses the same soft-float as the interpreter, in the target's format {#ext.13}.
- `print` converts to `float64` (`__long_double_to_float64` in `stdlib/Basic/Print.jai`, which decodes the target's bit pattern in Jai), so it shows at most float64's digits, and values outside float64's range print as `inf` or `0` {#ext.14}. Read the bytes for exact output. `Reflection.set_value_from_string` does not parse it {#ext.15}.

### Compiler and backends

The IR has no wide scalar register: like a small struct, a `Long_Double` lives in memory (16 bytes, 16-aligned) and IR values are its address. Every operation is one `Intrinsic::Wide(WideOp, WideFloat)` (`ir.rs`) taking addresses (`Arith`, `Neg`, `Cmp`, `FromF64/F32/S64/U64`, `ToF64/F32/S64/U64`). `sema/wide.rs` emits them, `sema/convert.rs` routes casts and implicit costs there.

- LLVM (`crates/jaic-llvm/src/lower.rs`, `wide`): loads `x86_fp80`/`fp128`, uses native `fadd`, `fcmp`, `fpext`, `fptosi`... and stores the result.
- Interpreter (`interp/mod.rs`): soft-float in `crates/jaic/src/wide_float.rs`, with round-to-nearest-even, subnormals, infinities and NaN for both formats. It matches x87 hardware bit for bit; the `c_long_double` native test compares a batch of operations against an x86-64 build run under Rosetta {#ext.16}.
- C ABI: see [C ABI](../native/c-abi.md) (`Ty::F80`/`Ty::F128`, `PieceTy::X87`/`PieceTy::F128`).

### Limits

- Passing a wide `Long_Double` to a C variadic procedure (`printf("%Lf", x)`) is a compile error; cast to `float64` and use `%f`/`%g` {#ext.17}. Variadic `long double` is not implemented in either backend.
- `jaic run`: a C library cannot call a Jai `#c_call` procedure that passes a wide `Long_Double` (the callback thunks have no x87/q-register path); native builds can. Foreign calls *to* C work in the interpreter on x86-64 and arm64 hosts.
- The interpreter's foreign calls need the host to match the target (an arm64 Linux host for binary128, an x86-64 host for x87); `jaic run -target x86_64-...` on an arm64 Mac still computes in soft-float, but cannot call x86-64 C code.

### Outside this module

Two jaic-only additions live elsewhere because they belong to native builds, not to the language a program sees:

- `#intrinsic "llvm.<name>"` calls an LLVM intrinsic directly ([intrinsics](intrinsics.md)).
- `stdlib/Wasi_Runtime`, the WASI runtime that `jaic build -os wasm` adds to a program ([wasm target](../native/wasm-target.md)). A program never imports it itself.

## How to change it

- Another extension: add it to `stdlib/Jaic_Extensions/module.jai`, add a name to `jaic_type` in `sema/expr.rs` (or another directive), document it here.
- Long_Double format for a target: `long_double_for` in `sema/mod.rs`, plus the C ABI rules in `abi.rs`. Check against `clang -target <triple> -S -emit-llvm`.
- New operation: add a `WideOp`, implement it in `wide_float.rs`, `interp/mod.rs` (`wide`) and `jaic-llvm` (`wide`), and emit it from `sema/wide.rs`.
- Gotcha: `ir_ty` returns `None` for the wide type (it is memory-class); `materialize` and `convert_const` encode constants into 16 bytes with `wide_const`.

## Configuration

- `-target <triple>` / `-os` pick the format (`Options.long_double`).
- `Generate_Bindings_Options.use_jaic_long_double` (default `true`) makes `Bindings_Generator` emit `Long_Double`; see [Bindings_Generator](../specs/stdlib/README.md).

## Dependencies

- `crates/jaic/src/wide_float.rs`, `sema/wide.rs`, `sema/expr.rs`, `sema/convert.rs`, `ir.rs`, `abi.rs`, `interp/native/wide.rs`, `crates/jaic-llvm/src/lower.rs`.
- `stdlib/Basic/Print.jai` for printing.
- Tests: `tests/stdlib/jaic-extensions-long-double.jai` (also run with four `-target`s by `long_double_extension_on_wide_targets` in `crates/jaic-cli/tests/cli.rs`), `tests/native/c-long-double/` (`c_long_double` in `crates/jaic-cli/tests/native.rs`), `tests/stdlib/bindings-generator-long-double.jai`.
