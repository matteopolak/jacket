# Build options and the metaprogram API

## What it is

What jaic does with each `Build_Options` / `Build_Options_During_Compile` field and `compiler_*` procedure. The rule: a request jaic cannot carry out is never dropped silently. Either it is honoured, or the metaprogram stops with an error naming the unavailable feature.

## How it works

`set_build_options` (`stdlib/internal/compiler-api/workspaces.jai`) forwards each field through a typed builtin (`__jaic_workspace_set_text/flag/number/list`) keyed by an enum, so a field cannot be forwarded and then dropped; see [compiler builtins](../compiler/compiler-builtins.md).

```jai
options := get_build_options(w);
options.output_type = .EXECUTABLE;
options.array_bounds_check = .OFF;
set_build_options(options, w);
```

- Honoured: output type, names and paths, `import_path`, linker arguments, `os_target`/`cpu_target` (an unbuildable target is an error), `intermediate_path`, `entry_point_name`, `append_executable_filename_extension`, `stack_trace`, `dead_code_elimination`, `array_bounds_check`, `null_pointer_check`, `cast_bounds_check`, `arithmetic_overflow_check`, `runtime_support_definitions`, `backtrace_on_crash`, `minimum_os_version`, `enable_frame_pointers`, `disable_redzone`, `emit_debug_info`, `temporary_storage_size` and the `Llvm_Options` switches.
- Rejected with a `require` error: `interactive_bytecode_debugger`, raw `llvm_options.command_line`, `function_sections` and `disable_mem2reg`, compiler memory breakpoints, `developer_debug`, `get_runtime_info` of another workspace, `add_build_string` with a message from another workspace, and `compiler_custom_link_command_is_complete` when no link is pending.
- Options without effect in jaic (no bytecode inliner, no `write_added_strings` output, `use_natvis_compatible_types`) are accepted and stored for `get_build_options` but change nothing.
- `compiler_begin_intercept`'s `SKIP_*` flags clear those lists in `TYPECHECKED` messages (`filter_typechecked` in `messages.jai`), then rebuild `all`. `DEBUG_DUMP` is sent for `#dump` procedures with a listing of the IR (do not parse it).

### Custom link commands

With `use_custom_link_command`, `write_output` writes objects via `OutputBackend::write_objects` instead of linking, then queues `PHASE READY_FOR_CUSTOM_LINK_COMMAND` with `compiler_generated_object_files`, `system_libraries`, `user_libraries` and `executable_name` (`support_object_files` is empty). The metaprogram links and calls `compiler_custom_link_command_is_complete`; the next step sends `POST_WRITE_EXECUTABLE` and `COMPLETE`. A non-zero exit code, or asking for the next message without reporting the link, fails the build. It is an error on the top-level program, which has no message receiver.

### Code generation

`LlvmBackend::prepare` (`crates/jaic-cli/src/main.rs`) turns settings into `jaic_llvm::Options`. `intermediate_path` holds objects and requested `.ll` / `.bc` (and `.unoptimized.*`). Frame pointers, red zone, inlining and tail calls become function attributes (`apply_codegen_attributes`, `crates/jaic-llvm/src/lib.rs`); unrolling, vectorization and function merging are pass switches; `.OS` / `.OZ` run `default<Os>` / `default<Oz>`; `JAIC_CODEGEN_UNITS=1` asks for whole-program optimization ([LLVM backend](../native/llvm-backend.md)). `entry_point_name` goes through `sema::Options::entry_point`. `runtime_support_definitions` and `backtrace_on_crash` become `Runtime_Support` module parameters (off for wasm, which has no signals). `minimum_os_version` sets the macOS triple and `-mmacosx-version-min`.

The program's own workspace reports real outputs from `get_build_options()`: the main file's stem, its absolute directory, and `<output_path>.build` for intermediates. A workspace from `compiler_create_workspace` starts with empty names and paths.

## How to change it

- New option jaic acts on: add the field to `Build_Options` (`stdlib/Compiler/contract-records.jai`), forward it in `set_build_options`, add a member to the matching `Workspace_*` enum (both sides) and handle it in `Workspaces::set_text/set_flag/set_number/append_list`, and read it in `new_compiler` (sema) or `LlvmBackend::prepare` (codegen).
- An option jaic cannot honour: add a `require(...)` with a clear message in `set_build_options`.
- Another backend implements `OutputBackend::write_objects` for custom links.

Configuration: `MACOSX_DEPLOYMENT_TARGET` when no `minimum_os_version` was forwarded; `-O0`..`-O3` replace code generation switches for the top-level program. Other: `crates/jaic/src/build/option_snapshot.rs` carries the full options between compilers. See [compiler module](compiler-module.md) and [workspaces](workspaces.md).

## Rules

Claims that tests cite by ID (`tools/check_rule_coverage.py`).

- `set_build_options` (`stdlib/Compiler/options.jai`) stores the options. `check_build_options` then compares them with the workspace's previous options and reports each field that this call changed to a value jaic does not act on. The report goes to the caller's location (`loc`) and appears once, not on every call {#bo.1}:
- Options left at their defaults, or set to values jaic acts on, say nothing {#bo.4}. Examples are `text_output_flags = 0`, `enable_bytecode_inliner = true` and `set_optimization(*options, .OPTIMIZED_SMALL)`. `set_build_options` then sends every field jaic acts on through the typed workspace-option builtins ([compiler builtins](../compiler/compiler-builtins.md)); an option that is not an enum member cannot be named, so a field cannot be forwarded and then dropped.
- `ERROR_CONTINUABLE` prints `error:`, lets the metaprogram go on and fails the build with exit code 1 {#bo.3}.
- `compiler_begin_intercept`'s `SKIP_DECLARATIONS`, `SKIP_PROCEDURE_HEADERS`, `SKIP_PROCEDURE_BODIES`, `SKIP_STRUCTS` and `SKIP_OTHERS` remove those kinds from `TYPECHECKED` messages, including `all`. A message left with nothing in it is not sent {#bo.2}.
- `DO_PERFORMANCE_REPORT_RUNS` and `DO_PERFORMANCE_REPORT_POLYMORPHS` ask for a `PERFORMANCE_REPORT` message. It is sent once per workspace that compiled, after every other message (the write phases included) and right before `COMPLETE`, and not at all when neither flag was given or the workspace failed {#bo.14}. The flag decides which sections are filled: `run_directive_report` (and its `reported` field) for `_RUNS`, `polymorph_report.records` (and `reported`) for `_POLYMORPHS`; the section that was not asked for is empty with `reported = false`. `time_report`, `bytecode_report` and the two solve counts are always there. What jaic can fill in, and what it cannot:
- `DEBUG_DUMP` is sent to an intercepting metaprogram for each lowered procedure declared `#dump`, in the order the compiler made the procedures, once the program is complete: after the last `TYPECHECKED_ALL_WE_CAN` and before `ALL_TARGET_CODE_BUILT`. It needs no flag. `dump_text` is a heading (the procedure, its file and line) and a listing of the procedure's jaic IR, one instruction per line; the format changes with the IR, so a metaprogram should print it and not parse it. A polymorphic `#dump` procedure is dumped once per instance. A workspace nobody intercepts prints nothing {#bo.15}.
- What jaic cannot do at all stops the metaprogram with an error {#bo.5}:
- `support_object_files` is empty because jaic links no support objects {#bo.6}. The metaprogram runs its linker and calls `compiler_custom_link_command_is_complete(w, exit_code)`. The next time the workspace is stepped, it sends `POST_WRITE_EXECUTABLE` and then `COMPLETE`. `POST_WRITE_EXECUTABLE` carries `linker_exit_code`, and `executable_write_failed` is set when the exit code is not 0.
- The build fails with an error in two cases {#bo.7}:
- `intermediate_path` holds the object files and the IR or bitcode a metaprogram asks for {#bo.8}:
- `enable_frame_pointers`, `disable_redzone`, `llvm_options.disable_inlining` and `enable_tail_calls` become function attributes (`apply_codegen_attributes` in `crates/jaic-llvm/src/lib.rs`) {#bo.12}:
- `entry_point_name` names the procedure the program starts in instead of `main`, through `sema::Options::entry_point` {#bo.9}. `runtime_support_definitions` and `backtrace_on_crash` become the `Runtime_Support` module parameters:
- `ENABLE_BACKTRACE_ON_CRASH`, which installs `Runtime_Support_Crash_Handler` {#bo.10}. Its default is `ON`, as in `Build_Options`. WebAssembly has no signals, so it is off there.
- `minimum_os_version` (`major.minor`) is the deployment version in a macOS target's triple, and the link adds `-mmacosx-version-min` {#bo.11}.
- `get_build_options()` of the program's own workspace already reports where the output goes: `output_executable_name` is the main file's stem, `output_path` its absolute directory with a trailing `/`, and `intermediate_path` is `<output_path>.build` (`derive_main_outputs` in `stdlib/Compiler/options.jai`, from the `__jaic_main_file` primitive that the command line fills in) {#bo.16}. Metaprograms build paths from these (`tprint("%/%", options.output_path, options.output_executable_name)`). `forward_options` does not send the derived `intermediate_path` back, so a round trip through `set_build_options` leaves intermediates where they were. A workspace made with `compiler_create_workspace` starts with empty names and paths.
- | `FAILED_IMPORT` | sent to an intercepting metaprogram when an import finds no module (`status = .NOT_FOUND`, `host_module_name`, `target_module_name`; `import_code` is null), right before `ERROR`. The import is still a compile error unless the metaprogram answers with `provide_import`, which reloads the workspace's sources. Test: `tests/corpus/negative/message-failed-import-and-error.jai` {#bo.13} |
