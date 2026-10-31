# Compiler module and compiler records

`stdlib/Compiler/` declares the public metaprogramming API: workspaces, build settings, compiler messages, editable syntax records, reflection and managed global data. Public `#compiler` operations run in Rust, independently of the Jai procedure bodies in the selected standard library.

## How it works

`contract-records.jai` defines the public records. `workspace.jai` declares compiler operations without bodies, except `get_type`, `get_type_table` and `get_runtime_info`. Formatting and construction helpers are Jai in `internal/compiler-api`.

- **Binding.** A procedure's `#compiler "name"` selector (or its own name) picks an `Operation` in `crates/jaic/src/build/public_api`. Binding checks the declaration against the operation's slots (`binding::slots`): kinds, pointer targets, view elements and the record fields the handler reads or writes (`Schema::require`). A declaration that does not fit is a compile error at the declaration, not a failure inside a handler, because `JAIC_STDLIB` can select another stdlib. Handlers use the resolved offsets, sizes, defaults and enums, never offsets from one bundled stdlib.
- **Dispatch.** At compile time `Hook::PublicCompiler` calls the Rust handler and the workspace engine. At runtime a supplied Jai body runs; a bodyless operation reports that it is compile-time only. Unknown operations fail explicitly.
- **Options and messages.** Rust stores full option snapshots. Defaults come from the declaration's own record. Interception advances workspaces fairly; PHASE and COMPLETE messages and their strings live until the interpreter is destroyed. Details: [build options](build-options.md).
- **Records** (`crates/jaic/src/records.rs`, `record_build.rs`). A target workspace exports neutral records keyed by id and named fields; `record_build.rs` materializes them into the caller's declared types, flattening promoted members and keeping pointer identity.
- **Code in and out.** `compiler_get_code` converts the editable public graph straight into a syntax tree (`public_api/convert.rs`), no source text or reparse. `compiler_modify_procedure` does the same for every current statement, then queues replacement and re-lowering at the next workspace step. The converter's spelled text becomes a virtual source (`compiler_get_code N`) so diagnostics quote it. Unsupported syntax is diagnosed, never silently dropped. `Program_Print` is the only printer.
- `compiler_get_nodes` may request typed export and replay a pure compile-time run; after observable effects it uses the untyped graph. `Operation::is_effect` decides what counts as an effect.

## How to change it

Add or extend an operation in `build/public_api/mod.rs`, its slots in `binding::slots`, and its handler in `operations.rs`, `options.rs`, `messages.rs`, `records.rs` or `convert.rs`, plus `Operation::is_effect` if it only reads. What it does to workspaces belongs in `build/engine.rs`. Test both bodyless compile-time dispatch and supplied runtime bodies.

New record members need exporter fields and a declared layout, not handwritten offsets. New syntax forms need a mapping in `convert/` and, if printable, a Jai printer; keep syntax metadata keys in step ([compiler builtins](../compiler/compiler-builtins.md)).

Workspace output needs an embedder backend; `-check` and `-no_workspace_output` emit none. Tests: `stdlib/Compiler/tests`, compiler regressions, `crates/jaic-cli/tests/compiler_primitives.rs`.

## Rules

Claims that tests cite by ID (`tools/dev/check_rule_coverage.py`).

### Compiler operations

- Every workspace starts from the declaration's struct defaults {#compiler.3}, reflecting the target (`os_target := OS`, `cpu_target := CPU`) {#compiler.1}; the running workspace also gets the top-level compile-time command line {#compiler.2}.
- `get_build_options(w)` returns the stored copy {#compiler.4}.
- `set_build_options(options, w, loc)` validates, forwards to the engine and snapshots the record. Relative `import_path` entries are anchored at `loc`'s directory {#compiler.5}.
- `set_build_options_dc(dc, w, loc)` updates the stored copy and forwards `do_output`, `append_executable_filename_extension`, the output name and path, and one `additional_linker_argument` per `append_linker_arguments` entry {#compiler.6}.
- Destroying a workspace releases its engine state. Empty declaration defaults do not replace engine-derived output names and search defaults {#compiler.7}.
- Once every intercepted workspace has finished, later calls keep returning a `COMPLETE` for the last one, so `if m.kind == .COMPLETE break;` terminates {#compiler.8}.
- Message records and copied strings stay valid for the interpreter's lifetime. `Message.workspace` is the workspace the event came from {#compiler.9}.
- `compiler_report(message, loc, mode)` reports through the engine. `ERROR` stops the metaprogram and fails the compile {#compiler.10}; `WARNING` and `INFO` print and do not fail it {#compiler.11}; `ERROR_CONTINUABLE` prints an error, lets the metaprogram go on and fails the build.
- `compiler_set_workspace_status(.FAILED, w)` makes the build exit nonzero {#compiler.12}; `.OK` on a workspace that did not fail changes nothing {#compiler.13} and cannot clear a failure.
- `compiler_get_version_info` returns the version text and fills `major`, `minor` and `micro` from `LANGUAGE_VERSION` {#compiler.14}.
- Syntax-tree APIs (`compiler_get_nodes`, `compiler_get_code`, `code_to_string`, `print_expression`, `add_global_data`, `compiler_modify_procedure`) are in the records section below {#compiler.15}.
- `add_build_string(text, w, message)` with a FILE or IMPORT message adds the text to that message's module; otherwise to the workspace's top level {#compiler.16}. With a non-null `code` it adds the text in that code's scope, and the code must belong to the running workspace.
- An API call jaic cannot carry out reports a fatal error when made, never silently nothing {#compiler.17}: `add_build_string` with a `Code` from another workspace, `compiler_set_workspace_status(.OK)` on a failed workspace, and `add_global_data` with `.USER_SEGMENT` but no segment.
- `get_name(w)` uses names remembered by `compiler_create_workspace` {#compiler.18}. `get_runtime_info` reads `__runtime_info` {#compiler.19}; another workspace's is an error.
- `compiler_set_type_info_flags(type, flags)` queues flags that merge after each `call_thunk` and rebuild the descriptor. `NO_TYPE_INFO` empties `members` {#compiler.20}; `PROCEDURES_ARE_VOID_POINTERS` reports procedure members as `*void` {#compiler.21}.
- `compiler_get_struct_location` gives the file, line and column where a struct is declared {#compiler.22}.
- `remap_import(w, host, name, replacement)` makes `#import "name"` in module `host` (empty: any) load `replacement`. The latest rule wins; imports already resolved keep their module {#compiler.23}.
- `provide_import(w, message, type, value)` answers a `FAILED_IMPORT` message with a module name, a file, a directory holding `module.jai` or the source text {#compiler.24}. The message is delivered after the load failed, so a workspace with a `FAILED_IMPORT` and an intercepting metaprogram keeps the error unreported until the metaprogram next asks for a message; if an offer arrived, the sources load again from scratch using it. An offer identical to an earlier one does not count, or a metaprogram answering every `FAILED_IMPORT` the same way would loop forever. Without an offer the error is reported, then `ERROR` and `COMPLETE` (`build::resume_failed_load`).
- `compiler_make_procedure_live(w, header)` keeps the procedure of a header the workspace sent although nothing calls it. It matters under `dead_code_elimination = .ALL`. Headers of polymorphic procedures and macros are ignored {#compiler.25}.
- `get_root_type(code)` returns `SUCCESS` with the typed root, `INPUT_IS_CODE_NULL` for null code, or `NOT_TYPED` {#compiler.26}.
- `compiler_get_base_path()` is the directory above `stdlib`. `add_data_segment` returns a managed handle and `false` for an actual named section. Compiler memory breakpoints and debugger requests fail explicitly {#compiler.27}.
- `compiler_add_library_search_directory(path)` adds a directory searched for `#library` names (compile-time `dlopen` and the link line), ahead of the driver's. It is process-wide, and not added to the executable's run path {#compiler.28}.
- In compile-time code `get_type_table(w)` lists a descriptor of every type the workspace has declared or used so far, unused structs and enums included {#compiler.29}. It is built on demand by `refresh_type_table` (`sema/runtime_info.rs`) from the plain structs and enums declared at file level, and kept in the interpreter. In a compiled program it returns the types the program reaches.

### Exported records

- `export_file_events` emits an IMPORT record per module before its first file {#records.1} and a FILE record per file {#records.2}. Module types: Preload is `PRELOAD`, Runtime_Support `RUNTIME_SUPPORT`, the main module `MAIN_PROGRAM`, the rest `UNINITIALIZED` {#records.3}.
- `export_typechecked` resolves and exports every new top-level declaration of a user module with its type {#records.4}. User means not Preload or Runtime_Support and not under the system module directory. Headers, bodies and struct literals go into `procedure_headers`, `procedure_bodies` and `structs`, and every node into `subexpressions`. Declarations that fail to resolve are skipped.
- What a TYPECHECKED message carries depends on what the sources can read (`Contents`, `build/drive.rs`). If no source names `Message_Typechecked`, the lists are empty. If some do but none can reach a procedure body, bodies are left out (headers have a null `body_or_null`), which is the largest part of the export. A source reaches bodies by naming `procedure_bodies`, `body_or_null`, `subexpressions`, `Code_Procedure_Body`, `PROCEDURE_BODY`, `.all` or `all` (`SourceMap::reads_procedure_bodies`). Left-out bodies are exported later if such a source turns up. The check looks at words, so printing a whole message with reflection does not count, and `Code_Procedure_Header.serial` shifts when bodies are skipped.
- Global declarations, parameters, returns and type instantiations get `type`; local declarations and other expressions do not {#records.5}. A variadic parameter's `type_inst` has `inst_flags = .VARARGS` (Check's `@PrintLike` looks for it). Notes after a procedure body attach to the declaration and the header {#records.6}. `Code_Node.serial` is the record id.
- `type_record` makes `Type_Info` records, memoised per `TypeId`. Primitive types carry `__builtin` so `decl.type == type_info(int)` works {#records.7}. Polymorphic struct instances get `specified_parameters`, `constant_storage` and a generic `polymorph_source_struct` {#records.8}.
- The record builder resolves declared member layouts, flattening promoted fields; a record maps to one pointer {#records.9}.
- Libraries like or_return, MetaThreadSafe and Jai-Shader-Transpiler read `resolved_procedure_expression`, `resolved_declaration` and `Code_Node.type` {#records.10}. Those need the compiler, which a compiler operation cannot reach while the interpreter runs, so jaic traps and reruns.
- Type records carry `__address`, the real descriptor in this program, so `type == type_info(T)` works {#records.11}.
- Rust converts the editable node graph into a new `Code`; names resolve in the copied scope {#records.12}.
- Value expressions get `type` by checking them in the code's scope without emitting. One that reads a local gets none. Untyped literals report their default (`s64`, `float32`) {#records.13}.
- `-1` and `-1.5` export as one negative `Code_Literal` {#records.14}.
- Backticked identifiers and declarations set `HAS_SCOPE_MODIFIER` {#records.15}. `#assert` and `#run` export as `Code_Directive_Run` {#records.16}, `#exists(x)` as `Code_Directive_Exists` {#records.17}.
- Procedure edits convert every current statement, including edits to existing records, and are queued until the next workspace step; the body is replaced and re-lowered, so the modified body runs {#records.18}.
- Public node defaults set the node kind for user-created records {#records.19}.
- `a, b := f()` is a `Code_Compound_Declaration`: names in `comma_separated_assignment`, shared type and values in a nameless `declaration_properties` {#records.20}.
- A top-level enum's `Code_Enum.external_type` is its `Type_Info_Enum` {#records.26}.
- A procedure whose body is not lowered yet is reported with a null `body_or_null` and queued; each later `export_typechecked` reports bodies lowered since and patches the header, including the cached Jai-side struct. Imported modules' unreachable bodies are never lowered, so their errors never surface. The program's own unreferenced bodies are checked once the workspace has finished ([dead-code elimination](../language/dead-code-elimination.md)), and one more TYPECHECKED message carries them right before the closing phases and COMPLETE. Declarations of file-level names carry `IS_GLOBAL`, and `SCOPE_FILE` below a `#scope_file` {#records.28}.
- Typed-export reruns replay a code when its conversion read the same values (a fingerprint of every field and attribute read) from the same scope, rather than allocating endlessly {#records.29}.
- Noted procedures (`@glsl`, `@thread`, on the header or after the body) whose headers went out are lowered leniently once the workspace runs out of sources, because metaprograms find shaders and checked procedures by note whether or not anything calls them. Only procedures in the main module qualify {#records.22}. An imported module's uncalled noted procedures stay unchecked {#records.23}, since imported modules ship stale ones.
- `ExportState::resolved_headers` keeps one header per procedure, shared by `resolved_procedure_expression` and the reported header, so a checker following calls reaches bodies {#records.24}. Resolved headers carry the procedure's notes, including those after the body, and list `using` parameters in `parameter_usings` {#records.25}.
- When a workspace runs out of sources, `step` lowers everything reachable (`lower_reachable`, lenient: a failing body stays queued). If that reports new declarations or bodies, the metaprogram gets another `TYPECHECKED_ALL_WE_CAN` and may add code; only a round that reports nothing new proceeds to code generation {#records.27}.
- A call to a polymorphic procedure resolves to the instance it made (`call_instances`, `instance_called_at`) {#records.30}. The instance's header has no `POLYMORPHIC` flag, and its body is reported later, in the scope of its bindings.
- A static `#if` in such a body has `static_if_flags` `EVALUATED_AS_TRUE` when the condition held, and a static `if x == {` switch has `static_if_accepted_case`, so a checker can follow only the branch the compiler took. In a generic body neither is set {#records.31}.
- Declarations of file-level names carry `IS_GLOBAL`, and `SCOPE_FILE` below a `#scope_file` {#records.32}.
