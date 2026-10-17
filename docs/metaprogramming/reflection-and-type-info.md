# Type values, identity and Type_Info

## What it is

Types as compile-time values (`Type`), how jaic decides two types are the same, and `type_info(T)`, which returns a pointer to a read-only `Type_Info_*` descriptor. The descriptor structs are plain Jai in `stdlib/prelude/reflection.jai`; the compiler builds one initialised global per type on demand, and a `Type` value at run time is that descriptor's address.

## How it works

Every type is a `TypeId` into `Types` (`crates/jaic/src/types.rs`), and `==` on types compares ids. `Types::intern` dedupes structural kinds (`Int`, `Pointer`, `Array`, `Proc`, ...); nominal kinds (`Struct`, `Enum`, `Distinct`) get a fresh id per declaration; parameterized structs are cached per argument values (`Gen(int) == Gen(s64)`).

```jai
Alias :: Inner;
Id :: #type,distinct int;
Meters :: #type,isa float;
Alias == Inner          // true
Id == int               // false: distinct is a new nominal type, same size
m: Meters = 2.5; f: float = m;   // isa converts implicitly to its base

Vec :: struct { x: float; y: float; tag: string; }
ti := cast(*Type_Info_Struct) type_info(Vec);
// name=Vec size=24, members x@0 FLOAT, y@4 FLOAT, tag@8 STRING
type_info(*int).type    // POINTER
```

- `Type` values can be stored, compared, printed and put in arrays. A variable holding a type cannot be a declaration's type (`t := Inner; v: t;` fails with `type must be known at compile time`). `size_of(Type)` is 8, `size_of(Any)` is 16, and `Any.type` is a `*Type_Info`.
- `type_info_global` (`sema/typeinfo.rs`) memoises one `ir::Global` named `type_info.<type>`. `type_info_struct_type` picks `Type_Info_Integer`, `_Float`, `_String`, `_Pointer`, `_Procedure`, `_Struct`, `_Array`, `_Enum`, `_Variant` (distinct types) or plain `Type_Info`; `build_type_info` fills fields by name with `set_field`. Every descriptor embeds the `Type_Info` header with `using #as`.
- Struct descriptors have `members` (`name`, `offset_in_bytes`, `type`, `flags` such as `USING`, `AS`, `OVERLAY`, `notes`), `textual_flags`, `polymorph_source_struct` and `specified_parameters`. Tagged unions are flattened ([unions](../language/unions.md)); enums are in [enums](../language/enums.md).
- `get_runtime_info()` returns the `__runtime_info` global whose `type_table` lists every descriptor, filled once all descriptors exist. At compile time `get_type_table()` is a snapshot of declared types ([Compiler module](compiler-module.md)). Metaprograms see a target workspace's types through exported records, not these globals.

## How to change it

- New descriptor field: add it to `stdlib/prelude/reflection.jai` (field order is the ABI) and set it by name in `build_type_info`.
- New descriptor kind: a tag in `Type_Info_Tag` (numbers in `tag`, `typeinfo.rs`, must match), a struct embedding `using #as info: Type_Info`, and an arm in `type_info_struct_type`.
- New type kind: extend `TypeKind` and `size_of`/`align_of`/`name` in `types.rs`, then emit its info.
- Descriptors are lazy, so `type_info` of a type forces its layout.

Tests: `tests/stdlib/compiler-reflection-pure.jai`, `type-field-constant.jai`, `poly-struct-type-names.jai`. Related: `intern.rs`, `sema/structs.rs`.

## Rules

Claims that tests cite by ID (`tools/check_rule_coverage.py`).

- Every type is a `TypeId` into the `Types` table, and `==` on types compares ids. `Types::intern` dedupes structural kinds (`Int`, `Float`, `Pointer`, `Array`, `Proc`, ...), so `*Derived` and `type_of(*d)` share an id {#typeval.1}. Nominal kinds (`Struct`, `Enum`, `Distinct`) get a fresh id per declaration {#typeval.2}. Parameterized structs are cached per argument values, so `Gen(int) == Gen(s64)` {#typeval.3}.
- An alias equals its target {#typeval.4}; a `#type,distinct` type differs from its base but has its size {#typeval.5}; a `#type,isa` value converts implicitly to its base {#typeval.6}.
- `Type` values can be stored, compared, printed and put in arrays (`tt: [2] Type = .[int, float]` prints `s64 float32`) {#typeval.7}. A variable holding a type cannot be a declaration's type: `t := Inner; v: t;` fails with `type must be known at compile time` {#typeval.8}.
- `size_of(Type)` is 8 and `size_of(Any)` is 16 {#typeval.9}. `Any.type` is a `*Type_Info`, so compare it with `type_info(int)`, not `int` {#typeval.10}.
- `type_info(T)` returns the matching `Type_Info_*`: `.type` {#typeval.11}, `.name` {#typeval.12}, `.members` (with `name`, `offset_in_bytes`, `type`, `flags` such as `USING`, `AS` and `OVERLAY`, and `notes`) {#typeval.13}, the struct's own `notes` {#typeval.14}, and the fields described in [enums](../language/enums.md) and [unions](../language/unions.md). See [reflection and Type_Info](../metaprogramming/reflection-and-type-info.md) for the runtime side.
- `type_info(T)` returns a pointer to a read-only `Type_Info_*` descriptor for `T` {#reflect.1}. The descriptor structs are plain Jai in `stdlib/prelude/module.jai`. The compiler builds one initialised global per type on demand, and a `Type` value at run time is that descriptor's address {#reflect.2}.
- `BuiltinProc::TypeInfo` in `sema/calls.rs` calls `type_info_global` (`sema/typeinfo.rs`), which memoises one `ir::Global` per `TypeId`, named `type_info.<type>`. `type_info_struct_type` picks the Preload struct by `TypeKind`: `Type_Info_Integer`, `_Float`, `_String`, `_Pointer`, `_Procedure`, `_Struct`, `_Array`, `_Enum`, `_Variant` (distinct types), else plain `Type_Info` {#reflect.3}. `build_type_info` fills fields by name through `set_field`, emitting arrays and strings as views over extra globals. `type_from_info_global` maps a descriptor back to its type.
- Every descriptor starts with the `Type_Info` header (`type: Type_Info_Tag`, `runtime_size`) embedded with `using #as`, so a `*Type_Info_Struct` converts to `*Type_Info` {#reflect.4}.
- The struct line {#reflect.5} and the pointer tag {#reflect.6} show the commented values.
- Struct descriptors carry `members` (name, `offset_in_bytes`, `type`) {#reflect.7}, `textual_flags`, `polymorph_source_struct` and, for polymorphic instances, `specified_parameters` (`poly_struct_info`) {#reflect.8}. Tagged unions are flattened: the tag, then each variant at its offset, with `UNION | UNION_IS_TAGGED` and `tagged_union_bindings` {#reflect.9}.
- `get_runtime_info()` returns the `__runtime_info` global, whose `type_table: [] *Type_Info` lists every descriptor {#reflect.10}; it is filled once every global and descriptor exists {#reflect.11}. At compile time `get_type_table()` is a snapshot of every declared type instead (see [the Compiler module](compiler-module.md)). See [compile-time values](compile-time-execution.md).
