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

The plugin runs successful executable workspace builds on macOS, Linux, and Windows. It joins the output directory and executable name, preserves absolute executable names, resolves a full path, and forwards arguments through `Process.run_command`. On Windows an extensionless request first checks the exact file and then tries `.exe`. Output directories do not need a trailing separator. Non-executable outputs are logged and skipped; process-launch and nonzero-exit failures are logged as errors.

The implementation is `stdlib/internal/build/autorun.jai`. Keep workspace-completion filtering and argument ownership in the plugin lifecycle, and executable resolution in `run_native`; extend `stdlib/Autorun/tests/basics.jai` when changing either. The regression builds a child workspace, forwards arguments, checks plugin shutdown, and exercises missing executables and nonzero exits on each native host. The module has no extra configuration: workspace build options select the executable, and plugin arguments become its argument array. Dependencies are `Compiler`, `Process`, `File`, `File_Utilities`, `String`, and the host's process services. Ordinary cross compilation does not run foreign-platform executables.
