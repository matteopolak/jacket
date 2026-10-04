# Structs and aggregate literals

## What it is

Struct declarations, defaults, literals, parameterized structs and layout rules as implemented by `jaic`. A struct is a `StructInfo` in `crates/jaic/src/types.rs`; declaration and layout live in `crates/jaic/src/sema/structs.rs`.

## How it works

Member defaults apply to `v: Vec;` and to omitted literal members. Literals can be named, positional or empty:

```jai
Vec :: struct { x: float = 1; y: float = 2; z: float = 3; }
w := Vec.{x=9};      // {9, 2, 3}
u := Vec.{4,5,6};    // positional, declaration order
z := Vec.{};         // all defaults: {1, 2, 3}
```

`check_struct_literal` / `literal_target` handle both forms; `default_initializer` and `init_default` build the zero/default value.

Layout (`layout_struct`) is lazy: a struct is laid out the first time its size or a member offset is needed (`LayoutState`). Fields are aligned naturally and the size is rounded up to the struct alignment:

```jai
Mix :: struct { a: u8; b: s64; c: u16; }          // size_of == 24
NoPad :: struct #no_padding { a: u8; b: s64; }    // size_of == 9
Al :: struct #align 16 { a: u8; b: s32; }         // size_of == 16
```

`#place a; b: int;` makes `b` overlay the offset of `a`. An anonymous `union { ... }` or `struct { ... }` inside a struct exposes its members directly (`c.r`, `c.packed` below); members of an anonymous `using c: struct {...}` are reachable through `c` and directly.

```jai
Rgba :: struct { union { using c: struct { r, g, b, a: u8; }; v: [4] u8; packed: u32; } }
c.packed = 0x04030201;   // c.r == 1, c.a == 4, c.v[3] == 4, size_of(Rgba) == 4
```

Parameterized structs (`struct(T: Type, N: int)`) are instantiated by `instantiate_struct`, cached per argument values (`PolyStruct::instances`), so `Gen(int)` and `Gen(s64)` are the same type. Type names print with parameter names: `Pair(int, 3)` shows as `Pair(T=s64, N=3)`.

A struct body may also hold constants (`K :: 3;`, readable as `Plain.K`) and member-override statements such as `kind = .B;` (see [using.md](using.md)).

## How to change it

Add new struct flags in `parser/aggregate.rs` (where `#no_padding` and `#align` are parsed into the literal flags) and apply them in `layout_struct_inner`. `offset_of` is not supported; use `type_info(T).members[i].offset_in_bytes`. Recursive layout is detected through `LayoutState::InProgress`. Regression programs: `tests/stdlib/poly-struct-type-names.jai`, `tests/stdlib/baked-struct-restriction.jai`, `tests/stdlib/struct-body-member-path-override.jai`.

## Configuration

None.

## Dependencies

`types.rs` (`Types`, `StructInfo`, `size_of`/`align_of`), `sema/value.rs` (aggregate constants), `parser/aggregate.rs`.
