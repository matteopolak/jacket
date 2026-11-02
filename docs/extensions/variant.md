# Variant

A tagged union with named cases, like a Rust enum. Name the cases yourself, or list types and let each case be named after its type.

```jai
#import "Extensions/Variant";

// Named cases: two cases may share a type.
Contact :: Variant(struct { email: string; phone: string; });

// A type list: the cases are `circle`, `rect` and `s64_`.
Shape :: Variant(Circle, Rect, s64);

c: Contact;                             // holds no case: c.which == .NONE
c.which = .PHONE;
c.phone = "555-0100";

s := variant(Shape, Circle.{ 2 });      // pick the case by the value's type
set(*s, Rect.{ 1, 2 });

if #complete s.which == {               // every tag must be handled
    case .NONE;   print("nothing\n");
    case .CIRCLE; print("circle %\n", s.circle.radius);
    case .RECT;   print("rect %\n", s.rect.width);
    case .S64;    print("number %\n", s.s64_);
}

if s.which == .RECT print("%\n", s.rect.width);
p := get_pointer(*s, Rect);             // pointer into the storage, or null
free_variant(*s);                       // frees what the live case owns; back to .NONE
```

## How it works

- Layout: `which: Which` then an anonymous union of one field per case and `payload: [N] u8`, sized and aligned for the largest and strictest case. `Which` is an `enum u8` (`u16` past 255 cases) with `NONE :: 0` and one UPPER_CASE tag per case, so a Variant of small cases is small: `Variant(u8)` is 2 bytes.
- `declare_variant` (`layout.jai`) writes all of it with `#insert`, plus `case_type_K` aliases, `case_names` and `case_infos` for printing, equality, freeing and Json.
- Case names: the named form uses the struct's field names. A type list uses the type's name in snake_case (`HttpServer` and `HTTP_Server` become `http_server`; `*Node` becomes `node`); built-in types get a trailing `_` (`s64_`, `string_`) because `s64: s64` is not allowed. Types without a name (arrays, procedures) and two cases with one name are compile errors that suggest the named form.
- Read and write cases as fields, checking `which` first. `variant`, `set` and `get_pointer` pick the case by type instead; a type that two cases share is a compile error there, so set `which` and the field yourself.
- `set` doesn't free the old case. Equality and `free_variant` (`reflect.jai`) walk the live case by type info; only free payloads that came from the allocator. Two empty Variants are equal.
- Printing: `use_variant_printing()` installs a struct printer so `print("%", v)` writes the payload (`none` when empty); `variant_to_string(v)` prints without installing anything. `print("%", v.which)` prints the tag.
- Json encodes variants by case name: [json](json.md#variants).

## How to change it

- New operation needing the live case: `live_case(info, data)` (`reflect.jai`) gives the case index or -1; the payload is at `offset_of_member(info, "payload")`.
- New naming rule or case-list check: `derived_name` and `collect_cases` in `layout.jai`, reporting with `compiler_report`. Names the Variant declares itself are in `RESERVED_NAMES`.
- A variant can contain itself only through a pointer that passes through a named struct: `Tree :: Variant(Leaf, Branch); Branch :: struct { children: [..] *Tree; }`.
- Tests: `stdlib/Extensions/Variant/tests/`, with compile errors in `negative/`.
