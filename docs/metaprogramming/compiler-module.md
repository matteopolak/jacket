# Compiler module (Jai side)

## What it is

`stdlib/Compiler/` is our implementation of Jai's public `Compiler` module (`#import "Compiler"`). Metaprograms use it to create workspaces, set `Build_Options`, add source, intercept compiler messages and report diagnostics. It is plain Jai over a set of bodiless `#compiler` primitives (`__jaic_*`) that the Rust side binds by name; the Rust half is described in [workspaces](workspaces.md).

## How it works

`module.jai` loads:

| File | Content |
| --- | --- |
| `nodes.jai` | `Message` and the `Message_*` records, `Typechecked(T)`, the `Code_Node` family, `Operator_Type`, `Provided_Import_Type`. Pure data; layout and member order follow the public API. |
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

`compiler_destroy_workspace` sends `destroy` (empty value). Only changed or non-empty values are forwarded for settings the Rust side defaults itself (output name, import paths, optimization, dead code elimination), so empty Jai defaults never clobber them. That is why the default `import_path` is empty {#compiler.7}. `write_added_strings` and `null_pointer_check` are not forwarded; [build options](build-options.md) says why.

### Messages

`compiler_begin_intercept(w, flags)` calls `__jaic_workspace_begin_intercept` and remembers `w` with its flags; `keep_typechecked` applies the `SKIP_` flags to each `TYPECHECKED` message (see [build options](build-options.md)). `compiler_wait_for_message()` walks the intercepted workspaces round-robin, starting after the one served last and skipping finished ones, calls `__jaic_workspace_next_event(ws)`, and `build_message` builds the message:

| Event | Message | Payload |
| --- | --- | --- |
| 1 FILE, 4 IMPORT, 5 TYPECHECKED | `Message_File`, `Message_Import`, `Message_Typechecked` | int 0 = the message's record id, filled by `record_struct` |
| 2 PHASE | `Message_Phase` | int 0 = phase. `PRE_WRITE_EXECUTABLE` and `READY_FOR_CUSTOM_LINK_COMMAND`: ints 1..4 = counts of object files, support objects, system and user libraries; the strings are those lists in that order, then the executable. `POST_WRITE_EXECUTABLE`: int 1 = write failed, int 2 = linker exit code, string 0 = executable |
| 3 COMPLETE | `Message_Complete` | int 0 = `error_code`; marks the workspace finished |
| 0 | `Message_Complete` (no error) | no more events; treated as finished |

Once every intercepted workspace has finished, later calls keep returning a `COMPLETE` for the last one, with its error code, so the usual `if m.kind == .COMPLETE break;` loop terminates {#compiler.8}.

PHASE and COMPLETE messages reuse one module-level struct per kind, so they and their strings are valid only until the next `compiler_wait_for_message()`. That matches the event payload strings, which aren't copied. `Message.workspace` is always the workspace the event came from {#compiler.9}.

### Reporting, version, stubs

- `compiler_report(message, loc, mode)` calls `__jaic_report` with the `Report` value. An `ERROR` report stops the metaprogram and fails the compile {#compiler.10}; `WARNING` and `INFO` print and do not fail it {#compiler.11}; `ERROR_CONTINUABLE` prints an error, lets the metaprogram go on and fails the build.
- `compiler_set_workspace_status(.FAILED, w)` has no primitive: unless `w` already failed it reports a fatal diagnostic so the exit code is non-zero {#compiler.12}. `.OK` on a workspace that did not fail changes nothing {#compiler.13}; on one that failed it is an error.
- `compiler_get_version_info` returns `__jaic_compiler_version()` and parses the first digit run (`"the, jaic"` gives 0, 2, 29) {#compiler.14}.
- `compiler_set_type_info_flags(type, flags)` queues the flags (`__jaic_set_type_info_flags`, `Interp.pending_type_flags`); after each `call_thunk`, `apply_type_info_flags` merges them and rebuilds the descriptor. `NO_TYPE_INFO` empties `members` {#compiler.20}; `PROCEDURES_ARE_VOID_POINTERS` reports procedure members as `*void` {#compiler.21}.
- `get_name(w)` uses names remembered by `compiler_create_workspace` {#compiler.18}. `get_runtime_info` and `get_type_table` read `__runtime_info` {#compiler.19}; asking for another workspace's is an error.
- These keep their public signatures but call `unsupported()`, a fatal report when invoked {#compiler.17}: `get_root_type`, `compiler_make_procedure_live`, `compiler_report_errors_for_*`, `compiler_set_memory_breakpoint`, `compiler_add_library_search_directory`, `compiler_get_base_path`, `remap_import`, `provide_import`, `add_data_segment`.

Syntax-tree APIs (`compiler_get_nodes`, `compiler_get_code`, `code_to_string`, `print_expression`, `add_global_data`, `compiler_modify_procedure`) are in [compiler records](compiler-records.md) {#compiler.15}. `add_build_string(text, w, message)` with a FILE or IMPORT message adds the text to that message's module; otherwise it goes to the workspace's top level {#compiler.16}. `add_build_string(text, w, code)` with a non-null `code` is an error (`__jaic_code_is_null`). `compiler_get_struct_location` gives the file, line and column where a struct is declared (`__jaic_struct_location`) {#compiler.22}.

## How to change it

- New forwarded option: add the field in `options.jai` (keep public member order and defaults), add a `__jaic_workspace_set_option` line in `forward_options`, handle the key in `Workspaces::set_option` in `build.rs` (unknown keys are errors), and update the table above and [build options](build-options.md).
- New event kind: an `EVENT_*` constant and a `case` in `build_message`. Prefer a record payload built with `record_struct`, which keeps pointer identity across messages, over a reused module-level struct.
- When a primitive for an unsupported API exists, replace the `unsupported(...)` body. Don't change the signature.
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
