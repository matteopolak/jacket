# Type values, identity and type info

## What it is

Types as first-class compile-time values (`Type`), how `jaic` decides two types are the same, and the `type_info` reflection structs. Core definitions are in `crates/jaic/src/types.rs`; reflection data is built in `crates/jaic/src/sema/typeinfo.rs`.

## How it works

Every type is a `TypeId` into the `Types` table. `Types::intern` dedupes structural kinds (`TypeKind`: `Int`, `Float`, `Pointer`, `Array`, `Proc`, ...), so `*Derived` and `type_of(*d)` are the same id and comparison `==` on types is id equality. Nominal kinds (`Struct`, `Enum`, `Distinct`) get a fresh id per declaration; parameterized structs are cached per argument values, so `Gen(int) == Gen(s64)` is true.

```jai
Alias :: Inner;
Id :: #type,distinct int;
Meters :: #type,isa float;
Alias == Inner          // true
Id == int               // false: distinct is a new nominal type
size_of(Id)             // 8
m: Meters = 2.5; f: float = m;   // #type,isa converts implicitly to its base
type_of(d) == Derived   // true
```

`Type` values can be stored, compared, printed and put in arrays (`tt: [2] Type = .[int, float]` prints `s64 float32`), but a variable holding a type cannot be used as a declaration type: `t := Inner; v: t;` is rejected with `type must be known at compile time`. `size_of(Type)` is 8 and `size_of(Any)` is 16. `Any.type` is a `*Type_Info`, so compare with `type_info(int)`, not `int`.

`type_info(T)` returns the matching `Type_Info_*` struct. Checked here: `.type` (`STRUCT`, `ARRAY`, `INTEGER`, `ENUM`), `.name`, `.members` with `name`, `offset_in_bytes`, `type` and `flags` (`USING`, `AS`), per-member `notes` and the struct's own `notes` (`S :: struct @thing { }`), plus the enum and tagged-union fields described in [enums.md](enums.md) and [unions.md](unions.md).

Sizes and alignments come from `Types::size_of` / `align_of`; struct layout is in [structs.md](structs.md).

## How to change it

To add a type kind, extend `TypeKind`, `size_of`/`align_of`/`name` in `types.rs`, then emit its info in `build_type_info`. Info globals are created once per type (`type_info_global`). Runtime pieces such as `Type_Info` struct definitions come from the bundled prelude; keep field names in sync with `type_info_struct_type`. Tests: `tests/stdlib/type-field-constant.jai`, `type-of-outer-local-expr.jai`, `poke-name-shared-type.jai`.

## Configuration

None.

## Dependencies

`types.rs`, `intern.rs` (symbols), `sema/typeinfo.rs`, `sema/runtime_info.rs`.
