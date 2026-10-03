# Minimal_Metaprogram

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `is_absolute_path :: (path: string) -> bool`
- `build :: ()`

## Public type members

No public aggregate fields or enum members were identified.

## Behavioral contract and constraints

`build()` creates a workspace named `Target Program`; if workspace creation fails, it prints a message and returns. It reads `options.compile_time_command_line`, skips empty strings, and treats every nonempty argument beginning with `-` as an option. It recognizes `-no_cwd`, `-release`, `-x64`, and `-llvm`; other dash-prefixed strings are ignored. Every non-option string is appended as a source file, preserving argument order. It does not intercept compiler messages or return a status.

`-release` calls `set_optimization(*options, .OPTIMIZED)`, disables stack traces, and commits the changed build options. This selects the `.OPTIMIZED` tier (LLVM O2 and machine-code `DEFAULT` in the public compiler contract), not `.VERY_OPTIMIZED`. `-no_cwd` disables the initial working-directory change. `-x64` and `-llvm` assign a local backend selection but do not themselves mark options modified; absent another option/output-path change that triggers `set_build_options`, these choices are not committed. The default backend/optimization settings therefore otherwise come from the workspace options.

The first source argument determines output naming and, when it includes a directory, output location. The path portion is made absolute relative to the original working directory using this module's `is_absolute_path` heuristic; the basename becomes `output_executable_name`. If no path portion is present, the original working directory is used as the source working directory. Unless `-no_cwd` is set, the process working directory is changed to that directory and each relative source argument is rewritten relative to the original working directory before being added to the workspace. This is intended to anchor relative `#import` paths to the first source's directory; the module does not itself implement additional import search or path resolution. With `-no_cwd`, it leaves relative source strings and the process cwd unchanged. Minimal does not pass a separate `source_relative_path` value or an alternate `loc` to `add_build_file`; that API's public signature has no `source_relative_path` argument, and its default `#caller_location` makes the Minimal_Metaprogram call site the base for a remaining relative filename. Thus under `-no_cwd`, a relative source argument is resolved against the directory containing the `add_build_file` call in Minimal_Metaprogram, not automatically against the metaprogram process cwd or the helper that invoked Minimal. In the default cwd-changing mode, relative arguments are rewritten to absolute paths before `add_build_file` is called. The working directory is not restored by `build()`. It adds all collected source paths through `add_build_file` and sets compile-time direct options to `do_output=false` and `write_added_strings=false` so the metaprogram itself does not request an executable or write added strings.

`is_absolute_path` is a deliberately limited string heuristic: empty strings are false; a leading `/` or `\\` is true; on Windows, a path longer than two bytes with `:` at index 1 is true; and on every target, a string of at least three bytes with `:` at index 1 is true. It does not normalize or validate a path, and a two-byte drive prefix alone is not accepted. No filesystem existence or import-resolution guarantee is added. This module's option and path handling is intentionally narrower than `Default_Metaprogram`.
