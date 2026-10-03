# Default_Metaprogram

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `is_absolute_path :: (path: string) -> bool`
- `build :: ()`
- `message_loop :: (w: Workspace)`
- `HELP_STRING :: #string DONE`

- `plugins: [..] *Metaprogram_Plugin`
- `verbose := false`

## Public type members

No public aggregate fields or enum members were identified.

## Behavioral contract and constraints

`build` creates a target workspace, parses command-line arguments, initializes requested plugins, applies build options, adds requested source/code, and intercepts compiler messages. It passes each message to every plugin that has a `message` callback until `COMPLETE`, ends interception, then calls plugin `finish` callbacks followed by `shutdown` callbacks. An invalid or unknown command-line option exits with status 1. A plugin initialization failure is logged and stops further build work through the source's `exit(0)` path. Workspace creation failure is logged and returns.

### Command-line behavior

Arguments before the first plugin segment remain metaprogram options or source inputs. A plugin segment begins with `+Name`; its following arguments belong to that plugin until another `+Name` or a standalone `-`. A standalone `-` ends plugin argument collection and marks the remaining arguments as target-program command-line arguments. Plugin modules are named by the text after `+`; an optional parenthesized module parameter suffix is supported by the loader. Plugin options are delivered to that plugin's `init` callback as strings. `-help`/`-?` print this metaprogram's option help and each initialized plugin's help callback; a help-only invocation exits successfully. `-version` prints compiler version information; version-only invocation exits successfully. With no source, added code, run code, help, or version request, the metaprogram reports that an input is required and exits unsuccessfully.

For each requested `+Name`, the plugin loader imports the named module and calls its module-qualified `get_plugin()` factory, which must return `*Compiler.Metaprogram_Plugin`. The built-in `Check` plugin uses that factory; its checker does not expose a separate constructor API.

Supported options:

- `-add CODE` queues code to add to the target workspace, with a statement terminator supplied by the metaprogram.
- `-run CODE` queues code as a compile-time `#run` directive.
- With source files present, `-exe NAME` and `-output_path PATH` explicitly set the output executable name and output directory. Otherwise, the first source file's basename and containing directory are used as defaults where available.
- `-import_dir PATH` adds an import search path. User-specified paths precede the target workspace's existing import paths; relative paths are resolved from the first source file's directory when one is provided.
- `-` puts all later arguments into the target program's `compile_time_command_line` field.
- `-o`/`-optimized` select very optimized output and disable stack traces; `-release` is a deprecated alias with a warning. `-od`/`-optimized_debug` select optimized output while retaining stack traces; `-release_debug` is a deprecated warning alias. `-very_debug` selects very-debug output.
- `-x64` selects the x64 backend; `-llvm` selects LLVM. `-no_inline` disables inlining, `-no_dce` disables dead-code elimination, `-no_split` disables LLVM split modules, and `-output_ir` enables LLVM IR output before optimization and regular LLVM IR output.
- `-quiet` suppresses compiler text output; `-no_color` disables ANSI color; `-msvc_format` selects Visual Studio diagnostic formatting; `-natvis` selects Natvis-compatible debug type names.
- `-debug_for` enables stepping into `for` expansion macros. `-debugger` enables the interactive compile-time debugger in this and the target workspace.
- `-no_backtrace_on_crash` sets crash backtrace handling off for the target workspace.
- `-no_check` omits the `Check` plugin. `-no_check_bindings` configures that plugin not to check module bindings; `-check_bindings` explicitly enables that check. Binding checks are on by default when `Check` is loaded.
- `-context_size N` sets the target context maximum when `N` parses as an integer, is at least `size_of(Context_Base)`, and is no greater than `0x40000` bytes. Invalid values are reported and ignored; the source does not mark these diagnostics as fatal.
- `-verbose` enables additional argument, inferred-output, working-directory and build summaries. The public global `verbose` defaults to false.

The path predicate `is_absolute_path` returns false for an empty string, true for paths beginning with `/` or `\\`, and true for any path of at least three bytes whose second byte is `:`. It does not validate drive-letter syntax; its source explicitly treats this as a limited heuristic, so callers should not rely on it as a general path normalizer.

The option parser is coupled to `Compiler.Build_Options`; this contract describes the supported options above, not a guarantee that every compiler option is forwarded. Added strings and `#run` snippets are compiler input, so they follow the language syntax and build workspace rules.
