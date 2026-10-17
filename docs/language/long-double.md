# `Long_Double` (jaic extension)

## What it is

`stdlib/Extensions/Long_Double/module.jai` gives Jai C's `long double` in the target's format, so programs and `Bindings_Generator` output can call C functions that take or return one. Like every [`stdlib/Extensions/`](../extensions/README.md) module it is not official Jai and may change between releases; it is visible only after `#import "Extensions/Long_Double";`, so other code stays portable.

```jai
#import "Basic";
#import "Extensions/Long_Double";

libm :: #system_library "libm";
sqrtl :: (x: Long_Double) -> Long_Double #foreign libm;

main :: () {
    one: Long_Double = 1;
    x := one / 3;                                    // full precision on x86-64 / arm64 Linux
    print("% %\n", sqrtl(x), LONG_DOUBLE_IS_WIDE);  // printed through float64
}
```

## How it works

The module names the builtin with `Long_Double :: #jaic_type long_double;`. `jaic_type` in `sema/expr.rs` resolves it (unknown names fail with ``unknown jaic extension type``); without the import `Long_Double` is an unknown identifier. A new compiler-backed extension follows the same pattern.

The format is chosen by `long_double_for(os, cpu, windows_gnu)` in `sema/mod.rs` and stored in `Options.long_double`:

| Target | C `long double` | `Long_Double` |
|---|---|---|
| x86-64 Linux, macOS (Intel), MinGW | x87 80-bit, 16 bytes | wide, `x86_fp80` |
| arm64 Linux, wasm32 | IEEE binary128 | wide, `fp128` |
| Apple arm64, Windows x64 MSVC, Windows arm64 | same as `double` | `float64` itself |

`LONG_DOUBLE_IS_WIDE` is `Long_Double != float64`. The wide type is `TypeKind::WideFloat(WideFloat::{X87, Binary128})` (`types.rs`); its `type_info` is a `Type_Info_Float` with `runtime_size` 16.

- Operators: `+ - * /`, unary `-` and comparisons at full precision. `%`, bit operations and math intrinsics are errors; cast to `float64` for `Math`.
- Casts to and from every integer and float type work. Implicit conversion follows float64's rules (floats, integers up to 32 bits, untyped literals); `s64`/`u64` and narrowing need a cast. `Long_Double + float64` is `Long_Double`.
- Literals are `float64` first (`cast(Long_Double) 0.1` is float64's 0.1); `cast(Long_Double) 1 / 10` folds at full precision (`wide_fold`, `sema/wide.rs`, using the interpreter's soft-float).
- `print` converts to `float64` (`stdlib/internal/basic/format-float.jai`), so it shows float64 digits and out-of-range values as `Inf` or `0`. `set_value_from_string` does not parse it.
- Out-of-range conversion to `s64`: x87 gives the minimum like `fistp`, binary128 saturates, matching native builds.

### Compiler and backends

The IR has no wide scalar register: a `Long_Double` lives in 16 aligned bytes and IR values are its address. Each operation is one `Intrinsic::Wide(WideOp, WideFloat)` (`ir.rs`: `Arith`, `Neg`, `Cmp`, `From*`, `To*`), emitted by `sema/wide.rs`; `sema/convert.rs` routes casts there. LLVM (`jaic-llvm/src/lower.rs`, `wide`) uses `x86_fp80`/`fp128` natively. The interpreter uses `crates/jaic/src/wide_float.rs` (round-to-nearest-even, subnormals, infinities, NaN) and matches x87 hardware bit for bit. C ABI: [C ABI](../native/c-abi.md).

Limits: a wide value can be passed to C variadics (`printf("%Lf", x)`). Under `jaic -interp`, C code cannot call a Jai `#c_call` procedure taking a wide `Long_Double`, and foreign calls need a host matching the target (arm64 Linux for binary128, x86-64 for x87).

## How to change it

- Another `#jaic_type` builtin: add a name in `jaic_type` (`sema/expr.rs`) and export it from `stdlib/Extensions/<Name>/module.jai`.
- Another target's format: `long_double_for` plus `abi.rs`; compare with `clang -target <triple> -S -emit-llvm`.
- New operation: add a `WideOp`, implement it in `wide_float.rs`, `interp/mod.rs` and `jaic-llvm`, emit it from `sema/wide.rs`. `ir_ty` returns `None` for the wide type (memory class); `materialize` and `convert_const` encode constants with `wide_const`.

Configuration: `-target` / `-os`; `Generate_Bindings_Options.use_jaic_long_double` (default `true`, see [Bindings_Generator](../specs/stdlib/modules/bindings-generator.md)). Tests: `tests/stdlib/extensions-long-double.jai` (run with four `-target`s by `long_double_extension_on_wide_targets` in `crates/jaic-cli/tests/cli.rs`), `tests/native/c-long-double/`.

## Rules

Claims that tests cite by ID (`tools/check_rule_coverage.py`).

- `stdlib/Extensions/Long_Double/module.jai` gives Jai C's `long double` in the target's format, so jaic programs and `Bindings_Generator` output can call C functions that take or return one {#ext.2}. Like every module under [`stdlib/Extensions/`](../extensions/README.md) it is not official Jai: no other compiler knows it, and it may change between jaic releases. A program sees none of it unless it writes `#import "Extensions/Long_Double";` {#ext.1}, so code that doesn't opt in stays portable.
- The module names compiler builtins with the `#jaic_type name` directive (`Long_Double :: #jaic_type long_double;`). `parser/directive.rs` parses it like any directive with an identifier operand, and `jaic_type` in `sema/expr.rs` resolves it; unknown names fail with `` unknown jaic extension type `x` `` {#ext.3}. Without the import, `Long_Double` is just an unknown identifier. A new compiler-backed extension follows the same pattern: a builtin named only by a module of its own under `stdlib/Extensions/`, documented like this page.
- The table holds for each target {#ext.4}.
- `LONG_DOUBLE_IS_WIDE` is `Long_Double != float64`. On the last row `Long_Double` is literally `float64` {#ext.5}, so everything below about the wide type does not apply.
- The wide type is `TypeKind::WideFloat(WideFloat::{X87, Binary128})` (`types.rs`), named `Long_Double`. Its `type_info` is `Type_Info_Float` with `runtime_size` 16, so `Any`, `print`, `type_of` and reflection see a float {#ext.6}.
- Operators: `+ - * /`, unary `-`, and `== != < <= > >=` (IEEE: NaN is unordered) at full precision {#ext.7}. `%`, bit operations and math intrinsics are errors; `Math` procedures take `float64`, so cast first {#ext.8}.
- Conversions: explicit casts to and from every integer and float type (integers go through `s64`/`u64`). Implicit conversions follow float64's rules so code stays portable to targets where it *is* float64: `float32`/`float64`, integers up to 32 bits and untyped literals convert implicitly, `s64`/`u64` and narrowing to `float64` need a cast {#ext.9}.
- Mixed arithmetic: `Long_Double + float64` is a `Long_Double` {#ext.10}.
- Out-of-range conversion to `s64` (undefined in C): x87 gives `-9223372036854775808` like the `fistp` instruction, binary128 saturates. Both match native builds on that target {#ext.11}.
- Literals: a decimal literal is a `float64` value first, so `cast(Long_Double) 0.1` holds float64's 0.1. Build exact values with arithmetic: `cast(Long_Double) 1 / 10` folds at full precision {#ext.12}.
- Constant folding (`wide_fold`, `sema/wide.rs`) uses the same soft-float as the interpreter, in the target's format {#ext.13}.
- `print` converts to `float64` (`__long_double_to_float64` in `stdlib/Basic/Print.jai`, which decodes the target's bit pattern in Jai), so it shows at most float64's digits, and values outside float64's range print as `Inf` or `0` {#ext.14}. Read the bytes for exact output. `Reflection.set_value_from_string` does not parse it {#ext.15}.
- Interpreter (`interp/mod.rs`): soft-float in `crates/jaic/src/wide_float.rs`, with round-to-nearest-even, subnormals, infinities and NaN for both formats. It matches x87 hardware bit for bit; the `c_long_double` native test compares a batch of operations against an x86-64 build run under Rosetta {#ext.16}.
- A wide `Long_Double` can be passed to a C variadic procedure (`printf("%Lf", x)`) {#ext.17}: the call site gives the extra argument a layout in `Sig.c_abi.params` after the declared ones, so both backends pass it exactly as they pass a fixed `long double` (x87 memory slot on x86-64, a q register on arm64 Linux). A memory-class result's out-pointer is now the last *declared* IR parameter (`Sig::out_index`), before the variadic arguments.
