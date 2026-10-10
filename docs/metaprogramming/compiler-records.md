# Compiler records (messages, syntax trees, type descriptors)

## What it is

The bridge that lets a metaprogram see another workspace's code: `Message_File`, `Message_Import`, `Message_Typechecked`, the `Code_*` nodes, the target program's `Type_Info_*` descriptors, and `compiler_get_nodes`, `compiler_get_code` and `compiler_modify_procedure`.

## How it works

### Why records

A metaprogram runs in its own compiler's interpreter, which borrows that compiler, and the target workspace's types are not the metaprogram's types. So the target's compiler exports neutral records (`crates/jaic/src/records.rs`): a tag naming a Jai struct (`"Code_Ident"`, `"Type_Info_Struct"`, `"Message_Import"`) and named fields, each an `Int` (floats as bits), `Str`, `Ref` (record id, 0 = null) or a list of those. Ids are global to the compilation, and records live until it ends.

### Rust exporter

`sema/code_export.rs`, with state in `Compiler::export`:

- `export_file_events`: an IMPORT record per module before its first file {#records.1}, and a FILE record per file {#records.2}. Module types: Preload is `PRELOAD`, Runtime_Support `RUNTIME_SUPPORT`, the main module `MAIN_PROGRAM`, the rest `UNINITIALIZED` {#records.3}. Hidden `__module`/`__file` fields let `add_build_string(.., message)` find the module.
- `export_typechecked`: every new top-level declaration of a user module is resolved and exported with its type {#records.4}. User means not Preload or Runtime_Support and not under the system module directory (the directory of `Runtime_Support.jai`). Procedure headers, bodies and struct literals inside go into `procedure_headers`, `procedure_bodies` and `structs`, and every node into `subexpressions`. Declarations that fail to resolve are skipped, where Jai would report an error.
- Global declarations, parameters and returns, and type instantiations get `type` {#records.5}; local declarations and other expressions don't. A variadic parameter's `type_inst` has `inst_flags = .VARARGS` (`mark_varargs`, in both the reported header and the resolved one); Check's @PrintLike test looks for it before the format string. Test: `tests/stdlib/code-export-varargs-flag.jai`. Notes after a procedure body attach to the declaration and the header {#records.6}. `Code_Node.serial` is the record id.
- `type_record` makes Type_Info records, memoised per `TypeId`. Primitive types carry `__builtin` (their name) so the metaprogram substitutes its own descriptor and `decl.type == type_info(int)` works {#records.7}. Polymorphic struct instances get `specified_parameters`, `constant_storage` and a generic `polymorph_source_struct` {#records.8}; `Type` parameters are pointers the Jai side patches from `__constant_pointers`.
- A top-level enum's `Code_Enum.external_type` is its `Type_Info_Enum` {#records.26}.
- `-1` and `-1.5` export as one negative `Code_Literal`, as Jai folds them (yield-jai overwrites `_s64` of `#code case -1`) {#records.14}.
- Backticked identifiers and declarations set `HAS_SCOPE_MODIFIER` {#records.15}. `#assert` and `#run` statements export as `Code_Directive_Run` (with jaic's extra `expression` and `message`) {#records.16}, `#exists(x)` as `Code_Directive_Exists` {#records.17}.
- `a, b := f()` is a `Code_Compound_Declaration`: the names in `comma_separated_assignment`, the shared type and values in a nameless `declaration_properties` {#records.20}.

`build::step` takes the record table out of the registry while exporting (resolving can run compile-time code) and queues IMPORT, FILE and TYPECHECKED events whose int 0 is the message record. A typed `compiler_get_nodes` export (`Compiler::export_code_typed`) does the same. `Records::lend` leaves the registry an empty part that draws ids from the same counter, so compile-time code run meanwhile (a pending `#insert` that a lookup expands, which may itself call `compiler_get_nodes`) makes records with new ids; `Records::give_back` merges them. Ids must never repeat: the Jai side builds each record's struct once and keeps it by id (`record_structs`), so a reused id hands code another record's struct.

### Procedures in TYPECHECKED

A procedure whose body isn't lowered yet is reported with a null `body_or_null` and queued in `ExportState::pending_bodies`. Each later `export_typechecked` reports the bodies lowered since, with local declaration types, and patches the header's `body_or_null`. The Jai side also patches the cached header struct, since a metaprogram may still hold it from the earlier message (MetaThreadSafe checks bodies at COMPLETE). An imported module's bodies that nothing reaches are never lowered, so they are never reported and their errors never surface, matching Jai's default dead-code elimination {#records.28}. The program's own unreferenced bodies are type-checked once the workspace has finished ([dead-code elimination](../language/dead-code-elimination.md)); `build::step` then sends one more TYPECHECKED message with those bodies right before the closing phases and COMPLETE, so a metaprogram that walks `procedure_bodies` also sees procedures nothing calls (it can no longer add code in reaction to them). Declarations of file-level names carry `IS_GLOBAL`, and `SCOPE_FILE` below a `#scope_file` {#records.32}.

Exception: noted procedures (`@glsl`, `@thread`, on the header or after the body) whose headers went out are lowered leniently once the workspace runs out of sources (`lower_reachable_inner`), because metaprograms find shaders and checked procedures by note whether or not anything calls them (Jai-Shader-Transpiler). Only procedures in the main module qualify {#records.22}. An imported module's uncalled noted procedures stay unchecked {#records.23}, since imported modules ship stale ones: Vk-Engine's `@PrintLike FormatToCString` calls a procedure that doesn't exist.

`ExportState::resolved_headers` keeps one header record per procedure, shared by `resolved_procedure_expression` and the reported header, so a checker following calls reaches bodies {#records.24}. The key includes `Exporter::own`: a compiler's own compile-time code (`#modify`) needs type records with real descriptors, which exports for a metaprogram don't have. Resolved headers carry the procedure's notes, including those after the body (`Compiler::proc_decl_notes`, copied to polymorph instances), and list `using` parameters in `parameter_usings` {#records.25}.

A call to a polymorphic procedure resolves to the instance it made, not the generic procedure: sema records the instances each call span made (`ExportState::call_instances`, filled in `match_candidate`), and `Exporter::instance_called_at` picks the one belonging to the resolved procedure {#records.30}. The instance's header has no `POLYMORPHIC` flag, and its body is reported in a later TYPECHECKED message once lowered, exported in the scope of its bindings. A static `#if` there has `static_if_flags` `EVALUATED_AS_TRUE` when the condition held, and a static `if x == {` switch has `static_if_accepted_case`, so a checker can follow only the branch the compiler took (MetaThreadSafe does) {#records.31}. In a generic body the condition is unknown and neither is set.

### Phases

When a workspace runs out of sources, `step` lowers everything reachable (`Compiler::lower_reachable`, lenient: a failing body stays queued). If that reports new declarations or bodies (a body can declare more through `#insert,scope(...)`), the metaprogram gets another `TYPECHECKED_ALL_WE_CAN` and may add code {#records.27}. Only a round that reports nothing new proceeds to `finish_program` and code generation.

### Jai side

`stdlib/Compiler/records.jai`: `record_struct(id, expected)` allocates the struct the tag names (`RECORD_TYPES`, else the expected pointee type), zero-fills it, and fills members by name through `Type_Info_Struct`: integers, enums, bools and floats by size, strings, pointers (recursively and memoised, so a record maps to one pointer forever {#records.9}), in-place structs, and arrays of those. `using` members and anonymous unions are filled from the same record. `record_of(pointer)` maps back. Every `Code_*` struct sets its own kind (`base.kind = .IDENT;`), so nodes a metaprogram makes with `New(Code_Ident)` behave like exported ones {#records.19}.

Large metaprograms build hundreds of thousands of records, so filling is split:

- `record_plan(info)` flattens a struct type's members once, cached per `Type_Info_Struct`.
- `__jaic_rec_fill` writes ints, strings and pointers to already-built records in one native call and returns a bit mask of members Jai must still fill (arrays, in-place structs, records not built yet). `__jaic_rec_fill_list` does the same for array elements. `__jaic_rec_tag_id` numbers tags so `record_type_of` can cache the struct type.
- Structs come from 256 KiB zeroed chunks (`record_memory`) and are never freed, like Jai's nodes. Array data is allocated separately because metaprograms grow those arrays with `array_add`.
- The pointer-to-record index behind `record_of` is built on its first call.
- Strings point into the records' own `Rc<[u8]>`, kept alive in `Workspaces::kept`.

### `compiler_get_nodes` and typed nodes

`Compiler::add_code` mirrors every `Code` value (AST plus source snippet) into `Interp::codes`. `__jaic_code_nodes` exports it, storing the snippet as `__source` on the root; the Jai side remembers each (root, code) pair in `__code_roots`.

Libraries like or_return, MetaThreadSafe and Jai-Shader-Transpiler read `Code_Procedure_Call.resolved_procedure_expression`, `Code_Ident.resolved_declaration` and `Code_Node.type` {#records.10}. Those need the compiler, which a `MetaOp` can't reach while the interpreter runs. So jaic traps and reruns:

1. `call_thunk` (`sema/consteval.rs`) counts the run's observable effects (`Interp::effects`: output, foreign calls other than memory and string helpers, workspace operations).
2. While nothing observable has happened yet, `__jaic_code_nodes` traps with `Interp::export_request`.
3. `call_thunk` exports the code with the compiler (`export_code_typed`, then `export_code_in`) into `Interp::code_exports` and reruns the thunk from the start.

A run that already did something observable gets untyped nodes. Each `compiler_get_nodes` call takes its own export (`code_export_cursor` resets every run), so in-place edits never leak into another call.

Gotcha: a run that writes globals before `compiler_get_nodes` writes them again on the rerun; only I/O and foreign calls count as effects.

In typed exports:

- Names resolve to locals first (`Exporter::locals`, scoped per block), then `Compiler::lookup` from the code's scope. A declaration resolves to a cached reference record (`resolved_decls`) with `type`, `type_inst.result` and, for procedures, a header as `expression`. Builtins like `size_of` resolve to nothing.
- Calls pick the only candidate, else the first overload whose parameter count fits.
- Value expressions get `type` by checking them in the code's scope without emitting (`Exporter::expr_type`); untyped literals report their default (`s64`, `float32`) {#records.13}.
- Type records carry `__address`, the real descriptor in this program's memory, so `type == type_info(T)` works {#records.11}.
- In `x := value`, the value takes the declaration's type when it has none.

### On-demand lowering

A compile-time run inside another body's lowering (`lowering_depth > 0`, such as a `#modify` or a `#run` in a macro) must not drain every queued body, since one of them may be the procedure being lowered. `lower_reachable_from(thunk)` lowers only the bodies the thunk's IR reaches (calls, function addresses, globals' relocations). A call that still reaches a bodiless function traps with `Interp::missing_func`; `call_thunk` lowers it (`lower_with_callees`) and reruns, under the same no-effects rule.

### `compiler_get_code`

Prints the node tree with `Program_Print` and passes the text to `__jaic_parse_code`, which parses it (`build::parse_code_text`) into a new `Interp::codes` entry and records in `Interp::made_codes` whose scope it takes (`code_to_copy_scope_from`). The compiler adopts such codes (`Compiler::adopt_made_codes`) when it reads a `Code` back or adds one of its own, re-parsing the text as a registered source so diagnostics can point into it. Node edits the printer can't express are lost (see [Program_Print](../specs/stdlib/README.md)). Names in the result resolve where the original code was written {#records.12}. A compile-time run that is repeated to serve a typed `compiler_get_nodes` replays the attempt before it: `Interp::run_codes` lists the codes the run made, in order, and the repeat's n-th `compiler_get_code` gets the n-th of them back (when its text and scope are the same) instead of a new code whose nodes it would ask for again. So `code_to_string(compiler_get_code(root))` in one expression finishes {#records.29}. Each call outside a repeat makes a code of its own, as each `#code` does.

Without `code_to_copy_scope_from` the code is unscoped (`Compiler::unscoped_codes`) and resolves at the insertion site, as in Jai; yield-jai builds `(self: *COROUTINE) -> ...` where only the inserting macro knows `COROUTINE`. Names the site lacks fall back to the scopes of codes handed to `compiler_get_nodes` (newest first, `Interp::nodes_codes`), because official nodes keep what they resolved to where they were written: Epic_Fail's `#code` blocks call `print_to_builder`, which the inserting user never imported. `Compiler::code_scope_at` makes a block scope under the site with `Scope::fallbacks`, and `lookup_full` consults fallbacks only when nothing else binds a name.

### `compiler_modify_procedure`

For each statement of `body.block.statements`, Jai sends the record id, or 0 plus the `Program_Print` text when the statement is new or was edited anywhere below. `record_differs` in `records.jai` compares every member with what `fill_struct` wrote, following `Code_*` pointers but not `resolved_*` or `type`, and skipping union members that overlap a present field.

The edit is queued and applied at the workspace's next step (`Compiler::modify_procedure`): unchanged statements map back to their AST, `#code` roots are re-parsed from `__source`, edited ones from their text, inside a dummy procedure. The modified body is what runs {#records.18}. The `ProcLit` is replaced, and if the body was already lowered it is lowered again into the same function once the registry borrow is released (`relower_modified`).

## How to change it

- New node kind: a case in `Exporter::expr`/`stmt` building a record with the `Code_*` member names, and an entry in `RECORD_TYPES` if it is reached through a base pointer.
- New field: add it to the record. The Jai filler matches by name, ignores unknown names and leaves missing fields zero. Keep names identical to `stdlib/Compiler/nodes.jai` and `stdlib/prelude/module.jai`.
- Only top-level statement lists can be modified; nested blocks would need their statement records mapped too.
- Gotcha: anything the exporter resolves can run compile-time code, so never hold the registry borrow.

Tests: `tests/stdlib/compiler-typechecked-messages.jai`, `compiler-resolved-nodes.jai`, `compiler-get-code.jai`, `compiler-get-code-rerun.jai`, `compiler-noted-module-procs.jai`, `compiler-header-own-types.jai`.

## Configuration

None. Exports happen only for intercepted workspaces.

## Dependencies

`sema` (resolution, signatures, `eval_type`, layouts), `build.rs` (events and the `__jaic_rec_*`, `__jaic_code_nodes`, `__jaic_modify_procedure`, `__jaic_workspace_add_string_to_module` primitives), and the parser for re-parsing inserted code.
