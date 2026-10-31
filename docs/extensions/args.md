# Args

A typed command-line parser generated at compile time, in the spirit of clap derive. Declare a struct whose fields are the options; notes refine them. `Args.parse` returns the filled struct, handles `--help`/`--version`, reports bad input with a usage hint (exit 2) and can print shell completions. [`jaifmt`](../tools/jaifmt.md) uses it.

```jai
Args :: #import "Extensions/Args";

Cli :: struct {
    paths:   [..] string; @positional
    check:   bool;        @short=c
    jobs:    int = 4;     @short=j @env=TOOL_JOBS @range(1,64)
    verbose: u8;          @short=v @count
}

HELP :: Args.Help(Cli).{
    about = "Does things to files.", version = "1.0.0",
    paths = "Files to process", check = "Only report", jobs = "Parallel jobs",
    verbose = "Say more (repeat for more)",
};

main :: () {
    cli, set := Args.parse(Cli, HELP);   // set.jobs: given by flag or env
}
```

## How it works

- `Args.parse` is an `#insert` of text produced by `generate.jai` at compile time. It reads fields and notes through `type_info`, validates them, and writes plain Jai: one procedure per option. Nothing reflects while parsing.
- `runtime.jai` is the shared token loop calling those procedures, with value conversion (range checks come free from the field type).
- Help and completions render from a `Command_Spec` tree built by a second generated procedure.
- `try_parse` never prints or exits; it returns an `Error` with a `kind` and a nearest-name `suggestion`.
- Subcommands: an enum field marked `@subcommand` selects a struct field (`REMOTE_ADD` picks `remote_add`, typed `remote-add`). A command with subcommands takes no positionals.
- Bad notes, duplicate names and unsupported types are compile errors naming the field. A missing help string is a warning.
- Colour follows `NO_COLOR`, `FORCE_COLOR`, `CLICOLOR_FORCE`, `TERM=dumb`, else whether the stream is a terminal.

## How to change it

- New note: read it in `read_field`, validate in `analyze`, emit in `generate_command`, show in `generate_description` and `render.jai`. Add a negative fixture `tests/corpus/negative/args-*.jai` (with a manifest entry) and a case in `tests/parse.jai`.
- New field type: `classify_scalar`, then `store_value` in `runtime.jai`.
- Regenerate goldens after an intended help change: `jaic stdlib/Extensions/Args/tests/fixtures/bless.jai -interp > stdlib/Extensions/Args/tests/fixtures/golden.jai`.
- Gotchas: generated code can't use `%` inside a `print` format; unquoted generator names become identifiers; notes are whitespace-delimited and never quoted.
