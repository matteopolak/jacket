# Workspaces, metaprograms and plugins

The Rust side of the `Compiler` module, in `crates/jaic/src/build.rs`. Metaprograms (`build.jai`, `first.jai`) create workspaces, add sources, set build options and read compiler messages through `#compiler` primitives; the Jai API on top is in the [Compiler module](compiler-module.md). This page also covers `+Name` plugins.

```
jaic build.jai - arg1 arg2      # arg1 arg2 go to the metaprogram
jaic file.jai +Check            # compile file.jai with the Check plugin hooked in
```

## How it works

The embedder creates `Workspaces` with a `BuildEnv`: file system, base `Options`, an optional `OutputBackend`, arguments after `-`, a report sink (it is told each message's severity, which is how `-quiet` drops warnings) and an optional `WorkspaceObserver` ([jailint](../tools/jailint.md) lints what a metaprogram builds). Workspace 2 is the top-level program; workspace 1 is reserved, so the first created is 3.

Public `#compiler` declarations bind to `Hook::PublicCompiler` through `build/public_api`, which validates the signature and resolves the real layouts; handlers then call the typed engine (`build/engine.rs`). Supplied Jai bodies are runtime implementations; compile-time execution does not run them.

### State machine

`build::step` advances a workspace when its events are read, or from `build::finish_all` for workspaces nobody intercepted.

1. **Open to Checked.** `Compiler::begin_sources` loads the bootstrap and sources and runs their `#run`s. Events: FILE..., PHASE `ALL_SOURCE_CODE_PARSED`, PHASE `TYPECHECKED_ALL_WE_CAN`.
2. **New sources.** `add_source` and `settle`, more FILE events, `TYPECHECKED_ALL_WE_CAN` again.
3. **Checked to Done.** `finish_program` lowers, output is written, COMPLETE.

Never hold `shared.borrow_mut()` across `compile_sources` or a backend call: the registry borrow is released while a compiler runs so nested metaprograms work. One interpreter budget (`Interp::block_budget`) bounds a whole compilation.

Events: `FILE` (1), `IMPORT` (4) and `TYPECHECKED` (5) exist only for intercepted workspaces. `PHASE` (2) carries 0 `ALL_SOURCE_CODE_PARSED`, 1 `TYPECHECKED_ALL_WE_CAN`, 2 `ALL_TARGET_CODE_BUILT`, 3 `PRE_WRITE_EXECUTABLE`, 4 `POST_WRITE_EXECUTABLE`, 5 `READY_FOR_CUSTOM_LINK_COMMAND`. `COMPLETE` (3) carries 0 or 1.

### Output

With `do_output` and `output_type != NO_OUTPUT`, `OutputBackend::write_output` gets the IR program and `BuildSettings`. `jaic` and `jaic -interp` pass an LLVM backend; `-interp` differs only in interpreting the top-level program, so a build metaprogram can launch what it builds. `-check`, `-no_workspace_output`, the browser, jailsp and jailint pass none, so workspaces are only checked. `BuildEnv::unwritten_output_hint` makes `-check` warn when a workspace asks for an executable.

### Plugins

`jaic file.jai +Name` builds the target in a child workspace using the `Metaprogram_Plugin` hooks (`stdlib/Compiler/workspace.jai`): `init`, `handle_one_option`, `before_intercept`, `add_source`, `message`, `finish`, `shutdown`, `log_help`. `build_with_plugins` (`stdlib/Metaprogram_Plugins.jai`) drives the lifecycle. Profilers use plugins to instrument procedures, and lint plugins to inspect typechecked code.

## How to change it

- New public operation: its registry entry, signature validation and handler under `build/public_api`, the typed engine call in `build/engine.rs`, a public `#compiler` declaration, and compile-time and runtime tests.
- New option: handle the key in `Workspaces::set_option` and read it in `new_compiler` or `write_output` ([build options](build-options.md)).
- New event kind: push it in `step`; strings given to Jai go through `keep_string`.
- New plugin hook: add the field to `Metaprogram_Plugin` and call it from `build_with_plugins`. Options go through `deliver_plugin_options`.
- Tests: `plug_hooks_a_plugin_into_check` in `crates/jaic-cli/tests/cli.rs`, `crates/jaic-cli/tests/plugins.rs`.

## Rules

Claims that tests cite by ID (`tools/dev/check_rule_coverage.py`).

### Workspaces

- Workspace 2 is the top-level program {#ws.1}. Workspace 1 is reserved, so the first workspace a metaprogram creates is 3 {#ws.2}.
- `compiler_create_workspace(name)` returns a new workspace id {#ws.3}. `get_current_workspace()` returns the workspace whose compile-time code is running {#ws.4}.
- `compiler_report(msg, loc, mode)`: mode 0 (`ERROR`) fails the compile {#ws.5}; 2 and 3 (`WARNING`, `INFO`) go to the report sink {#ws.6}; 1 (`ERROR_CONTINUABLE`) goes to the sink as an error and marks the current workspace failed.
- Open to Checked emits FILE..., PHASE `ALL_SOURCE_CODE_PARSED`, PHASE `TYPECHECKED_ALL_WE_CAN` {#ws.7}.
- If the metaprogram calls `add_build_string` after `TYPECHECKED_ALL_WE_CAN`, `add_source` and `settle` run, new FILE events follow, and `TYPECHECKED_ALL_WE_CAN` is sent again {#ws.8}.
- The registry borrow is released while a compiler runs, so nested metaprograms work {#ws.9}.
- Sources a `#run` adds to its own workspace (`add_build_string(s, -1)`, say to define a `#placeholder`) are pulled by `pull_workspace_sources` right after that `#run`; `run_top_level` is incremental {#ws.10}. `compiler_modify_procedure` calls are queued and applied at the start of the next `step`, so bodies are replaced before they are lowered {#ws.14}.
- `FILE` (1): int 0 = record, one per loaded file {#ws.11}. `TYPECHECKED` (5): int 0 = record, just before PHASE `TYPECHECKED_ALL_WE_CAN` {#ws.12}. `COMPLETE` (3): int 0 = error code (0 none, 1 failed) {#ws.13}.
- When `do_output` is set and `output_type != NO_OUTPUT`, `write_output` gets the IR program and `BuildSettings` (path `output_path/output_executable_name`). `jaic` and `jaic -interp` pass an LLVM backend (object file, `cc` link, `-shared` for `DYNAMIC_LIBRARY`, `ar` for `STATIC_LIBRARY`). `jaic -check`, `jaic -interp -no_workspace_output`, the browser, jailsp and jailint pass none, so workspaces are only checked {#ws.15}.
- Workspace 2's own settings decide whether `jaic` writes the top-level program, so `set_build_options_dc(.{do_output = false})` produces no output {#ws.16}.
- Without a backend, `unwritten_output_hint` tells a workspace asking for output that it was not written: `jaic -check` warns ``warning: `jaic -check` does not write build/game (workspace `Build` asks for an executable)`` plus ``help: `jaic first.jai` (or `jaic first.jai -interp`) writes it``. `-no_workspace_output` and the tools say nothing {#ws.17}.

### Plugins

- `jaic file.jai +Name` compiles `file.jai` with the plugin module `Name` hooked in {#plugin.1}.
- A plugin is a module with `get_plugin :: () -> *Metaprogram_Plugin`, exported or under `#scope_module`; file-scope factories are inaccessible {#plugin.2}.
- `build_with_plugins` creates the "Target Program" workspace, delivers options, calls `before_intercept`, starts intercepting, calls `add_source`, adds the file, forwards every message to each plugin's `message` until `COMPLETE`, then calls `finish` and `shutdown` {#plugin.3}. A failed compilation, or an option no plugin accepts, makes jaic exit non-zero {#plugin.4}.
- `+Name` may repeat, and each plugin gets its own options {#plugin.5}. Module parameters are part of the name: `'+Check(CHECK_BINDINGS=false)'` {#plugin.6}.
- Everything after `+Name` up to the next `+Name`, a lone `-` or `--` belongs to that plugin (`+tracy -min_size 1`) {#plugin.7}. jaic's own options before the first plugin stay jaic's {#plugin.8}.
- An option nobody takes is a usage error (exit 2) {#plugin.9}. With `-interp` the plugin workspace is compiled as for a build but its program is run instead of written (`run_instead_of_writing`, `take_program_to_run`); a workspace that asks for no output leaves nothing to run {#plugin.10}. `main` (or a plugin's `entry_point_name`) gets the arguments after `--`, and its status is jaic's.
- The plugin workspace's program is written to `-exe` (a bare name goes in the starting directory) or, without it, next to the source under the source's name {#plugin.11}.

### Bootstrap modules

- Every program implicitly loads two bootstrap modules {#prelude.1}: Preload (compiler-owned declarations: `Type_Info`, `Allocator`, intrinsics) and Runtime_Support (entry point, context initialisation, write helpers).
- `Compiler::load_bootstrap` (`sema/driver.rs`) loads Preload, then, if `Options::runtime_support` is set, Runtime_Support with `DEFINE_SYSTEM_ENTRY_POINT=true`, `DEFINE_INITIALIZATION=true`, `ENABLE_BACKTRACE_ON_CRASH=false` and `TEMPORARY_STORAGE_SIZE` from `Options::temporary_storage_size`. Both are visible from every module {#prelude.2}.
- A program can import Runtime_Support itself with other arguments and gets its own instance {#prelude.3}. Without the entry point there is no exported `main`.
- `context_type` (`sema/structs.rs`) builds `Context`: Preload's `FIRST_ADD_CONTEXT` first, resolved in Runtime_Support's scope (which defines `Context_Base`), then every `#add_context` in load order {#prelude.4}.
- The intrinsics in `prelude/intrinsics.jai` are bound by name in `sema/calls/intrinsics.rs` {#prelude.5}; see [intrinsics](../language/intrinsics.md).

Keep Preload declarative: no imports, `#run` or side effects. Runtime behaviour (allocators, temporary storage, crash output) belongs in Runtime_Support, whose module parameters are the contract with `load_bootstrap`.
