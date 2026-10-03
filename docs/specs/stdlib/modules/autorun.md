# Autorun

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `run_build_result_of_workspace :: (w: Workspace, args : [] string = .[])`
- `run_native :: (executable_path_without_extension: string, args: []string)`
- `get_plugin :: () -> *Plugin`
- `init :: (_p: *Plugin, options: [] string) -> bool`
- `message :: (_p: *Plugin, message: *Compiler.Message)`
- `shutdown :: (_p: *Plugin)`

## Behavioral contract and constraints

The plugin runs a workspace build only when its output type is executable. It resolves the executable name from build options, appends `.exe` on Windows, forwards the argument array, and uses `Process.run_command`; a callback module parameter can replace native execution. Non-executable outputs are logged and skipped. Process launch and exit errors are reported through the Process result/log path.
