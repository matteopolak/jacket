# using

`using` on struct members, locals, parameters, enum values and module imports makes the target's members reachable by bare name.

```jai
Inner :: struct { n: int = 5; K :: 3; }
Plain :: struct { using i: Inner; }       // p.n == p.i.n, Plain.K == 3

Derived :: struct { using #as b: Base; kind = .B; y: int; }   // kind = .B overrides Base's default
get_x(*d);                                // get_x :: (b: *Base) -> int
```

## On struct members

- `using i: Inner` embeds `Inner` and promotes its members {#using.1} and constants {#using.2}.
- `#as` also lets the outer type convert implicitly to the embedded one, so a `Derived` passes where `Base` or `*Base` is expected {#using.3}. `using #as x` and `#as using x` both parse {#using.16}. Without `#as`, passing `Plain` to `(i: Inner)` fails with ``argument of type `Plain` does not match parameter type `Inner` `` {#using.4}.
- A pointer to such a struct also passes where the `#as` member's own type is expected; the pointer is dereferenced and the member read. With `Device :: struct { #as handle: VkDevice; }`, a `*Device` passes as a `VkDevice` {#using.18}. Converting `*Derived` to `*Base` is preferred when both apply.
- The member has `USING | AS` in its `type_info` flags {#using.5}.
- A `using` member of pointer type (`using meta: *Meta`) promotes the pointee's members {#using.6}; writes go through the pointer {#using.7}.
- `member = value;` in a struct body overrides a promoted member's default {#using.8}, also through a dotted path (`base.f = show;`) {#using.9}. Both `Derived` and `New(Derived)` get it; other `Base` members keep their defaults {#using.10}.
- Direct members shadow promoted ones {#using.15}. Two `using` members bringing in the same field name are an error (``the name `x` is brought in twice``).
- `using g;` of a global pointer to a struct doesn't make the struct contain itself {#using.17}.

## In scopes

- `using p;` in a block, or a `using p: *Pt` parameter, brings the fields into scope {#using.11}.
- `using basket.tag;` on an enum-typed value brings in the enum's members, so `case MANGO;` works {#using.14}.
- `using Type.{...};` copies the literal into a writable anonymous local {#using.12}.
- `using name := value;` at file scope works for variables, not only constants {#using.13}.
- A declaration in the scope that holds `using o;` can't reuse the name of one of `o`'s members: `x := 1;` is `` `x` is already declared in this scope ``. A nested block can shadow it as any outer name.
- If two `using` locals or parameters of one scope both provide a name, using it is `` `model` is ambiguous ``; a `using` in an inner block takes the name over.
- `using,only(...)` and `using,except(...)` filter locals and statements; names in a filter need not exist.
- `using,map(mapper) value;` runs a compile-time procedure over copied member-name strings: a replacement resolves to the original member, an empty one omits the short name. On an import (`using,map(f) M :: #import "X"`), the mapper sees the module's exports, including re-exports; names it doesn't keep are hidden, while the alias `M` keeps the original names. The mapper runs at the import's first use, with the module visible unmapped meanwhile.

## How to change it

Member-side logic: `using_member`, `using_target`, `find_member_in`, `override_target` in `sema/structs.rs`; clashes are `check_using_clashes` (end of layout, field names only). Scope-level: `sema/scope.rs`, `check_using` in `sema/stmt.rs`, `resolve_module_filter` in `sema/using.rs`. Inline `#overlay` and `#place` targets use the same promoted-field resolver, limited to fields already laid out; pointer-backed promoted storage has no inline offset and is rejected. Tests: `tests/compiler/regressions/using-*.jai`.
