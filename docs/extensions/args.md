# Args (`Extensions/Args`)

## What it is

A typed command-line parser generated at compile time, in the spirit of clap derive. Declare a struct whose fields are the options; notes (`@short=c`, `@env=NAME`, `@range(1,64)`) refine them. `Args.parse(Cli, HELP)` returns the filled struct, handles `--help`/`--version`, prints bad input with a usage hint and exits 2, and can print shell completions. [`jaifmt`](../tools/jaifmt.md) uses it.

```jai
Args :: #import "Extensions/Args";

Cli :: struct {
    paths:   [..] string; @positional
    check:   bool;        @short=c
    jobs:    int = 4;     @short=j @env=TOOL_JOBS @range(1,64)
    config:  string;      @value=FILE
    verbose: u8;          @short=v @count
}

HELP :: Args.Help(Cli).{
    about = "Does things to files.", version = "1.0.0",
    paths = "Files to process", check = "Only report", jobs = "Parallel jobs",
    config = "Config file", verbose = "Say more (repeat for more)",
};

main :: () {
    cli, set := Args.parse(Cli, HELP);          // `set.jobs` is true when given by flag or env
}
```

`[default: ...]` shows in help only for non-zero initial values (type info cannot tell whether an initializer was written).

## How it works

`Args.parse` is polymorphic; its body is an `#insert` of text from `generate.jai`, which runs in the compiler: it reads fields and notes through `type_info`, validates them, and writes plain Jai (one procedure per option, `h_option`, `h_positional`, `h_subcommand`, `h_finish` for env fallbacks, required, conflicts). `runtime.jai` is the shared token loop calling them through procedure pointers, with value conversion (`string_to_int` with the field's type, so range checks are free). Nothing reflects while parsing. Help and completions render from a `Command_Spec` tree built by a second generated procedure (`describe`). `Help(T)` and `Is_Set(T)` are generated structs; a subcommand's `Help` is that type's own, so literals nest.

- `try_parse(T, HELP, argv, environment) -> T, Error, Is_Set(T)` never prints or exits. `Error.kind`: `UNKNOWN_OPTION`, `UNKNOWN_COMMAND`, `MISSING_VALUE`, `INVALID_VALUE`, `OUT_OF_RANGE`, `UNEXPECTED_VALUE`, `UNEXPECTED_ARGUMENT`, `MISSING_REQUIRED`, `MISSING_COMMAND`, `CONFLICT`, `REQUIRES`, plus `HELP`/`VERSION` (requests, page in `error.text`). It carries a `suggestion` (nearest name by edit distance). The first problem wins.
- `help_text`, `completions(T, HELP, shell)` (`.BASH .ZSH .FISH .POWERSHELL`), `report(error)`. An `Args.Shell` field gives `--completions`.
- Syntax: `--long v`, `--long=v`, `-s v`, `-sv`, bundled flags (`-abc`), `--` ends options, bool flags take `--no-name`, option names turn `_` into `-`. Enums match case-insensitively against `kebab-case` member names.
- Field types: bool, integers (range-checked), floats, string, enum, `[..] T` (repeatable, or the trailing `@positional` list), `@count` integers, `@subcommand` structs.

| Note | Meaning |
| --- | --- |
| `@positional` | by position; only the last may be a list; no required after optional |
| `@short=x`, `@long=name`, `@value=NAME` | short letter (`h`, `V` reserved), long name, help placeholder |
| `@env=NAME` | fallback when not given (empty counts as unset); not on lists |
| `@range(a,b)`, `@count`, `@required` | bounds, occurrence counter, must be given |
| `@conflicts=f`, `@requires=f` | cross-field checks |
| `@hidden`, `@global` | omit from help; also accepted after subcommands |
| `@subcommand`, `@optional` | struct fields / selector enum; `@optional` allows no subcommand |

Subcommands: an enum field marked `@subcommand` selects one; its members name the struct fields (`REMOTE_ADD` is the field `remote_add`, typed `remote-add`). Each struct has its own options and help (`tool build --help`). A command with subcommands takes no positionals.

Notes are whitespace-delimited tokens (never quoted) so every Jai parser accepts them. All problems are compile errors naming the field (`Args: Cli.jobs: ...`): bad notes, duplicate/reserved names, unsupported types, `@range` on non-numbers, misplaced positionals, `@conflicts` naming a missing field, fields called `about`/`version`. A missing help string is a warning (`@hidden` exempt).

Colour: `Color_Mode` `AUTO` follows `NO_COLOR` (off), `FORCE_COLOR`/`CLICOLOR_FORCE` (on), `TERM=dumb` (off), else whether the stream is a terminal. A field of type `Args.Color_Mode` is applied when parsed; otherwise call `Args.set_color_mode`.

## How to change it

- **New note:** read it in `read_field`, validate in `analyze` (`generate.jai`), emit in `generate_command`, show in `generate_description` and `render.jai`; add a negative fixture `tests/corpus/negative/args-*.jai` (with its `tests/corpus/manifest.json` entry) and a case in `tests/parse.jai`.
- **New field type:** `classify_scalar`, then `store_value` in `runtime.jai`.
- **Help layout:** `render_help`; completions: `completions.jai`. After an intended change regenerate goldens: `jaic stdlib/Extensions/Args/tests/fixtures/bless.jai -interp > stdlib/Extensions/Args/tests/fixtures/golden.jai`.
- Gotchas: generated code cannot use `%` inside a `print` format; unquoted generator names become Jai identifiers; compile errors point at the Args source, so messages name the field.

## Configuration

None of its own. Colour reads `NO_COLOR`, `FORCE_COLOR`, `CLICOLOR_FORCE`, `TERM`.

## Dependencies

`Basic`, `String`, `Compiler` (`compiler_report`), `Windows` on Windows; libc `getenv`/`isatty` elsewhere (WASI has no `isatty`: never coloured). Tests: `stdlib/Extensions/Args/tests/` and the `args-*` negative corpus cases.
