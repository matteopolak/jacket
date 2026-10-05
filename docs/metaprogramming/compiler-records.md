# Compiler records (messages, syntax trees, type descriptors)

## What it is

The bridge that lets metaprograms see a workspace's code: `Message_File`, `Message_Import`,
`Message_Typechecked`, the `Code_*` nodes, the `Type_Info_*` descriptors of the *target* program, plus
`compiler_get_nodes` and `compiler_modify_procedure`.

## How it works

- **Why records.** A metaprogram runs in its own compiler's interpreter, which borrows that compiler, and the
  target workspace's types are not the metaprogram's types. So the target's compiler exports neutral
  *records* (`crates/jaic/src/records.rs`): a tag naming a Jai struct (`"Code_Ident"`, `"Type_Info_Struct"`,
  `"Message_Import"`) and named fields — `Int` (floats as bits), `Str`, `Ref` (record id, 0 = null) or a list
  of those. Ids are global to the compilation and records live until it ends.
- **Rust exporter** (`crates/jaic/src/sema/code_export.rs`, state in `Compiler::export`):
  - `export_file_events`: an IMPORT record per module (before its first file) and a FILE record per file.
    Module types: Preload → PRELOAD, Runtime_Support → RUNTIME_SUPPORT, main → MAIN_PROGRAM, others
    UNINITIALIZED. Hidden fields `__module`/`__file` name the module for `add_build_string(.., message)`.
  - `export_typechecked`: every new top-level declaration of a *user* module (not Preload/Runtime_Support,
    not under the system module directory = the directory of `Runtime_Support.jai`) is resolved and exported
    with its type; procedure headers, bodies and struct literals inside it are collected into
    `procedure_headers`/`procedure_bodies`/`structs`, every node made into `subexpressions`. Declarations that
    fail to resolve (or procedures whose signature fails) are skipped, as Jai would report an error instead.
  - Global declarations, procedure parameters/returns (from the signature) and type instantiations that
    evaluate get `type`; local declarations and other expressions do not. Notes after a procedure body
    attach to the declaration and the header. `Code_Node.serial` is the record id.
  - `type_record`: Type_Info records, memoized per `TypeId`. Primitive types carry `__builtin` (their name) so
    the metaprogram uses its own `type_info(int)` etc. — `decl.type == type_info(int)` works. Polymorphic
    struct instances get `specified_parameters`, `constant_storage` and a generic `polymorph_source_struct`;
    `Type` parameters are pointers patched by the Jai side from `__constant_pointers`.
- **Events**: `build::step` takes the record table out of the registry while exporting (resolving may run
  compile-time code) and queues IMPORT/FILE/TYPECHECKED events whose `ints[0]` is the message record.
- **Jai side** (`stdlib/Compiler/records.jai`): `record_struct(id, expected)` allocates the struct named by the
  tag (`RECORD_TYPES`, falling back to the expected pointee type), zero-fills it and fills members by name
  through `Type_Info_Struct`: ints/enums/bools/floats by size, strings, pointers (recursively, memoized so a
  record is one pointer forever), in-place structs and arrays of any of those. `using`/`#as using` members and
  anonymous unions are filled from the same record. `record_of(pointer)` maps back.
- **Speed**: a large metaprogram builds hundreds of thousands of records (Focus: 300k), so the member walk is
  split. `record_plan(info)` flattens a struct type's members once (offset, kind, size, name, type; cached by
  `Type_Info_Struct` pointer). `__jaic_rec_fill` then writes the ints, strings and pointers to records already
  built in one native call, and returns a bit mask of the members Jai must still fill (arrays, in-place
  structs, records not built yet). `__jaic_rec_fill_list` does the same for an array's elements.
  `__jaic_rec_tag_id` numbers tags so `record_type_of` caches the tag's struct type. The pointer-to-record
  index behind `record_of` is built lazily, on the first `record_of` call. Strings handed to Jai point into
  the records' own `Rc<[u8]>` (kept alive in `Workspaces::kept`); tags are static text.
- **compiler_get_nodes**: `Compiler::add_code` mirrors every `Code` value (AST + source snippet) into
  `Interp::codes`; `__jaic_code_nodes` exports it and stores the snippet as `__source` on the root. The Jai side
  remembers each (root, code) pair it hands out (`__code_roots` in `workspace.jai`).
- **Typed nodes (trap and run again)**: modern libraries (or_return, MetaThreadSafe, Jai-Shader-Transpiler) read
  `Code_Procedure_Call.resolved_procedure_expression` (the callee's header, with `arguments`/`returns` and their
  `type_inst.result`), `Code_Ident.resolved_declaration` and `Code_Node.type`. Those need the compiler, which the
  MetaOp cannot reach while the interpreter runs. So `call_thunk` (`sema/consteval.rs`) counts the run's
  observable effects (`Interp::effects`: output, foreign calls other than memory/string helpers, workspace ops);
  while the count is unchanged since the run started, `__jaic_code_nodes` traps with `Interp::export_request`.
  `call_thunk` then exports that code with the compiler (`Compiler::export_code_typed` → `export_code_in` with
  `Some(compiler)`) into `Interp::code_exports` and runs the thunk again from the start. A run that already did
  something observable gets untyped nodes instead. Each `compiler_get_nodes` call takes its own export (a list
  per code, `code_export_cursor` reset on every run), so in-place edits never leak into another call.
  - Names resolve locals first (`Exporter::locals`, pushed by declarations and parameters, scoped per block),
    then `Compiler::lookup` from the code's scope. A declaration resolves to a cached reference record
    (`ExportState::resolved_decls`) with `type`, `type_inst.result` and, for procedures, `expression` = a
    header (`resolved_headers`). Builtin procedures (`size_of`...) resolve to nothing.
  - Calls pick the only candidate, else the first overload whose parameter count fits.
  - Type records in these exports carry `__address`: the real descriptor in this program's memory, so
    `get_type(x.type_inst.result)` and `type == type_info(T)` work in the same program.
  - Gotcha: a run that writes globals before `compiler_get_nodes` writes them again when it reruns (only I/O
    and foreign calls count as effects).
- **On-demand lowering**: a compile-time run inside another body's lowering (`Compiler::lowering_depth > 0`,
  e.g. a `#modify` or `#run` in a macro) must not drain every queued body: one of them may be the procedure
  being lowered. Instead `lower_reachable_from(thunk)` lowers only the queued bodies the thunk's IR reaches
  (calls, function addresses, globals' relocations), and the context global does the same. A call that still
  hits a body-less function traps with `Interp::missing_func`; `call_thunk` lowers it (`lower_with_callees`) and
  runs again, under the same no-effects rule.
- **compiler_get_code**: prints the node tree with `Program_Print` and passes the text to `__jaic_parse_code`,
  which parses it (`build::parse_code_text`) into a new entry of `Interp::codes` and records in
  `Interp::made_codes` which code's scope it takes (`code_to_copy_scope_from`). The compiler adopts such codes
  when it reads a `Code` value back from compile-time code or adds a code of its own
  (`Compiler::adopt_made_codes`), re-parsing the text as a registered source so diagnostics can point into it.
  Node edits the printer cannot express (see [program-print](../specs/stdlib/README.md)) are lost.
  - Without `code_to_copy_scope_from` the code is *unscoped* (`Compiler::unscoped_codes`): it resolves names at
    the insertion site (or the `#insert,scope(target)` scope), as in Jai — yield-jai builds
    `(self: *COROUTINE) -> ...` where only the inserting macro knows `COROUTINE`. Names the site lacks fall back
    on the scopes of the codes handed to `compiler_get_nodes` (newest first, `Interp::nodes_codes`), because Jai
    nodes keep what they resolved to where they were written (Epic_Fail's `#code` blocks call
    `print_to_builder`, which the inserting user never imported). `Compiler::code_scope_at` makes a block scope
    under the site with `Scope::fallbacks`; `lookup_full` consults fallbacks only when nothing else binds a name.
- **Expression types**: from compile-time code, value expressions (operators, literals, members, subscripts,
  casts) get `type` by checking them in the code's scope without emitting (`Exporter::expr_type`); untyped
  literals report their default type (`s64`, `float64`). Identifiers and calls get theirs from the resolved
  declaration or header.
- **Literal and flag details**: `-1` and `-1.5` export as one negative `Code_Literal` (Jai folds them; yield-jai
  overwrites `_s64` of `#code case -1`). Backticked identifiers set `Code_Ident.flags.HAS_SCOPE_MODIFIER` and
  backticked declarations `Code_Declaration.flags.HAS_SCOPE_MODIFIER`; `#assert`/`#run` statements export as
  `Code_Directive_Run` (with jaic's extra `expression`/`message`), `#exists(x)` as `Code_Directive_Exists`.
- **compiler_modify_procedure**: for each statement of `body.block.statements`, Jai sends its record id, or 0
  and its text when the statement is new or was edited in place anywhere below it (`record_differs` in
  `records.jai` compares every member with the record, as `fill_struct` wrote it, following `Code_*` pointers
  but not `resolved_*`/`type`; union members overlapping a present field are skipped). Text comes from
  `Program_Print`. The modifications are queued on the workspace and applied at its next step
  (`Compiler::modify_procedure`): unchanged statements map back to their AST, `#code` roots are re-parsed from
  `__source` and edited ones from their text, inside a dummy procedure. The procedure's `ProcLit` is replaced
  and, if the body was already lowered, it is lowered again into the same function after the registry borrow
  is released (`relower_modified`).
- **Node kinds**: every `Code_*` struct sets its kind (`base.kind = .IDENT;`), so nodes a metaprogram makes with
  `New(Code_Ident)` print and compare like exported ones. `a, b := f()` is a `Code_Compound_Declaration`: the
  names (declarations, or identifiers for `=` targets) in `comma_separated_assignment`, the shared type and
  value(s) in a nameless `declaration_properties`.
- **TYPECHECKED for procedures**: a procedure whose body is not lowered yet is reported with a null
  `body_or_null` and queued in `ExportState::pending_bodies`. Each later `export_typechecked` reports the
  queued bodies that have been lowered since (records carry local declaration types) and patches the header's
  `body_or_null`. Bodies the program never reaches are never lowered, so their errors never surface — as in Jai.
  - Exception: procedures with notes (`@glsl`, `@thread`, header or after the body) whose headers went out are
    lowered leniently when the workspace runs out of sources (`lower_reachable_inner`), because metaprograms
    find shaders and checked procedures by note whether or not anything calls them (Jai-Shader-Transpiler).
  - The Jai side (`workspace.jai`, TYPECHECKED) also sets `body.header.body_or_null` on the cached header
    struct: a metaprogram may hold that header from the earlier message (MetaThreadSafe checks bodies at
    COMPLETE).
  - One header record per procedure: `ExportState::resolved_headers` maps a procedure to the header record
    that both `resolved_procedure_expression` and the reported header use, so a checker following calls
    reaches bodies. The key includes `Exporter::own`: a compiler's own compile-time code (`#modify`) needs type
    records with real descriptors, which message exports for a metaprogram do not have
    (`tests/stdlib/compiler-header-own-types.jai`, or_return's shape). Resolved headers carry the procedure's notes, including notes after the body
    (`Compiler::proc_decl_notes`, copied to polymorph instances).
  - Headers list `using` parameters in `parameter_usings` (a `Code_Using` of an ident resolved to the
    argument). In `x := value` the value takes the declaration's lowered type when it has none.
- **Enums**: a top-level enum declaration's `Code_Enum.external_type` is its `Type_Info_Enum`.
- **Phases**: when a workspace runs out of sources, `build.rs::step` first lowers everything reachable
  (`Compiler::lower_reachable`, lenient: a failing body stays queued). If that reports new declarations or bodies (a body may declare more through
  `#insert,scope(...)`), the metaprogram gets another `TYPECHECKED_ALL_WE_CAN` and may add code; only a
  lowering round that reports nothing new ends in `finish_program` and code generation.

## How to change it

- New node kind: add a case in `Exporter::expr`/`stmt` building a record with the `Code_*` member names, and
  add the struct to `RECORD_TYPES` in `records.jai` if it is reached through a base pointer.
- New field: just add it to the record — the Jai filler matches names, unknown names are ignored, missing
  fields stay zero. Keep names identical to `stdlib/Compiler/nodes.jai`/`stdlib/prelude/module.jai`.
- Only top-level statement lists can be modified; nested blocks would need their statement records mapped too.
- Gotcha: anything that resolves in the exporter can run compile-time code; never hold the registry borrow.

## Configuration

None. Exports happen only for intercepted workspaces.

## Dependencies

`sema` (entity resolution, signatures, `eval_type`, struct layouts), `build.rs` (events, `__jaic_rec_*`,
`__jaic_code_nodes`, `__jaic_modify_procedure`, `__jaic_workspace_add_string_to_module`), the parser (re-parsing
inserted code). Tests: `tests/stdlib/compiler-typechecked-messages.jai`, `tests/stdlib/compiler-resolved-nodes.jai`,
`tests/stdlib/compiler-get-code.jai`.
