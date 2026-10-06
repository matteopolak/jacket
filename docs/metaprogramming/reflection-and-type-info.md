# Reflection and Type_Info

## What it is

`type_info(T)` returns a pointer to a read-only `Type_Info_*` descriptor for `T`. The descriptor structs are plain Jai in `stdlib/prelude/module.jai`. The compiler builds one initialised global per type on demand, and a `Type` value at run time is that descriptor's address.

## How it works

`BuiltinProc::TypeInfo` in `sema/calls.rs` calls `type_info_global` (`sema/typeinfo.rs`), which memoises one `ir::Global` per `TypeId`, named `type_info.<type>`. `type_info_struct_type` picks the Preload struct by `TypeKind`: `Type_Info_Integer`, `_Float`, `_String`, `_Pointer`, `_Procedure`, `_Struct`, `_Array`, `_Enum`, `_Variant` (distinct types), else plain `Type_Info`. `build_type_info` fills fields by name through `set_field`, emitting arrays and strings as views over extra globals. `type_from_info_global` maps a descriptor back to its type.

Every descriptor starts with the `Type_Info` header (`type: Type_Info_Tag`, `runtime_size`) embedded with `using #as`, so a `*Type_Info_Struct` converts to `*Type_Info`.

```jai
Vec :: struct { x: float; y: float; tag: string; }
ti := cast(*Type_Info_Struct) type_info(Vec);
// name=Vec size=24, members x@0 FLOAT, y@4 FLOAT, tag@8 STRING
type_info(*int).type       // POINTER
```

Struct descriptors carry `members`, `textual_flags`, `polymorph_source_struct` and, for polymorphic instances, `specified_parameters` (`poly_struct_info`). Tagged unions are flattened: the tag, then each variant at its offset, with `UNION | UNION_IS_TAGGED` and `tagged_union_bindings`.

`get_runtime_info()` returns the `__runtime_info` global; see [compile-time values](compile-time-data-and-state.md#runtime-info).

Metaprograms see a target workspace's types through exported records, not these globals; see [compiler records](compiler-records.md).

## How to change it

- New descriptor field: add it to `stdlib/prelude/module.jai` (field order is the ABI), then set it by name in `build_type_info`.
- New descriptor kind: a tag in `Type_Info_Tag`, a struct embedding `using #as info: Type_Info`, and an arm in `type_info_struct_type`. The tag numbers in `tag` (`typeinfo.rs`) must equal `Type_Info_Tag`.
- Descriptors are built lazily, so `type_info` of a type forces its layout.

Tests: `tests/stdlib/compiler-reflection-pure.jai`, `compiler-enum-external-type.jai`, `poly-struct-type-names.jai`.

## Dependencies

`stdlib/prelude/module.jai` (layout), `sema/structs.rs` (offsets), `ir::Global` relocations between descriptors.
