# Metaprogram plugins (`+Name`)

## What it is

`jaic file.jai +Name` compiles `file.jai` with the plugin module `Name` hooked into the compilation {#plugin.1}. Profilers use this to instrument every procedure (`+tracy` from rluba/jai-tracy, the stdlib's `+Iprof`); lint plugins use it to inspect typechecked code. `-check` runs the plugins without writing anything.

## How it works

A plugin is a module with `get_plugin :: () -> *Metaprogram_Plugin` {#plugin.2}. The struct (in `stdlib/Compiler/workspace.jai`) holds hooks: `init` (newer plugins: all options at once, may refuse them), `handle_one_option` (older plugins: one option at a time), `before_intercept`, `add_source`, `message`, `finish`, `shutdown`, `log_help`.

The official default metaprogram imports plugins by name from inside its `#run`. jaic can't `#import` a name computed during `#run`, but with `+Name` the names are known up front, so `plugin_metaprogram` in `crates/jaic-cli/src/main.rs` generates a small program and compiles that instead:

```jai
Compiler :: #import "Compiler";
Plugins :: #import "Metaprogram_Plugins";
#import "Basic";
__plugin_0 :: #import "tracy";
#run,stallable {
    plugins: [..] *Compiler.Metaprogram_Plugin;
    array_add(*plugins, __plugin_0.get_plugin());
    options: [..] string;
    counts: [..] s64;
    array_add(*options, "-min_size");
    array_add(*options, "1");
    array_add(*counts, 2);
    added: [..] string;
    Plugins.build_with_plugin_groups("/abs/path/file.jai", plugins, options, counts, added, "file", "");
}
```

`build_with_plugin_groups` (`stdlib/Metaprogram_Plugins.jai`; `build_with_plugins` is the same with one option list shared by all plugins) creates the "Target Program" workspace, delivers options, calls `before_intercept`, starts intercepting, calls `add_source`, adds the file, forwards every compiler message to each plugin's `message` until `COMPLETE`, then calls `finish` and `shutdown` {#plugin.3}. A failed compilation marks the workspace failed, so jaic exits non-zero {#plugin.4}.

Command line (`parse` in `main.rs`):

- `+Name` starts a plugin; it may repeat {#plugin.5}. Module parameters go in the name: `'+Check(CHECK_BINDINGS=false)'` (quote it, since shells treat the parentheses specially) {#plugin.6}.
- Every argument after `+Name`, up to the next `+Name`, a lone `-` or `--`, is an option of that plugin and of no other (`jaic file.jai +tracy -min_size 1 +Iprof -modules`). Each plugin is handed only its own list: `init` gets it all at once, and a plugin with `handle_one_option` has to claim every entry of it {#plugin.7}. So jaic's own options (`-I`, `-exe`, `-os`) and the file go before the first `+Name`; after it they are the plugin's {#plugin.8}. An option jaic does not know before the first plugin is a usage error (exit 2) {#plugin.9}.
- `-interp` with a plugin is a usage error: the program compiles in its own workspace, which `-interp` can't start {#plugin.10}. Build it and run the executable.
- `-exe` and `-output_path` name the executable; otherwise it is named after the file and written next to it {#plugin.11}. `-add` and `-run` strings are added to the plugin workspace after the file.
- `-no_check_bindings` sets `CHECK_BINDINGS=false` on a `+Check` that has no value for it.
- The old `-plug Name` and `-plugin Name` are an error that names `+Name`.

## How to change it

- Option delivery: `deliver_grouped_options` and `deliver_plugin_options` in `Metaprogram_Plugins.jai`. Plugins with `init` get their options; the rest take them through `handle_one_option`, and an option nobody claims fails the build.
- New hook: add the field to `Metaprogram_Plugin` and call it from `build_plugin_workspace`.
- `init_plugins` (choosing plugins from inside a running metaprogram) still reports an error, because it needs `#import` of a name computed during `#run`.
- Tests: `plug_hooks_a_plugin_into_check` in `crates/jaic-cli/tests/cli.rs`, `plug_builds_the_plugin_workspace` in `crates/jaic-cli/tests/native.rs` (fixture plugin in `tests/common/mod.rs`), and the `jai-tracy-plugin-*` upstream sweep cases.

## Configuration

Command-line arguments only. Plugins are found on the normal import path: the file's `modules/`, `-I` directories, the stdlib.

## Dependencies

The `Compiler` module (workspaces, `compiler_begin_intercept`, `compiler_wait_for_message`, `compiler_modify_procedure`); see [workspaces](workspaces.md) and [Compiler module](compiler-module.md).
