# Args (jaic extension)

## What it is

`Extensions/Args` is a typed command-line parser generated at compile time, in the spirit of Rust's clap derive. You declare a struct whose fields are the options; notes on the fields (`@short=c`, `@env=NAME`, `@range(1,64)`) refine them. `Args.parse(Cli, HELP)` returns the filled struct. It handles `--help` and `--version`, reports bad input with a usage hint and exits 2, and can print shell completion scripts. It is a jaic extension, not official Jai: other compilers do not have it, and it may change between releases. [`jaifmt`](../tools/jaifmt.md) uses it.

```jai
#import "Basic";
Args :: #import "Extensions/Args";

Color :: enum { AUTO; ALWAYS; NEVER; }

Cli :: struct {
    paths:   [..] string; @positional
    check:   bool;        @short=c
    jobs:    int = 4;     @short=j @env=TOOL_JOBS @range(1,64)
    color:   Color = .AUTO;
    config:  string;      @value=FILE
    verbose: u8;          @short=v @count
}

HELP :: Args.Help(Cli).{
    about = "Does things to files.",
    version = "1.0.0",
    paths = "Files or directories to process",
    check = "Do not write; only report",
    jobs = "Parallel jobs",
    color = "When to colour the output",
    config = "Use this config file",
    verbose = "Say more (repeat for more)",
};

main :: () {
    cli, set := Args.parse(Cli, HELP);
    if set.jobs print("jobs given explicitly: %\n", cli.jobs);
    print("% paths, color %, verbosity %\n", cli.paths.count, cli.color, cli.verbose);
}
```

```
$ tool --help
Does things to files.

Usage: tool [OPTIONS] [PATHS]...

Arguments:
  [PATHS]...           Files or directories to process

Options:
  -c, --[no-]check     Do not write; only report
  -j, --jobs <JOBS>    Parallel jobs [default: 4] [range: 1..64] [env: TOOL_JOBS]
      --color <COLOR>  When to colour the output [possible values: auto, always, never]
      --config <FILE>  Use this config file
  -v, --verbose        Say more (repeat for more)
  -h, --help           Print help
  -V, --version        Print version

$ tool --chek
error: unknown option `--chek`
help: did you mean `--check`?
usage: tool [OPTIONS] [PATHS]...
For more information, run `tool --help`.
```

`[default: ...]` appears only when the field's initial value is not the zero value (`0`, `""`, the first enum member), because Jai's type info does not say whether an initializer was written. An explicit `= .AUTO` on the first member is therefore not shown; bool flags never show one.


## API

| Name | What it does |
| --- | --- |
| `Args.parse(T, HELP) -> T, Is_Set(T)` | Parses the process arguments. `--help`/`--version` print and exit 0; an error is printed to stderr and exits 2. |
| `Args.try_parse(T, HELP, argv, environment = .[]) -> T, Error, Is_Set(T)` | No printing, no exit. `argv[0]` is the program name. `error.kind == .NONE` on success. |
| `Args.Help(T)` | The type of the help literal: `about`, `version`, then one `string` per option, named after the field. |
| `Args.Is_Set(T)` | One `bool` per field: true when the option was supplied (on the command line or by its `@env`). |
| `Args.completions(T, HELP, shell, program = "") -> string` | A completion script for `.BASH`, `.ZSH`, `.FISH` or `.POWERSHELL`. |
| `Args.help_text(T, HELP, program = "", color = false) -> string` | The page `--help` prints. |
| `Args.report(error)` | Prints an `Error` the way `parse` does. |
| `Args.Color_Mode`, `Args.set_color_mode`, `Args.color_enabled(stream)` | The colour rules (below). |

**Knowing whether an option was set.** `parse` and `try_parse` return a generated `Is_Set(T)` next to the struct, like [`Command_Line`](../specs/stdlib/modules/command-line.md)'s: `set.jobs` is true when the user gave `--jobs` or `TOOL_JOBS` filled it in. Because it is a struct with the same field names, a misspelled field is a compile error. A field that is a subcommand holds that subcommand's own `Is_Set`; a list is set once it has an item; `--no-check` counts as setting `check`.

**Errors.** `Error` has a `kind` (`UNKNOWN_OPTION`, `UNKNOWN_COMMAND`, `MISSING_VALUE`, `INVALID_VALUE`, `OUT_OF_RANGE`, `UNEXPECTED_VALUE`, `UNEXPECTED_ARGUMENT`, `MISSING_REQUIRED`, `MISSING_COMMAND`, `CONFLICT`, `REQUIRES`, plus `HELP` and `VERSION`, which are requests rather than failures and carry the page in `error.text`), the `message`, the `option` and `value` concerned, a `suggestion` (the nearest known name, by edit distance), and the `command` and `usage` of the command that failed. The first problem found wins. Strings in the result point into `argv`; lists are allocated with the context allocator.

## Syntax

`--long value`, `--long=value`, `-s value`, `-svalue`, `-s=value`, bundled short flags (`-abc`, `-cj8`), `--` ends the options (everything after it is a positional), and `-` alone is a positional. A bool field is a flag: `--check` sets it, `--no-check` clears it. A value may start with a dash when it is the argument of an option (`--wide -5`). Repeating a single-valued option keeps the last value. Enum values are matched case-insensitively against the lowercased, `-`-separated member names (`SAFE_MODE` is `safe-mode`).

## Field types

| Type | Option |
| --- | --- |
| `bool` | Flag, plus `--no-<name>` |
| integers (`u8` ... `s64`) | `--name <N>`, range-checked against the type |
| `float`, `float64` | `--name <N>` |
| `string` | `--name <TEXT>` |
| enum (not `enum_flags`) | `--name <choice>` |
| `[..] T` for a number, string or enum | A repeatable option (`--tag a --tag b`), or with `@positional` the trailing list |
| integer with `@count` | Counts occurrences (`-vvv` is 3) |
| a struct with `@subcommand` | A subcommand |

The option name is the field name with `_` turned into `-` (`output_dir` is `--output-dir`).

## Notes

Notes are whitespace-delimited tokens (`@range(1,64)`, never a quoted string), so they work in every Jai compiler's parser. All are checked when the program is compiled.

| Note | On | Meaning |
| --- | --- | --- |
| `@positional` | value fields | Read by position instead of by name. Positionals are taken in declaration order; only the last may be a list; a required one may not follow an optional one. |
| `@short=x` | options | One letter or digit. `h` (and `V`, when `version` is set) are taken. |
| `@long=name` | options | Replaces the name taken from the field. |
| `@env=NAME` | single-valued options | Used when the option was not given. An empty variable counts as unset. Errors name it as `$NAME`. |
| `@range(a,b)` | numeric fields | Inclusive bounds; an integer field needs integer bounds. |
| `@value=NAME` | value options, positionals | The placeholder shown in help (`--config <FILE>`). |
| `@count` | integer fields | Counts occurrences; takes no value. |
| `@required` | options, positionals | Must be given (by name or by `@env`). Scalar positionals are optional unless required. |
| `@conflicts=field` | any | Parse error when both are set. Repeat the note for several fields. |
| `@requires=field` | any | Parse error when this is set and `field` is not. |
| `@hidden` | options | Parsed, but left out of help and completions, and needs no help string. |
| `@global` | options | A top-level option that is also accepted after a subcommand. |
| `@subcommand` | struct fields; the selector enum | See below. |
| `@optional` | the selector enum | Running without a subcommand is allowed. |

## Compile-time checks

Every problem below is a compile error naming the field (`Args: Cli.jobs: ...`), found before the program runs:

unknown or malformed notes; a duplicate or reserved short or long name (including the `--no-` form of a bool, `--help`, `-h`); `@range` on a non-number or with bounds that are not numbers; unsupported field types (pointers, fixed arrays, `enum_flags`, structs without `@subcommand`); `@count` on a non-integer; `@env` on lists; `@required` on flags; a list positional that is not last, a required positional after an optional one; `@conflicts`/`@requires` naming a missing field; a field called `about` or `version` (reserved by `Help`); selector and subcommand mismatches.

A missing help string for an option (one not in the `Help` literal) is a compile-time **warning**; `@hidden` options are exempt. A misspelled field in the `Help` literal is the compiler's own "no member" error.

## Subcommands

```jai
Command :: enum { BUILD; RUN; REMOTE_ADD; }

Build_Args :: struct { release: bool; @short=r  files: [..] string; @positional }
Run_Args   :: struct { program: string; @positional @required  arguments: [..] string; @positional }

Tool :: struct {
    verbose:    u8; @short=v @count @global
    command:    Command; @subcommand
    build:      Build_Args; @subcommand
    run:        Run_Args; @subcommand
    remote_add: Run_Args; @subcommand
}

TOOL_HELP :: Args.Help(Tool).{
    about = "A build tool.", verbose = "More output",
    build = .{ about = "Compile the project", release = "Optimise", files = "Files" },
    run = .{ about = "Run a program", program = "Program", arguments = "Arguments" },
    remote_add = .{ about = "Add a remote", program = "Name", arguments = "Rest" },
};

tool, set := Args.parse(Tool, TOOL_HELP);
if #complete tool.command == {
    case .BUILD;      build(tool.build);
    case .RUN;        run(tool.run);
    case .REMOTE_ADD; remote_add(tool.remote_add);
}
```

The enum field marked `@subcommand` says which subcommand ran; its members name the struct fields (`REMOTE_ADD` is the field `remote_add`, typed `remote-add`). Each struct is parsed with its own options, positionals and help (`tool build --help`), and `HELP` literals nest the same way. A command with subcommands takes no positionals, and a subcommand is required unless the enum is `@optional` (then check `set.command`). `@global` options of outer commands are accepted inside; each command's `@conflicts`, `@requires` and `@required` are checked when that command's arguments end.

## Colour

Help and errors are coloured by `Color_Mode`: `AUTO` follows `NO_COLOR` (off), `FORCE_COLOR`/`CLICOLOR_FORCE` that are not empty or `0` (on), `TERM=dumb` (off), and otherwise whether the stream is a terminal (on Windows, a console known to understand ANSI codes). It is the rule `jaic`, `jailint` and `jaifmt --color auto` use. A struct field of type `Args.Color_Mode` (`color: Args.Color_Mode = .AUTO;`, giving `--color auto|always|never`) is applied the moment it is parsed. Otherwise call `Args.set_color_mode`.

## Completions

```jai
// An option of type Args.Shell gives `--completions bash|zsh|fish|powershell`:
//     completions: Args.Shell;
cli, set := Args.parse(Cli, HELP);
if set.completions {
    print("%", Args.completions(Cli, HELP, cli.completions));
    exit(0);
}
```

Scripts list every non-hidden option, subcommand (with its own options plus inherited `@global` ones), enum choices after enum options, and file completion for positionals and for values whose `@value` mentions file, path or dir. Install them where each shell looks: `/etc/bash_completion.d/` or `source` for bash, a `_name` file on `$fpath` for zsh, `~/.config/fish/completions/name.fish`, and your `$PROFILE` for PowerShell. The program name defaults to `argv[0]`'s basename; pass `program` to choose it.

## How it works

`Args.parse` is a polymorphic procedure. Its body is an `#insert` whose string comes from `generate.jai`, which runs inside the compiler: it reads the struct's fields and notes with `type_info`, validates them, and writes plain Jai: one procedure per option (`opt_jobs`), a `h_option` dispatch on the option name, `h_positional`, `h_subcommand` and `h_finish` (environment fallbacks, required, conflicts). `runtime.jai` is the shared token loop that calls them through procedure pointers and the value conversion (`string_to_int` with the field's type, so range-checking costs nothing extra). Nothing reflects over the struct while arguments are parsed. Help and completions are rendered from a `Command_Spec` tree built by a second generated procedure (`describe`), also from the compile-time analysis.

`Help(T)` and `Is_Set(T)` are structs generated the same way. A subcommand's `Help` is the subcommand type's own `Help`, which is why the literals nest.

## How to change it

- A new note: read it in `read_field` and validate it in `analyze` (`generate.jai`), emit code in `generate_command`, show it in `generate_description` and `render.jai` if help should mention it, and add a negative fixture under `tests/corpus/negative/args-*.jai` (with its `tests/corpus/manifest.json` entry) plus a case in `tests/parse.jai`.
- A new field type: `classify_scalar` for its kind, then `store_value` in `runtime.jai` (or a per-field store, as enums have).
- Help layout: `render_help`. Completions: `completions.jai`. After an intended change to either, regenerate the golden texts: `jaic stdlib/Extensions/Args/tests/data/bless.jai -interp > stdlib/Extensions/Args/tests/data/golden.jai`.
- Gotchas: generated code cannot use `%` inside a `print` format; names in the generator that are not `quote`d end up as Jai identifiers; `Help` reserves `about` and `version`; compile errors point at the `Args` source, so they name the field in the message instead (`Cli.jobs`).

## Configuration

No flags or environment variables of its own besides those the program declares. The colour rules read `NO_COLOR`, `FORCE_COLOR`, `CLICOLOR_FORCE` and `TERM`.

## Dependencies

`Basic`, `String`, `Compiler` (for `compiler_report` at compile time) and, on Windows, `Windows`. On other systems libc's `getenv` and `isatty` are called directly (WASI has no `isatty`, so output is never coloured by `auto` there). Tests: `stdlib/Extensions/Args/tests/` (run in all four [stdlib runtime](../tools/stdlib-runtime-tests.md) modes) and the `args-*` cases of `tests/corpus/negative`.
