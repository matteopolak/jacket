# Variant

A tagged union whose cases are types, like a Rust enum: `Variant(s64, string)` holds either an `s64` or a `string` and knows which.

```jai
#import "Extensions/Variant";

Number_Or_Text :: Variant(s64, string);

v := variant(Number_Or_Text, 42);
w: Number_Or_Text;                      // zeroed first case
set(*w, "hi");

is(v, s64)                              // bool
n, ok := get(v, s64);                   // zero and false when another case is live
p := get_pointer(*v, s64);              // pointer into the storage, or null

variant_switch(v,                       // one handler per case, checked at compile time
    (it: *s64)    { print("number %\n", it.*); },
    (it: *string) { print("text %\n", it.*); },
);
free_variant(*v);                       // frees what the live case owns
```

## How it works

- Layout: `index: s32` plus `payload: [N] u8` sized and aligned for the biggest and strictest case. `declare_variant` (`layout.jai`) writes the struct with `#insert`, including `case_type_K` aliases and `case_infos` for run-time printing, equality and freeing.
- The case comes from the value's type at compile time. A non-case type or a repeated type (`Variant(s64, s64)`) is a compile error; wrap each in a struct. Literals are `s64`/`float32`, so cast to the case you mean.
- `variant_switch` (`switch.jai`) matches handlers to cases by argument type, in any order. Missing, extra or repeated handlers are compile errors. Pass `*v` to let handlers change the variant. Handlers are procedures, so they can't see surrounding locals.
- Equality and `free_variant` (`reflect.jai`) walk the live case by type info. Only free payloads that came from the allocator, not literals.
- Printing: `use_variant_printing()` installs a struct printer so `print("%", v)` writes the payload; `variant_to_string(v)` prints without installing anything.
- Json encodes variants: [json](json.md#variants).

## How to change it

- New operation needing the live case: use `V.case_infos[v.index]` and `v.payload.data` (see `values_equal`).
- New case-list check: `declare_variant` or `case_of`, reporting with `compiler_report`.
- More than 16 handlers: raise `MAX_SWITCH_CASES` (overloads come from `switch_overloads`).
- A variant can contain itself only through a pointer that passes through a named struct: `Tree :: Variant(Leaf, Branch); Branch :: struct { children: [..] *Tree; }`.
- Tests: `stdlib/Extensions/Variant/tests/`, including `negative/` programs listed in `manifest.json`.
