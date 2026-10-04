# Declarations, polymorphism and `#modify` in sema

## What it is

Notes on the `crates/jaic/src/sema` pieces that handle multi-value declarations, `#this`, polymorphic
procedure arguments, `#bake_constants`, `#modify`, `#poke_name` and a few operator forms.

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
  rejects the call. Result bindings replace the inferred ones.
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
- **Polymorphic struct parameters**: a bare `Base` / `*Base` parameter accepts a struct whose `#as` member is
  an instance of `Base` (`instance_or_as_base`); the call converts through the `#as` offset.
- **Baked parameter defaults**: `$mode: Mode = .fast` and `$info := Info.{}` give the default (and `.X` /
  `.{...}` arguments) the declared or default type.
- **Conversions**: `*[..] T` → `*[] T`; a one-character string constant → `u8`; an `ifx` argument fits an
  overload only if both branches do.
- **Tagged unions** (`union kind: Kind { .A ,, a: X; }`): laid out as a struct holding the tag field, then an
  anonymous union of the members (`layout_struct_inner`). Tags on members are parsed but not enforced.
- **`#poke_name Module name`** adds the importer's entity itself to the module scope, so both see one type.
- **Struct field types in constants**: while a struct is laid out, earlier field types are recorded in
  `Compiler::field_types`, so `type_of(field)` works in a struct constant (`FnCtx::type_only`).
- **Macros**: constants declared in a macro body exist once per expansion (`hoisted_consts` is keyed by scope).
  A constant string argument the macro never writes keeps a constant `.count` (`const_string_params`, checked
  textually by `text_may_write`). An expression `#insert code` checks the code in the scope it was written in.
- **`#insert (break=..., continue=..., remove=...) body`**: inside the inserted for-loop body, `break` /
  `continue` / `remove` aimed at that loop run the replacement at the insertion site (`InsertReplacements`,
  `try_insert_replacement`).
- **For loops**: `for *=cond` / `for <=cond` take compile-time flags; `remove it` works in reverse loops too.
  A struct iterated through `for_expansion` is passed by address unless an overload takes that very type by value.
- **Arguments**: a multi-value call passed as one argument gives its first value (spreading is the fallback);
  `null` binds a type variable only when no other argument does (`*void`); `#char` binds `u8`; `.A | .B` and
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
  (they need a layout still in progress) stay queued and are reported by the final `drain_bodies`.

## How to change it

New operator forms belong next to `try_operator_assign`. New `#modify` features (messages, value variables) go
in `run_modify`; it builds one IR function per call, so keep per-call cost in mind.

## Dependencies

`interp` (compile-time execution), `parser/decl.rs` (mixed declaration lists), `calls.rs` (candidate matching).
