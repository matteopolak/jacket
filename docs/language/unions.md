# Unions and tagged unions

## What it is

Plain `union` types, where all members share offset 0 {#union.1} (`StructInfo::is_union`), and tagged unions (`union kind: Enum { ... }`), which are laid out as structs.

## How it works

A plain union has the size of its largest member {#union.2} and the alignment of its most aligned one {#union.3}:

```jai
U :: union { i: s32; f: float; b: [4] u8; }
uu: U; uu.f = 1.0;
print("% %\n", uu.i, size_of(U));   // 1065353216 4
```

The program prints `1065353216 4` {#union.4}.

A tagged union is a struct: the tag field, then an anonymous union of the variants {#union.5}, all reachable directly {#union.6}. Each variant is `.TAG ,, member: Type;` {#union.7}:

```jai
Kind :: enum u8 { INT; TEXT; PAIR; }
Value :: union kind: Kind {
    .INT  ,, int_value: s64;
    .TEXT ,, text: string;
    .PAIR ,, pair: struct { a: s32; b: s32; };
}
v: Value = .{ kind = .INT, int_value = 42 };   // size_of(Value) == 24
```

The tag may have a default, `union kind: Kind = .TEXT { ... }` or `union kind := Kind.TEXT { ... }` (the tag type then comes from the default) {#union.8}; a value of the union starts with that tag {#union.9}.

`type_info(Value)` has `.UNION` and `.UNION_IS_TAGGED` in `textual_flags` {#union.10}; `members` lists the tag and then each variant {#union.11}, and `tagged_union_bindings` maps each tag value (`constant_value`) to a `member_index` {#union.12}. `tagged_union_members` and `set_tagged_union_bindings` in `sema/typeinfo.rs` build these.

A tag need not be an enum member: any constant of the tag's type selects a variant, such as `4,, a: u8;` with an `s8` tag or `u16,, a: s8;` with a `Type` tag {#union.13}. A member without one is part of the union but has no binding {#union.14}.

Anonymous `union { }` blocks inside structs are covered in [structs](structs.md).

## How to change it

Layout is the `is_union` branch of `layout_struct_inner` in `sema/structs.rs`. `parser/aggregate.rs` parses the tag into `lit.tag`; a union with a tag is not `is_union`. Tests: `tests/stdlib/tagged-union-layout.jai`, `print-null-pointer-union-forms.jai`.

## Dependencies

`sema/structs.rs`, `sema/typeinfo.rs`, `parser/aggregate.rs`, and `Type_Info_Struct` in the prelude.
