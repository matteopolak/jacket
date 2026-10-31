# `Long_Double` (jaic extension)

`stdlib/Extensions/Long_Double/module.jai` gives Jai C's `long double` in the target's format, so programs and `Bindings_Generator` output can call C functions that take or return one {#ext.2}. It is visible only after `#import "Extensions/Long_Double";` {#ext.1}, so code that doesn't opt in stays portable. See [extensions](../extensions/README.md).

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

The module names the builtin with `Long_Double :: #jaic_type long_double;`. Unknown names fail with `` unknown jaic extension type `x` `` {#ext.3}; without the import, `Long_Double` is just an unknown identifier. A new compiler-backed extension follows the same pattern.

The format depends on the target {#ext.4}:

| Target | C `long double` | `Long_Double` |
|---|---|---|
| x86-64 Linux, macOS (Intel), MinGW | x87 80-bit, 16 bytes | wide, `x86_fp80` |
| arm64 Linux, wasm32 | IEEE binary128 | wide, `fp128` |
| Apple arm64, Windows x64 MSVC, Windows arm64 | same as `double` | `float64` itself |

- `LONG_DOUBLE_IS_WIDE` is `Long_Double != float64`. On the last row `Long_Double` is literally `float64` {#ext.5}, so the rest of this list doesn't apply.
- The wide type's `type_info` is a `Type_Info_Float` with `runtime_size` 16, so `Any`, `print`, `type_of` and reflection see a float {#ext.6}.
- Operators: `+ - * /`, unary `-`, and `== != < <= > >=` (IEEE: NaN is unordered) at full precision {#ext.7}. `%`, bit operations and math intrinsics are errors; `Math` procedures take `float64`, so cast first {#ext.8}.
- Casts work to and from every integer and float type. Implicit conversion follows float64's rules: floats, integers up to 32 bits and untyped literals convert; `s64`/`u64` and narrowing need a cast {#ext.9}. `Long_Double + float64` is a `Long_Double` {#ext.10}.
- Out-of-range conversion to `s64`: x87 gives the minimum like `fistp`, binary128 saturates, matching native builds {#ext.11}.
- Decimal literals are `float64` first, so `cast(Long_Double) 0.1` holds float64's 0.1. `cast(Long_Double) 1 / 10` folds at full precision {#ext.12}, using the same soft-float as the interpreter {#ext.13}.
- `print` converts to `float64`, so it shows at most float64's digits and out-of-range values as `Inf` or `0` {#ext.14}. Read the bytes for exact output. `set_value_from_string` doesn't parse it {#ext.15}.
- The interpreter's soft-float (round-to-nearest-even, subnormals, infinities, NaN) matches x87 hardware bit for bit {#ext.16}.
- A wide value can be passed to a C variadic (`printf("%Lf", x)`) {#ext.17}. Under `jaic -interp`, C code can't call a Jai `#c_call` procedure taking one, and foreign calls need a host matching the target.

## How to change it

- The IR has no wide scalar register: a value lives in 16 aligned bytes and each operation is one `Intrinsic::Wide(WideOp, WideFloat)`, emitted by `sema/wide.rs` and lowered in `jaic-llvm`. The interpreter uses `crates/jaic/src/wide_float.rs`.
- Another `#jaic_type` builtin: add a name in `jaic_type` (`sema/expr.rs`) and export it from `stdlib/Extensions/<Name>/module.jai`.
- Another target's format: `long_double_for` in `sema/mod.rs` plus `abi.rs`; compare with `clang -target <triple> -S -emit-llvm`.
- A new operation: add a `WideOp`, implement it in `wide_float.rs`, the interpreter and `jaic-llvm`, emit it from `sema/wide.rs`.

Config: `-target`/`-os`; `Generate_Bindings_Options.use_jaic_long_double` (default `true`). See also the [C ABI](../native/c-abi.md).
