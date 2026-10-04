# Unions and tagged unions

## What it is

Plain `union` types (all members share offset 0, `StructInfo::is_union` in `crates/jaic/src/types.rs`) and tagged unions (`union kind: Enum { ... }`), which are laid out as structs.

## How it works

A plain union has the size of its largest member and the alignment of its most aligned one:

```jai
U :: union { i: s32; f: float; b: [4] u8; }
uu: U; uu.f = 1.0;
print("% %\n", uu.i, size_of(U));   // 1065353216 4
```

A tagged union is an ordinary struct: the tag field first, then an anonymous union of the variants, all reachable directly. Each variant is introduced by `.TAG ,, member: Type;`:

```jai
Kind :: enum u8 { INT; TEXT; PAIR; }
Value :: union kind: Kind {
    .INT  ,, int_value: s64;
    .TEXT ,, text: string;
    .PAIR ,, pair: struct { a: s32; b: s32; };
}
v: Value = .{ kind = .INT, int_value = 42 };   // size_of(Value) == 24
```

`type_info(Value)` has `textual_flags` with `.UNION` and `.UNION_IS_TAGGED`; `members` lists the tag, then each variant (`text` at offset 8), and `tagged_union_bindings` maps each tag value (`constant_value`) to its `member_index`. This is built by `tagged_union_members` and `set_tagged_union_bindings` in `crates/jaic/src/sema/typeinfo.rs`. The full checked example is `tests/stdlib/tagged-union-layout.jai`.

Anonymous `union { ... }` blocks inside a struct work like inline unions; see [structs.md](structs.md).

## How to change it

Union layout is the `is_union` branch of `layout_struct_inner` in `sema/structs.rs`; the tag is parsed into `lit.tag` by `parser/aggregate.rs`, and a union with a tag is not `is_union`. Tests that print union pointers: `tests/stdlib/print-null-pointer-union-forms.jai`.

## Configuration

None.

## Dependencies

`sema/structs.rs`, `sema/typeinfo.rs`, `parser/aggregate.rs`, and the `Type_Info_Struct` definitions in the bundled prelude.
