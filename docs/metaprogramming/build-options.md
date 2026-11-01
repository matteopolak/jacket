# Build options and the metaprogram API

What jaic does with each `Build_Options` and `Build_Options_During_Compile` field and each `compiler_*` procedure. A request jaic cannot carry out is never dropped silently: it is honoured, or the metaprogram stops with an error naming the unavailable feature.

```jai
options := get_build_options(w);
options.output_type = .EXECUTABLE;
options.array_bounds_check = .OFF;
set_build_options(options, w);
```

## How it works

`set_build_options` is bound to `build/public_api/options.rs`. It reads each field through the declared `Build_Options` layout and hands it to `Workspaces::set_text/set_flag/set_number/append_list`, keyed by the `Workspace_*` enums in `builtin_keys.rs`, so a field cannot be read and then dropped ([compiler builtins](../compiler/compiler-builtins.md)).

- **Honoured:** output type, names and paths, `import_path`, linker arguments, `os_target`/`cpu_target`, `intermediate_path`, `entry_point_name`, the runtime checks, `stack_trace`, `dead_code_elimination`, `backtrace_on_crash`, `minimum_os_version`, frame pointer and red zone switches, `emit_debug_info`, `temporary_storage_size` and the `Llvm_Options` switches.
- **Rejected** with a `require` error: `interactive_bytecode_debugger`, raw `llvm_options.command_line`, `disable_mem2reg`, memory breakpoints, `developer_debug`, another workspace's `get_runtime_info`, and `add_build_string` with another workspace's message.
- **Accepted and ignored:** options with no meaning here (the bytecode inliner, `write_added_strings`, `use_natvis_compatible_types`).
- Before `set_build_options`, a workspace reports the command line's options (`-o`, `-O0..-O3`, `-no_debug_info`, `-no_dce`, ...) and builds with them. This holds for the top-level program and for the first workspace plugins compile; other workspaces start at defaults. A plugin that gets, edits and sets keeps the command line; one that sets a field overrides it. See `requested_settings` in `crates/jaic-cli/src/run.rs`.

### Custom link commands

With `use_custom_link_command`, `write_output` writes objects instead of linking, then queues `PHASE READY_FOR_CUSTOM_LINK_COMMAND` with the object files, libraries and `executable_name`. The metaprogram links and calls `compiler_custom_link_command_is_complete`. A non-zero exit code, or asking for the next message without reporting the link, fails the build. On the top-level program, which has no message receiver, it is an error.

### Code generation

`LlvmBackend::prepare` (`crates/jaic-cli/src/main.rs`) turns settings into `jaic_native::Options`. Frame pointers, red zone, inlining and tail calls become function attributes (`apply_codegen_attributes`); unrolling, vectorization and merging are pass switches. `function_sections` applies on ELF only. `.OS`/`.OZ` run `default<Os>`/`default<Oz>`. `JAIC_CODEGEN_UNITS=1` asks for whole-program optimization ([LLVM backend](../native/llvm-backend.md)).

## Target constants

`IS_CROSS_COMPILING` compares the workspace's target OS and CPU with the compiler host; child workspaces resolve it independently. `MACHINE_OPTIONS_SIZE` (128) is the byte capacity of `Build_Options.machine_options`; architecture option records must fit it. Change it in `sema/mod.rs`.

## How to change it

- A new option jaic acts on: add the field to `Build_Options` (`stdlib/Compiler/contract-records.jai`), list it in `options.rs` (`NUMBERS`, `FLAGS` or `TEXTS`), add a key to the matching `Workspace*` enum in `builtin_keys.rs`, handle it in `Workspaces::set_*`, and read it in `new_compiler` (sema) or `LlvmBackend::prepare`.
- An option jaic cannot honour: return a `Trap` with a clear message from `options.rs::set`.
- `option_snapshot.rs` carries the full options between compilers. Another backend implements `OutputBackend::write_objects` for custom links.
- `MACOSX_DEPLOYMENT_TARGET` applies when no `minimum_os_version` was forwarded.

See [compiler module](compiler-module.md) and [workspaces](workspaces.md).

## Rules

Claims that tests cite by ID (`tools/dev/check_rule_coverage.py`).

- `set_build_options` (`stdlib/Compiler/options.jai`) stores the options, then `check_build_options` reports each field this call changed to a value jaic does not act on, once, at the caller's `loc` {#bo.1}.
- Options left at defaults, or set to values jaic acts on, say nothing, for example `text_output_flags = 0`, `enable_bytecode_inliner = true`, `set_optimization(*options, .OPTIMIZED_SMALL)` {#bo.4}. Every field jaic acts on goes through the typed workspace-option builtins, so a field cannot be forwarded and then dropped.
- `ERROR_CONTINUABLE` prints `error:`, lets the metaprogram go on and fails the build with exit code 1 {#bo.3}.
- `compiler_begin_intercept`'s `SKIP_DECLARATIONS`, `SKIP_PROCEDURE_HEADERS`, `SKIP_PROCEDURE_BODIES`, `SKIP_STRUCTS` and `SKIP_OTHERS` remove those kinds from `TYPECHECKED` messages, including `all`; a message left empty is not sent {#bo.2}.
- `DO_PERFORMANCE_REPORT_RUNS` and `DO_PERFORMANCE_REPORT_POLYMORPHS` ask for a `PERFORMANCE_REPORT`, sent once per compiled workspace after every other message and right before `COMPLETE`, and not at all if neither flag was given or the workspace failed {#bo.14}. The flag decides which section is filled (`run_directive_report`, `polymorph_report.records`); the other has `reported = false`. `time_report`, `bytecode_report` and the solve counts are always there.
- `DEBUG_DUMP` is sent for each lowered `#dump` procedure, after the last `TYPECHECKED_ALL_WE_CAN` and before `ALL_TARGET_CODE_BUILT`, with no flag needed. `dump_text` is a heading and a jaic IR listing; the format changes with the IR, so print it and do not parse it. A polymorphic `#dump` procedure is dumped per instance. An unintercepted workspace prints nothing {#bo.15}.
- What jaic cannot do at all stops the metaprogram with an error {#bo.5}.
- `support_object_files` is empty because jaic links no support objects {#bo.6}. The metaprogram runs its linker and calls `compiler_custom_link_command_is_complete(w, exit_code)`; the next step sends `POST_WRITE_EXECUTABLE` (with `linker_exit_code`, and `executable_write_failed` when non-zero) and then `COMPLETE`.
- A custom link fails the build when the exit code is non-zero, or when the next message is requested without reporting the link {#bo.7}.
- `intermediate_path` holds the object files and the IR or bitcode a metaprogram asks for {#bo.8}.
- `enable_frame_pointers`, `disable_redzone`, `llvm_options.disable_inlining` and `enable_tail_calls` become function attributes {#bo.12}.
- `entry_point_name` names the starting procedure instead of `main` {#bo.9}. A `#run` or metaprogram may set it on the workspace being compiled; `-interp` and native builds read it back before picking the entry point. `runtime_support_definitions` and `backtrace_on_crash` become `Runtime_Support` module parameters.
- `ENABLE_BACKTRACE_ON_CRASH` installs `Runtime_Support_Crash_Handler`. `Build_Options` defaults it to `ON`, so a metaprogram's build has the handler; a plain `jaic` build has it only with `-backtrace_on_crash` (`jacket test` passes that for native tests). Off for WebAssembly, which has no signals {#bo.10}.
- `minimum_os_version` (`major.minor`) is the deployment version in a macOS triple, and the link adds `-mmacosx-version-min` {#bo.11}.
- The program's own workspace reports its output location: `output_executable_name` is the main file's stem, `output_path` its absolute directory with a trailing `/`, `intermediate_path` is `<output_path>.build` {#bo.16}. `forward_options` does not send the derived `intermediate_path` back. A workspace from `compiler_create_workspace` starts with empty names and paths.
- A workspace reports the command-line options until `set_build_options` is called on it (see above); `create_workspaces`, `Workspaces::start_from` and `start_first_created_from` install them {#bo.17}.
- `FAILED_IMPORT` is sent to an intercepting metaprogram when an import finds no module (`status = .NOT_FOUND`, `host_module_name`, `target_module_name`), right before `ERROR`. It is still a compile error unless the metaprogram answers with `provide_import` {#bo.13}.
