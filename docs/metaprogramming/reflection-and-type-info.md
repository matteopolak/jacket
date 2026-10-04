# Reflection and Type_Info

## What it is

`type_info(T)` returns a pointer to a read-only `Type_Info_*` descriptor for `T`. The descriptor structs are plain Jai declared in `stdlib/prelude/module.jai`; the compiler builds one initialized global per type on demand, and a `Type` value at run time is the address of that descriptor.

## How it works

`BuiltinProc::TypeInfo` in `sema/calls.rs` calls `Compiler::type_info_global` (`sema/typeinfo.rs`), which memoizes one `ir::Global` per `TypeId` (named `type_info.<type>`). `type_info_struct_type` picks the Preload struct by `TypeKind`: `Type_Info_Integer`, `_Float`, `_String`, `_Pointer`, `_Procedure`, `_Struct`, `_Array`, `_Enum`, `_Variant` (distinct types), else plain `Type_Info`. `build_type_info` fills the fields by name through `set_field`, with arrays and strings emitted as views over extra globals. `type_from_info_global` maps a descriptor back to a type.

Every descriptor starts with the `Type_Info` header (`type: Type_Info_Tag`, `runtime_size`), embedded with `using #as`, so a `*Type_Info_Struct` converts to `*Type_Info`. The tag numbers in `typeinfo.rs::tag` must equal `Type_Info_Tag` in the prelude.

Verified behavior:

```jai
Color :: enum u8 { RED; GREEN; BLUE; }
Vec :: struct { x: float; y: float; tag: string; }
ti := cast(*Type_Info_Struct) type_info(Vec);
// tag=STRUCT name=Vec size=24 members=3
//   x offset=0 type=FLOAT / y offset=4 type=FLOAT / tag offset=8 type=STRING
e := cast(*Type_Info_Enum) type_info(Color);   // names RED GREEN BLUE, values 0 1 2
type_info(*int).type       // POINTER
type_info([4] int).type    // ARRAY
```

Struct descriptors carry `members` (name, `offset_in_bytes`, `type`), `textual_flags`, `polymorph_source_struct` and, for polymorphic instances, `specified_parameters` (`poly_struct_info`). Tagged unions are flattened: the tag member, then each variant at its offset, with `UNION | UNION_IS_TAGGED` and `tagged_union_bindings`.

`get_runtime_info()` (Compiler module) returns the `__runtime_info` global from `sema/runtime_info.rs`: `type_table: [] *Type_Info` plus `global_data_info` segments (data and rdata globals). It is created on first reference and filled by `fill_runtime_info` in `finish_program`, once every global and descriptor exists.

Metaprograms see the types of a *target* workspace through exported records instead of these globals; see [compiler-records.md](compiler-records.md).

## How to change it

- New field in a descriptor: add it to `stdlib/prelude/module.jai` (field order is the ABI), then set it in `build_type_info` by name. Keep the tag constants in sync.
- New descriptor kind: add a tag to `Type_Info_Tag`, a struct embedding `using #as info: Type_Info`, and a match arm in `type_info_struct_type`.
- Gotcha: descriptors are emitted lazily, so `type_info` of a type forces its layout.
- Regression programs: `tests/stdlib/compiler-reflection-pure.jai`, `compiler-enum-external-type.jai`, `poly-struct-type-names.jai`.

## Configuration

None.

## Dependencies

`stdlib/prelude/module.jai` (descriptor layout), `sema/structs.rs` (layout and offsets), `ir::Global` relocations for pointers between descriptors.
