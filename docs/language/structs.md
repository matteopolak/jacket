# Structs and aggregate literals

Struct declarations, defaults, literals, parameterized structs and layout.

```jai
Vec :: struct { x: float = 1; y: float = 2; z: float = 3; }
w := Vec.{x=9};      // {9, 2, 3}
u := Vec.{4,5,6};    // positional
e: Extent = {1280, 720};   // dot optional when the type is known

Mix   :: struct { a: u8; b: s64; c: u16; }        // size_of == 24
NoPad :: struct #no_padding { a: u8; b: s64; }    // 9
Al    :: struct #align 16 { a: u8; b: s32; }      // 16
Rgba  :: struct { union { using c: struct { r, g, b, a: u8; }; packed: u32; } }
```

## Literals

- Member defaults apply to `v: Vec;` {#struct.1} and to members a literal omits {#struct.2}.
- Positional literals fill members in declaration order {#struct.3}; `Vec.{}` is all defaults {#struct.4}. Every member without an initializer needs a value (`not enough values in this struct literal`); trailing members with initializers may be left out.
- A literal's field values replace defaults wholly, `null` included: `U.{p = null}` clears the bytes a union's `s: string = "default"` put there {#struct.19}.
- Where the type is known, the dot may be left out: `gpu_init(1, {.GENERAL, 1})`, `return {w, h};`, `e = {width = 5};` {#struct.17}. A `{` that starts with a statement keyword or directive, declares a name, holds a `;`, or is a lambda body is a block instead.
- An overlay takes no positional value, so `Rect.{0, 0, 640, 480}` fills `x`, `y`, `w`, `h` of a `Rect` that overlays `pos` on `x` and `size` on `w` {#struct.25}.
- A constant view (`[] T`) given for a struct parameter declared `[$N] T` is taken as the fixed array of its elements, so `N` is its count: `Wrapper(registry)` with `registry :: #run make_types();` {#struct.26}.

## Default overrides

Struct-body assignments replace subobjects of the default value. Targets can be direct
or promoted fields, aliases, and nested fixed-array elements. Selected `#if` branches
and expanded `#insert` text use the same rules and assignment order.

```jai
Grid :: struct {
    rows: [2][3]int;
    #insert "rows[1][2] = 17;";
    rows[0][0], rows[0][1] = 3, 5;
}
grid: Grid; // grid.rows[1][2] == 17
```

Indexes must be constant and within the fixed array. Defaults cannot write through
pointers or index views or dynamic arrays. Invalid targets, incompatible values,
compound operators and mismatched value counts are errors. A single value broadcasts
to several targets; multiple results pair with targets in order. Nested defaults and
pointer relocations survive writes to other subobjects.

The compiler retains assignments during `sema/structs/fields.rs` expansion and stores
them with the completed layout. `defaults.rs` builds the shared image used by variables,
literals and `initializer_of`; `using_members.rs` resolves targets with the ordinary
filtered member resolver. Extend these paths together when adding a target form.
There are no default-override flags or external dependencies.

## Layout

- Layout is lazy, but every top-level struct is laid out before compilation ends, so a member of an undefined type is always an error {#struct.5}. By-value recursion is caught. The member defaults of every non-polymorphic top-level struct are evaluated too, so a default of the wrong type is an error even in an unused struct.
- Fields are naturally aligned and the size rounds up to the struct alignment {#struct.6}. `#no_padding` packs members with no gaps {#struct.7}; `#align N` raises the alignment {#struct.8}.
- `#place a; b: int;` overlays `b` on `a`'s offset {#struct.9}. `#overlay(a) b: u32;` overlays only `b` and leaves following members where they were; `b` gets the `OVERLAY` flag in its `Type_Info_Struct_Member`, so serializers can skip it {#struct.18}.
- `#overlay(a)` is allowed only in struct bodies (including `#if`, `#insert`ed members, unions and nested anonymous structs); several may share a target, and the struct's size is the farthest member's end {#struct.22}. Elsewhere it is an error. The target may be a path, `#overlay(base.table)`, adding the inner member's offset {#struct.24}. `#place` takes a path too.
- Members of an anonymous `union { }` or `struct { }` are reachable directly {#struct.10}, and through `c` with `using c:` {#struct.11}. They share storage as the example shows {#struct.12}.
- `offset_of(T, "member")` or `offset_of(T.member)` is the byte offset as a constant `s64`. Dotted paths and members behind a `using` member work; a member behind a `using` pointer or a missing one is an error {#struct.20}.
- `is_value_type(T)` is `true` when copying a `T`'s bytes copies all of it: numbers, `bool`, enums, distinct types, fixed arrays, and structs or unions of those. Pointers, `string`, views, resizable arrays, procedures, `Any`, `Type` and `Code` anywhere inside make it `false` {#struct.21}.

## Parameterized structs and constants

- `struct(T: Type, N: int)` is instantiated per distinct argument values, so `Gen(int)` and `Gen(s64)` are one type {#struct.13}. Names print with parameter names: `Pair(T=s64, N=3)` {#struct.14}.
- A body can hold constants (`K :: 3;`, read as `Plain.K`) {#struct.15} and default overrides like `kind = .B;` (see [using](using.md)) {#struct.16}.
- `Plain.field` and `Plain.inner.field` name the field's type where only a type is wanted (`x: Plain.inner`). Where a value of another type is expected (`color = Theme.button.surface_color;`) they read the field's default from a default-initialized temporary. With no expected type, `v := Plain.count;` stays a type {#struct.23}.

## How to change it

New struct flags: parse them in `parser/aggregate.rs` next to `#no_padding` and `#align`, and apply them in `layout_struct_inner` (`sema/structs.rs`). Literals: `check_struct_literal`. `offset_of` and `is_value_type` live in `check_builtin` (`sema/calls/builtins.rs`). Looking up a declared member's type doesn't need its offset (`declared_member_type`, `sema/structs/access.rs`), so a nested array bound can ask for a type mid-layout; extend that path together with the layout solver when adding member forms. See [sema](../compiler/sema.md).
