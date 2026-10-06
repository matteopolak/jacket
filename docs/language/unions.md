# Unions and tagged unions

## What it is

Plain `union` types, where all members share offset 0 (`StructInfo::is_union`), and tagged unions (`union kind: Enum { ... }`), which are laid out as structs.

## How it works

A plain union has the size of its largest member and the alignment of its most aligned one:

```jai
U :: union { i: s32; f: float; b: [4] u8; }
uu: U; uu.f = 1.0;
print("% %\n", uu.i, size_of(U));   // 1065353216 4
```

A tagged union is a struct: the tag field, then an anonymous union of the variants, all reachable directly. Each variant is `.TAG ,, member: Type;`:

```jai
Kind :: enum u8 { INT; TEXT; PAIR; }
Value :: union kind: Kind {
    .INT  ,, int_value: s64;
    .TEXT ,, text: string;
    .PAIR ,, pair: struct { a: s32; b: s32; };
}
v: Value = .{ kind = .INT, int_value = 42 };   // size_of(Value) == 24
```

`type_info(Value)` has `.UNION` and `.UNION_IS_TAGGED` in `textual_flags`; `members` lists the tag and then each variant, and `tagged_union_bindings` maps each tag value (`constant_value`) to a `member_index`. `tagged_union_members` and `set_tagged_union_bindings` in `sema/typeinfo.rs` build these.

Anonymous `union { }` blocks inside structs are covered in [structs](structs.md).

## How to change it

Layout is the `is_union` branch of `layout_struct_inner` in `sema/structs.rs`. `parser/aggregate.rs` parses the tag into `lit.tag`; a union with a tag is not `is_union`. Tests: `tests/stdlib/tagged-union-layout.jai`, `print-null-pointer-union-forms.jai`.

## Dependencies

`sema/structs.rs`, `sema/typeinfo.rs`, `parser/aggregate.rs`, and `Type_Info_Struct` in the prelude.
