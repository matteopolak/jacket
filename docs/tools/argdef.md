# argdef

`argdef` (`crates/argdef`, with the derive in `crates/argdef-derive`) declares a tool's command-line options as one struct. `jaic` is the user: its options are the fields of `jaic_cli_args::cli::Cli`. Parsing, `-help`, "did you mean" suggestions, shell completions and the inspection `jacket` does of a compiler command line all read that struct.

## How it works

`#[derive(Args)]` turns each field with an `#[arg(...)]` into an `Opt` (spellings, kind, value name, help, section, tag) and writes `apply`, which stores a value into the field. `Args::spec()` collects the options; `Args::parse(&args)` walks the arguments against that spec.

```rust
use argdef::Args;

/// demo: a tool
#[derive(Args, Default)]
#[args(name = "demo", usage = "demo [options] file", sections = ["Options"])]
struct Demo {
    /// print more
    #[arg(name = "-verbose", section = "Options")]
    verbose: bool,
    /// the output file
    #[arg(name = "-out", value = "file", needs = "a file name", hint = "file")]
    out: Option<String>,
    #[arg(positional)]
    files: Vec<String>,
}
```

- Options are spelt as written: `-check`, `--color`. A value goes after a space or an `=`; `attached` also accepts `-Dwarnings`.
- The field type decides what an option takes. `bool` is a switch; `Option<T>` stores a value; `Vec<T>` repeats; any other `T` is assigned. `T` is read with `FromArg` (`String`, `PathBuf`, integers, `Shell`), or with `parse = path`. `with = path` stores a value in any way a function can (`-A`, `-W` and `-D` all push to one list). `set = expr` makes a switch that stores a constant, and `conflict = "..."` rejects two different constants. `skip` marks a field that is not an option. `os` keeps a value as the OS gave it (`OsString`, `PathBuf`, read with `FromOsArg`); `Args::parse` takes any `AsRef<OsStr>` arguments, and a non-UTF-8 argument anywhere but an `os` option's value is an error.
- The derive refuses, with the error on the attribute: an option with no `name`, `conflict` without `set` on an `Option` field, one spelling on two options, generic structs, a `positional` that is not `Vec<String>`, a `rest` that is not `Option<Vec<String>>`, and an unknown key in `#[args(...)]` (the message names it). `crates/argdef-derive` tests each.
- `group = "+"` collects `+Name` and the arguments after it up to the next marker; `rest = "-"` with `until = "--"` collects everything after a marker. A marker seen with nothing after it still yields `Some(vec![])`.
- `exit` stops parsing at that option (`-help`, `-version`). Errors carry a kind (`Unknown`, `Missing`, `Invalid`), so the tool words its own message.
- `Spec::scan` reads only the options of an argument list, without storing them, up to the first marker. Tools that inspect a command line (`jacket`) use it and pick options by `tag`.

### Help

The struct's doc comment is the title (first line) and a paragraph. A field's doc comment is its help; further lines are indented under the first. Fields are listed under `section = "..."`, which a later field without one inherits; `sections = [...]` in `#[args]` fixes the order. `Spec::help()` returns the pieces, so the tool can add sections and paragraphs (`jaic` adds its environment variables and exit statuses) before `render(color)`.

Headings and flags are bold in colour. `jaic` decides colour with `jaic::render::detect_with` on standard output, so `NO_COLOR`, `FORCE_COLOR`, `CLICOLOR_FORCE`, `--color` and `-no_color` apply as they do to diagnostics.

### Completions

`Spec::completions(Shell::Bash | Zsh | Fish)` writes a script for the options that are not `hidden`. A value option with `hint = "file"` or `"dir"` completes paths. `jaic -completions <shell>` prints jaic's; `jacket completions <shell>` prints jacket's own (made by `clap_complete`), and `jacket completions <shell> --tool jaic` prints jaic's.

## How to change it

- A new option is a new field. See [the jaic command line](cli.md#how-to-change-it).
- A new kind of value is a `FromArg` impl (`FromOsArg` for an `os` option) or a `parse` function.
- A new attribute key is a field of `Arg` in `crates/argdef-derive/src/lib.rs` and, if the parser needs it, of `Opt` in `crates/argdef/src/spec.rs`.
- Tests are `crates/argdef/tests/args.rs`: a struct shaped like jaic's, with the parsing, help and completions output they must produce.

`jacket` itself is parsed by `clap`, not `argdef`: its commands are subcommands, which `argdef` does not model.

## Dependencies

`argdef-derive` uses `syn`, `quote` and `proc-macro2`, which `serde_derive` already brings in. `argdef` has no other dependency. `jacket completions` adds `clap_complete`.
