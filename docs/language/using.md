# using

## What it is

`using` on struct members, locals, parameters, enum values and module imports makes the target's members reachable by bare name. Member-side logic is in `sema/structs.rs` (`using_member`, `using_target`, `find_used_member`, `override_target`); scope-level `using` is in `sema/scope.rs` and `check_using` in `sema/stmt.rs`.

## How it works

On a struct member, `using i: Inner` embeds `Inner` and promotes its members {#using.1} and constants {#using.2}:

```jai
Inner :: struct { n: int = 5; K :: 3; }
Plain :: struct { using i: Inner; }
p: Plain;   // p.n == 5, p.i.n == 5, Plain.K == 3
```

`using #as` also lets the outer type convert implicitly to the embedded one, so a `Derived` passes where `Base` or `*Base` is expected {#using.3}. Without `#as`, passing `Plain` to `(i: Inner)` fails with `argument of type Plain does not match parameter type Inner` {#using.4}.

```jai
Derived :: struct { using #as b: Base; y: int; }
get_x(*d);   // get_x :: (b: *Base) -> int
```

Such a member has `USING | AS` in its `type_info` flags {#using.5}. `using #as x` and `#as using x` both parse {#using.16}.

A `using` member of pointer type (`using meta: *Meta`) promotes the pointee's members {#using.6}; writes go through the pointer {#using.7}.

A `member = value;` statement in a struct body overrides a promoted member's default {#using.8}, also through a dotted path (`base.f = show;`) {#using.9}. Both `Derived` and `New(Derived)` get the override; other `Base` members keep their defaults {#using.10}.

```jai
Derived :: struct { #as using base: Base; kind = .B; value: int; }
```

Elsewhere:

- `using p;` in a block, or a `using p: *Pt` parameter, brings the fields into scope {#using.11}.
- `using basket.tag;` on an enum-typed value brings in the enum's members, so `case MANGO;` works in a switch {#using.14}.
- `using Type.{...};` in a procedure copies the literal into an anonymous local. It is writable, where Jai uses read-only data {#using.12}.
- `using name := value;` at file scope works for variables, not only constants (`sema/modules.rs`) {#using.13}.

## How to change it

Member lookup order is in `find_member`: direct members come before promoted ones, so a direct member shadows a promoted one {#using.15}. Default overrides resolve through `override_target`/`override_path` and apply when `default_initializer` builds the default value.

Tests: `tests/stdlib/using-pointer-member.jai`, `using-member-default-override.jai`, `struct-body-member-path-override.jai`, `module-using-global.jai`, `module-using-import-reexport.jai`, `using-discard-struct-constants.jai`.

## Dependencies

`sema/structs.rs`, `sema/scope.rs`, `sema/stmt.rs`, `parser/decl.rs`.
