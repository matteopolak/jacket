# Workspaces, metaprograms and plugins

## What it is

The Rust side of the `Compiler` module, in `crates/jaic/src/build.rs`. Metaprograms (`build.jai`, `first.jai`) create workspaces, add sources, set build options and read compiler messages through `#compiler` primitives; the Jai API on top is in [Compiler module](compiler-module.md). The page also covers `+Name` plugins.

## How it works

The embedder creates `Workspaces` (shared as `SharedWorkspaces`) with a `BuildEnv`: file system, base `Options`, an optional `OutputBackend`, arguments after `-`, a report sink, and an optional `WorkspaceObserver` ([jailint](../tools/jailint.md) uses it to lint what a metaprogram builds). Workspace 2 is the top-level program (`TOP_LEVEL_WORKSPACE`); workspace 1 is reserved as in Jai, so the first created workspace is 3.

Bodiless `#compiler` procedures named `__jaic_*` bind to `Hook::Meta(MetaOp, has_context)` in `proc_func` (`sema/procs.rs`); `interp::run_hook` forwards them to `build::call`. Key primitives:

| Primitive | Meaning |
|---|---|
| `__jaic_workspace_create(name)`, `__jaic_current_workspace()` | new id; workspace whose code is running |
| `__jaic_workspace_add_file/add_string(ws, s)` | queue a source |
| `__jaic_workspace_set_text/flag/number(ws, option, value)`, `_clear_list/_append_list`, `_destroy` | set a build option; `option` is a `Workspace_*` enum member |
| `__jaic_workspace_begin_intercept(ws, flags)`, `__jaic_workspace_next_event(ws)` | intercept; advance and pop the next event kind |
| `__jaic_report(msg, file, line, col, mode)` | mode 0 `ERROR` fails the compile; 2, 3 go to the report sink; 1 `ERROR_CONTINUABLE` reports and marks the workspace failed |
| `__jaic_custom_link_complete(ws, code)` | linker exit code after `READY_FOR_CUSTOM_LINK_COMMAND` |

The `__jaic_rec_*`, `__jaic_code_nodes`, `__jaic_parse_code` and `__jaic_modify_procedure` primitives are in [compiler module](compiler-module.md#records).

### State machine

`build::step` advances a workspace whenever its events are read, or from `build::finish_all` for workspaces nobody intercepted (skipping those with no sources).

1. Open to Checked: `Compiler::begin_sources` loads the bootstrap and sources and runs their `#run`s. Events: FILE..., PHASE `ALL_SOURCE_CODE_PARSED`, PHASE `TYPECHECKED_ALL_WE_CAN`.
2. New sources after that: `add_source` and `settle`, more FILE events, `TYPECHECKED_ALL_WE_CAN` again.
3. Checked to Done: `finish_program` lowers, output is written, COMPLETE.

The registry borrow is released while a compiler runs, so nested metaprograms work; never hold `shared.borrow_mut()` across `compile_sources` or a backend call. Sources a `#run` adds to its own workspace (`add_build_string(s, -1)`) are pulled right after that `#run` (`pull_workspace_sources`). `compiler_modify_procedure` calls are applied at the start of the next `step`. One interpreter budget (`Interp::block_budget`, set by the playground, jailsp and jailint) bounds a whole compilation.

Events: `IMPORT` (4), `FILE` (1), `TYPECHECKED` (5) are produced only for intercepted workspaces; `PHASE` (2) carries 0 `ALL_SOURCE_CODE_PARSED`, 1 `TYPECHECKED_ALL_WE_CAN`, 2 `ALL_TARGET_CODE_BUILT`, 3 `PRE_WRITE_EXECUTABLE`, 4 `POST_WRITE_EXECUTABLE`, 5 `READY_FOR_CUSTOM_LINK_COMMAND`; `COMPLETE` (3) carries 0 or 1.

### Output

With `do_output` set and `output_type != NO_OUTPUT`, `OutputBackend::write_output` gets the IR program and `BuildSettings`. `jaic` and `jaic -interp` pass an LLVM backend (`-interp` only interprets the top-level program instead of compiling it, so a build metaprogram can launch what it builds). `jaic -check`, `-no_workspace_output`, the browser, jailsp and jailint pass none, so workspaces are only checked; `BuildEnv::unwritten_output_hint` makes `-check` warn when a workspace asks for an executable. Workspace 2's own settings decide whether the top-level program is written. A workspace with `use_custom_link_command` stops after writing objects until `__jaic_custom_link_complete` (`finish_custom_link`).

### Plugins (`+Name`)

`jaic file.jai +Name` builds the target in a child workspace using the `Compiler.Metaprogram_Plugin` hooks; `-check` runs the same lifecycle without writing an executable. The CLI (privately, in `crates/jaic-cli/src/main.rs`) generates source that imports each plugin, creates a workspace, sets its options, calls `init` with each plugin's option array, then `before_intercept`, `add_source`, adds the target and CLI sources, dispatches messages until COMPLETE, and calls finish and shutdown. Null plugins and refused options fail; init failures shut down acquired plugins first. The wrapper requests `NO_OUTPUT`; the child keeps the real settings.

- `+Name` may repeat; module parameters go in the name (`'+Check(CHECK_BINDINGS=false)'`). Arguments after a plugin up to the next plugin or separator belong to it; put compiler options first.
- `-interp` with plugins is a usage error. `-exe`, `-output_path`, `-add` and `-run` apply to the child.
- Regression: `crates/jaic-cli/tests/plugins.rs`.

## How to change it

- New primitive: a `MetaOp` variant and name in `MetaOp::from_name`, a case in `build::call`, and a bodiless `#compiler` declaration in `stdlib/internal/compiler-api/primitives.jai`.
- New option: handle the key in `Workspaces::set_option` and read it in `new_compiler` or `write_output`. Unknown keys and unbuildable `os_target`/`cpu_target` are errors ([build options](build-options.md)).
- New event kind: push it in `step`; strings given to Jai go through `keep_string`.

`jaic build.jai - arg1 arg2` passes `arg1 arg2` to the metaprogram. Depends on `sema::Compiler`, interpreter hooks, `jaic-llvm` and `stdlib/internal/compiler-api`.

## Rules

Claims that tests cite by ID (`tools/check_rule_coverage.py`).

- Workspace 2 is the top-level program (`TOP_LEVEL_WORKSPACE`) {#ws.1}. Workspace 1 is reserved, as in Jai, so the first workspace a metaprogram creates is 3 {#ws.2}.
- | `__jaic_workspace_create(name) -> s64` | new workspace id {#ws.3} |
- | `__jaic_current_workspace() -> s64` | workspace whose compile-time code is running {#ws.4} |
- | `__jaic_report(msg, file, line, col, mode)` | `mode` 0 (`ERROR`) fails the compile {#ws.5}; 2 and 3 (`WARNING`, `INFO`) go to the report sink {#ws.6}; 1 (`ERROR_CONTINUABLE`) goes to the sink as an error and marks the current workspace failed |
- 1. **Open to Checked**: `Compiler::begin_sources` loads the bootstrap and queued sources and runs their `#run`s. Events: FILE..., PHASE `ALL_SOURCE_CODE_PARSED`, PHASE `TYPECHECKED_ALL_WE_CAN` {#ws.7}.
- 2. **Checked with new sources** (the metaprogram called `add_build_string` after `TYPECHECKED_ALL_WE_CAN`): `add_source` and `settle`, new FILE events, `TYPECHECKED_ALL_WE_CAN` again {#ws.8}.
- The registry borrow is released while a compiler runs, so nested metaprograms work {#ws.9}. Sources a `#run` adds to its own workspace (`add_build_string(s, -1)`, for example to define a `#placeholder`) are pulled by `Compiler::pull_workspace_sources` right after that `#run`; `run_top_level` is incremental {#ws.10}. `compiler_modify_procedure` calls are queued and applied at the start of the next `step`, so bodies are replaced before they are lowered {#ws.14}.
- | `FILE` (1) | int 0 = record; one per loaded file {#ws.11} |
- | `TYPECHECKED` (5) | int 0 = record; just before PHASE `TYPECHECKED_ALL_WE_CAN` {#ws.12} |
- | `COMPLETE` (3) | int 0 = error code (0 none, 1 failed) {#ws.13} |
- When `do_output` is set and `output_type != NO_OUTPUT`, the embedder's `OutputBackend::write_output` gets the IR program and `BuildSettings` (path `output_path/output_executable_name`). `jaic` and `jaic -interp` pass an LLVM backend (object file, `cc` link, `-shared` for `DYNAMIC_LIBRARY`, `ar` for `STATIC_LIBRARY`): `-interp` differs from a build only in interpreting the top-level program instead of compiling it, so a build metaprogram (`jaic first.jai -interp`) writes and can launch what it builds. `jaic -check`, `jaic -interp -no_workspace_output` (used by `tools/jaic-sweep.py`), the browser, jailsp and jailint pass none, so workspaces are only checked {#ws.15}. Without a backend, `BuildEnv::unwritten_output_hint` decides whether a workspace that asks for output is told it was not written: `jaic -check` warns ``warning: `jaic -check` does not write build/game (workspace `Build` asks for an executable)`` with ``help: `jaic first.jai` (or `jaic first.jai -interp`) writes it`` (the path is shown relative to where jaic was started, the main file as it was typed); `-no_workspace_output` and the tools set `None` and say nothing {#ws.17}.
- Workspace 2's own settings decide whether `jaic` writes the top-level program, so a metaprogram calling `set_build_options_dc(.{do_output = false})` produces no output of its own {#ws.16}.
