# Semantic analysis

Notes on `crates/jaic/src/sema`: module loading, declarations, calls and polymorphism, macros, and the end-of-compile checks. User-facing behaviour is in the [language docs](../README.md#language); this page is for people changing the checker.

Sema is demand-driven: a declaration is resolved when something needs it, and checking a body lowers it to [IR](ir.md) in the same pass.

## How it works

### Compiler state

`Compiler` (`mod.rs`) keeps its state in one struct per concern, each in its own file:

- `ct` compile-time state (interpreter, top-level `#run`s), `src` files and parse-ahead, `diag` warnings, `epochs` counters that stamp cached failures, `ty` the type table, `ns` modules, scopes and entities, `caches` memoised lookups, `pr` procedures and the body queue, `out` the IR program and pooled constants, `meta` workspace and records, `ed` the editor analysis.
- A new field goes in the group it belongs to. Keep groups disjoint so a method can borrow one mutably while reading another. A helper that touches one group is a method on that group.

### Modules and top-level expansion

- `load_module` keys a module by path and parameter values. The first `#module_parameters` list is part of the instance key; the second is shared program parameters.
- All member lookup goes through `module_lookup` (plain `#import`, `M.name`, `using M;`, re-exports). A new kind of module member goes there.
- `expand_all` first expands imports and `#if` items whose condition is a plain constant, so `#add_context` lands before a `#run` lays out the Context. It reruns after each source a metaprogram adds, visiting only unsettled scopes.
- Queue scope items with `push_pending`/`push_import`, never directly.
- Import search: `<main file dir>/modules`, `-I` directories, then the stdlib; `<importing file dir>/modules` is tried first.
- An item that fails mid-lowering is retried; if the program fails first, the error carries the deferred item's error as a note.

### Declarations

- `#run`, `#assert` and constant initialisers are checked in a thunk scope one level deeper, so a runtime local is an error.
- A parameter or result type that mentions an earlier parameter is checked with the parameters in a scratch scope.
- Every top-level non-polymorphic struct is laid out at the end (`check_declared_structs`), so `a: Missing;` is reported whether or not the struct is used.

### Calls and polymorphism

`calls/` is split by phase: `mod.rs` ranks candidates, `infer.rs` binds `$T`, `baking.rs` builds baked values, `emit.rs` lowers the chosen call.

- Overload ties: equal conversion cost prefers non-polymorphic procedures, then more constrained patterns (`*$T` beats `$T`).
- `null` binds a type variable only when no other argument does.
- Baked variadics (`$types: ..Type`) become one constant `[] T`, deduplicated by contents. Instance keys include a constant view's relocation targets.
- `$$x` auto-bakes: a constant argument selects a cached variant.
- `type_of(poly_proc)` gives a procedure type for inspection only; a polymorphic struct used as a value is a template, never a layout.

### Operators, macros, loops

- `a op= b` calls `operator op=`; `x[i]` tries `operator []`, then `operator *[]`.
- Constants in a macro body exist once per expansion. A constant macro argument the macro never writes stays a typed constant so `#if n <= 1` works in recursive macros.
- `#insert (break=..., continue=...)` runs the replacement at the insertion site.
- `check_for` lowers loops rotated: a range loop tests `it` against the bound before stepping (so a bound at the type's limit cannot wrap); an array loop reloads the count each step.

### Unreachable code

The editor fades code that cannot run, and batch compiles warn after `return`, `break` and `continue`. The walk is `reachability.rs`; value knowledge comes from editor facts (`ide/values.rs`, `ide/interval.rs`), so a batch compile decides by jumps alone.

- Each integer or bool expression gets an `Interval` from its type, constants and arithmetic. Unknown must stay "no information", never a guess.
- Dead: code after an exit, a branch whose condition has one value, an infinite loop's tail. A polymorphic body is walked per instance and shown dead only if dead in all.
- `ide/never_returns.rs` finds procedures that never return by fixed point.
- In an editor analysis the ruled-out branch is still checked so hover works there; its errors are dropped.

### Pending and deferred work

- Resolving an undefined `#placeholder` makes dependent items wait and retry at the next settle, since a metaprogram may define it later.
- `drain_bodies_lenient` lowers what it can before a thunk runs; `drain_bodies` retries failures while any progress is made, then reports the first error.
- The parser keeps how a literal was spelled; sema carries it on the constant for messages.

## How to change it

- Per-module state goes on `Module`. Anything needed before compile-time code runs belongs in the plain pass of `expand_all`, which must stay free of calls.
- New operator forms go next to `try_operator_assign`; `#modify` features in `run_modify_block`; a binary-operator rule in the `check_binary` phase it belongs to.
- `StructInfo::fields` is an `Rc<[Field]>`: clone it to walk fields while calling `&mut self` methods.
- A new condition form for unreachability needs a rule in both `compute_ainfo` and `IdeFacts::eval`.
- Tests: `tests/compiler/regressions/` (`module-member-resolution.jai`, `struct-modify.jai`, ...) and `tests/corpus/negative/`.

## Dependencies

The parser, `consteval.rs`, `structs.rs`, `interp`, `calls/`.
