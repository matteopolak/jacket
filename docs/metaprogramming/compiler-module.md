# Compiler module (Jai side)

## What it is

`stdlib/Compiler/` is the independently authored implementation of Jai's public `Compiler` module (`#import "Compiler"`). Metaprograms use it to create workspaces, set `Build_Options`, add source, intercept compiler messages and report diagnostics. It is plain Jai on top of a small set of `#compiler` primitives (`__jaic_*`) that the Rust side binds by name.

## How it works

The module is split into three files, all loaded by `module.jai`:

| File | Content |
| --- | --- |
| `nodes.jai` | `Message` and every `Message_*` record, `Typechecked(T)`, the `Code_Node` family, `Operator_Type`, `Provided_Import_Type`. Pure data; layout and member order follow the public API. |
| `options.jai` | `Build_Options` (with the `Commonly_Propagated` using-struct), `Llvm_Options`, `Build_Options_During_Compile`, `Optimization_Type`, `set_optimization`, `copy_commonly_propagated_fields`, the per-workspace option store and the forwarding to Rust. |
| `workspace.jai` | Workspaces, `add_build_file/string`, interception and message construction, `compiler_report`, version info, location helpers, stubs, and the primitive declarations. |

### Options

Jai owns `Build_Options` storage: `option_store` is an array indexed by workspace id, filled lazily by `ensure_option_slot`. Workspace 1 (the metaprogram's own) starts from the struct defaults, which already reflect the target through `os_target := OS` / `cpu_target := CPU`, plus `compile_time_command_line` collected from `__jaic_command_line_*`. New workspaces start from the defaults.

- `get_build_options(w)` returns the stored copy.
- `set_build_options(options, w, loc)` stores the copy and forwards settings with `__jaic_workspace_set_option(ws, key, value)`. Relative `import_path` entries that changed are anchored at the directory of `loc`.
- `set_build_options_dc(dc, w)` updates the stored copy and forwards `do_output` and the output fields.

Keys sent by `set_build_options` (in this order; values are text):

| Key | Value | Sent when |
| --- | --- | --- |
| `output_executable_name` | string | non-empty |
| `output_path` | string | non-empty |
| `output_type` | `NO_OUTPUT`, `EXECUTABLE`, `DYNAMIC_LIBRARY`, `STATIC_LIBRARY`, `OBJECT_FILE` | always |
| `import_path_clear` | empty | `import_path` differs from the previously stored value |
| `import_path` | one entry per call | same condition, one call per entry after the clear |
| `os_target` | `Operating_System_Tag` member name (`MACOS`, `LINUX`, `WINDOWS`, `WASM`, ...) | always |
| `cpu_target` | `CPU_Tag` member name (`X64`, `ARM64`, `CUSTOM`, ...) | always |
| `optimization` | `llvm_options.bitcode_optimization_setting` member name: `O0 O1 O2 O3 OS OZ` | not `UNSET` (set it with `set_optimization`) |
| `entry_point_name` | string | non-empty |
| `temporary_storage_size` | decimal | always |
| `additional_linker_arguments_clear` | empty | `additional_linker_arguments` differs from the stored value |
| `additional_linker_argument` | one entry per call | same condition |
| `write_added_strings` | `true` / `false` | always |
| `stack_trace` | `true` / `false` | always |
| `emit_debug_info` | `NONE`, `DWARF`, `CODEVIEW`, `DEFAULT` (only `NONE` matters: it turns [native debug info](../native/debug-info.md) off) | always |
| `array_bounds_check` | `OFF`, `ON`, `ALWAYS` | always |
| `null_pointer_check` | `OFF`, `ON` | always |
| `arithmetic_overflow_check` | `OFF`, `NONFATAL`, `FATAL` | always |

`set_build_options_dc` sends `do_output` (`true`/`false`), `output_executable_name` and `output_path` when non-empty, and one `additional_linker_argument` per `append_linker_arguments` entry. It targets `w`, which defaults to the current workspace.

Defaults the Rust side owns (entry point, output name, import paths, optimization level) are therefore never overwritten by empty or unset Jai defaults. That is also why the default `import_path` is empty: only a change is forwarded.

### Messages

`compiler_begin_intercept(w, flags)` calls `__jaic_workspace_begin_intercept` and records `w`. `compiler_wait_for_message()` walks the intercepted workspaces round-robin starting after the one served last, skips finished ones, calls `__jaic_workspace_next_event(ws)`, and builds a message from the payload slots:

| Event | Message | Payload |
| --- | --- | --- |
| 1 FILE | `Message_File` | string 0 = `fully_pathed_filename` |
| 2 PHASE | `Message_Phase` | int 0 = phase ordinal. For `PRE_WRITE_EXECUTABLE` (3) and `READY_FOR_CUSTOM_LINK_COMMAND` (5): int 1 = object count `n`, strings `0..n-1` = `compiler_generated_object_files`, string `n` = `executable_name` |
| 3 COMPLETE | `Message_Complete` | int 0 = `error_code` ordinal; marks the workspace finished |
| 4 IMPORT | `Message_Import` | string 0 = `module_name`, string 1 = `fully_pathed_filename`; `module_type` is `FILE` when the path is non-empty |
| 0 | `Message_Complete` (NONE) | workspace has no more events; treated as finished |

When every intercepted workspace has finished, further calls keep returning a `COMPLETE` message for the workspace that finished last, with its error code, so `while true { m := compiler_wait_for_message(); if m.kind == .COMPLETE break; }` terminates.

Messages live in module-level records (one per kind), so a message and the strings in it are valid only until the next `compiler_wait_for_message()`; this matches the lifetime of the event payload strings, which are not copied. `Message.workspace` is always the id the event came from. `Intercept_Flags` are accepted but not forwarded.

### Reporting

`compiler_report(message, loc := #caller_location, mode := Report.ERROR)` calls `__jaic_report` with `is_error = (mode == .ERROR)`. The other severities (`ERROR_CONTINUABLE`, `WARNING`, `INFO`) are reported with `is_error = false`; the primitive cannot distinguish them. The four-argument overload builds a `Source_Code_Location` and forwards. `compiler_set_workspace_status(.FAILED, w)` has no primitive: unless `w` already completed with an error it reports a fatal diagnostic so the exit code is nonzero; setting `.OK` is ignored.

### Version

`compiler_get_version_info(*info)` returns `__jaic_compiler_version()` unchanged and parses the first digit run split on dots (`"the, jaic"` gives 0, 2, 29).

### Syntax trees and unsupported APIs

Messages and syntax trees are built from compiler records (`records.jai`, see [compiler-records.md](compiler-records.md)): FILE, IMPORT and TYPECHECKED messages, `compiler_get_nodes`, `compiler_get_code`, `code_to_string`, `print_expression` (through `Program_Print`), `add_global_data` and `compiler_modify_procedure` work. `add_build_string(text, w, message)` with a FILE or IMPORT message adds the text to that message's module; other `code`/`message` scoping goes to the workspace's top level. These keep the public signatures but call `unsupported()`, which issues a fatal `__jaic_report` only when invoked: `get_root_type`, `compiler_make_procedure_live`, `compiler_get_struct_location`, `compiler_report_errors_for_*`, `compiler_set_memory_breakpoint`, `compiler_add_library_search_directory`, `compiler_get_base_path`, `remap_import`, `provide_import`, `add_data_segment`.

`get_name(w)` is answered from names remembered by `compiler_create_workspace`. `get_runtime_info` / `get_type_table` read the `#elsewhere` `__runtime_info` symbol like the original API.

## How to change it

- New option forwarded to Rust: add the field in `options.jai` (keep member order and defaults of the public API), then add a `__jaic_workspace_set_option` line in `forward_options` and handle the key on the Rust side. Update the table above.
- New event kind: add an `EVENT_*` constant and a `case` in `build_message`. Prefer a record payload built with `record_struct` (pointer identity is kept across messages) over a reused module-level instance.
- When a primitive for an unsupported API appears, replace the `unsupported(...)` body with the real call. Do not change the signature.
- Anonymous enums inside `Build_Options` / `Message_*` cannot be named in other declarations, so helper state stores their values as integers (`last_complete_error`) or uses the polymorphic `enum_member_name`.
- Gotchas: workspace id `-1` means the current workspace (`resolve_workspace`); `jaic check` only type-checks code reachable from `main`, so probes must call new procedures behind a runtime (non-constant) condition.

## Configuration

No environment variables. Behaviour depends on the build target constants `OS` and `CPU` (default `os_target` / `cpu_target`) and on `MACHINE_OPTIONS_SIZE` (size of `machine_options`, a layout placeholder). `jaic check file.jai -os linux|windows|macos` sets the target `OS` (for checking code written for another platform; compile-time code still runs on the host). Command-line arguments come from the compiler through `__jaic_command_line_count` / `__jaic_command_line_arg`.

## Dependencies

- Compiler primitives, declared (exported, bodiless, `#compiler`) at the end of `workspace.jai`: `__jaic_workspace_create`, `__jaic_current_workspace`, `__jaic_workspace_add_file`, `__jaic_workspace_add_string`, `__jaic_workspace_set_option`, `__jaic_workspace_begin_intercept`, `__jaic_workspace_next_event`, `__jaic_event_int`, `__jaic_event_string`, `__jaic_command_line_count`, `__jaic_command_line_arg`, `__jaic_report`, `__jaic_compiler_version`, `__jaic_custom_link_complete`. Until Rust binds them, calling any of them fails at compile-time execution with "runtime check failed".
- Prelude types: `Workspace`, `Source_Code_Location`, `Type_Info*`, `Struct_Textual_Flags`, `For_Flags`, `OS`, `CPU`.
- `Basic` (private import) for `array_add`, `tprint`, `copy_string`, `New`, `String_Builder`.
- Users inside the repo: `stdlib/Default_Metaprogram.jai`, `stdlib/Minimal_Metaprogram.jai`, `stdlib/Print_Vars.jai`, `stdlib/Icon.jai`. `stdlib/Metaprogram_Plugins.jai`, `stdlib/Example_Plugin.jai` and `Basic/Memory_Debugger.jai` import `Compiler` as well.
- Probe: `tests/stdlib/compiler-api-shapes.jai` type-checks the option, message and stub surface.

`compiler_set_type_info_flags(type, flags)` records the flags (`__jaic_set_type_info_flags`, `Interp.pending_type_flags`); `Compiler::apply_type_info_flags` (run after each `call_thunk`) or-s them into `Compiler.type_info_flags` and rebuilds the struct's descriptor in place. `NO_TYPE_INFO` (like `#type_info_none`) empties `members`; `PROCEDURES_ARE_VOID_POINTERS` (like `#type_info_procedures_are_void_pointers`) reports procedure-typed members as `*void`. Test: `tests/stdlib/type-info-flags.jai`.
