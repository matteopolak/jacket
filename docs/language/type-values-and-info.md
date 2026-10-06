# Type values, identity and type info

## What it is

Types as compile-time values (`Type`), how jaic decides two types are the same, and the `type_info` reflection structs. Core definitions are in `crates/jaic/src/types.rs`; reflection data is built in `sema/typeinfo.rs`.

## How it works

Every type is a `TypeId` into the `Types` table, and `==` on types compares ids. `Types::intern` dedupes structural kinds (`Int`, `Float`, `Pointer`, `Array`, `Proc`, ...), so `*Derived` and `type_of(*d)` share an id. Nominal kinds (`Struct`, `Enum`, `Distinct`) get a fresh id per declaration. Parameterized structs are cached per argument values, so `Gen(int) == Gen(s64)`.

```jai
Alias :: Inner;
Id :: #type,distinct int;
Meters :: #type,isa float;
Alias == Inner          // true
Id == int               // false: distinct is a new nominal type
m: Meters = 2.5; f: float = m;   // isa converts implicitly to its base
```

`Type` values can be stored, compared, printed and put in arrays (`tt: [2] Type = .[int, float]` prints `s64 float32`). A variable holding a type cannot be a declaration's type: `t := Inner; v: t;` fails with `type must be known at compile time`.

`size_of(Type)` is 8 and `size_of(Any)` is 16. `Any.type` is a `*Type_Info`, so compare it with `type_info(int)`, not `int`.

`type_info(T)` returns the matching `Type_Info_*`: `.type`, `.name`, `.members` (with `name`, `offset_in_bytes`, `type`, `flags` such as `USING` and `AS`, and `notes`), the struct's own `notes`, and the fields described in [enums](enums.md) and [unions](unions.md). See [reflection and Type_Info](../metaprogramming/reflection-and-type-info.md) for the runtime side.

## How to change it

To add a type kind, extend `TypeKind` and `size_of`/`align_of`/`name` in `types.rs`, then emit its info in `build_type_info`. Info globals are created once per type (`type_info_global`). The `Type_Info` struct definitions come from the prelude; keep their field names in sync with `type_info_struct_type`.

Tests: `tests/stdlib/type-field-constant.jai`, `type-of-outer-local-expr.jai`, `poke-name-shared-type.jai`.

## Dependencies

`types.rs`, `intern.rs`, `sema/typeinfo.rs`, `sema/runtime_info.rs`.
