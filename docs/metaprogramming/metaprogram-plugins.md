# Metaprogram plugins (`-plug`)

## What it is

`jaic check|build file.jai -plug Name` compiles `file.jai` with the metaprogram plugin module `Name` hooked into the compilation, like `jai file.jai -plug Name`. Profilers use this to instrument every procedure (`-plug tracy` from rluba/jai-tracy, the stdlib's `-plug Iprof`); lint plugins use it to inspect typechecked code.

## How it works

A plugin is a module with `get_plugin :: () -> *Metaprogram_Plugin`. The struct (in `stdlib/Compiler/workspace.jai`) holds hooks: `init` (newer plugins: all options at once, may refuse them), `handle_one_option` (older plugins: one option at a time), `before_intercept`, `add_source`, `message`, `finish`, `shutdown`, `log_help`.

The Jai's default metaprogram imports plugins by name while its `#run` executes. jaic cannot `#import` a computed name in the middle of a `#run`, but with `-plug` the names are known before compilation starts, so `crates/jaic-cli/src/main.rs` (`plugin_metaprogram`) writes a small program instead of compiling `file.jai` directly:

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

`build_with_plugins` (`stdlib/Metaprogram_Plugins.jai`) creates the "Target Program" workspace, hands the options to the plugins, calls `before_intercept`, starts intercepting, calls `add_source`, adds the file, forwards every compiler message to each plugin's `message` until `COMPLETE`, then calls `finish` and `shutdown`. A failed compilation marks the workspace failed, so jaic exits non-zero.

Command line rules (`parse` in `main.rs`):

- `-plug Name` (or `-plugin`) may repeat. `Name(PARAM=value)` passes module parameters: `-plug "Check(CHECK_BINDINGS=false)"`.
- An argument jaic does not know, and every non-option argument after it, is a plugin option (`-plug tracy -min_size 1 -modules`). Options jaic knows (`-I`, `-o`, `-os`) are still taken by jaic wherever they appear. Unknown options without any `-plug` are a usage error (exit 2).
- `run` with `-plug` is a usage error: the program is compiled in its own workspace, which `jaic run` cannot start. Use `build` and run the executable.
- `build -o path` names the executable; otherwise it is named after the file and written next to it.

## How to change it

- Option delivery: `deliver_plugin_options` in `Metaprogram_Plugins.jai`. Plugins with `init` get all options; the rest share them through `handle_one_option`, and an option no plugin claims fails the build.
- New hooks: add the field to `Metaprogram_Plugin` and call it in `build_with_plugins`.
- `init_plugins` (choosing plugins from inside a running metaprogram) still reports an error; it needs `#import` of a name computed during `#run`.
- Tests: `plug_hooks_a_plugin_into_check` in `crates/jaic-cli/tests/cli.rs`, `plug_builds_the_plugin_workspace` in `crates/jaic-cli/tests/native.rs` (fixture plugin in `tests/common/mod.rs`), and the `jai-tracy-plugin-*` upstream sweep cases.

## Configuration

Command-line flags only. Plugins are found on the normal import path (the file's `modules/`, `-I` directories, the stdlib).

## Dependencies

`Compiler` module (workspaces, `compiler_begin_intercept`, `compiler_wait_for_message`, `compiler_modify_procedure`); [workspaces](workspaces.md); [Compiler module](compiler-module.md).
