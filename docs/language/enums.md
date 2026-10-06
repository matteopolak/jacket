# Enums

## What it is

`enum` and `enum_flags` declarations, their backing integer types, loose enums, and the reflection data they expose. The type is `EnumInfo` in `crates/jaic/src/types.rs`; `new_enum_type` and `enum_items` in `sema/structs.rs` declare it.

## How it works

Members count up from zero {#enum.1}; `NAME :: value;` sets a value and numbering continues after it {#enum.2}. The default backing type is 8 bytes {#enum.3}; `enum u8` picks another {#enum.4}.

```jai
Color :: enum { RED; GREEN :: 5; BLUE; }    // 0, 5, 6
Small :: enum u8 { A; B; C; }
Flags :: enum_flags u32 { READ; WRITE; EXEC; }
print("% % %\n", Color.GREEN, cast(int) Color.GREEN, Color.BLUE);   // GREEN 5 BLUE
print("% % %\n", size_of(Small), size_of(Color), size_of(Flags));   // 1 8 4
print("%\n", Flags.READ | Flags.EXEC);                              // READ | EXEC
```

A member prints by name, `cast(int)` gives its value, and an `enum_flags` combination prints as its member names joined by ` | ` {#enum.5}.

- A value with no matching name prints as a number (`cast(Color) 99` prints `99`) {#enum.6}.
- `enum_flags` members are powers of two and combine with `|` and `&` {#enum.7}; `fl & .WRITE` works as an `if` condition {#enum.8}.
- `Color.loose` (`Types::loose_enum`) converts implicitly to and from integers and `Color` (`implicit_cost` in `sema/convert.rs`) {#enum.9}.
- `.FIRST == x` works: `check_binary` checks the right side first, so the inferred member takes its type {#enum.14}.
- A polymorphic struct's enum-typed parameter types `.MEMBER` arguments and defaults (`Ticket(.SECOND)`, `struct(_order: Stuff = .FIRST)`) via `poly_struct_param_type` {#enum.15}.
- `#insert` inside the enum body can generate members {#enum.13}.

Reflection: `type_info(Color)` has `names` and `values` {#enum.10}, and `enum_type_flags` (`.FLAGS` for `enum_flags`, `.COMPLETE` for `#complete`, `.SPECIFIED` for `#specified`) {#enum.11}; `enum_highest_value(Color)` works {#enum.12}. `Color.names` is not a thing; use `type_info`.

## How to change it

Member evaluation is in `enum_items`; inserted members go through `eval_insert_enum_items`. Reflection data is built by `build_type_info` in `sema/typeinfo.rs`. Tests: `tests/stdlib/enum-insert-members.jai`, `baked-enum-default.jai`, `compiler-enum-external-type.jai`.

## Dependencies

`types.rs`, `sema/consteval.rs` (member values), `sema/typeinfo.rs` (reflection).
