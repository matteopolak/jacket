# Enums

## What it is

`enum` and `enum_flags` declarations, backing types, loose enums and their reflection. `EnumInfo` is in `crates/jaic/src/types.rs`; collection and lazy value resolution are in `sema/enums.rs`.

## How it works

```jai
Color :: enum { RED; GREEN :: 5; BLUE; }    // 0, 5, 6
Small :: enum u8 { A; B; C; }
Flags :: enum_flags u32 { READ; WRITE; EXEC; }
print("% % %\n", Color.GREEN, cast(int) Color.GREEN, Color.BLUE);   // GREEN 5 BLUE
print("% %\n", size_of(Small), size_of(Color));                     // 1 8
print("%\n", Flags.READ | Flags.EXEC);                              // READ | EXEC
```

- The default backing type is 8 bytes. A value with no name prints as `(enum out of range: 99)`; an `enum_flags` value prints its named bits, then leftover bits in hex (`X | Z | 0xf8`).
- Names are unique and values must fit the backing type. Explicit values may refer to later members; values resolve by dependency, while numbering and reflection keep declaration order. A value may name the enum itself (`new_enum_type` binds it).
- `enum_flags` combine with `|` and `&`; `fl & .WRITE` works as a condition.
- `Color.loose` converts implicitly to and from integers (`implicit_cost`, `sema/convert.rs`).
- `.FIRST == x` works because `check_binary` checks the right side first. Polymorphic struct parameters of enum type type `.MEMBER` arguments (`poly_struct_param_type`).
- `#insert` in the body can generate members (`eval_insert_enum_items`).
- Reflection: `type_info(Color)` has `names`, `values`, `enum_type_flags` (`.FLAGS`, `.COMPLETE`, `.SPECIFIED`); `enum_highest_value(Color)` works. `Color.names` does not exist.

## How to change it

`sema/enums.rs`; reflection in `build_type_info` (`sema/typeinfo.rs`); member values via `sema/consteval.rs`. Regression: `crates/jaic/tests/enum_dependencies.rs` with `tests/compiler/semantics/enum-dependencies.jai`.

## Rules

Claims that tests cite by ID (`tools/check_rule_coverage.py`).

- Members count up from zero {#enum.1}; `NAME :: value;` sets a value and numbering continues after it {#enum.2}. The default backing type is 8 bytes {#enum.3}; `enum u8` picks another {#enum.4}.
- A member prints by name, `cast(int)` gives its value, and an `enum_flags` combination prints as its member names joined by ` | ` {#enum.5}.
- A value with no matching name prints as `(enum out of range: 99)` for `cast(Color) 99` {#enum.6}. An `enum_flags` value lists its named bits and then any leftover bits in hex (`X | Z | 0xf8`); with no named bit it is the hex number alone, and zero prints `0`.
- `enum_flags` members are powers of two and combine with `|` and `&` {#enum.7}; `fl & .WRITE` works as an `if` condition {#enum.8}.
- `Color.loose` (`Types::loose_enum`) converts implicitly to and from integers and `Color` (`implicit_cost` in `sema/convert.rs`) {#enum.9}.
- `.FIRST == x` works: `check_binary` checks the right side first, so the inferred member takes its type {#enum.14}.
- A polymorphic struct's enum-typed parameter types `.MEMBER` arguments and defaults (`Ticket(.SECOND)`, `struct(_order: Stuff = .FIRST)`) via `poly_struct_param_type` {#enum.15}.
- `#insert` inside the enum body can generate members {#enum.13}.
- A member's value may name the enum itself, whose declaration is still being resolved (`IPV6 :: LAST + cast(Sc) 50`): `new_enum_type` binds the enum's name to its type in the members' scope {#enum.16}.
- A member's value may follow `::`, `=` or `: :` (the constant form with its empty type written out, `INVERT : :5;`) {#enum.17}.
- Reflection: `type_info(Color)` has `names` and `values` {#enum.10}, and `enum_type_flags` (`.FLAGS` for `enum_flags`, `.COMPLETE` for `#complete`, `.SPECIFIED` for `#specified`) {#enum.11}; `enum_highest_value(Color)` works {#enum.12}. `Color.names` is not a thing; use `type_info`.
