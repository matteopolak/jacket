# Enums

## What it is

`enum` and `enum_flags` declarations, their backing integer types, loose enums, and the reflection data they expose. The type is `EnumInfo` in `crates/jaic/src/types.rs`; `new_enum_type` and `enum_items` in `sema/structs.rs` declare it.

## How it works

Members count up from zero; `NAME :: value;` sets a value and numbering continues after it. The default backing type is 8 bytes; `enum u8` picks another.

```jai
Color :: enum { RED; GREEN :: 5; BLUE; }    // 0, 5, 6
Small :: enum u8 { A; B; C; }
Flags :: enum_flags u32 { READ; WRITE; EXEC; }
print("% % %\n", Color.GREEN, cast(int) Color.GREEN, Color.BLUE);   // GREEN 5 BLUE
print("% % %\n", size_of(Small), size_of(Color), size_of(Flags));   // 1 8 4
print("%\n", Flags.READ | Flags.EXEC);                              // READ | EXEC
```

- A value with no matching name prints as a number (`cast(Color) 99` prints `99`).
- `enum_flags` members are powers of two and combine with `|` and `&`; `fl & .WRITE` works as an `if` condition.
- `Color.loose` (`Types::loose_enum`) converts implicitly to and from integers and `Color` (`implicit_cost` in `sema/convert.rs`).
- `.FIRST == x` works: `check_binary` checks the right side first, so the inferred member takes its type.
- A polymorphic struct's enum-typed parameter types `.MEMBER` arguments and defaults (`Ticket(.SECOND)`, `struct(_order: Stuff = .FIRST)`) via `poly_struct_param_type`.
- `#insert` inside the enum body can generate members.

Reflection: `type_info(Color)` has `names`, `values` and `enum_type_flags` (`.FLAGS` for `enum_flags`); `enum_highest_value(Color)` works. `Color.names` is not a thing; use `type_info`.

## How to change it

Member evaluation is in `enum_items`; inserted members go through `eval_insert_enum_items`. Reflection data is built by `build_type_info` in `sema/typeinfo.rs`. Tests: `tests/stdlib/enum-insert-members.jai`, `baked-enum-default.jai`, `compiler-enum-external-type.jai`.

## Dependencies

`types.rs`, `sema/consteval.rs` (member values), `sema/typeinfo.rs` (reflection).
