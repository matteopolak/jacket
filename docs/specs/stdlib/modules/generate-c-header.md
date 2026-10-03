# generate_c_header

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `Header_Info :: struct`
- `handle_message :: (using info: *Header_Info, message: *Message)`
- `generate_header :: (using info: *Header_Info, path: string) -> bool`

## Public type members

#### `Header_Info` (struct)

- `prefix_enum_values:   bool`
- `enum_internal_types:  bool`
- `jai_type_prefix:      string`
- `exported_procedures:  [..] *Code_Declaration`
- `exported_types:       Table(*Type_Info, string)`
- `name_types:           Table(string, *Type_Info)`
- `ordered_types:        [..] *Type_Info`
- `type_resizable_array: *Type_Info`
- `type_array_view_8:    *Type_Info`
- `type_array_view_16:   *Type_Info`
- `type_array_view_32:   *Type_Info`
- `type_array_view_64:   *Type_Info`
- `type_type_info:       *Type_Info`
- `type_newstring:       *Type_Info`
- `type_any:             *Type_Info`

## Behavioral contract and constraints

This metaprogram emits a C header for exported Jai declarations. It deliberately excludes `main`. The output is a C ABI projection, so only types and signatures supported by the generator can be represented; unsupported declarations or ABI mismatches should be treated as generation errors. The generated header does not guarantee ownership rules for pointers.

## Generation behavior and ABI limits

`generate_header(info, path)` writes the generated text to `path` and returns the file-write result. `handle_message` collects exported procedure declarations on the compiler's typechecked message and also records the compiler type descriptors for support types named `Resizable_Array`, `Array_View_8/16/32/64`, `Type_Info`, `Newstring`, and `Any_Struct`. Those support types are compiler/runtime dependencies; `Header_Info` does not define their field layouts itself. Its fields have no explicit initializers; a zero-initialized value therefore starts with both Boolean options false, an empty prefix, empty collections, and null support descriptors. The module provides no dedicated reset/initialization procedure. When a generated signature uses a Jai `string`, `Any`, `Type`, array view, or resizable array, the generator includes the corresponding runtime type descriptor and emits its non-imported, non-constant, non-void members as an ordinary C record. Array views and resizable arrays are projected using the discovered 64-bit view and generic resizable-array descriptors, respectively; this is not specialized per element type.

`jai_type_prefix` is prepended to emitted type tags, typedef aliases, and runtime support type names. It does not prefix ordinary function names, struct field names, or enum value names. With `prefix_enum_values` enabled, each enum member receives a prefix formed from the uppercase letters of the emitted enum type name, followed by `_`; otherwise member names are emitted as declared, subject to the reserved-name mapping. Name clashes among projected types receive numeric suffixes.

`enum_internal_types` defaults to false under zero initialization. When false, an enum is emitted as a normal C enum; its C implementation chooses the enum representation, which need not match the Jai underlying integer width or signedness. When true, the output uses C23 fixed-underlying-type enum syntax, naming the Jai-width integer alias (`s8/u8` through `s64/u64`). Consumers requiring exact Jai enum ABI must use the fixed-underlying mode and a C compiler supporting that syntax. The generated preamble includes `<stdint.h>`, fixed-width integer aliases, C/C++ linkage guards, and a C-only `bool` alias plus `false`/`true` constants. This indicates intended use from C and C++, but the module does not promise compatibility with every language-standard/compiler combination.

Only procedures marked with the C calling convention are emitted. A procedure with no return values is emitted as returning `void`; a single return type is emitted directly. Multiple return values are not supported: the source asserts that at most one exists rather than packaging them into a C result record. Procedure-valued types are projected as `void *`, and fixed arrays, primitive integers/floats, pointers, enums, and structs receive direct declarators. Polymorphic structs are omitted; imported, constant, and void-valued struct members are omitted. These limitations are observable generator constraints, not permission to silently change an ABI. The module does not emit C++ wrapper overloads or make non-C-call procedures callable from C.
