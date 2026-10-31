# Type values, identity and Type_Info

Types are compile-time values (`Type`). `type_info(T)` returns a pointer to a read-only `Type_Info_*` descriptor. The descriptor structs are plain Jai in `stdlib/prelude/reflection.jai`; the compiler builds one initialised global per type on demand, and a `Type` value at run time is that descriptor's address.

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

## How it works

Every type is a `TypeId` into `Types` (`crates/jaic/src/types.rs`), and `==` on types compares ids. `Types::intern` dedupes structural kinds; nominal kinds get a fresh id per declaration; parameterized structs are cached per argument values.

- `type_info_global` (`sema/typeinfo.rs`) memoises one `ir::Global` per type, named `type_info.<type>`. `type_info_struct_type` picks the descriptor struct and `build_type_info` fills fields by name with `set_field`.
- A pointer's `pointer_to` descriptor for a struct not laid out yet is reserved and filled when the layout finishes (`type_info_global_later`), so describing `*Tree` never needs `Tree`'s size. That is what lets `Tree :: Box(Branch); Branch :: struct { c: *Tree; }` resolve. A case that needs the struct by value is still `contains itself`.
- Struct descriptors have `members` (`name`, `offset_in_bytes`, `type`, `flags` like `USING`, `AS`, `OVERLAY`, `notes`), `textual_flags`, `polymorph_source_struct` and `specified_parameters`. See [unions](../language/unions.md) and [enums](../language/enums.md).
- Metaprograms see a target workspace's types through exported records, not these globals.

## How to change it

- New descriptor field: add it to `stdlib/prelude/reflection.jai` (field order is the ABI) and set it by name in `build_type_info`.
- New descriptor kind: a tag in `Type_Info_Tag` (numbers in `typeinfo.rs` must match), a struct embedding `using #as info: Type_Info`, and an arm in `type_info_struct_type`.
- New type kind: extend `TypeKind` and `size_of`/`align_of`/`name` in `types.rs`, then emit its info.
- Descriptors are lazy, so `type_info` of a type forces its layout.

Tests: `tests/compiler/regressions/compiler-reflection-pure.jai`, `crates/jaic/tests/recursive_variadic_struct.rs`, `type-field-constant.jai`, `poly-struct-type-names.jai`.

## Rules

Claims that tests cite by ID (`tools/dev/check_rule_coverage.py`).

### Type identity

- `Types::intern` dedupes structural kinds (`Int`, `Float`, `Pointer`, `Array`, `Proc`, ...), so `*Derived` and `type_of(*d)` share an id {#typeval.1}. Nominal kinds (`Struct`, `Enum`, `Distinct`) get a fresh id per declaration {#typeval.2}. Parameterized structs are cached per argument values, so `Gen(int) == Gen(s64)` {#typeval.3}.
- An alias equals its target {#typeval.4}. A `#type,distinct` type differs from its base but has its size {#typeval.5}. A `#type,isa` value converts implicitly to its base {#typeval.6}.
- `Type` values can be stored, compared, printed and put in arrays (`tt: [2] Type = .[int, float]` prints `s64 float32`) {#typeval.7}. A variable holding a type cannot be a declaration's type: `t := Inner; v: t;` fails with `type must be known at compile time` {#typeval.8}.
- `size_of(Type)` is 8 and `size_of(Any)` is 16 {#typeval.9}. `Any.type` is a `*Type_Info`, so compare with `type_info(int)`, not `int` {#typeval.10}.
- `type_info(T)` returns the matching `Type_Info_*` with `.type` {#typeval.11}, `.name` {#typeval.12}, `.members` {#typeval.13} and the struct's own `notes` {#typeval.14}.

### Descriptors

- `type_info(T)` returns a pointer to a read-only `Type_Info_*` descriptor {#reflect.1}. The compiler builds one initialised global per type on demand, and a `Type` at run time is that descriptor's address {#reflect.2}.
- `type_info_struct_type` picks `Type_Info_Integer`, `_Float`, `_String`, `_Pointer`, `_Procedure`, `_Struct`, `_Array`, `_Enum`, `_Variant` (distinct types), else plain `Type_Info` {#reflect.3}.
- Every descriptor starts with the `Type_Info` header (`type`, `runtime_size`) embedded with `using #as`, so a `*Type_Info_Struct` converts to `*Type_Info` {#reflect.4}.
- The struct line {#reflect.5} and the pointer tag {#reflect.6} in the example show the commented values.
- Struct descriptors carry `members` {#reflect.7}, `textual_flags`, `polymorph_source_struct` and `specified_parameters` for polymorphic instances {#reflect.8}. Tagged unions are flattened: the tag, then each variant at its offset, with `UNION | UNION_IS_TAGGED` and `tagged_union_bindings` {#reflect.9}.
- `get_runtime_info()` returns the `__runtime_info` global, whose `type_table: [] *Type_Info` lists every descriptor {#reflect.10}, filled once every global and descriptor exists {#reflect.11}. At compile time `get_type_table()` is a snapshot of declared types instead ([Compiler module](compiler-module.md)).
