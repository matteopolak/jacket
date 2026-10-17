# Metaprogram plugins (`+Name`)

`jaic file.jai +Name` builds the target in a child workspace using the reviewed
`Compiler.Metaprogram_Plugin` hooks. `-check` runs the same lifecycle without
writing an executable. Plugins expose a compatible `get_plugin`; opaque plugin
pointers may convert through their leading `Metaprogram_Plugin` base.

The CLI generates ordinary source that imports each plugin, creates a workspace,
sets its build options, assigns the workspace to each plugin and calls `init`
with that plugin's own option array. Null plugin results and refused options fail
compilation. Options without an init hook are diagnosed. Initialization failures
call shutdown on acquired plugins before reporting the failure.

It then calls `before_intercept`, starts interception, invokes `add_source`, adds
the target and CLI-added sources, and dispatches compiler messages until the
workspace's COMPLETE event. It ends interception and calls finish and shutdown.
A failed COMPLETE marks the workspace failed. The wrapper workspace requests
NO_OUTPUT; the child retains the intended build settings. Plugin initialization
can amend those child settings before compilation starts.

This orchestration lives privately in `crates/jaic-cli/src/main.rs`. It does not
require `build_with_plugin_groups`, `build_with_plugins`, `handle_one_option`, or
other helpers absent from reviewed public declarations. `Metaprogram_Plugins`
retains its separately reviewed parsing/initialization API.

- `+Name` may repeat; module parameters belong in the name, for example
  `'+Check(CHECK_BINDINGS=false)'`.
- Arguments following a plugin, until another plugin or the argument separator,
  belong to that plugin. Put compiler options before the first plugin.
- `-interp` with plugins is a usage error; build and run the child executable.
- `-exe`, `-output_path`, `-add` and `-run` apply to the child. WASI runtime
  insertion follows the same build-target decision as a normal CLI build.

Regression: `crates/jaic-cli/tests/plugins.rs` checks lifecycle,
option refusal, check-only behavior and actual native child execution.
