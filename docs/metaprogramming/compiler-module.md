# Compiler module (Jai side)

## What it is

`stdlib/Compiler/` is our implementation of Jai's public `Compiler` module (`#import "Compiler"`). Metaprograms use it to create workspaces, set `Build_Options`, add source, intercept compiler messages and report diagnostics. It is plain Jai over a set of bodiless `#compiler` primitives (`__jaic_*`) that the Rust side binds by name; the Rust half is described in [workspaces](workspaces.md).

## How it works

`module.jai` loads:

| File | Content |
| --- | --- |
| `messages.jai` | `Message` and the `Message_*` records, `Typechecked(T)` and the report records. Pure data; layout and member order follow the public API. |
| `syntax_nodes.jai` | The `Code_Node` family for declarations, expressions and statements, `Operator_Type`, `Provided_Import_Type`. Pure data. |
| `directive_nodes.jai` | The `Code_Directive_*` nodes for `#` directives. Pure data. |
| `options.jai` | `Build_Options` (with `Commonly_Propagated`), `Llvm_Options`, `Build_Options_During_Compile`, `set_optimization`, `copy_commonly_propagated_fields`, the per-workspace option store and forwarding to Rust. |
| `workspace.jai` | Workspaces, `add_build_file/string`, interception and message construction, `compiler_report`, version info, stubs, and the primitive declarations. |
| `records.jai` | Turning compiler records into Jai structs; see [compiler records](compiler-records.md). |

### Options

Jai owns `Build_Options` storage: `option_store` is indexed by workspace id and filled lazily by `ensure_option_slot`. Every workspace starts from the struct defaults {#compiler.3}, which already reflect the target (`os_target := OS`, `cpu_target := CPU`) {#compiler.1}; the metaprogram's own also gets `compile_time_command_line` from `__jaic_command_line_*` {#compiler.2}.

- `get_build_options(w)` returns the stored copy {#compiler.4}.
- `set_build_options(options, w, loc)` stores it, reports what jaic does not act on (`check_build_options`, see [build options](build-options.md)) and forwards settings with `__jaic_workspace_set_option(ws, key, value)`. Changed relative `import_path` entries are anchored at `loc`'s directory {#compiler.5}.
- `set_build_options_dc(dc, w, loc)` updates the stored copy and forwards `do_output`, `append_executable_filename_extension`, the output name and path, and one `additional_linker_argument` per `append_linker_arguments` entry {#compiler.6}.

Keys `set_build_options` sends (values are text):

| Key | Value | Sent when |
| --- | --- | --- |
| `output_executable_name`, `output_path` | string | non-empty |
| `output_type` | `NO_OUTPUT`, `EXECUTABLE`, `DYNAMIC_LIBRARY`, `STATIC_LIBRARY`, `OBJECT_FILE` | always |
| `import_path_clear`, then one `import_path` per entry | string | `import_path` changed |
| `os_target` | `Operating_System_Tag` name (`MACOS`, `LINUX`, `WINDOWS`, `WASM`, ...) | always |
| `cpu_target` | `CPU_Tag` name (`X64`, `ARM64`, ...) | always |
| `optimization` | `O0 O1 O2 O3 OS OZ` from `llvm_options.bitcode_optimization_setting` | not `UNSET` (use `set_optimization`) |
| `llvm_target_system_triple`, `llvm_target_system_cpu`, `llvm_target_system_features` | string | always |
| `entry_point_name`, `intermediate_path` | string (empty: `main`, next to the output) | always |
| `temporary_storage_size` | decimal | always |
| `additional_linker_arguments_clear`, then one `additional_linker_argument` each | string | the list changed |
| `append_executable_filename_extension`, `use_custom_link_command`, `disable_redzone`, `enable_frame_pointers`, `stack_trace` | `true` / `false` | always |
| `runtime_support_definitions` | `AUTO`, `ENTRY_POINT_AND_INIT`, `ONLY_INIT`, `OMIT` | changed |
| `backtrace_on_crash` | `OFF`, `ON` | always |
| `minimum_os_version` | `major.minor` | always |
| `machine_code_optimization` | `NONE`, `LESS`, `DEFAULT`, `AGGRESSIVE` | not `UNSET` |
| `llvm_enable_loop_unrolling`, `llvm_enable_loop_vectorization`, `llvm_enable_slp_vectorization`, `llvm_merge_functions`, `llvm_disable_inlining`, `llvm_enable_tail_calls`, `llvm_enable_split_modules`, `llvm_preserve_debug_info` | `true` / `false` | always |
| `output_llvm_ir`, `output_bitcode`, `output_llvm_ir_before_optimizations`, `output_bitcode_before_optimizations` | `true` / `false` | always |
| `emit_debug_info` | `NONE`, `DWARF`, `CODEVIEW`, `DEFAULT`; only `NONE` matters (turns off [debug info](../native/debug-info.md)) | always |
| `array_bounds_check` | `OFF`, `ON`, `ALWAYS` | always |
| `arithmetic_overflow_check`, `cast_bounds_check` | `OFF`, `NONFATAL`, `FATAL` | always |
| `dead_code_elimination` | `NONE`, `ALL`, `MODULES_ONLY` | changed |

`compiler_destroy_workspace` sends `destroy` (empty value). Only changed or non-empty values are forwarded for settings the Rust side defaults itself (output name, import paths, optimization, dead code elimination), so empty Jai defaults never clobber them. That is why the default `import_path` is empty {#compiler.7}. `write_added_strings` is not forwarded; [build options](build-options.md) says why.

### Messages

`compiler_begin_intercept(w, flags)` calls `__jaic_workspace_begin_intercept` and remembers `w` with its flags; `keep_typechecked` applies the `SKIP_` flags to each `TYPECHECKED` message (see [build options](build-options.md)). `compiler_wait_for_message()` walks the intercepted workspaces round-robin, starting after the one served last and skipping finished ones, calls `__jaic_workspace_next_event(ws)`, and `build_message` builds the message:

| Event | Message | Payload |
| --- | --- | --- |
| 1 FILE, 4 IMPORT, 5 TYPECHECKED | `Message_File`, `Message_Import`, `Message_Typechecked` | int 0 = the message's record id, filled by `record_struct` |
| 2 PHASE | `Message_Phase` | int 0 = phase. `PRE_WRITE_EXECUTABLE` and `READY_FOR_CUSTOM_LINK_COMMAND`: ints 1..4 = counts of object files, support objects, system and user libraries; the strings are those lists in that order, then the executable. `POST_WRITE_EXECUTABLE`: int 1 = write failed, int 2 = linker exit code, string 0 = executable |
| 3 COMPLETE | `Message_Complete` | int 0 = `error_code`; marks the workspace finished |
| 7 ERROR | `Message` | none |
| 8 DEBUG_DUMP | `Message_Debug_Dump` | string 0 = `dump_text` |
| 9 PERFORMANCE_REPORT | `Message_Performance_Report` | int 0 = the message's record id (nested records for the report sections) |
| 0 | `Message_Complete` (no error) | no more events; treated as finished |

Once every intercepted workspace has finished, later calls keep returning a `COMPLETE` for the last one, with its error code, so the usual `if m.kind == .COMPLETE break;` loop terminates {#compiler.8}.

PHASE and COMPLETE messages reuse one module-level struct per kind, so they and their strings are valid only until the next `compiler_wait_for_message()`. That matches the event payload strings, which aren't copied. `Message.workspace` is always the workspace the event came from {#compiler.9}.

### Reporting, version, stubs

- `compiler_report(message, loc, mode)` calls `__jaic_report` with the `Report` value. An `ERROR` report stops the metaprogram and fails the compile {#compiler.10}; `WARNING` and `INFO` print and do not fail it {#compiler.11}; `ERROR_CONTINUABLE` prints an error, lets the metaprogram go on and fails the build.
- `compiler_set_workspace_status(.FAILED, w)` has no primitive: unless `w` already failed it reports a fatal diagnostic so the exit code is non-zero {#compiler.12}. `.OK` on a workspace that did not fail changes nothing {#compiler.13}; on one that failed it is an error.
- `compiler_get_version_info` returns `__jaic_compiler_version()` and parses the first digit run (`"the, jaic"` gives 0, 2, 29) {#compiler.14}.
- `compiler_set_type_info_flags(type, flags)` queues the flags (`__jaic_set_type_info_flags`, `Interp.pending_type_flags`); after each `call_thunk`, `apply_type_info_flags` merges them and rebuilds the descriptor. `NO_TYPE_INFO` empties `members` {#compiler.20}; `PROCEDURES_ARE_VOID_POINTERS` reports procedure members as `*void` {#compiler.21}.
- `get_name(w)` uses names remembered by `compiler_create_workspace` {#compiler.18}. `get_runtime_info` reads `__runtime_info` {#compiler.19}; asking for another workspace's is an error. In compile-time code `get_type_table(w)` instead lists a descriptor of every type the workspace has declared or used so far, unused structs and enums included {#compiler.29}. The table is built on demand, in `refresh_type_table` (`sema/runtime_info.rs`): it resolves the plain structs and enums declared at file level (one that cannot be resolved yet is left out), gives each type a descriptor with `type_info_global` and keeps the descriptors' addresses in a vector the interpreter owns (`ct_type_table`). The running workspace's table is made when a compile-time run that can reach `get_type_table` starts; another workspace's is made at the call from its compiler, which a workspace keeps after `COMPLETE` once any compile-time code references the primitive (`__jaic_type_table`). Descriptors made only for the table stay out of the running program's own table until something else asks for them. In a compiled program `get_type_table` still returns the types the program reaches.
- An API call jaic cannot carry out reports a fatal error when it is made, never silently nothing {#compiler.17}: `add_build_string(text, w, code)` with a `Code` value, `compiler_set_workspace_status(.OK)` on a failed workspace, and `add_global_data` with `.USER_SEGMENT` but no segment.
- `remap_import(w, host, name, replacement)` makes `#import "name"` in module `host` (empty: any module) load `replacement`; the latest rule wins, and imports resolved before the call keep their module {#compiler.23}.
- `provide_import(w, message, type, value)` answers a `FAILED_IMPORT` message with a module name, a file, a directory holding `module.jai` or the source text (`Provided_Import_Type`) {#compiler.24}. jaic delivers the message after the load failed, so a workspace with a `FAILED_IMPORT` and an intercepting metaprogram is held back: it keeps the error unreported until the metaprogram next asks for a message, and if an offer arrived by then the workspace loads its sources again from scratch (new compiler, same sources) using it. An offer identical to one already made does not count (the retry would fail the same way, and a metaprogram answering every `FAILED_IMPORT` with it would loop for ever; test: `fuzz/regressions/check/provide-import-missing-file.jai`). Without an offer the error is reported, followed by `ERROR` and `COMPLETE` as before (`build::resume_failed_load`).
- `compiler_make_procedure_live(w, header)` keeps the procedure of a header the workspace sent, though nothing calls it: the header's record maps to its procedure (`Compiler::make_procedure_live`), which `lower_reachable_inner` lowers with the program's exports. It matters under `dead_code_elimination = .ALL`; the default modes already check and keep program procedures. Headers of polymorphic procedures and macros are ignored {#compiler.25}.
- `get_root_type(code)` is `SUCCESS` with the type of the typed root node (`Code_Node.type`), `INPUT_IS_CODE_NULL` for `#code,null`, and `NOT_TYPED` for code whose nodes were not typed (see [compiler records](compiler-records.md)) {#compiler.26}. `compiler_get_base_path()` is the directory above `stdlib` with a trailing slash. `add_data_segment` returns a handle and `false` for `actual_segment_will_be_created`, and `add_global_data` then places the bytes with the other constant data. `compiler_report_errors_for_unresolved_identifiers`, `compiler_report_errors_for_untyped_declarations_with_these_notes` and `compiler_set_memory_breakpoint` do nothing: jaic reports unresolved names as errors where they occur, checks every declaration of the program's own files by default, and its compile-time interpreter has no write watch {#compiler.27}.
- `compiler_add_library_search_directory(path)` adds a directory searched for `#library` names (compile-time `dlopen` and the link line), ahead of the driver's. It is process-wide, not per workspace; the directory is not added to the executable's run path, so a shared library there still has to be found by the loader at run time {#compiler.28}.

Syntax-tree APIs (`compiler_get_nodes`, `compiler_get_code`, `code_to_string`, `print_expression`, `add_global_data`, `compiler_modify_procedure`) are in [compiler records](compiler-records.md) {#compiler.15}. `add_build_string(text, w, message)` with a FILE or IMPORT message adds the text to that message's module; otherwise it goes to the workspace's top level {#compiler.16}. `add_build_string(text, w, code)` with a non-null `code` is an error (`__jaic_code_is_null`). `compiler_get_struct_location` gives the file, line and column where a struct is declared (`__jaic_struct_location`) {#compiler.22}.

## How to change it

- New forwarded option: add the field in `options.jai` (keep public member order and defaults), add a `__jaic_workspace_set_option` line in `forward_options`, handle the key in `Workspaces::set_option` in `build.rs` (unknown keys are errors), and update the table above and [build options](build-options.md).
- New event kind: an `EVENT_*` constant and a `case` in `build_message`. Prefer a record payload built with `record_struct`, which keeps pointer identity across messages, over a reused module-level struct.
- A new primitive is a `MetaOp` in `build.rs` plus a `#compiler` declaration at the end of `workspace.jai`. Keep public signatures as they are; something jaic cannot do either does nothing and says why in its comment, or reports an error through `__jaic_report`.
- Anonymous enums inside `Build_Options` and `Message_*` can't be named elsewhere, so helper state stores them as integers (`last_complete_error`) or uses the polymorphic `enum_member_name`.
- Workspace id `-1` means the current workspace (`resolve_workspace`).
- `jaic check` only type-checks code reachable from `main`, so a probe must call new procedures behind a runtime condition. `tests/stdlib/compiler-api-shapes.jai` is that probe for the option, message and stub surface; `tests/stdlib/type-info-flags.jai` covers type-info flags.

## Configuration

`MACHINE_OPTIONS_SIZE` sizes the `machine_options` layout placeholder. `jaic check file.jai -os linux|windows|macos` changes the target `OS`, and with it the default `os_target`; compile-time code still runs on the host.

## Dependencies

- The primitives at the end of `workspace.jai` and `records.jai`, bound in `MetaOp::from_name` (`build.rs`).
- Prelude types: `Workspace`, `Source_Code_Location`, `Type_Info*`, `For_Flags`, `OS`, `CPU`.
- `Basic`, imported privately.
- In-repo users: `stdlib/Default_Metaprogram.jai`, `Minimal_Metaprogram.jai`, `Print_Vars.jai`, `Icon.jai`, `Metaprogram_Plugins.jai`, `Example_Plugin.jai`, `Basic/Memory_Debugger.jai`.
