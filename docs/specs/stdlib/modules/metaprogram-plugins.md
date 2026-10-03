# Metaprogram_Plugins

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `Plugin_To_Create :: struct`
- `parse_plugin_arguments :: (args: [] string, starting_index := 0) -> (success: bool, plugins: [..] Plugin_To_Create, remaining_args: [] string)`
- `init_plugins :: (to_create: [] Plugin_To_Create, results: *[..] *Metaprogram_Plugin, w: Workspace) -> (success: bool)`

## Public type members

#### `Plugin_To_Create` (struct)

- `name:    string`
- `options: [..] string`

## Behavioral contract and constraints

`parse_plugin_arguments` starts scanning at `starting_index`. Before the first plugin marker, arguments are retained as ordinary arguments. A plugin marker is any nonempty argument whose first character is `+`; the marker character is removed and the rest of that argument becomes the plugin name. Following arguments belong to that plugin as options until the next `+` marker or an argument exactly equal to `-`; other dash-prefixed strings remain plugin options. The exact `-` ends plugin parsing and is consumed; all later arguments are returned as ordinary arguments. Arguments before the first plugin and after the consumed terminator are concatenated in their original order. If a standalone `-` occurs before any plugin marker, scanning stops and the no-plugin path returns the original `args` slice unchanged. If no plugin marker is found for any reason, no plugins are returned. For `0 <= starting_index <= args.count`, parsing reports success; behavior outside that range is not specified.

Plugin records and their option arrays are constructed as output collections. Plugin names and option strings are slices of the input argument strings, and the ordinary-argument result is built from those input strings when plugin parsing occurs. Callers must keep the input string storage alive while using these values. The no-plugin path returns the original argument slice. No release procedure is exported for these collections; their allocation follows the active allocation context.

An optional module-parameter suffix may follow a plugin name, beginning at its first `(`. The portion before that character is the module to import; the suffix is passed through as the import parameter text. The name/options syntax is passed to the plugin's `get_plugin` and `init` interfaces; this parser does not promise a separate validation grammar for module names, parameter expressions, or option spellings.

`init_plugins` is intended to run during the metaprogram's compile-time build/setup phase, before the caller needs the plugin objects. The request names may be computed by that metaprogram; they are not restricted to top-level command-line strings, and the API accepts no factory callback or pre-registered pointer. The helper uses the public compiler build-source facility (`Compiler.add_build_string`) to request imports for the supplied module names and factory calls for those imports. A successful call completes this plugin setup before returning: `results` contains the corresponding plugin objects and their initializers have run. This is compile-time module loading, not runtime module discovery. The requested names must be usable as imports in the metaprogram's build context, and each imported module must provide the expected factory.

Neither exported procedure carries a `#expand`, `#compile_time`, or `#bake` attribute. Synchronous completion relies on public language facilities: `#placeholder` declares a name whose definition may arrive later, a procedure value can be available from its signature while compilation of its body waits on that placeholder, and compile-time invocation through that value can wait for the body to become available. `#insert` can consume the resulting string as code in a procedure body. The compiler therefore must permit a running compile-time computation to resume after generated declarations resolve an explicitly deferred body dependency. A direct call to an undeclared generated function is not equivalent to this supported boundary.

The public scoped overload of `Compiler.add_build_string` accepts a `code` argument that identifies the destination scope. A code value captured in the module's scope permits a generated declaration to satisfy a placeholder in that scope when targeting the caller's own workspace (`w = -1`). The plugin imports and typed factory uses are inserted in a procedure body after this dependency resolves. Message interception, a message pump, and an explicit yield are not requirements of this helper. The contract does not promise general-purpose immediate execution of arbitrary build strings or runtime module discovery.

Importing this module creates a deferred definition requirement: callers must invoke `init_plugins`, including with an empty request list when appropriate, to satisfy it. Merely importing the module without an initialization call can leave compilation unable to complete.

`init_plugins` attaches the supplied workspace to each returned plugin and invokes its initializer with its associated options. A plugin without an initializer accepts no options; supplying options to it causes failure. The success result aggregates initializer success. Compiler module-load/type errors remain compiler diagnostics.

The compiler's `Metaprogram_Plugin.communication_proc` is a separate string-based way to communicate with plugin instances after they have been created. It is not a factory-registration callback and does not make a runtime string into an import by itself.

The generated import declarations are fixed by the first call. A later call with a different ordered plugin-name list, including any parameter suffix, reports failure. The interface is intended for one initialization pass; it does not promise successful repeated initialization or replacement/removal of plugin modules. Plugin option values are supplied to initialization and are not part of this name-list consistency check. A call with no plugins succeeds without producing plugin entries. The input list may be formed by compile-time metaprogram logic; the interface does not require names to originate from the top-level compiler command line.
