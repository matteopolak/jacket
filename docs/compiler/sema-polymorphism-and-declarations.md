# Sema: polymorphism and declarations

## What it is

Notes on the `crates/jaic/src/sema` pieces that handle multi-value declarations, `#this`, polymorphic
procedure arguments, `#bake_constants`, `#modify`, `#poke_name`, declaration checking and a few operator forms.

## How it works

- **Local declarations** (`stmt.rs`, `check_local_decl` / `check_decl_values`): `a, b := 1, "x"` pairs values
  with names (`Decl::extra_values`); `a, b := f()` splits a multi-value call; `a=, b := ...` and
  `a:, b = ...` assign to names marked existing (`Decl::existing`). A name declared twice in one scope is an
  error. Procedure bodies get their own block scope, so a body can shadow a parameter.
- **Local constants** are hoisted: `check_block_stmts` declares every constant of a block before checking
  statements, so a nested procedure can be called above its declaration.
- **Compile-time code and locals**: `#run`, `#assert` and constant initializers are checked in a thunk scope
  (`thunk_scope`) one procedure level deeper than the block, so runtime locals give a "compile-time expression"
  error. `#if x.CONST` on a runtime local is the exception: only the local's type is used.
- **`#this`** is the enclosing struct, or inside a procedure body the procedure (`Scope::proc`).
- **Procedure arguments**: a polymorphic procedure passed to a procedure-typed parameter is instantiated from
  the parameter's types (`instantiate_for_proc_type`). A parenthesized parameter type such as `(*Vector3)` is a
  procedure type returning nothing (`parse_param_type`). `v = a, b, c` gives a variadic parameter its list.
- **Baking** (`bake.rs`): `T = Type` where `T` is a `$T` variable instantiates the baked copy. Constants baked
  into a polymorphic struct are stored in `PolyStruct::baked` and become members of every instance.
- **`#modify`** (`modify.rs`): after inference, the block runs in the interpreter with each type variable as a
  mutable `Type` variable (globals read back afterwards). Unbound variables start as `void`; `return false`
  rejects the call (`return false, "why";` adds the message to the error; it is stored in the `modify.message`
  global). Result bindings replace the inferred ones.
- **Struct `#modify`** (`run_struct_modify`, called from `instantiate_struct`): every struct parameter, type or
  value, is an assignable variable (`if N < 8 N = 8;`); the final values are the instance key, so `Holder(3, T)`
  and `Holder(8, T)` can be the same type. Test: `tests/stdlib/struct-modify.jai`.
- **Operators**: `a op= b` calls `operator op=` for struct-like targets (`try_operator_assign`); `*x[i]` calls
  `operator *[]`.
- **Aliases**: `name :: overloaded;` resolves to `Resolved::ProcSet`. `#poke_name Module name;` copies the
  visible declarations into the module's scope (`apply_pokes`, run after `expand_all`). An alias whose value is
  a name (`starts_with :: begins_with;`) joins an overload set from another scope like a procedure does: the
  lookup resolves it on demand (`overloadable_or_alias` in `scope.rs`).
- **Mixed arithmetic**: an integer value meeting a float in a binary operator converts to that float
  (`binary_operand_type` picks the float type; `binary` casts the integer side).
- **Pending top-level items** (`#if`, `#insert`, `#run`-driven declarations) expand lazily when a lookup reaches
  their scope (`expand_pending`). An item that fails while some procedure body is mid-lowering (its
  compile-time code may need that body) is put back to waiting and recorded in `Compiler::deferred_pending`;
  `expand_all` retries it once nothing is lowering, and only then is a failure final.
- **Indexing**: `x[i]` calls a matching `operator []`, else `operator *[]` and dereferences the result
  (an assignable place); a pointer with no fitting operator indexes memory (`try_pointer_index_operator`).
- **Polymorphic struct parameters**: a bare `Base` / `*Base` parameter (and `$T/Base`) accepts a struct whose `#as` member is
  an instance of `Base` (`instance_or_as_base`); the call converts through the `#as` offset.
- **Baked parameter defaults**: `$mode: Mode = .fast` and `$info := Info.{}` give the default (and `.X` /
  `.{...}` arguments) the declared or default type. A declared type naming earlier type variables
  (`$compare: (T, T) -> bool = (a, b) => a == b`) is read in a scratch scope holding the bindings so far.
- **Conversions**: `*[..] T` → `*[] T`; a one-character string constant → `u8`; an `ifx` argument fits an
  overload only if both branches do.
- **Tagged unions** (`union kind: Kind { .A ,, a: X; }`): laid out as a struct holding the tag field, then an
  anonymous union of the members (`layout_struct_inner`). Tags on members are parsed but not enforced.
  Type info flattens it: `members` lists the tag, then each variant at its offset; `textual_flags` has
  `UNION | UNION_IS_TAGGED`; `tagged_union_bindings` pairs each tag value with its member index
  (`typeinfo.rs`, `tagged_union_members`).
- **`#poke_name Module name`** adds the importer's entity itself to the module scope, so both see one type.
- **Member aliases in structs**: `using,only(width, height) texture.desc;` (also `except` / `map` / no
  filter) in a struct body, where the path starts at a field, makes those members of the nested field
  reachable on the struct (`Compiler::member_aliases`, followed last by `find_member`). No storage is added.
- **Struct field types in constants**: while a struct is laid out, earlier field types are recorded in
  `Compiler::field_types`, so `type_of(field)` works in a struct constant (`FnCtx::type_only`).
- **Macros**: constants declared in a macro body exist once per expansion (`hoisted_consts` is keyed by scope).
  A constant argument (string, integer, bool, float) the macro never writes stays a typed constant inside the
  body (`const_macro_params`, checked textually by `text_may_write`; any `#asm` in the body counts as a write), so
  `#if n <= 1` and a constant string's `.count` work in recursive macros. A Code argument naming a `#code`
  constant passes that code; one naming a `Code` variable passes its value (a runtime `Code` local in the
  macro). An expression `#insert code` checks the code in the scope it was written in;
  `#insert,scope()` uses the insertion scope.
- **Backtick in defers**: a macro's deferred statement keeps the scope of the caller it was written for
  (`DeferEntry::caller_scope`), so `` `name `` inside it resolves where the defer runs (`FnCtx::backtick_scope`).
- **Overload ties**: candidates with equal conversion cost prefer non-polymorphic procedures, then the
  polymorphic header whose parameter types pin down more structure (`pattern_specificity`: `*$T` beats `$T`).
- **`#insert (break=..., continue=..., remove=...) body`**: inside the inserted for-loop body, `break` /
  `continue` / `remove` aimed at that loop run the replacement at the insertion site (`InsertReplacements`,
  `try_insert_replacement`).
- **For loops**: `for *=cond` / `for <=cond` take compile-time flags; `remove it` works in reverse loops too.
  A struct iterated through `for_expansion` is passed by address unless an overload takes that very type by value.
- **Arguments**: a multi-value call passed as one argument gives its first value (spreading is the fallback);
  `null` binds a type variable only when no other argument does (`*void`); a defaulted baked parameter whose
  type names that variable (`$is_equal: (T, T) -> bool = null`) is evaluated after the `null` binding
  (`deferred_defaults` in `infer_bindings`, `tests/stdlib/null-poly-baked-default.jai`); `#char` binds `u8`; `.A | .B` and
  `xx a + 1` are deferred like `.A` and `xx a`.
- **Procedure values**: a procedure type may take a parameter's type from its default
  (`(s: string, start := 0) -> s64`). Calls through a procedure value use the names and defaults recorded for its
  type (`Compiler::proc_type_params`, from a procedure-type header or a procedure used as a value).
- **`ifx`**: a numeric then-value widens to an expected numeric type; an overload set takes the expected or
  else-value's procedure type; a branch may be a block whose last expression is the value.
- **`using`** on an expression whose value is a pointer (`using editors.active_pane;`) binds the pointer itself.
- **Compile-time constants** (`eval_const`) run with the compile-time Context, so `#assert` can call procedures
  that take one. `type_of(local.*)` in compile-time code only needs the local's type.
- **Thunks and queued bodies**: `drain_bodies_lenient` lowers what it can before a thunk runs; bodies that fail
  (they need a layout still in progress) stay queued and are reported by the final `drain_bodies`. A body whose
  failure is memoized for the current `lower_epoch` is parked (`parked_bodies`), so later thunks in the same
  epoch skip the whole group instead of looking at each body; `drain_bodies` and a new epoch put them back.
  `drain_bodies` itself retries failed bodies while other bodies still lower, since a later body may declare
  what an earlier one uses (`#insert,scope(Top)`); only a pass with no progress reports its first error.
  Progress means a body that was queued when the pass began lowered: a failing body re-creates its nested
  procedures on each retry, and counting those once made the loop run forever.
- **Auto-dereference** (`convert.rs`): `*Thing` converts to a `Thing` value (structs only, one level), also to
  an `#as` member's value. Poly patterns (`Base`, `$T/Base`, `Base($T)`) accept `*Instance` the same way.
- **Types naming parameters**: a parameter or result type that mentions an earlier parameter
  (`proc: cache.Proc`, `-> type_of(asset)`) is checked with the parameters in a scratch scope
  (`procs.rs`, `type_from_params`; a text scan of the type span decides).
- **`#bake_arguments` structs as restrictions**: `$V/Vec3` with `Vec3 :: #bake_arguments Vector(N = 3)` accepts
  instances of the origin whose parameters match the baked values (`instance_or_as_base`).
- **Float literal types** (`float_literal_type` in `expr.rs`): a float literal is an untyped `float32` constant
  unless it has more than 7 significant figures and `float32` cannot hold it exactly (`12342345234.0`,
  `3.14159265358979`); then it defaults to `float64`. Either still converts to an expected float type.
- **Literal overloads**: between `float32` and `float64` overloads an untyped literal picks `float32`
  (`log(2)`); integer parameters still win over floats.
- **`!=` fallback**: with no matching `operator !=`, `a != b` is `!(a == b)` through `operator ==`.
- **`Type.field`** names the field's type, so `Anim.joints.Load_Factor` reaches its constants. Reading a
  runtime local's field in compile-time code is still an error (`names_field` keeps the local shortcuts to
  constants), so a field read cannot bind a baked parameter.
- **Enum bodies**: `#insert -> string { ... }` inside an enum adds members (parsed as an enum body).
- **`push_context,defer_pop ctx;`** holds the context for the rest of its block (`check_block_stmts`).
- **Named for-expansion iterators**: `for slot, _ : list` hides the expansion's own `it_index` from the body,
  so an enclosing `it_index` stays visible (`insert_for_body`). A for_expansion may forward its body
  (`for_expansion(*inner, body, flags)`): the inserting macro's `it` / `it_index` are borrowed into the
  loop scope for that insertion.
- **Parenthesized types**: `x: (*T);` and `#type (*T)` are procedure types (`parse_param_type`).
- **`#insert,scope(Top)`** where `Top :: #code()` is top-level: the code (also a string) is checked in that
  file's scope and its constants are declared there, so the metaprogram sees them as top-level declarations.
- **Waiting for `#placeholder`s**: resolving an undefined placeholder bumps `placeholder_misses`. A pending
  top-level item, top-level `#run` or `#assert` whose failure reached one stays waiting (a metaprogram may
  define the name at `TYPECHECKED_ALL_WE_CAN`) and is retried at the next settle; `finish_program` sets
  `placeholders_final` and settles once more, so a placeholder never defined is still an error.
- **`using _ :: struct {...}`** at top level may repeat: each gets a hidden entity name (`__using_N`).
- **Declared structs are always checked**: sema is demand-driven, so a struct nothing needs would never be
  laid out. Jai type-checks every declaration (its dead-code elimination only skips *procedure bodies*, and
  only in modules by default), so after the reachable code is lowered `finish_program` lays out every
  top-level, non-polymorphic struct and union in every loaded file and module (`check_declared_structs` in
  `driver.rs`). A member typed by an undefined name (`a: Missing;`, `[4] Missing`, `*Missing`,
  `(x: Missing) -> s32`) is reported as `unknown identifier` at the member, whether the struct is used, only
  reached through `*S` / `type_info` / `size_of`, or not used at all. Polymorphic structs are checked per
  instance, when one is made; structs declared inside procedure bodies are checked with their body. Not
  covered: unused top-level constants and globals that are not structs (`T :: Missing;`, `g: Missing;`).
- **Failed type descriptors are not cached**: `type_info_global` registers a descriptor's global before
  building it (descriptors refer to themselves). If the build fails, the type is unregistered again
  (`failed_type_infos` keeps the global for the retry), so a retried body reports the layout error instead of
  getting an empty descriptor (`runtime_size` 0, no members) from the cache.

- Compile-time pointer constants that are plain integers (handle-like values such as `cast(*void) 32512` or `cast(HANDLE) -1`) are frozen as raw values by `freeze_pointer` in `sema/consteval.rs` instead of erroring with "unknown size".
- **Overloaded / polymorphic procedure arguments**: for a procedure-typed polymorphic parameter
  (`f: (T) -> $R`), an overload set or polymorphic procedure/lambda is resolved after the other bindings are known
  (`deferred_procs` in `infer_bindings`, `proc_for_param_types`), then `$R` is bound from the chosen instance.
- **Named variadics**: after `v = x`, further positional arguments extend the variadic until a later named argument
  (`assign_slots`).
- **Pointer for_expansion parameters**: `for :iter CONST` / any non-pointer iterable passed to a `*T` expansion
  parameter is spilled to a temporary and its address taken (`stmt.rs`, `wants_pointer`). `utf8_iter` follows jai: it
  backtick-declares `it` / `it_index` (character count; `-1` backwards).
- **Aggregate baked arguments**: `$a: [$N] float` bound to `.[...]` infers `N` from the literal's own type.
- **`type_of(poly_proc)`** gives a procedure type with `void` for type-variable parameters (`poly_proc_type`), for
  compile-time inspection only.
- **Proc vs `*void`**: `proc == ptr` compares addresses (`binary_operand_type` in `expr.rs`).

## How to change it

New operator forms belong next to `try_operator_assign`. To check more kinds of unused declarations
eagerly, extend the filter in `check_declared_structs`; anything it resolves must not depend on code that
only a metaprogram adds later (it runs after `placeholders_final`). Negative tests for it are the
`tests/corpus/negative/undefined-*.jai` cases. New `#modify` features go in `run_modify_block`
(shared by procedures and structs); it builds one IR function per call, so keep per-call cost in mind.

## Configuration

None.

## Dependencies

`interp` (compile-time execution), `parser/decl.rs` (mixed declaration lists), `calls.rs` (candidate matching).
- **`$$x` auto-bake parameters**: the parser sets `Param::auto_bake`. In `match_candidate` (`calls.rs`), a constant argument
  (`Operand::is_const`, a proc) selects `auto_bake_variant`, a copy of the procedure whose `$$` params are turned into
  `$` ones, cached per (proc, `AutoBake` mask) so `#procedure_of_call` identity is per baked value. Non-constant calls
  use the original proc (`is_constant(x)` false). Constants with no `Value` (`"s".data`, `*global`, `type_info(T)`,
  constant pointer arithmetic: `is_constant_pointer`) keep the param at runtime but bind a hidden `$const:x` marker
  that `is_constant` reads. Two procedure names now compare by address (`expr.rs`).
