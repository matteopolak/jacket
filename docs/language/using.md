# using

## What it is

`using` on struct members, locals, parameters and module imports. It makes the members of the target reachable by bare name. The member-side logic is `using_member`, `using_target`, `find_used_member` and `override_target` in `crates/jaic/src/sema/structs.rs`; scope-level `using` is in `sema/scope.rs`.

## How it works

On a struct member, `using i: Inner` embeds `Inner` and promotes its members and constants:

```jai
Inner :: struct { n: int = 5; K :: 3; }
Plain :: struct { using i: Inner; }
p: Plain;   // p.n == 5, p.i.n == 5, Plain.K == 3
```

`using #as` additionally makes the outer type implicitly convert to the embedded one, so a `Derived` can be passed where `*Base` or `Base` is expected. Without `#as`, passing `Plain` to `(i: Inner)` is rejected (`argument of type Plain does not match parameter type Inner`). With it:

```jai
Derived :: struct { using #as b: Base; y: int; }
get_x(*d);   // d: Derived, get_x :: (b: *Base) -> int
```

In `type_info(Derived).members`, the member has `flags` `USING | AS`.

A `using` member of pointer type (`using meta: *Meta`) promotes the pointee's members too; writes go through the pointer (`tests/stdlib/using-pointer-member.jai`).

A `member = value;` statement in a struct body overrides the default of a promoted member, including through a dotted path (`base.f = show;`):

```jai
Derived :: struct { #as using base: Base; kind = .B; value: int; }
```

Both `Derived` and `New(Derived)` get `kind == .B` while `extra` keeps `Base`'s default. See `tests/stdlib/using-member-default-override.jai` and `struct-body-member-path-override.jai`.

`using` on a parameter (`move :: (using p: *Pt, dx: int) { x += dx; }`) or a local statement (`using p;` in a block) brings the fields into that scope. `using` of a module or struct constant at file scope is covered by `tests/stdlib/module-using-global.jai`, `module-using-import-reexport.jai` and `using-discard-struct-constants.jai`.

- `using Type.{...};` in a procedure copies the literal into an anonymous local and uses it (`check_using`, `sema/stmt.rs`); it is writable, unlike the read-only literal data real Jai uses.
- `using name := value;` at file scope registers the `using` for variables too, not only constants (`sema/modules.rs`).

## How to change it

Member lookup order lives in `find_member`; promoted members come after direct ones, so a direct member shadows a promoted one. Default overrides resolve through `override_target`/`override_path` and are applied when the default initializer is built (`default_initializer`). Gotcha: `using` is a declaration modifier, so `#as` must appear next to it (`using #as x` and `#as using x` both parse).

## Configuration

None.

## Dependencies

`sema/structs.rs`, `sema/scope.rs`, `parser/decl.rs`.
