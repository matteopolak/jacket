# Example_Plugin

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `HELP_STRING :: #string`
- `My_Plugin :: struct`
- `add_source :: (p: *Metaprogram_Plugin)`
- `before_intercept :: (p: *Metaprogram_Plugin, flags: *Intercept_Flags)`
- `finish :: (p: *Metaprogram_Plugin)`
- `get_plugin :: () -> *Metaprogram_Plugin`
- `init :: (p: *Metaprogram_Plugin, options: [] string) -> bool`
- `log_help :: (p: *Metaprogram_Plugin)`
- `message :: (p: *Metaprogram_Plugin, message: *Message)`
- `shutdown :: (p: *Metaprogram_Plugin)`

## Public type members

#### `My_Plugin` (struct)

- `base: Metaprogram_Plugin` (embedded with `#as using`)
- `some_state := 0`

No public aggregate fields or enum members were identified.

## Behavioral contract and constraints

`get_plugin` returns a newly allocated `My_Plugin`, whose embedded `Metaprogram_Plugin` is the loader-facing record. The sample initializes `some_state` to `33` during `init`; it accepts zero or more `-option VALUE` pairs, requires a value after each `-option`, and rejects any other plugin option by returning false. The option's value is reported, while the sample does not use it to change later behavior.

The lifecycle illustrates the compiler callbacks: `before_intercept` announces startup after target build options are set; `add_source` runs after options are finalized and before compiler messages begin; `message` receives each compiler message and reports phase, typecheck counts, and completion; `finish` reports the number of messages seen; `shutdown` announces teardown and frees the plugin allocation. The sample's `.PHASE` handling reports the executable name at `PRE_WRITE_EXECUTABLE`. It does not add source code or create build artifacts. The `log_help` callback prints an independently described option synopsis: `-option` requires one value.

All callbacks are optional in the general plugin record; absent callbacks are skipped by the loader. This example installs all available lifecycle callbacks and owns the allocated plugin record until `shutdown`. It is a template for plugin lifecycle and option handling, not a general-purpose service API.
