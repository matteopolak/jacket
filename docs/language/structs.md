# Structs and aggregate literals

## What it is

Struct declarations, defaults, literals, parameterized structs and layout. A struct is a `StructInfo` in `crates/jaic/src/types.rs`; declaration and layout are in `sema/structs.rs`.

## How it works

Member defaults apply to `v: Vec;` {#struct.1} and to members a literal omits {#struct.2}:

```jai
Vec :: struct { x: float = 1; y: float = 2; z: float = 3; }
w := Vec.{x=9};      // {9, 2, 3}
u := Vec.{4,5,6};    // positional, declaration order
z := Vec.{};         // all defaults
```

Positional literals fill members in declaration order {#struct.3} and `Vec.{}` is all defaults {#struct.4}.

Where a value is expected and its type is known, the dot may be left out: `gpu_init(1, {.GENERAL, 1})`, `e: Extent = {1280, 720};`, `return {w, h};`, `e = {width = 5};` and fields of another literal all build a struct like `.{...}` {#struct.17}. A `{` that starts with a statement keyword or directive, declares a name, holds a `;` or is a lambda's body (`x => { ... }`) is a block. Third-party Jai code relies on the dotless form (`UnNabbo/no_api`'s examples and README, and the `jai_parser` used by the Jails language server parses it in arguments, returns, declarations and operands).

`check_struct_literal` and `literal_target` handle literals; `default_initializer` and `init_default` build default values.

### Layout

`layout_struct` runs lazily, the first time a size or offset is needed (`LayoutState`). Every top-level struct is still laid out before compilation finishes, so a member of an undefined type is always an error {#struct.5} (see [sema: polymorphism and declarations](../compiler/sema-polymorphism-and-declarations.md)). Recursion is caught by `LayoutState::InProgress`.

Fields are naturally aligned and the size rounds up to the struct alignment {#struct.6}; `#no_padding` packs members with no gaps {#struct.7} and `#align N` raises the struct's alignment {#struct.8}:

```jai
Mix :: struct { a: u8; b: s64; c: u16; }          // size_of == 24
NoPad :: struct #no_padding { a: u8; b: s64; }    // size_of == 9
Al :: struct #align 16 { a: u8; b: s32; }         // size_of == 16
```

`#place a; b: int;` overlays `b` on `a`'s offset {#struct.9}. Members of an anonymous `union { }` or `struct { }` inside a struct are reachable directly {#struct.10}; with `using c:` they are also reachable through `c` {#struct.11}:

```jai
Rgba :: struct { union { using c: struct { r, g, b, a: u8; }; v: [4] u8; packed: u32; } }
c.packed = 0x04030201;   // c.r == 1, c.a == 4, c.v[3] == 4, size_of(Rgba) == 4
```

The members share storage as the comment shows {#struct.12}.

`offset_of` is not supported; use `type_info(T).members[i].offset_in_bytes`.

### Parameterized structs

`struct(T: Type, N: int)` is instantiated by `instantiate_struct` and cached per argument values (`PolyStruct::instances`), so `Gen(int)` and `Gen(s64)` are the same type {#struct.13}. Names print with parameter names: `Pair(int, 3)` shows as `Pair(T=s64, N=3)` {#struct.14}.

A struct body can also hold constants (`K :: 3;`, read as `Plain.K`) {#struct.15} and default overrides like `kind = .B;` (see [using](using.md)) {#struct.16}.

## How to change it

New struct flags: parse them in `parser/aggregate.rs` next to `#no_padding` and `#align`, and apply them in `layout_struct_inner`. Tests: `tests/stdlib/poly-struct-type-names.jai`, `baked-struct-restriction.jai`, `struct-body-member-path-override.jai`.

## Dependencies

`types.rs` (`Types`, `StructInfo`, `size_of`, `align_of`), `sema/value.rs` (aggregate constants), `parser/aggregate.rs`.
