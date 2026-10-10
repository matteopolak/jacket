# Build options and the metaprogram API: what jaic honours

## What it is

The support matrix for everything a metaprogram can ask of the compiler: every `Build_Options` and `Build_Options_During_Compile` field, the `compiler_*` procedures, message kinds and `Code_Node` kinds. jaic's rule is that a request it cannot carry out is never dropped without a word. It is either honoured, reported as an error, or reported as a warning where projects commonly set it and the program comes out the same.

## How it works

`set_build_options` (`stdlib/Compiler/options.jai`) stores the options. `check_build_options` then compares them with the workspace's previous options and reports each field that this call changed to a value jaic does not act on. The report goes to the caller's location (`loc`) and appears once, not on every call {#bo.1}:

```
first.jai:14:5: warning: jaic ignores Build_Options.backend: jaic always generates code with LLVM, and x64_options do not apply
```

Options left at their defaults, or set to values jaic acts on, say nothing {#bo.4}. Examples are `text_output_flags = 0`, `enable_bytecode_inliner = true` and `set_optimization(*options, .OPTIMIZED_SMALL)`. `forward_options` then sends every field jaic acts on to `Workspaces::set_option` (`crates/jaic/src/build.rs`) as a key and a text value. `set_option` rejects keys it does not know, so a field cannot be forwarded and then dropped.

Warnings and errors use `__jaic_report`'s `mode`, which is `Report` in the Compiler module:

- `ERROR` stops the metaprogram.
- `ERROR_CONTINUABLE` prints `error:`, lets the metaprogram go on and fails the build with exit code 1 {#bo.3}.
- `WARNING` and `INFO` only print.

`compiler_begin_intercept`'s `SKIP_DECLARATIONS`, `SKIP_PROCEDURE_HEADERS`, `SKIP_PROCEDURE_BODIES`, `SKIP_STRUCTS` and `SKIP_OTHERS` remove those kinds from `TYPECHECKED` messages, including `all`. A message left with nothing in it is not sent {#bo.2}.

`DO_PERFORMANCE_REPORT_RUNS` and `DO_PERFORMANCE_REPORT_POLYMORPHS` ask for a `PERFORMANCE_REPORT` message. It is sent once per workspace that compiled, after every other message (the write phases included) and right before `COMPLETE`, and not at all when neither flag was given or the workspace failed {#bo.14}. The flag decides which sections are filled: `run_directive_report` (and its `reported` field) for `_RUNS`, `polymorph_report.records` (and `reported`) for `_POLYMORPHS`; the section that was not asked for is empty with `reported = false`. `time_report`, `bytecode_report` and the two solve counts are always there. What jaic can fill in, and what it cannot:

| Field | What jaic puts there |
| --- | --- |
| `time_report.run_directives` | seconds the interpreter spent executing compile-time code (a `#run` inside another counts once) |
| `time_report.bytecode_generating` | seconds spent lowering procedure bodies to jaic's IR, without the compile-time code they ran. Type checking happens during the lowering, so it is in here |
| `time_report.bytecode_inlining` | always `0`: jaic's IR is not inlined; the native backend inlines when it optimizes (`--timings` shows that time) |
| `bytecode_report.num_calls_total`, `num_instructions_total` | calls and instructions (blocks' terminators included) of the IR lowered in the workspace so far, the runtime support's included |
| `bytecode_report.num_calls_inlined`, `num_instructions_inlined` | always `0`, for the same reason |
| `run_directive_report.records` | one per execution of compile-time code, in order: `callee` is the first procedure the code calls (the `f` of `#run f()`; null when it calls none directly), `num_stalls` how many times the run stopped for a body that was not lowered yet and started over, `elapsed_time` the seconds in the interpreter |
| `polymorph_report.records` | one per polymorphic procedure that was used, in the order of first use: `source` its header, `num_call_sites` how many places asked for an instance, `num_deduplications_by_constants` how many of those found an instance made earlier for the same constants, `distinct_polymorphs` the instances made (`polymorphed` the instance's header, `location` the place that made it, `call_site` null). `num_deduplications_by_bytecode` is always `0`: instances are shared by their constants only |
| `polymorph_report.num_solves_invoked`, `num_solves_used` | requests (one per call site) and instances made; polymorphic structs are not counted |

`DEBUG_DUMP` is sent to an intercepting metaprogram for each lowered procedure declared `#dump`, in the order the compiler made the procedures, once the program is complete: after the last `TYPECHECKED_ALL_WE_CAN` and before `ALL_TARGET_CODE_BUILT`. It needs no flag. `dump_text` is a heading (the procedure, its file and line) and a listing of the procedure's jaic IR, one instruction per line; the format changes with the IR, so a metaprogram should print it and not parse it. A polymorphic `#dump` procedure is dumped once per instance. A workspace nobody intercepts prints nothing {#bo.15}.

What jaic cannot do at all stops the metaprogram with an error {#bo.5}:

- an `os_target` or `cpu_target` it cannot build for (`.PS5`);
- `add_build_string` with a `code` scope;
- `add_global_data` into a user segment;
- `get_runtime_info` or `get_type_table` of another workspace;
- `compiler_custom_link_command_is_complete` for a workspace that is not waiting for a link;
- `compiler_set_workspace_status(.OK)` on a failed workspace.

### Custom link commands

With `use_custom_link_command` on a workspace that writes an executable or a dynamic library, `write_output` does not link. It calls `OutputBackend::write_objects` instead, which writes the objects to `intermediate_path` (or next to the output). It then queues `PHASE` `READY_FOR_CUSTOM_LINK_COMMAND` with:

- `compiler_generated_object_files`;
- `system_libraries`, linker arguments for libraries linked by name (`-lm`, `-framework Metal`, `user32.lib` for MSVC);
- `user_libraries`, library files by path with their `-L` and `-Wl,-rpath,` arguments;
- `executable_name`.

`support_object_files` is empty because jaic links no support objects {#bo.6}. The metaprogram runs its linker and calls `compiler_custom_link_command_is_complete(w, exit_code)`. The next time the workspace is stepped, it sends `POST_WRITE_EXECUTABLE` and then `COMPLETE`. `POST_WRITE_EXECUTABLE` carries `linker_exit_code`, and `executable_write_failed` is set when the exit code is not 0.

The build fails with an error in two cases {#bo.7}:

- a non-zero exit code: `the custom link command for out/game failed with exit code 3`;
- a metaprogram that asks for the next message without reporting the link: `the metaprogram did not link ...` plus a `help:` naming the call.

`use_custom_link_command` on the top-level program is an error: nothing receives its messages.

### Code generation

`prepare` in `LlvmBackend` (`crates/jaic-cli/src/main.rs`) turns the settings into `jaic_llvm::Options` and `Codegen`:

- `intermediate_path` holds the object files and the IR or bitcode a metaprogram asks for {#bo.8}:
  - `<output name>.ll` for `output_llvm_ir`;
  - `.bc` for `output_bitcode`;
  - `.unoptimized.ll` and `.unoptimized.bc` for the `_before_optimizations` variants.
- `enable_frame_pointers`, `disable_redzone`, `llvm_options.disable_inlining` and `enable_tail_calls` become function attributes (`apply_codegen_attributes` in `crates/jaic-llvm/src/lib.rs`) {#bo.12}:
  - `"frame-pointer"="all"` or `"none"`. Apple targets keep their ABI's frame records either way.
  - `noredzone`.
  - `noinline`, except on `inline` procedures.
  - `"disable-tail-calls"`.
- `enable_loop_unrolling`, `enable_loop_vectorization` (with interleaving), `enable_slp_vectorization` and `merge_functions` are the pass builder's switches.
- `enable_split_modules = false` keeps one codegen unit of an unoptimized build. Optimized builds ignore it: a large program is divided by call graph before the optimizer and each part is optimized on its own thread, small ones are optimized whole and split afterwards (see [LLVM backend](../native/llvm-backend.md)). `JAIC_CODEGEN_UNITS=1` in the environment asks for whole-program optimization.
- `machine_code_optimization_setting` sets the target machine's level apart from the IR's.
- `bitcode_optimization_setting` `.OS` and `.OZ` run `default<Os>` and `default<Oz>`.
- `preserve_debug_info = false` turns debug information off.

Every `set_build_options` call forwards these, so a workspace built from the defaults gets the debug flavor they describe. `set_optimization` sets them per flavor. `-O` on the command line replaces them for the top-level program.

`entry_point_name` names the procedure the program starts in instead of `main`, through `sema::Options::entry_point` {#bo.9}. `runtime_support_definitions` and `backtrace_on_crash` become the `Runtime_Support` module parameters:

- `DEFINE_SYSTEM_ENTRY_POINT` and `DEFINE_INITIALIZATION`: `ONLY_INIT` leaves out the entry point and `OMIT` leaves out both.
- `ENABLE_BACKTRACE_ON_CRASH`, which installs `Runtime_Support_Crash_Handler` {#bo.10}. Its default is `ON`, as in `Build_Options`. WebAssembly has no signals, so it is off there.

`minimum_os_version` (`major.minor`) is the deployment version in a macOS target's triple, and the link adds `-mmacosx-version-min` {#bo.11}.

`get_build_options()` of the program's own workspace already reports where the output goes: `output_executable_name` is the main file's stem, `output_path` its absolute directory with a trailing `/`, and `intermediate_path` is `<output_path>.build` (`derive_main_outputs` in `stdlib/Compiler/options.jai`, from the `__jaic_main_file` primitive that the command line fills in) {#bo.16}. Metaprograms build paths from these (`tprint("%/%", options.output_path, options.output_executable_name)`). `forward_options` does not send the derived `intermediate_path` back, so a round trip through `set_build_options` leaves intermediates where they were. A workspace made with `compiler_create_workspace` starts with empty names and paths.

## Support matrix

The before column is jaic 0.4.1. "Silently ignored" means the value was accepted or forwarded but changed nothing, with no message.

### `Build_Options`

| Field | Before | After |
| --- | --- | --- |
| `output_type`, `output_executable_name`, `output_path`, `import_path`, `additional_linker_arguments` | honoured | honoured |
| `os_target`, `cpu_target` | honoured; targets jaic cannot build for silently ignored | honoured; others an error |
| `intermediate_path` | silently ignored | honoured: objects, IR and bitcode go there |
| `entry_point_name` | silently ignored | honoured |
| `append_executable_filename_extension` | silently ignored | honoured |
| `use_custom_link_command` | silently ignored (jaic linked itself, `READY_FOR_CUSTOM_LINK_COMMAND` never came) | honoured (see above); an error for the top-level program |
| `temporary_storage_size`, `stack_trace`, `dead_code_elimination`, `cast_bounds_check`, `arithmetic_overflow_check` | honoured | honoured |
| `array_bounds_check` | honoured; `.ALWAYS` silently `.ON` | `.ALWAYS` warns (`#no_abc` still turns checks off) |
| `null_pointer_check` | silently ignored | honoured: `.ON` (the default) makes native code check the pointer of every load, store and copy that is not a local or global; `.OFF` leaves them unchecked. Gated like `array_bounds_check`, per workspace; `#no_abc` on a procedure also skips it |
| `emit_debug_info` | honoured (`.NONE` off; `CODEVIEW`/`DWARF` pick nothing, the target's format is used) | same |
| `write_added_strings` | silently ignored | not forwarded, documented: jaic never writes added strings to disk, which is what `false` asks |
| `runtime_support_definitions` | silently ignored | honoured (module parameters) |
| `backtrace_on_crash` | silently ignored (never installed) | honoured |
| `minimum_os_version` | silently ignored | honoured on macOS (triple and link) |
| `enable_frame_pointers`, `disable_redzone` | silently ignored | honoured (function attributes) |
| `backend` | silently ignored | `.X64` warns |
| `x64_options`, `machine_options` | silently ignored | covered by the `backend` warning; `machine_options` is an opaque layout placeholder |
| `runtime_storageless_type_info`, `shorten_filenames_in_error_messages`, `use_visual_studio_message_format`, `use_natvis_compatible_types`, `lazy_foreign_function_lookups`, `interactive_bytecode_debugger`, `debug_for_expansions`, `prevent_compile_time_calls_from_runtime` | silently ignored | `true` warns |
| `enable_bytecode_inliner`, `enable_bytecode_deduplication` | silently ignored | documented: jaic has no bytecode inliner or deduplication, which is what `false` asks; `true` (the default) is Jai's speed-up and changes no program |
| `max_bytecode_instructions_for_inlined_initializer`, `context_size_max`, `maximum_polymorph_depth`, `maximum_array_count_before_compile_time_returns_are_not_reflected_in_ast` | silently ignored | a change warns |
| `info_flags` | silently ignored | non-zero warns |
| `text_output_flags` | silently ignored | non-zero warns (jaic prints neither link lines nor timings; `-time` does timings) |
| `compile_time_command_line`, `user_data_*` | stored | stored (returned by `get_build_options`) |

### `Llvm_Options`

| Field | Before | After |
| --- | --- | --- |
| `bitcode_optimization_setting` | honoured; `.OS`, `.OZ` silently `.O2` | honoured, with size pipelines |
| `machine_code_optimization_setting` | silently ignored | honoured |
| `enable_tail_calls`, `enable_loop_unrolling`, `enable_slp_vectorization`, `enable_loop_vectorization`, `merge_functions`, `disable_inlining`, `enable_split_modules`, `preserve_debug_info` | silently ignored | honoured |
| `output_llvm_ir`, `output_bitcode`, `output_llvm_ir_before_optimizations`, `output_bitcode_before_optimizations` | silently ignored | honoured |
| `target_system_triple`, `target_system_cpu`, `target_system_features` | honoured (an empty CPU meant the build machine's) | honoured; an empty CPU is the target's baseline, `"native"` the build machine's ([LLVM backend](../native/llvm-backend.md)) |
| `function_sections`, `disable_mem2reg` | silently ignored | `true` warns |
| `command_line` | silently ignored | non-empty warns |

### `Build_Options_During_Compile`

| Field | Before | After |
| --- | --- | --- |
| `do_output`, `output_executable_name`, `output_path`, `append_linker_arguments` | honoured | honoured |
| `append_executable_filename_extension` | silently ignored | honoured |
| `write_added_strings` | silently ignored | documented as above |
| `interactive_bytecode_debugger` | silently ignored | `true` warns |

### Procedures

| Procedure | Before | After |
| --- | --- | --- |
| `compiler_create_workspace`, `get_current_workspace`, `get_name`, `add_build_file`, `add_build_string(data, w, message)`, `set_build_options`, `get_build_options`, `set_optimization`, `copy_commonly_propagated_fields`, `compiler_wait_for_message`, `compiler_end_intercept`, `compiler_get_version_info`, `get_toplevel_command_line`, `compiler_get_nodes`, `compiler_get_code`, `code_to_string`, `print_expression`, `compiler_modify_procedure`, `compiler_set_type_info_flags`, `compiler_get_struct_location`, `make_location`, `get_filename`, `is_subclass_of` | honoured | honoured |
| `compiler_destroy_workspace` | silently ignored (the workspace was still compiled) | a workspace not compiled yet never is |
| `add_build_string(data, w, code)` | `code` silently ignored | a non-null `code` is an error |
| `compiler_begin_intercept` flags | silently ignored | `SKIP_*` filter `TYPECHECKED`; `DO_PERFORMANCE_REPORT_*` now ask for `PERFORMANCE_REPORT`; `SKIP_EXPRESSIONS_WITHOUT_NOTES` is a hint jaic does not need (every expression is delivered) |
| `compiler_custom_link_command_is_complete` | silently ignored | honoured; an error without a pending link |
| `compiler_report` | `ERROR_CONTINUABLE` silently a warning | `ERROR_CONTINUABLE` prints an error and fails the build; `INFO` prints `info:` |
| `compiler_set_workspace_status` | `.OK` silently ignored | `.OK` on a failed workspace is an error |
| `get_runtime_info`, `get_type_table` | another workspace's `w` silently gave the running program's | an error for another workspace |
| `add_global_data` | segment silently ignored | data goes with the program's other constant data whatever the segment (a `.USER_SEGMENT` needs the segment `add_data_segment` returned) |
| `get_root_type`, `compiler_make_procedure_live`, `compiler_get_base_path`, `compiler_add_library_search_directory`, `remap_import`, `provide_import`, `add_data_segment` | error when called | implemented, see [the Compiler module](compiler-module.md) |
| `compiler_report_errors_for_*`, `compiler_set_memory_breakpoint` | error when called | accepted and ignored, for the reasons in [the Compiler module](compiler-module.md) |

### Messages and nodes

| What | Status |
| --- | --- |
| `FILE`, `IMPORT`, `TYPECHECKED`, `COMPLETE` | sent |
| `PHASE` | `ALL_SOURCE_CODE_PARSED`, `TYPECHECKED_ALL_WE_CAN`, `ALL_TARGET_CODE_BUILT`, `PRE_WRITE_EXECUTABLE`, `POST_WRITE_EXECUTABLE` sent; `READY_FOR_CUSTOM_LINK_COMMAND` now sent with `use_custom_link_command`; `POST_WRITE_EXECUTABLE` now fills `executable_write_failed` and `linker_exit_code` |
| `FAILED_IMPORT` | sent to an intercepting metaprogram when an import finds no module (`status = .NOT_FOUND`, `host_module_name`, `target_module_name`; `import_code` is null), right before `ERROR`. The import is still a compile error unless the metaprogram answers with `provide_import`, which reloads the workspace's sources. Test: `tests/corpus/negative/message-failed-import-and-error.jai` {#bo.13} |
| `ERROR` | sent once when the workspace fails with a compile error, before `COMPLETE` (which carries `COMPILATION_FAILED`). Only for errors that stop the workspace, not for `ERROR_CONTINUABLE` reports or link failures. The message has no extra fields |
| `PERFORMANCE_REPORT`, `DEBUG_DUMP` | `PERFORMANCE_REPORT` is sent when the intercept flags ask for it (`DO_PERFORMANCE_REPORT_RUNS`/`_POLYMORPHS`), `DEBUG_DUMP` for `#dump` procedures; what they hold is described above. Tests: `tests/stdlib/compiler-performance-and-dump-messages.jai` |
| `Code_Node` kinds | see [compiler records](compiler-records.md). `#this`, `#compile_time`, `#bytes`, `#procedure_name`, `#bake_arguments`/`#bake_constants` (as `Code_Directive_Bake` around the call), `#asm` (an opaque `Code_Asm`, without its instructions), `#load` (`short_name`, `fully_pathed_filename`), `#place` and `#overlay` are modelled, and `Program_Print` prints all of them but `#asm`. `#caller_code` (a `Code_Directive_Location` with the jaic field `is_caller_code`), `$T`/`$$x`/`$T/Restriction` (a `Code_Ident` with `DEFINES_POLYMORPH_VARIABLE`, the jaic flags `BAKED_POLYMORPH_VARIABLE` and `INTERFACE_RESTRICTED`, and the jaic field `polymorph_restriction`), `#add_context` (`Code_Directive_Add_Context` around the declaration) and `#module_parameters` (`Code_Directive_Module_Parameters`, its two lists as body-less procedure headers) are modelled too. `#file`, `#filepath` and `#line` are exported as the string or integer literal they evaluate to, when the export has the compiler's sources (`compiler_get_nodes` of a `#run`'s code, `TYPECHECKED` messages); a bare parse of the text, which has no file, leaves them as `.PLACEHOLDER`, as it does any statement or expression with no model, which `Program_Print` (and so `compiler_get_code` and edited statements in `compiler_modify_procedure`) reports as unsupported |

## How to change it

- **A new option jaic acts on.** Forward it in `forward_options` and add the key to `Workspaces::set_option`. Read it in `new_compiler` (sema options) or `LlvmBackend::prepare` (code generation). Then update the matrix above and the key table in [compiler module](compiler-module.md).
- **An option jaic can only report.** Add a line to `check_build_options`. Use `option_warning` when the program comes out the same and `option_error` when it would not. Compare with `previous` so it is reported once.
- **Unknown keys.** `set_option` returns an error for any key it does not know. Forgetting the Rust side therefore fails loudly in every test that sets options.
- **Custom links.** Another backend implements `OutputBackend::write_objects`. The default refuses with an error.

## Configuration

- `MACOSX_DEPLOYMENT_TARGET` is used when no `minimum_os_version` was forwarded, for example for the top-level program.
- `-O0`..`-O3` replace the metaprogram's code generation switches for the top-level program.

## Dependencies

- `stdlib/Compiler/options.jai`: `check_build_options`, `forward_options`.
- `stdlib/Compiler/workspace.jai`: reports, intercept flags and phase messages.
- `crates/jaic/src/build.rs`: `BuildSettings`, `set_option`, custom link state.
- `crates/jaic-cli/src/main.rs`: `LlvmBackend::prepare`, `write_objects`.
- `crates/jaic-llvm/src/lib.rs`: `Codegen`, `link_inputs`.
- `stdlib/Runtime_Support.jai` and `Runtime_Support_Crash_Handler.jai`.
