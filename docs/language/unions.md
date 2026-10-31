# Unions and tagged unions

Plain `union` types have every member at offset 0 {#union.1}. Tagged unions (`union kind: Enum { ... }`) are laid out as structs.

```jai
U :: union { i: s32; f: float; b: [4] u8; }
uu: U; uu.f = 1.0;      // uu.i == 1065353216, size_of(U) == 4

Kind :: enum u8 { INT; TEXT; PAIR; }
Value :: union kind: Kind {
    .INT  ,, int_value: s64;
    .TEXT ,, text: string;
    .PAIR ,, pair: struct { a: s32; b: s32; };
}
v: Value = .{ kind = .INT, int_value = 42 };   // size_of(Value) == 24
```

The first program prints `1065353216 4` {#union.4}.

## How it works

- A plain union has the size of its largest member {#union.2} and the alignment of its most aligned one {#union.3}.
- A tagged union is a struct: the tag field, then an anonymous union of the variants {#union.5}, all reachable directly {#union.6}. Each variant is `.TAG ,, member: Type;` {#union.7}.
- The tag may have a default, `union kind: Kind = .TEXT { ... }` or `union kind := Kind.TEXT { ... }` (the tag type then comes from the default) {#union.8}. A new value starts with that tag {#union.9}.
- Any constant of the tag's type can select a variant, not only enum members: `4,, a: u8;` with an `s8` tag, `u16,, a: s8;` with a `Type` tag {#union.13}. A member without one is part of the union but has no binding {#union.14}.

## Reflection

`type_info(Value)` has `.UNION` and `.UNION_IS_TAGGED` in `textual_flags` {#union.10}. `members` lists the tag, then each variant {#union.11}. `tagged_union_bindings` maps each tag value (`constant_value`) to a `member_index` {#union.12}.

Anonymous unions inside structs are covered in [structs](structs.md).

## How to change it

Layout is the `is_union` branch of `layout_struct_inner` in `sema/structs.rs`. `parser/aggregate.rs` parses the tag into `lit.tag` (a tagged union is not `is_union`). Reflection is built by `tagged_union_members` and `set_tagged_union_bindings` in `sema/typeinfo.rs`. Test: `tests/compiler/regressions/tagged-union-layout.jai`.
