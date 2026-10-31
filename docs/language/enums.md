# Enums

`enum` and `enum_flags` declarations, backing types, loose enums and their reflection.

```jai
Color :: enum { RED; GREEN :: 5; BLUE; }    // 0, 5, 6
Small :: enum u8 { A; B; C; }
Flags :: enum_flags u32 { READ; WRITE; EXEC; }
print("% % %\n", Color.GREEN, cast(int) Color.GREEN, Color.BLUE);   // GREEN 5 BLUE
print("% %\n", size_of(Small), size_of(Color));                     // 1 8
print("%\n", Flags.READ | Flags.EXEC);                              // READ | EXEC
```

## How it works

- Members count up from zero {#enum.1}; `NAME :: value;` sets a value and numbering continues after it {#enum.2}.
- The default backing type is 8 bytes {#enum.3}; `enum u8` picks another {#enum.4}.
- A member prints by name, `cast(int)` gives its value, and an `enum_flags` combination prints its names joined by ` | ` {#enum.5}.
- A value with no name prints as `(enum out of range: 99)` {#enum.6}. A flags value lists named bits, then leftover bits in hex (`X | Z | 0xf8`); with no named bit it is the hex number, and zero prints `0`.
- `enum_flags` members are powers of two and combine with `|` and `&` {#enum.7}; `fl & .WRITE` works as an `if` condition {#enum.8}.
- `Color.loose` converts implicitly to and from integers and `Color` {#enum.9}.
- `.FIRST == x` works: the right side is checked first, so the inferred member takes its type {#enum.14}.
- A polymorphic struct's enum-typed parameter types `.MEMBER` arguments and defaults (`Ticket(.SECOND)`) {#enum.15}.
- Names are unique and values must fit the backing type. Values may refer to later members; they resolve by dependency, while numbering and reflection keep declaration order.
- A value may name the enum itself, even while it is still being resolved (`IPV6 :: LAST + cast(Sc) 50`) {#enum.16}.
- A value may follow `::`, `=` or `: :` (`INVERT : :5;`) {#enum.17}.
- `#insert` in the body can generate members {#enum.13}.
- `using Other;` in the body makes another enum's names reachable through this one without making them members. `using,only(...)` and `using,except(...)` (names or a computed array of strings) select which; `using,map` is rejected.

## Reflection

`type_info(Color)` has `names` and `values` {#enum.10}, and `enum_type_flags`: `.FLAGS` for `enum_flags`, `.COMPLETE` for `#complete`, `.SPECIFIED` for `#specified` {#enum.11}. `enum_highest_value(Color)` works {#enum.12}. `Color.names` doesn't exist; use `type_info`.

## How to change it

Collection and lazy value resolution are in `sema/enums.rs`; reflection in `build_type_info` (`sema/typeinfo.rs`); member values via `sema/consteval.rs`. Dependency-order cases are in `crates/jaic/tests/enum_dependencies.rs`.
