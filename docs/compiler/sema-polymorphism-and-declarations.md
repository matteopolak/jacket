# Sema: polymorphism and declarations

## What it is

Implementation notes for the parts of `crates/jaic/src/sema` that handle declarations, polymorphic calls, baking, `#modify`, macros, deferred top-level items and the end-of-compile declaration check. User-facing behaviour is in the [language docs](../README.md#language); this page is about where things live and why they work the way they do.

## How it works

### Declarations

- **Locals** (`check_local_decl`, `check_decl_values` in `stmt.rs`): `a, b := 1, "x"` pairs values with names (`Decl::extra_values`); `a, b := f()` splits a multi-value call; `a=, b := ...` and `a:, b = ...` assign to names marked `Decl::existing`; a place that is not a plain name (`ok:, t.str = f()`) is in `Decl::targets` and checked as an expression. Declaring a name twice in one scope is an error. A procedure body gets its own block scope, so it can shadow a parameter.
- **Local constants** are hoisted by `check_block_stmts`, so a nested procedure can be called above its declaration.
- **Compile-time code and locals**: `#run`, `#assert` and constant initialisers are checked in a thunk scope (`thunk_scope`) one procedure level deeper, so using a runtime local is a "compile-time expression" error. Exceptions that only need the local's type: `#if x.CONST` and `type_of(local.*)`.
- **`#this`** is the enclosing struct, or inside a procedure body the procedure (`Scope::proc`).
- **Aliases**: `name :: overloaded;` resolves to `Resolved::ProcSet`. An alias to a name in another scope (`starts_with :: begins_with;`) joins an overload set like a procedure (`overloadable_or_alias` in `scope.rs`).
- **`#poke_name Module name;`** adds the importer's entity itself to the module scope after `expand_all` (`apply_pokes`), so both see one type.
- **`using _ :: struct {...}`** may repeat at top level; each gets a hidden name (`__using_N`).
- **Member aliases**: `using,only(width, height) texture.desc;` in a struct body (also `except`, `map` or no filter) makes members of a nested field reachable on the struct without adding storage (`Compiler::member_aliases`, consulted last by `find_member`).
- **Field types in constants**: while a struct is laid out, earlier field types go into `Compiler::field_types`, so `type_of(field)` works in a struct constant (`FnCtx::type_only`). `Type.field` names the field's type, so `Anim.joints.Load_Factor` reaches its constants.
- **Types naming parameters**: a parameter or result type that mentions an earlier parameter (`proc: cache.Proc`, `-> type_of(asset)`) is checked with the parameters in a scratch scope (`type_from_params` in `procs.rs`; a text scan of the type span decides).
- **Parenthesized types**: `x: (*T);` and `#type (*T)` are procedure types returning nothing (`parse_param_type`).

### Calls and polymorphism

- A polymorphic procedure passed to a procedure-typed parameter is instantiated from the parameter's types (`instantiate_for_proc_type`). For `f: (T) -> $R`, an overload set or polymorphic procedure is resolved after the other bindings (`deferred_procs` in `infer_bindings`, `proc_for_param_types`), then `$R` binds from the chosen instance.
- `null` binds a type variable only when no other argument does (it binds `*void`). A defaulted baked parameter whose type names that variable (`$is_equal: (T, T) -> bool = null`) is evaluated after the `null` binding (`deferred_defaults`).
- `#char` binds `u8`. `.A | .B` and `xx a + 1` are deferred like `.A` and `xx a`.
- A multi-value call passed as one argument gives its first value; spreading is the fallback.
- After a named `v = x` for a variadic, further positional arguments extend it until another named argument (`assign_slots`).
- `$a: [$N] float` bound to `.[...]` infers `N` from the literal.
- **Baked variadics** (`$types: ..Type`): the arguments become one constant `[] T` (`baked_pack` in `calls.rs`) whose data global `const_pack` deduplicates by contents (`Compiler::packs`). Instance keys go through `instance_key_value` (`sema/mod.rs`), which adds a constant view's relocation targets to its bytes: `Value` equality compares only an aggregate's bytes, which for a view is its count and a zero pointer, so `f(A)` and `f(G)` used to share one instance. Equal lists share the global and so the instance.
- Baked parameter defaults (`$mode: Mode = .fast`, `$info := Info.{}`) give `.X` and `.{...}` the declared or default type. A type naming earlier type variables (`$compare: (T, T) -> bool = (a, b) => a == b`) is read in a scratch scope holding the bindings so far.
- A bare `Base`, `*Base` or `$T/Base` parameter accepts a struct whose `#as` member is an instance of `Base` (`instance_or_as_base`); the call converts through the `#as` offset. A `#bake_arguments` struct as restriction (`$V/Vec3`) accepts instances of the origin whose parameters match the baked values.
- **Overload ties**: equal conversion cost prefers non-polymorphic procedures, then the header whose parameter types pin down more structure (`pattern_specificity`: `*$T` beats `$T`). Between `float32` and `float64`, an untyped literal picks `float32`; integer parameters still beat floats.
- **Procedure values**: calls through a procedure value use the parameter names and defaults recorded for its type (`Compiler::proc_type_params`), and a procedure type can take a parameter's type from its default. `type_of(poly_proc)` gives a procedure type for compile-time inspection only (`poly_proc_type`): a parameter or result that depends on a type variable is `TypeKind::PolyParam` (printed `$`, type-info tag `POLYMORPHIC_VARIABLE`; `*$` for a pointer to one), the rest keeps its type, and baked parameters keep their place. A polymorphic struct used as a value (`type_of(Table)`, `print(Table)`, `t := Table`) is the `Type` value `TypeKind::PolyStruct` (`poly_struct_type`), which prints as the struct's name; it is only a template, never a layout, and `x: Table;` is still an error.
- **`$$x` auto-bake**: the parser sets `Param::auto_bake`. In `match_candidate`, a constant argument selects `auto_bake_variant`, a copy with those parameters turned into `$` ones (an omitted argument counts as constant when its default folds without running code: `default_is_constant`), cached per procedure and `AutoBake` mask so `#procedure_of_call` identity is per baked value. Non-constant calls use the original (`is_constant(x)` is false). Constants with no `Value` (`"s".data`, `*global`, `type_info(T)`, constant pointer arithmetic: `is_constant_pointer`) stay runtime parameters but bind a hidden `$const:x` marker that `is_constant` reads.

### Baking and `#modify`

- `bake.rs`: `T = Type` for a `$T` variable instantiates the baked copy. Constants baked into a polymorphic struct live in `PolyStruct::baked` and become members of every instance.
- `#modify` (`modify.rs`, `run_modify_block`): after inference, the block runs in the interpreter with each type variable as a mutable `Type` (globals read back afterwards). Unbound variables start as `void`. `return false` rejects the candidate; `return false, "why";` adds the message to the error via the `modify.message` global. The resulting bindings replace the inferred ones.
- A `#modify` with nothing to modify (`check_proc_modify` from `build_signature`, `check_struct_modify` where a parameterless struct type is made) is an error with kind `ModifyWithoutPolymorphs`. `call_procs` returns that error as it is instead of wrapping it as the call's mismatch, so it points at the `#modify` whether the procedure is called or only checked as unreferenced code. Instances (`bindings` set) and `$$` procedures and their auto-bake variants are skipped: the block runs for the variants that bake.
- Struct `#modify` (`run_struct_modify`, from `instantiate_struct`): every struct parameter is assignable (`if N < 8 N = 8;`) and the final values form the instance key, so `Holder(3, T)` and `Holder(8, T)` can be one type.

### Operators and conversions

- `a op= b` calls `operator op=` for struct-like targets (`try_operator_assign`).
- `x[i]` tries `operator []`, then `operator *[]` (dereferenced, so assignable); a pointer with no fitting operator indexes memory (`try_pointer_index_operator`).
- An integer meeting a float in a binary operator converts to that float (`binary_operand_type`, `binary`). `proc == ptr` compares addresses, and so do two procedure names.
- `convert.rs` auto-dereferences `*Thing` to a `Thing` value (structs only, one level, also to an `#as` member's value); poly patterns accept `*Instance` the same way. Also: `*[..] T` to `*[] T`, a one-character string constant to `u8`, and an `ifx` argument fits an overload only if both branches do.
- `ifx`: a numeric then-value widens to the expected numeric type; an overload set takes the expected or else-value's procedure type; a branch can be a block whose last expression is the value.
- `using` on a pointer-valued expression (`using editors.active_pane;`) binds the pointer itself.

### Macros and loops

- Constants declared in a macro body exist once per expansion (`hoisted_consts` keyed by scope).
- A constant argument (string, integer, bool, float) the macro never writes stays a typed constant inside the body (`const_macro_params`), so `#if n <= 1` and a constant string's `.count` work in recursive macros. "Never writes" is a text check (`text_may_write`); any `#asm` counts as a write.
- A macro's deferred statement keeps the caller scope it was written for (`DeferEntry::caller_scope`), so `` `name `` in it resolves where the defer runs (`FnCtx::backtick_scope`).
- `#insert (break=..., continue=..., remove=...) body` runs the replacement at the insertion site (`InsertReplacements`, `try_insert_replacement`).
- `#insert,scope(Top)` with a top-level `Top :: #code()` checks the code in that file's scope and declares its constants there, so a metaprogram sees them as top-level declarations.
- `for *=cond` / `for <=cond` take compile-time flags; `remove it` works in reverse loops.
- `check_for` lowers range and array loops rotated: the bound is tested once before the body and again after each step, which branches straight back to the body, so a loop costs one compare and one branch per iteration. A range loop tests `it` against the bound before stepping (`>=`, or `<=` reversed), so a bound at the type's limit cannot wrap, and a body that moves `it` past the bound still ends the loop. An array loop reloads the count after each step, so `array_add` and `remove it` in the body take effect. `tests/stdlib/for-loop-bounds.jai` covers these edges.
- A struct iterated through `for_expansion` is passed by address unless an overload takes that type by value. A non-pointer iterable passed to a `*T` expansion parameter is spilled to a temporary (`wants_pointer` in `stmt.rs`). A forwarded body (`for_expansion(*inner, body, flags)`) borrows the inserting macro's `it`/`it_index` into the loop scope. `utf8_iter` backtick-declares `it` and `it_index` (character count; `-1` backwards).

### Pending and deferred work

- Pending top-level items (`#if`, `#insert`, `#run`-driven declarations) expand lazily when a lookup reaches their scope (`expand_pending`). An item that fails while some body is mid-lowering (its compile-time code may need that body) goes back to waiting in `Compiler::deferred_pending`; `expand_all` retries it once nothing is lowering, and only that failure is final.
- **Placeholders**: resolving an undefined `#placeholder` bumps `placeholder_misses`. A pending item, top-level `#run` or `#assert` whose failure reached one keeps waiting, since a metaprogram may define the name at `TYPECHECKED_ALL_WE_CAN`, and is retried at the next settle. `finish_program` sets `placeholders_final` and settles once more, so a never-defined placeholder is still an error. A failed top-level `#insert` always waits until then, because its generator body may have been parked by an earlier miss (Vk-Engine builds `EntityTypeId :: enum` from generated `Entity_Types`). `lookup_full` drops a placeholder found alongside its definition, which happens when the name arrives through an import of the module the metaprogram filled.
- **Queued bodies**: `drain_bodies_lenient` lowers what it can before a thunk runs; bodies that fail (they need a layout still in progress) stay queued for the final `drain_bodies`. A body whose failure is memoised for the current `lower_epoch` is parked (`parked_bodies`), so later thunks in the epoch skip the group; `drain_bodies` and a new epoch put them back. `drain_bodies` retries failures while other bodies still lower, since a later body may declare what an earlier one uses (`#insert,scope(Top)`), and reports the first error only after a pass with no progress. Progress counts only bodies queued when the pass began: a failing body re-creates its nested procedures on each retry, and counting those would loop forever.
- Compile-time constants (`eval_const`) run with the compile-time Context, so `#assert` can call procedures that need one. Integer-valued pointer constants (`cast(*void) 32512`, `cast(HANDLE) -1`) freeze as raw values (`freeze_pointer`).

### Declared structs are always checked

Sema is demand-driven, so a struct nothing needs would never be laid out. Jai type-checks every declaration (dead-code elimination only skips procedure bodies), so after reachable code is lowered, `check_declared_structs` in `driver.rs` lays out every top-level, non-polymorphic struct and union in every loaded file. A member typed by an undefined name (`a: Missing;`, `[4] Missing`, `*Missing`, `(x: Missing) -> s32`) is reported as `unknown identifier` at the member whether or not the struct is used. Polymorphic structs are checked per instance; structs inside procedure bodies with their body. Unused constants, globals and procedure bodies are checked afterwards in the program's own files only (in every file with `-no_dce`); see [dead-code elimination](../language/dead-code-elimination.md).

`type_info_global` registers a descriptor's global before building it, because descriptors refer to themselves. If the build fails the type is unregistered (`failed_type_infos` keeps the global for the retry), so a retried body reports the layout error instead of getting an empty cached descriptor.

## How to change it

- New operator forms go next to `try_operator_assign`.
- To check more kinds of unused declarations, extend the filter in `check_declared_structs`. Whatever it resolves must not depend on code a metaprogram adds later; it runs after `placeholders_final`. Negative tests: `tests/corpus/negative/undefined-*.jai`.
- New `#modify` features go in `run_modify_block`, shared by procedures and structs. It builds one IR function per call, so watch per-call cost.

Tests: `tests/stdlib/struct-modify.jai`, `modify-polymorph-forms.jai`, `tests/corpus/negative/modify-without-polymorphs*.jai`, `null-poly-baked-default.jai`, `compiler-module-placeholder-insert.jai`.

## Dependencies

`interp` (compile-time execution), `parser/decl.rs` (mixed declaration lists), `calls.rs` (candidate matching).
