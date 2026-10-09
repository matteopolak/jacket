# Workspaces and metaprograms

## What it is

The Rust side of the `Compiler` module, in `crates/jaic/src/build.rs`. Metaprograms (`build.jai`, `first.jai`) create workspaces, add files and strings, set build options and read compiler messages through `#compiler` primitives; the Jai API on top of them is in [Compiler module](compiler-module.md).

## How it works

### Registry

The embedder creates `Workspaces` (shared as `SharedWorkspaces`, an `Rc<RefCell<..>>`) with a `BuildEnv`: file system, base `Options`, an optional `OutputBackend`, command-line args after `-`, a host factory, a report sink, and an optional `WorkspaceObserver` that sees each workspace's compiler when it is made and gets it back when the workspace is done ([jailint](../tools/jailint.md) uses it to lint what a metaprogram builds). `Compiler::attach_workspaces` gives the compile-time interpreter access.

Workspace 2 is the top-level program (`TOP_LEVEL_WORKSPACE`) {#ws.1}. Workspace 1 is reserved, as in Jai, so the first workspace a metaprogram creates is 3 {#ws.2}.

### Primitives

Bodiless `#compiler` procedures named `__jaic_*` are bound to `Hook::Meta(MetaOp, has_context)` in `proc_func` (`sema/procs.rs`); `interp::run_hook` forwards them to `build::call`. The ABI is: optional context pointer, the parameters (memory types such as `string` by address), then out-addresses for memory-typed results.

| Primitive | Meaning |
|---|---|
| `__jaic_workspace_create(name) -> s64` | new workspace id {#ws.3} |
| `__jaic_current_workspace() -> s64` | workspace whose compile-time code is running {#ws.4} |
| `__jaic_workspace_add_file/add_string(ws, s)` | queue a source (`ProgramSource`) |
| `__jaic_workspace_set_option(ws, key, value)` | set a build option by name |
| `__jaic_workspace_begin_intercept(ws, flags)` | mark intercepted; `flags` are the `Intercept_Flags`, of which the Rust side reads `DO_PERFORMANCE_REPORT_*` |
| `__jaic_workspace_next_event(ws) -> s64` | advance the workspace if needed, pop the next event kind (0 = none) |
| `__jaic_event_int(i)`, `__jaic_event_string(i)` | fields of the current event |
| `__jaic_command_line_count/arg(i)` | metaprogram arguments |
| `__jaic_report(msg, file, line, col, mode)` | `mode` 0 (`ERROR`) fails the compile {#ws.5}; 2 and 3 (`WARNING`, `INFO`) go to the report sink {#ws.6}; 1 (`ERROR_CONTINUABLE`) goes to the sink as an error and marks the current workspace failed |
| `__jaic_compiler_version()` | |
| `__jaic_custom_link_complete(ws, code)` | the metaprogram's linker exit code for a workspace waiting after `READY_FOR_CUSTOM_LINK_COMMAND`; an error otherwise |
| `__jaic_code_is_null(code)` | whether a `Code` value is `#code,null` |

The `__jaic_rec_*`, `__jaic_code_nodes`, `__jaic_parse_code` and `__jaic_modify_procedure` primitives belong to [compiler records](compiler-records.md).

### Workspace state machine

`build::step` advances a workspace whenever its events are read, or from `build::finish_all` after the top-level compile for workspaces nobody intercepted. `finish_all` skips a workspace that never got a file or string: there is nothing to compile, and building it would link an executable without `main`. Its `Compiler` lives in the registry between steps.

1. **Open to Checked**: `Compiler::begin_sources` loads the bootstrap and queued sources and runs their `#run`s. Events: FILE..., PHASE `ALL_SOURCE_CODE_PARSED`, PHASE `TYPECHECKED_ALL_WE_CAN` {#ws.7}.
2. **Checked with new sources** (the metaprogram called `add_build_string` after `TYPECHECKED_ALL_WE_CAN`): `add_source` and `settle`, new FILE events, `TYPECHECKED_ALL_WE_CAN` again {#ws.8}.
3. **Checked to Done**: `finish_program` lowers, output is written, COMPLETE.

The registry borrow is released while a compiler runs, so nested metaprograms work {#ws.9}. Sources a `#run` adds to its own workspace (`add_build_string(s, -1)`, for example to define a `#placeholder`) are pulled by `Compiler::pull_workspace_sources` right after that `#run`; `run_top_level` is incremental {#ws.10}. `compiler_modify_procedure` calls are queued and applied at the start of the next `step`, so bodies are replaced before they are lowered {#ws.14}.

A workspace's compile-time code runs on its driver's interpreter budget (`Interp::block_budget`, set by the playground, jailsp and jailint; the CLI has none): `step` takes the budget of the interpreter reading the events (or, from `finish_all`, the top-level compiler's), gives it to the workspace's compiler and hands back what is left. One budget therefore bounds the whole compilation; an endless `#run` in a workspace ends in `execution budget exhausted` for the workspace and then for the metaprogram waiting on it.

### Events

| Kind | Payload |
|---|---|
| `IMPORT` (4) | int 0 = record; sent before a module's first file |
| `FILE` (1) | int 0 = record; one per loaded file {#ws.11} |
| `TYPECHECKED` (5) | int 0 = record; just before PHASE `TYPECHECKED_ALL_WE_CAN` {#ws.12} |
| `PHASE` (2) | int 0 = phase: 0 `ALL_SOURCE_CODE_PARSED`, 1 `TYPECHECKED_ALL_WE_CAN`, 2 `ALL_TARGET_CODE_BUILT`, 3 `PRE_WRITE_EXECUTABLE` (int 1 = object count `n`, strings = objects then output path), 4 `POST_WRITE_EXECUTABLE` (int 1 = write failed, int 2 = linker exit code, string 0 = output path), 5 `READY_FOR_CUSTOM_LINK_COMMAND` (ints 1..4 = counts of objects, support objects, system and user libraries; strings = those lists, then the output path) |
| `COMPLETE` (3) | int 0 = error code (0 none, 1 failed) {#ws.13} |

IMPORT, FILE and TYPECHECKED are only produced for intercepted workspaces.

### Output

When `do_output` is set and `output_type != NO_OUTPUT`, the embedder's `OutputBackend::write_output` gets the IR program and `BuildSettings` (path `output_path/output_executable_name`). `jaic build` and `jaic run` pass an LLVM backend (object file, `cc` link, `-shared` for `DYNAMIC_LIBRARY`, `ar` for `STATIC_LIBRARY`): `run` differs from `build` only in interpreting the top-level program instead of compiling it, so a build metaprogram (`jaic run first.jai`) writes and can launch what it builds. `jaic check`, `jaic run -no_workspace_output` (used by `tools/jaic-sweep.py`), the browser, jailsp and jailint pass none, so workspaces are only checked {#ws.15}. Without a backend, `BuildEnv::unwritten_output_hint` decides whether a workspace that asks for output is told it was not written: `jaic check` warns ``warning: `jaic check` does not write build/game (workspace `Build` asks for an executable)`` with ``help: `jaic build first.jai` (or `jaic run first.jai`) writes it`` (the path is shown relative to where jaic was started, the main file as it was typed); `-no_workspace_output` and the tools set `None` and say nothing {#ws.17}.

Workspace 2's own settings decide whether `jaic build` writes the top-level program, so a metaprogram calling `set_build_options_dc(.{do_output = false})` produces no output of its own {#ws.16}.

## How to change it

- New primitive: a `MetaOp` variant and name in `MetaOp::from_name`, a case in `build::call`, and a bodiless `#compiler` declaration in `stdlib/Compiler`.
- New option: handle the key in `Workspaces::set_option` and read it where the compiler is created (`new_compiler`) or output is written (`write_output`).
- New event kind: push it in `step`. Strings handed to Jai must go through `keep_string` so they outlive the call; structured payloads should be records (`crate::records`).
- Gotcha: never hold `shared.borrow_mut()` across `compile_sources` or a backend call; both can re-enter.

## Configuration

Option keys handled by `set_option` are listed in [compiler module](compiler-module.md); `os_target` and `cpu_target` values jaic can't build for, and keys it does not know, are errors. [Build options](build-options.md) says what each does. A workspace with `use_custom_link_command` stops after writing its objects (`Workspace::link`) until `__jaic_custom_link_complete` gives the exit code; the next step sends `POST_WRITE_EXECUTABLE` and `COMPLETE` (`finish_custom_link`). `arithmetic_overflow_check` applies to workspaces a metaprogram creates, not the program's own.

`jaic build build.jai - arg1 arg2` passes `arg1 arg2` to the metaprogram.

## Dependencies

`sema::Compiler` (`compile_sources`, `ProgramSource`), interpreter hooks, `jaic-llvm` (the CLI's backend), `stdlib/Compiler`.
