# Core syntax, declarations, types and values

Jai is statically typed, with declaration-driven inference, value aggregates, explicit casts, and compile-time type values. This page specifies the value-level language.

## Declarations and scope

The declaration shape is `name: Type = value;`. Omitting the type infers it from the initializer; omitting the initializer produces the type's default value. A second colon denotes an immutable binding, commonly used for procedures, types, and constants. `:=` is a concise inferred mutable declaration.

```jai
limit :: 12;
current := limit;
next: s32 = current + 1;
zeroed: [3] u16;
```

A declaration needs enough information to determine a type: either a type or an initializer. Data scopes (file, enum and struct declaration scopes) are distinct from imperative procedure scopes, and runtime statements belong in procedures. A nested procedure does not close over runtime locals; constants and declarations from data scopes can still be referenced. Braced runtime blocks create nested scopes. `#if` is a special case: its selected branch contributes declarations to the surrounding scope rather than introducing a normal block scope.

Declaration-order and name-shadowing corner cases are unspecified.

## Primitive types and literals

The primitive types are `bool`, signed `s8/s16/s32/s64`, unsigned `u8/u16/u32/u64`, `float32`, and `float64`, plus the aliases `int` for `s64` and `float` for `float32`. Integer literals default to `s64`.

Decimal floating literal precision follows a lexer heuristic, not a test that the value round-trips exactly through `float32`:

- a normal binary exponent outside -126 through 127 selects `float64`;
- otherwise, more than eight significant fractional digits require `float64`;
- exactly eight counted fractional digits default to `float64` when the expression has no stronger contextual type;
- fewer than eight default to `float32`.

Leading integer digits do not count toward this fractional-digit threshold, trailing fractional zeroes are ignored, and so are the leading fractional zeroes of a literal below one (`0.000000001` counts one digit and defaults to `float32`). Thus `12342345234.0`, `16777216.0`, and `9876543210.125` default to `float32`, while `0.123456789` requires `float64`. Context may select a type for a literal that merely defaults to `float64`, but a literal that requires `float64` cannot implicitly narrow to `float32`; use an explicit cast to permit rounding. A numeric literal fits implicitly into a requested numeric type when its precision permits it and the value is in range. A mutable variable's declared type does not shrink merely because its current value would fit.

Integer syntax includes decimal, `0x` hexadecimal, `0b` binary, and underscores for visual grouping. The `0h` form expresses IEEE bit patterns for floating values. `type_of` yields a compile-time type value, and `size_of` queries a type's storage size.

Implicit conversions are deliberately constrained: integer and float variables do not receive C-style general arithmetic conversions. A conversion widens implicitly only when the destination can represent the entire source type range, not merely its current value, so signed-to-unsigned and unsigned-to-signed conversions depend on representable ranges. Explicit `cast(T)` permits a conversion; checked narrowing can fail at runtime, while `cast,no_check(T)` suppresses that range check. Literal fit is a separate compile-time convenience.

For prefix casts combined with shifts and other bitwise operators, those operators are part of the cast operand. The cast does not change the type used to type an already typed operand expression: `cast(u32) byte << 2` groups as `cast(u32) (byte << 2)`. Write `(cast(u32) byte) << 2` to make the shift's left operand `u32`. See the [operator precedence](../../language/operators.md) and [cast](../../language/casts-and-conversions.md) pages. Whether a shift on a narrow typed operand promotes its result or keeps that operand's width is unspecified, so the numeric result of the first form is unspecified. Do not substitute C precedence or integer-promotion rules for this grouping.

Numeric-to-bool conversion is false for zero and true for nonzero, and `if` uses the same truth rule. Negative floating zero is false; a nonzero bit pattern is true even if ordinary printing rounds it to zero. Pointers and strings are also valid conditions. Truth conversion for other types is unspecified.

## Arrays and strings

`[N] T` is a fixed-size, inline array whose elements are contiguous and homogeneous. Its length is part of the type. An array view `[] T` carries a count and data pointer and does not own the storage. Fixed arrays convert to views for calls and assignments. Indexing and iteration are bounds checked when bounds checks are enabled; statically known invalid indices can be diagnosed during compilation. A caller can change a view's count or data pointer, so its validity depends on the backing storage and correct metadata.

```jai
samples: [4] s16 = .[2, 4, 8, 16];
window: [] s16 = samples;
window.count = 2;
for value, index: window {
    samples[index] = value + 1;
}
```

Resizable arrays are written `[..] T`. Array literals are `.[...]`, element types can be inferred, and empty literals take their type from context. `..` forms range expressions. A resizable array is not a fixed array or view: its storage can move when it grows, invalidating saved element pointers or views. Range endpoints are covered on the control-flow page.

`string` is a byte-oriented view with a count and data pointer, and string literals provide convenient values. Ordinary strings are distinct from C-style zero-terminated data. Strings support formatting, concatenation, comparison, and indexing. A string view does not imply ownership or lifetime extension. Unicode normalization and general text-encoding semantics are not part of the language; string operations are byte-oriented.

For a named fixed-array constant, the declaration fixes its element values and array type. It is unspecified whether repeated `.data` evaluations, explicit address-taking, or conversions to views yield one canonical address, whether writes through such a pointer are accepted, and how long such an address stays valid. Code that needs stable mutable storage should declare an explicitly mutable object with a suitable lifetime. Whether equal-valued constants share storage is unspecified.

Formatting helpers differ in ownership: console formatting returns a byte count, `sprint` returns text allocated through the active context allocator, and `tprint` returns text backed by temporary storage. The format language supports sequential `%` slots and numbered slots such as `%2`; the default metaprogram checks argument counts and indices. `Any` is a uniform type-erased wrapper carrying type metadata and a pointer to its value. It does not own or copy the payload, so it can alias a mutable value and can dangle when that value's storage expires. Assigning one `Any` to another copies the wrapper.

## Structs, unions and enums

`Name :: struct { ... }` declares an aggregate type. `.{field=value}` is a named struct literal, and positional literals are also allowed. Omitted members receive defaults. Nested literals can use contextual type information. Structs are plain data layouts: procedures declared inside a struct declaration are not instance methods and do not capture an instance.

`#as using field: Base` embeds a base-shaped struct as the first member and exposes its members through `using`. Struct composition and `using` do not imply virtual dispatch or class inheritance.

Enums declare named constants with an integer backing type, with explicit values or automatic sequencing. `#specified` marks enums whose numeric values are explicit and stable. `.MEMBER` is contextual enum-member syntax when the expected enum type is known. `enum_flags` declares bit flags. `#complete` applies to enum `if`/`case` branches (see the control-flow page), not to enum declarations. Implicit enum conversions and duplicate-value rules are unspecified.

A tagged union is built from an enum discriminator and a union payload; code branches on the discriminator before reading the matching field. `#type,distinct T` declares a non-interchangeable variant, and `#type,isa T` declares a type that can be used where its underlying type is accepted. A distinct type keeps its own nominal identity and is not an ordinary alias.

## Type values and restrictions

Types are first-class compile-time values. `Type` parameters and expressions such as `type_of(x)` let metaprograms and polymorphic procedures inspect or construct types. Runtime reflection through `type_info(T)` is separate: it embeds requested metadata into the program and can be trimmed with the type-info directives.

Member names, types, offsets, and sizes from type info can be used to copy matching fields between two structs. This is reflection over emitted metadata, not structural typing or an implicit assignment rule. Metadata reduction is covered on the metaprogramming page.

Named type restrictions constrain polymorphic types, and compile-time predicates and `#assert` check other conditions. The full restriction-expression grammar is unspecified. A failed restriction is a type error; the compiler does not silently fall back to a different overload.

## Implementation notes

- Keep literal typing separate from variable conversion. A variable initialized from `9` is `s64` unless explicitly typed or held as a constant.
- Keep compile-time type identity distinct from runtime values; `Type` is not a universal boxed value.
- Array views are non-owning. Bounds checking cannot make a forged count or stale pointer safe.
- Default initialization zero-fills the representation unless a type declares member defaults. Other aggregate initialization corner cases are unspecified.
- Numeric overflow, cast-check switches, and optimizer effects are build options, not language rules on this page.
