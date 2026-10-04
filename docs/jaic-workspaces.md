# jaic compiler workspaces (metaprograms)

## What it is

The compiler side of the `Compiler` module: metaprograms (`build.jai`, `first.jai`) create workspaces, add files and
strings, set build options, and read compiler messages. Implemented in `crates/jaic/src/build.rs`; the public Jai API
(`compiler_create_workspace`, `Build_Options`, `Message`...) lives in `stdlib/Compiler/` and is built on a handful of
`#compiler` primitives.

## How it works

- `Workspaces` (shared as `Rc<RefCell<..>>`, `SharedWorkspaces`) is created by the embedder with a `BuildEnv`
  (file system, base `Options`, optional `OutputBackend`, command-line args after `-`, a host factory, a report sink).
  Workspace 1 is the top-level program. `Compiler::attach_workspaces` gives the compile-time interpreter access.
- Bodiless `#compiler` procedures named `__jaic_*` are bound to `Hook::Meta(MetaOp, has_context)` in
  `sema/procs.rs::proc_func`; `interp::run_hook` forwards them to `build::call`. ABI: optional context pointer, then
  params (memory types such as `string` by address), then out-addresses for memory-typed results.

  | primitive | meaning |
  |---|---|
  | `__jaic_workspace_create(name) -> s64` | new workspace id |
  | `__jaic_current_workspace() -> s64` | workspace whose compile-time code runs |
  | `__jaic_workspace_add_file/add_string(ws, s)` | queue a source (`ProgramSource`) |
  | `__jaic_workspace_set_option(ws, key, value)` | string key/value build option (see below) |
  | `__jaic_workspace_begin_intercept(ws)` | mark intercepted |
  | `__jaic_workspace_next_event(ws) -> s64` | compile lazily, pop the next event kind (0 = none) |
  | `__jaic_event_int(i)`, `__jaic_event_string(i)` | fields of the current event |
  | `__jaic_command_line_count/arg(i)` | metaprogram args |
  | `__jaic_report(msg, file, line, col, is_error)` | error traps the compile; warnings go to `report` |
  | `__jaic_compiler_version()`, `__jaic_custom_link_complete(ws, code)` | |

- Each workspace is a small state machine (`build::step`), advanced whenever its events are read (or by
  `build::finish_all` after the top-level compile, for workspaces nobody intercepted). Its `Compiler` lives in the
  registry between steps:
  - **Open → Checked**: `Compiler::begin_sources` loads bootstrap + queued sources and runs their `#run`s;
    events FILE…, PHASE parsed, PHASE TYPECHECKED_ALL_WE_CAN.
  - **Checked + new sources** (a metaprogram called `add_build_string` after seeing TYPECHECKED_ALL_WE_CAN):
    `add_source` + `settle`, new FILE events, PHASE TYPECHECKED_ALL_WE_CAN again.
  - **Checked → Done**: `finish_program` (lowering), output, COMPLETE.
  The registry borrow is released while a compiler runs, so nested metaprograms work.
- Sources a `#run` adds to *its own* workspace (`add_build_string(s, -1)`, e.g. to define a `#placeholder`) are
  pulled by `Compiler::pull_workspace_sources` right after that `#run` (`run_top_level` is incremental).
- Events, in order: `IMPORT(4)` before a module's first file and `FILE(1)` per loaded file (ints[0] = message
  record); `TYPECHECKED(5)` (ints[0] = message record) just before PHASE TYPECHECKED_ALL_WE_CAN; `PHASE(2)` with ints `[phase, 0]`: 0 ALL_SOURCE_CODE_PARSED,
  1 TYPECHECKED_ALL_WE_CAN, 2 ALL_TARGET_CODE_BUILT, 3 PRE_WRITE_EXECUTABLE (ints[1] = object count n, strings[0..n]
  objects then output path), 4 POST_WRITE_EXECUTABLE (strings[0] = output path); `COMPLETE(3)` with ints `[error_code]` (0 none, 1 failed). IMPORT/FILE/TYPECHECKED are only produced for
  intercepted workspaces; their payloads are records, see [compiler-records.md](compiler-records.md).
- `compiler_modify_procedure` calls are queued on the workspace and applied by the next `step`, before its stage
  work, so bodies are replaced before they are lowered.
- Output: when `do_output` and `output_type != NO_OUTPUT`, the embedder's `OutputBackend::write_output` is called
  with the IR program and `BuildSettings` (path = `output_path/output_executable_name`). `jaic build` passes an LLVM
  backend (object, `cc` link, `-shared` for DYNAMIC_LIBRARY, `ar` for STATIC_LIBRARY); `jaic run/check` and the
  browser pass none (workspaces are only checked).
- The top-level program's own settings (workspace 1) decide whether `jaic build` writes it: a metaprogram calling
  `set_build_options_dc(.{do_output = false})` produces no output of its own.

## How to change it

- New primitive: add a `MetaOp` variant + name in `MetaOp::from_name`, handle it in `build::call`, declare it in
  `stdlib/Compiler` as `#compiler` without a body.
- New option: handle the key in `Workspaces::set_option` and use it in `ensure_compiled` or the backend.
- New event kind: push it in `step`; strings handed to Jai must go through `keep_string` so they outlive the
  call. Structured payloads should be records (`crate::records`) rather than strings.
- Gotcha: never hold `shared.borrow_mut()` across `compile_sources` or a backend call that may re-enter.

## Configuration

Option keys: `output_executable_name`, `output_path`, `output_type`, `do_output`, `import_path_clear`, `import_path`,
`os_target`, `cpu_target`, `optimization`, `entry_point_name`, `temporary_storage_size`,
`additional_linker_arguments_clear`, `additional_linker_argument`; unknown keys are ignored. CLI:
`jaic build build.jai - arg1 arg2` passes args to the metaprogram.

## Dependencies

`sema::Compiler` (`compile_sources`, `ProgramSource`), `interp` hooks, `jaic-llvm` (CLI backend),
`stdlib/Compiler` (Jai-side API).
