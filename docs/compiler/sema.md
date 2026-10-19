# Semantic analysis

## What it is

Implementation notes for `crates/jaic/src/sema`: module loading, top-level expansion, declarations, polymorphic calls, baking, `#modify`, macros and the end-of-compile declaration check. User-facing behaviour is in the [language docs](../README.md#language) and [modules and imports](../language/modules-and-imports.md); this page covers where things live and why.

## How it works

### Modules and top-level expansion (`modules.rs`, `scope.rs`)

- `load_module` keys a module by entry path and parameter values. The first `#module_parameters (...)` list is part of the instance key; the second holds program parameters (`Module.program_params`) shared by every instance. `#import "Basic"()(MEMORY_DEBUGGER = DEBUG)` records a setting in `Compiler::program_param_settings`; `apply_program_params` patches the declaration value as soon as both instance and setting exist, before anything reads it. See [module parameters](../language/module-parameters.md).
- **All member lookup goes through `module_lookup`** (plain `#import`, `M.name`, `using M;`, re-exports). Separate per-path lookups once diverged (`GL.glViewport` failed while `glViewport` worked), so a new kind of module member goes into `module_lookup`, with a line in `tests/compiler/regressions/module-member-resolution.jai`. A module-level `using global;` outside `#scope_file` is recorded in `Module.exported_usings`; an exported `using X :: #import "Y";` re-exports `Y` (`exported_using_imports`, cycle-guarded). `module_declarations` is only for fixed compiler names (`Context`, `__arithmetic_overflow`). A re-export applies the originating import's `only`/`except` filter.
- `expand_all` first expands imports and `#if` items whose condition is a plain constant (`expand_plain_ifs`: no calls, no `#run`), so a module's `#if FLAG #load "x.jai"` and `#add_context` land before a `#run` lays out the Context. While checking a condition `lookup_without_expansion` is set so a lookup never runs compile-time code early. Later lookups expand lazily (`expand_pending`), skipped when the scope already binds the name to a non-overloadable entity.
- `expand_all` runs again after each source a metaprogram adds, visiting only scopes in `Compiler::unsettled` (without this, a metaprogram adding code each round was quadratic). Queue items with `push_pending` / `push_import`, never directly onto `Scope::pending`/`imports`, or a retired scope never sees them.
- An `#add_context` after the Context type exists but before layout is appended to its struct; after layout it is an error.
- An item that fails while a body is mid-lowering is retried (`deferred_pending`); if the program fails first, the error carries the deferred item's error as a note (`with_deferred_errors`).
- Import search: `Options::import_paths` is `<main file dir>/modules`, `-I` directories, then the stdlib (built in `crates/jaic-cli/src/main.rs`); `find_module` tries `<importing file dir>/modules` first.

### Declarations

- Locals (`check_local_decl`, `check_decl_values` in `stmt.rs`): `a, b := 1, "x"` pairs values (`Decl::extra_values`); `a=, b := ...` assigns to names marked `Decl::existing`; a non-name place (`ok:, t.str = f()`) is in `Decl::targets`. Local constants are hoisted by `check_block_stmts`.
- `#run`, `#assert` and constant initialisers are checked in a thunk scope one procedure level deeper, so using a runtime local is an error (except `#if x.CONST` and `type_of(local.*)`).
- `name :: overloaded;` resolves to `Resolved::ProcSet`; an alias to a name in another scope joins an overload set (`overloadable_or_alias`).
- `#poke_name Module name;` adds the importer's entity to the module scope after `expand_all` (`apply_pokes`).
- `using,only(a, b) field.desc;` in a struct body makes nested members reachable without storage (`Compiler::member_aliases`, consulted last by `find_member`).
- A parameter or result type mentioning an earlier parameter (`proc: cache.Proc`, `-> type_of(asset)`) is checked with the parameters in a scratch scope (`type_from_params`).
- **Declared structs are always checked.** Sema is demand-driven, but Jai type-checks every declaration, so `check_declared_structs` (`driver.rs`) lays out every top-level non-polymorphic struct and union after reachable code is lowered; `a: Missing;` is reported whether or not the struct is used. Unused constants, globals and bodies are checked afterwards in the program's own files only (all files with `-no_dce`; [dead-code elimination](../language/dead-code-elimination.md)). `type_info_global` registers a descriptor before building it because descriptors refer to themselves; on failure the type is unregistered (`failed_type_infos`).

### Calls and polymorphism

`calls/` is split by phase: `mod.rs` checks a call and ranks candidates (`check_call`, `call_procs`, `arg_cost`), `infer.rs` binds `$T` and matches type patterns, `baking.rs` builds baked parameter values, `emit.rs` lowers the chosen call and its arguments, and `intrinsics.rs`, `builtins.rs`, `macro_expand.rs` and `operators.rs` handle those kinds of callee. Binary operators are in `binary.rs`: `check_binary` checks the operands (`binary_operands`), folds or dispatches to an overload, does pointer arithmetic (`pointer_offset`, `mask_pointer`), then converts both sides (`unify_binary_operands`) and emits (`emit_binary`).

- A polymorphic procedure passed to a procedure-typed parameter is instantiated from the parameter's types (`instantiate_for_proc_type`); for `f: (T) -> $R` the overload set resolves after other bindings (`deferred_procs`), then `$R` binds.
- `null` binds a type variable only when no other argument does (as `*void`); a baked default naming that variable is evaluated afterwards (`deferred_defaults`). `#char` binds `u8`. `$a: [$N] float` bound to `.[...]` infers `N`.
- **Baked variadics** (`$types: ..Type`) become one constant `[] T` (`baked_pack`) deduplicated by contents (`Compiler::packs`). Instance keys go through `instance_key_value`, which adds a constant view's relocation targets to its bytes; without it `f(A)` and `f(G)` shared one instance.
- A bare `Base`, `*Base` or `$T/Base` parameter accepts a struct whose `#as` member is a `Base` (`instance_or_as_base`).
- **Overload ties**: equal conversion cost prefers non-polymorphic procedures, then more structure (`pattern_specificity`: `*$T` beats `$T`). An untyped literal between `float32` and `float64` picks `float32`; integers beat floats.
- `type_of(poly_proc)` gives a procedure type for inspection only (`poly_proc_type`; type variables are `TypeKind::PolyParam`, tag `POLYMORPHIC_VARIABLE`). A polymorphic struct used as a value is `TypeKind::PolyStruct`, a template, never a layout.
- **`$$x` auto-bake**: `Param::auto_bake`; a constant argument selects `auto_bake_variant`, cached per procedure and `AutoBake` mask. Constants with no `Value` (`"s".data`, `*global`) stay runtime parameters but bind a hidden `$const:x` marker that `is_constant` reads.
- `bake.rs`: `T = Type` instantiates the baked copy; constants baked into a polymorphic struct live in `PolyStruct::baked`.
- **`#modify`** (`modify.rs`, `run_modify_block`): after inference the block runs in the interpreter with each type variable as a mutable `Type`; unbound ones start as `void`. `return false, "why";` rejects and adds the message. A `#modify` with nothing to modify is an error of kind `ModifyWithoutPolymorphs`, returned as is by `call_procs`. Struct `#modify` (`run_struct_modify`) makes every struct parameter assignable and the final values form the instance key. It builds one IR function per call, so watch per-call cost.

### Operators, conversions, macros, loops

- `a op= b` calls `operator op=` for struct targets (`try_operator_assign`); `x[i]` tries `operator []`, then `operator *[]` (`try_pointer_index_operator`).
- `convert.rs` auto-dereferences `*Thing` to a `Thing` value (structs, one level); an `ifx` argument fits an overload only if both branches do.
- Constants declared in a macro body exist once per expansion (`hoisted_consts`). A constant macro argument the macro never writes (`text_may_write`; any `#asm` counts) stays a typed constant so `#if n <= 1` works in recursive macros. A macro's deferred statement keeps its caller scope (`DeferEntry::caller_scope`).
- `#insert (break=..., continue=..., remove=...) body` runs the replacement at the insertion site; `#insert,scope(Top)` with a top-level `Top :: #code()` declares its constants in that file's scope.
- `check_for` lowers loops rotated: the bound is tested once before the body and again after each step. A range loop tests `it` against the bound before stepping so a bound at the type's limit cannot wrap; an array loop reloads the count each step so `array_add`/`remove it` work. `tests/compiler/regressions/for-loop-bounds.jai` covers the edges.
- A struct iterated through `for_expansion` is passed by address unless an overload takes it by value (`wants_pointer` in `stmt.rs`).

### Unreachable code (`reachability.rs`, `interval.rs`)

The editor fades code that can never run. Two parts, both only with `Compiler::ide` set and only for recorded files.

- While a body is checked, `ide_note_ainfo` gives every integer or bool expression an `AInfo` (`IdeFacts::ainfo`): the `Interval` its operands allow, from the type's range, constants (the checker's own evaluation), arithmetic, bit ops, shifts, casts and `!`/`&&`/`||`. Anything that may wrap in the result type, or that mixes a negative with an unsigned type, falls back to the type's range. `interval.rs` holds the domain (`compare`, `binary`, `unary`) with no compiler dependency, so a lint such as "condition always true" or a bounds-check proof can reuse it.
- `warn_unreachable` (called once per checked body) walks the statements. Dead: code after `return`/`break`/`continue`, after an `if`/`switch` whose paths all exit, after a loop whose condition is always true and that no reached `break` leaves; the branch of an `if` whose condition has one value, a `while` body that never starts, a `case` its value cannot select. Inside a branch the walk narrows the locals the condition constrains, but only locals never address-taken or named by a macro (`addr_taken`) and not assigned in that branch (`assigned`). `defer` bodies are read but are not "after" anything. The first statement after a direct `return`/`break`/`continue` still gets Jai's warning.
- A polymorphic body is walked once per instance: `body_evals` counts walks and `dead_runs` how many found a run dead; only runs dead in every walk are shown. `#if` branches and `ifx` arms use `static_branches` (`ide_note_ran`), where a branch is faded only if no evaluation ran it. `if`/`while` on a constant are decided by the same walk, not by special cases.

- After `if c { exits }` (or `if c {} else { exits }`) the rest of the block is narrowed by `!c` (`c`), under the same write rules. A `for` over a range with `from > to` is a dead body.
- **Procedures that never return** (`never_returns.rs`). Once per editor analysis (end of `ide_check_all`) a fixed point finds the procedures whose every path ends in a call to one that never returns; the roots are the body-less `#foreign` declarations of the C runtime's process and thread exits (`NEVER_RETURNING_SYMBOLS`: `exit`, `_exit`, `_Exit`, `quick_exit`, `abort`, `exit_group`, `pthread_exit`, `ExitProcess`, `ExitThread`), so the stdlib's `exit` and everything that ends in it follow. The read is syntactic (every branch possible, only `while true` without a `break` diverges), the callee of each call comes from `IdeFacts::callees` (`None` where instances of a polymorphic procedure resolved a call differently), and a procedure that expanded a macro which returns from its caller (`macro_returners`) is never in the set. The statements after such a call in each analysed body are added to `dead_runs` (counted `body_evals` times), which is what the editor fades and `constant_condition` consults. To teach it another primitive, add its C symbol to `NEVER_RETURNING_SYMBOLS`. The batch compiler is unchanged: "not all control paths return a value" still follows the reference rules.
- The branch a constant `if` rules out is still checked in an editor analysis (`check_dead_for_facts`), lowered into an unreachable block with its errors and warnings dropped, so hover, definition and tokens work there. The batch compiler never checks it. Constant `ifx` arms are already checked by both. Inactive `#if` branches are not checked, only tokenized by the syntax layer.
- `warn_unreachable` returns whether the end of the body is reachable. With an analysis on, "not all control paths return a value" is also dropped when it is not (batch output is unchanged: `stmts_return` keeps the reference compiler's rules, e.g. a `while` whose body returns counts as returning).

To extend: a new condition form needs an `AInfo` rule in `compute_ainfo` and the same in `eval`; narrowing is `narrow`/`constrain`. Unknown must stay "no information", never a guess. No procedure is known not to return, so calls never end a block.

### Pending and deferred work

- **Placeholders**: resolving an undefined `#placeholder` bumps `placeholder_misses`. Pending items, `#run`s and `#assert`s whose failure reached one keep waiting (a metaprogram may define the name at `TYPECHECKED_ALL_WE_CAN`) and retry at the next settle. `finish_program` sets `placeholders_final` and settles once more, so a never-defined placeholder is still an error. A failed top-level `#insert` always waits until then.
- **Queued bodies**: `drain_bodies_lenient` lowers what it can before a thunk runs; failures stay queued for `drain_bodies`, which retries while other bodies still lower (a later body may declare what an earlier uses) and reports the first error after a pass with no progress. Progress counts only bodies queued when the pass began, or a failing body re-creating nested procedures would loop forever. A body whose failure is memoised for the current `lower_epoch` is parked (`parked_bodies`).
- Compile-time constants (`eval_const`) run with the compile-time Context, so `#assert` can call procedures that need one.

## How to change it

- Per-module state goes on `Module` (`mod.rs`). Anything that must be visible before compile-time code runs belongs in the plain pass of `expand_all`, which must stay free of calls.
- New operator forms go next to `try_operator_assign`; new `#modify` features in `run_modify_block`. A new binary-operator rule goes in the `check_binary` phase it belongs to, not in front of all of them.
- `StructInfo::fields` is an `Rc<[Field]>`: clone it (a reference-count bump) to walk the fields while calling `&mut self` methods.
- To check more kinds of unused declarations, extend `check_declared_structs`; what it resolves must not depend on code a metaprogram adds later. Negative tests: `tests/corpus/negative/undefined-*.jai`.
- Tests: `tests/compiler/regressions/module-member-resolution.jai`, `static-switch-add-context.jai`, `struct-modify.jai`, `modify-polymorph-forms.jai`, `tests/corpus/negative/modify-without-polymorphs*.jai`.

## Configuration

`Options::import_paths` (above). No other settings.

## Dependencies

The parser, `consteval.rs` (conditions and `#run`), `structs.rs` (Context layout), `interp` (compile-time execution), `calls/mod.rs` (candidate matching).
