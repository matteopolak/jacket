# Structs and aggregate literals

## What it is

Struct declarations, defaults, literals, parameterized structs and layout. A struct is a `StructInfo` in `crates/jaic/src/types.rs`; declaration and layout are in `sema/structs.rs`.

## How it works

Member defaults apply to `v: Vec;` and to members a literal omits:

```jai
Vec :: struct { x: float = 1; y: float = 2; z: float = 3; }
w := Vec.{x=9};      // {9, 2, 3}
u := Vec.{4,5,6};    // positional, declaration order
z := Vec.{};         // all defaults
```

`check_struct_literal` and `literal_target` handle literals; `default_initializer` and `init_default` build default values.

### Layout

`layout_struct` runs lazily, the first time a size or offset is needed (`LayoutState`). Every top-level struct is still laid out before compilation finishes, so a member of an undefined type is always an error (see [sema: polymorphism and declarations](../compiler/sema-polymorphism-and-declarations.md)). Recursion is caught by `LayoutState::InProgress`.

Fields are naturally aligned and the size rounds up to the struct alignment:

```jai
Mix :: struct { a: u8; b: s64; c: u16; }          // size_of == 24
NoPad :: struct #no_padding { a: u8; b: s64; }    // size_of == 9
Al :: struct #align 16 { a: u8; b: s32; }         // size_of == 16
```

`#place a; b: int;` overlays `b` on `a`'s offset. Members of an anonymous `union { }` or `struct { }` inside a struct are reachable directly:

```jai
Rgba :: struct { union { using c: struct { r, g, b, a: u8; }; v: [4] u8; packed: u32; } }
c.packed = 0x04030201;   // c.r == 1, c.a == 4, c.v[3] == 4, size_of(Rgba) == 4
```

`offset_of` is not supported; use `type_info(T).members[i].offset_in_bytes`.

### Parameterized structs

`struct(T: Type, N: int)` is instantiated by `instantiate_struct` and cached per argument values (`PolyStruct::instances`), so `Gen(int)` and `Gen(s64)` are the same type. Names print with parameter names: `Pair(T=s64, N=3)`.

A struct body can also hold constants (`K :: 3;`, read as `Plain.K`) and default overrides like `kind = .B;` (see [using](using.md)).

## How to change it

New struct flags: parse them in `parser/aggregate.rs` next to `#no_padding` and `#align`, and apply them in `layout_struct_inner`. Tests: `tests/stdlib/poly-struct-type-names.jai`, `baked-struct-restriction.jai`, `struct-body-member-path-override.jai`.

## Dependencies

`types.rs` (`Types`, `StructInfo`, `size_of`, `align_of`), `sema/value.rs` (aggregate constants), `parser/aggregate.rs`.
