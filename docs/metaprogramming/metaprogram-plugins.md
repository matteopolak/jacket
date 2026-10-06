# Metaprogram plugins (`-plug`)

## What it is

`jaic check|build file.jai -plug Name` compiles `file.jai` with the plugin module `Name` hooked into the compilation, as `jai file.jai -plug Name` does {#plugin.1}. Profilers use this to instrument every procedure (`-plug tracy` from rluba/jai-tracy, the stdlib's `-plug Iprof`); lint plugins use it to inspect typechecked code.

## How it works

A plugin is a module with `get_plugin :: () -> *Metaprogram_Plugin` {#plugin.2}. The struct (in `stdlib/Compiler/workspace.jai`) holds hooks: `init` (newer plugins: all options at once, may refuse them), `handle_one_option` (older plugins: one option at a time), `before_intercept`, `add_source`, `message`, `finish`, `shutdown`, `log_help`.

The official default metaprogram imports plugins by name from inside its `#run`. jaic can't `#import` a name computed during `#run`, but with `-plug` the names are known up front, so `plugin_metaprogram` in `crates/jaic-cli/src/main.rs` generates a small program and compiles that instead:

```jai
Compiler :: #import "Compiler";
Plugins :: #import "Metaprogram_Plugins";
#import "Basic";
__plugin_0 :: #import "tracy";
#run,stallable {
    plugins: [..] *Compiler.Metaprogram_Plugin;
    array_add(*plugins, __plugin_0.get_plugin());
    options: [..] string;
    array_add(*options, "-min_size");
    array_add(*options, "1");
    Plugins.build_with_plugins("/abs/path/file.jai", plugins, options, "file", "");
}
```

`build_with_plugins` (`stdlib/Metaprogram_Plugins.jai`) creates the "Target Program" workspace, delivers options, calls `before_intercept`, starts intercepting, calls `add_source`, adds the file, forwards every compiler message to each plugin's `message` until `COMPLETE`, then calls `finish` and `shutdown` {#plugin.3}. A failed compilation marks the workspace failed, so jaic exits non-zero {#plugin.4}.

Command line (`parse` in `main.rs`):

- `-plug Name` (or `-plugin`) may repeat {#plugin.5}. `-plug "Check(CHECK_BINDINGS=false)"` passes module parameters {#plugin.6}.
- The first argument jaic doesn't know, and every non-option argument after it, is a plugin option (`-plug tracy -min_size 1 -modules`) {#plugin.7}. Options jaic knows (`-I`, `-o`, `-os`) are still taken by jaic wherever they appear {#plugin.8}. Unknown options without `-plug` are a usage error (exit 2) {#plugin.9}.
- `run` with `-plug` is a usage error: the program compiles in its own workspace, which `jaic run` can't start {#plugin.10}. Use `build` and run the executable.
- `build -o path` names the executable; otherwise it is named after the file and written next to it {#plugin.11}.

## How to change it

- Option delivery: `deliver_plugin_options` in `Metaprogram_Plugins.jai`. Plugins with `init` get all options; the rest share them through `handle_one_option`, and an option no plugin claims fails the build.
- New hook: add the field to `Metaprogram_Plugin` and call it from `build_with_plugins`.
- `init_plugins` (choosing plugins from inside a running metaprogram) still reports an error, because it needs `#import` of a name computed during `#run`.
- Tests: `plug_hooks_a_plugin_into_check` in `crates/jaic-cli/tests/cli.rs`, `plug_builds_the_plugin_workspace` in `crates/jaic-cli/tests/native.rs` (fixture plugin in `tests/common/mod.rs`), and the `jai-tracy-plugin-*` upstream sweep cases.

## Configuration

Command-line flags only. Plugins are found on the normal import path: the file's `modules/`, `-I` directories, the stdlib.

## Dependencies

The `Compiler` module (workspaces, `compiler_begin_intercept`, `compiler_wait_for_message`, `compiler_modify_procedure`); see [workspaces](workspaces.md) and [Compiler module](compiler-module.md).
