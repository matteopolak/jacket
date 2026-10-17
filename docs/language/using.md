# using

## What it is

`using` on struct members, locals, parameters, enum values and module imports makes the target's members reachable by bare name. Member-side logic is in `sema/structs.rs` (`using_member`, `using_target`, `find_used_member`, `override_target`); scope-level `using` is in `sema/scope.rs` and `check_using` in `sema/stmt.rs`.

## How it works

```jai
Inner :: struct { n: int = 5; K :: 3; }
Plain :: struct { using i: Inner; }       // p.n == p.i.n, Plain.K == 3

Derived :: struct { using #as b: Base; kind = .B; y: int; }   // kind = .B overrides Base's default
get_x(*d);                                // get_x :: (b: *Base) -> int
```

- `using x: T` promotes `T`'s members and constants. `#as` (either order with `using`) also lets the outer type convert to the embedded one, by value or pointer; without it `Plain` does not match `(i: Inner)`. A pointer to such a struct passes where the `#as` member's type is expected (dereferenced and read); converting `*Derived` to `*Base` is preferred when both apply. The member has `USING | AS` in its `type_info` flags.
- A `using` member of pointer type promotes the pointee's members; writes go through the pointer.
- `member = value;` in a struct body overrides a promoted default, also through a dotted path (`base.f = show;`), via `override_target` / `override_path` when `default_initializer` builds the value.
- `using p;` in a block, or a `using p: *Pt` parameter, brings fields into scope. `using basket.tag;` on an enum value brings its members in (`case MANGO;`). `using name := value;` works for file-scope variables. `using Type.{...};` copies the literal into a writable anonymous local (Jai uses read-only data).
- `using,only(...)` and `using,except(...)` filters apply to locals and statements; names in a filter need not exist. `using,map(mapper) value;` runs a compile-time procedure over copied member-name strings; a replacement resolves to the original member, an empty one omits the short name. Module-import mapping is rejected.
- Direct members shadow promoted ones (`find_member`). Two `using` members bringing in the same field name are an error (``the name `x` is brought in twice``); `check_using_clashes` runs at the end of layout and compares field names only.
- `using g;` of a global whose type is being laid out does not force recursive layout merely to query a declared member; real storage cycles still diagnose.

## How to change it

Inline `#overlay` and `#place` targets use the same promoted-field resolver as member lookup, limited to fields already laid out; pointer-backed promoted storage has no inline offset and is rejected there. Tests: `tests/stdlib/using-pointer-member.jai`, `using-member-default-override.jai`, `module-using-import-reexport.jai`, `using-global-of-own-struct.jai`. Related: `parser/decl.rs`.

## Rules

Claims that tests cite by ID (`tools/check_rule_coverage.py`).

- On a struct member, `using i: Inner` embeds `Inner` and promotes its members {#using.1} and constants {#using.2}:
- `using #as` also lets the outer type convert implicitly to the embedded one, so a `Derived` passes where `Base` or `*Base` is expected {#using.3}. Without `#as`, passing `Plain` to `(i: Inner)` fails with ``argument of type `Plain` does not match parameter type `Inner` `` {#using.4}.
- A pointer to such a struct also passes where the `#as` member's own type is expected: the pointer is dereferenced and the member read, as for `*Thing` passed as `Thing`. This holds for any member type, including pointer types, so with `Device :: struct { #as handle: VkDevice; }` a `*Device` passes as a `VkDevice` (`*VkDevice_T`) {#using.18}. Converting `*Derived` to `*Base` (the pointer to an `#as` member) is preferred when both apply.
- Such a member has `USING | AS` in its `type_info` flags {#using.5}. `using #as x` and `#as using x` both parse {#using.16}.
- A `using` member of pointer type (`using meta: *Meta`) promotes the pointee's members {#using.6}; writes go through the pointer {#using.7}.
- A `member = value;` statement in a struct body overrides a promoted member's default {#using.8}, also through a dotted path (`base.f = show;`) {#using.9}. Both `Derived` and `New(Derived)` get the override; other `Base` members keep their defaults {#using.10}.
- `using p;` in a block, or a `using p: *Pt` parameter, brings the fields into scope {#using.11}.
- `using basket.tag;` on an enum-typed value brings in the enum's members, so `case MANGO;` works in a switch {#using.14}.
- `using Type.{...};` in a procedure copies the literal into an anonymous local. It is writable, where Jai uses read-only data {#using.12}.
- `using name := value;` at file scope works for variables, not only constants (`sema/modules.rs`) {#using.13}.
- Member lookup order is in `find_member`: direct members come before promoted ones, so a direct member shadows a promoted one {#using.15}. Default overrides resolve through `override_target`/`override_path` and apply when `default_initializer` builds the default value.
- `using g;` of a global pointer to a struct does not make the struct contain itself: names in its field types are looked up while it is laid out {#using.17}.
