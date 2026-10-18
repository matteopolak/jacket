# jacket

## What it is

`jacket` is one front door to the toolchain: it runs `jaic`, `jailint`, `jaifmt` and `jailsp`, and it owns `jacket.toml`, the single project config file those tools share. Source: `crates/jacket` (the dispatcher) and `crates/jacket-config` (schema, lookup, migration, entry inference).

## The rule: a drop-in for the tools

`jacket build <args>` is `jaic <args>`. The arguments are forwarded byte for byte, single-dash Jai flags included; clap never parses, translates or reorders them. `jacket build -help` prints the compiler's own help.

```
jacket build main.jai -o -exe app -import_dir vendor      # jaic main.jai -o -exe app -import_dir vendor
jacket check                                              # jaic -check <entry>
jacket run -- --port 8080                                 # jaic <entry>, then ./<entry> --port 8080
jacket run main.jai -interp -- one two                    # jaic main.jai -interp -- one two
jacket lint --fix src                                     # jailint --fix src
jacket fmt --check                                        # jaifmt --check <project>
```

What `jacket` adds is only the project:

- With no `.jai` file named (before any `-` or `--`), the project's entry file goes first: `[project] entry`, else the first of `build.jai`, `first.jai`, `main.jai`, `src/main.jai`, `module.jai` that exists.
- `[project] import_path` becomes `-import_dir`.
- `[experimental]` settings become the matching jaic flags, each only when you did not pass that flag yourself.
- `jacket fmt` with no path formats the project (the directory with the jacket.toml, else the current one).
- `-help`/`-version` are never given anything extra.

`run` is `jaic <args>` followed by starting the executable (found by jaic's output rules, [CLI](cli.md#where-the-executable-goes)); arguments after `--` go to the program. With `-interp` jaic runs the program itself and the whole command line is jaic's.

## Commands

```
jacket build   Compile a program: `jaic <args>`, flags untouched
jacket check   Type-check without writing anything: `jaic -check <args>`
jacket run     Build, then run the program; arguments after `--` go to it
jacket lint    Lint with `jailint` (`--fix`, `-D warnings`, paths)
jacket fmt     Format with `jaifmt` (`--check`, paths; the project by default)
jacket lsp     Run the language server over stdio (`jailsp`)
jacket new     Create a project in a new directory        (--lib for a library)
jacket init    Create a project in the current directory  (--lib for a library)
jacket config  migrate [--dry-run] | show
jacket clean   Remove the executable `jacket build` writes
jacket version Print the version of jacket and of every tool it fronts
jacket jaic|jailint|jailsp|jaifmt <args>   The tool with exactly these arguments
jacket <other> ...                         Runs `jacket-<other>` from PATH, as cargo does
```

`new`, `init`, `config`, `clean` and `version` are `jacket`'s own commands and use `--long` flags. Tools are found next to the `jacket` executable (symlinks are followed, so a `~/.local/bin/jacket` link finds the installed release), else on `PATH`.

## jacket.toml

The nearest directory above the working directory (or the open file, for `jailsp`) that holds a `jacket.toml` or any legacy file is the project. Every section and key is optional.

```toml
[project]
entry = ["src/main.jai"]        # string or array; relative to this file
import_path = ["vendor"]       # more module folders (-import_dir)

[toolchain]
version = "0.19.1"              # jacket warns when its release differs

[lint]
exclude = ["tests/negative/**"]
max_nesting = 5                 # excessive_nesting threshold

[lint.rules]
unused_variable = "deny"        # allow | warn | deny

[format]                        # every jaifmt key, see jaifmt.md
indent_width = 4
ignore = ["gen/**"]

[experimental]                  # may change between releases; each is a jaic flag
parse_threads = 4               # -parse_threads
serial_parse = false            # -serial_parse
codegen_units = 4               # -codegen_units
```

Precedence: command-line flags, then `jacket.toml`, then legacy files, then defaults. Unknown sections and keys are kept and reported by `jacket config show`, not fatal, except where a tool has always been strict: `jailint` rejects unknown lint settings, rules and levels, and `jaifmt` unknown format keys. A file that is not valid TOML, or has a wrongly typed value, is an error naming the file and line.

`jacket config show` prints the effective settings annotated with the file each value came from.

### Legacy files

`jai.toml`, `jailint.toml` and `jaifmt.toml` are still read when the directory has no `jacket.toml`:

| legacy | becomes |
| --- | --- |
| `jai.toml` `build_files`, `import_path` | `[project] entry`, `import_path` |
| `jailint.toml` top-level keys, `[rules]` | `[lint]`, `[lint.rules]` |
| `jaifmt.toml` keys | `[format]` |

A directory with both uses `jacket.toml` and ignores the legacy file, with a one-line note on stderr (`jailsp` sends a log message instead). The legacy files are looked up per directory: a `jacket.toml` in a deeper directory wins over legacy files above it.

## Migration

`jacket config migrate [--dry-run]` writes `jacket.toml` next to the legacy files, merging the ones in that directory. Comments and layout move with their keys (`toml_edit`), unknown keys are kept and reported, the header says where the file came from, and **the old files are never deleted**. It is idempotent: with a `jacket.toml` present it does nothing. `--dry-run` prints the file and where it would go.

| situation | what happens |
| --- | --- |
| `jacket build/check/run/lint/fmt/lsp/clean` in a tree with legacy files and no `jacket.toml` | The first such command writes `jacket.toml` next to them and prints one line saying so. |
| same, with `CI` set | Nothing is written; a warning names `jacket config migrate`. |
| `jaic`, `jailint`, `jaifmt` or `jailsp` run directly | They read the legacy files and never write; they warn once with a hint to run `jacket config migrate`. |
| `jacket config show` | Read-only; notes the legacy files. |
| `jacket.toml` and a legacy file in one directory | `jacket.toml` wins; the legacy file is ignored with a note (not repeated for a file `migrate` wrote itself). |
| a legacy file that does not parse | An error naming that file; nothing is written. |

The notices go to stderr, since stdout may be an LSP stream. The VS Code extension offers a one-time notice with a button that runs the migration; the Neovim plugin reads `jacket.toml` and needs none.

## How to change it

- A new `jaic`-flag-backed setting: add the field to `Experimental` in `crates/jacket-config/src/schema.rs`, forward it in `defaults` in `crates/jacket/src/compile.rs`, and add a case to the `compile.rs` tests; extend `editors/vscode/scripts/build-schemas.ts` and run it.
- A new section read by a tool: add the struct in `schema.rs` (with `extra` flattened so unknown keys survive), then read it from `Loaded`; give a legacy file a mapping in `migrate.rs` only if one existed.
- Entry inference is `jacket_config::first_entry` and `INFERRED_ENTRIES`; `jaic-cli` and `jailsp` both use it, so keep one order.
- The compiler library (`crates/jaic`) never reads `jacket.toml` and stays dependency-free; `jaic` takes its settings as flags from `jacket`. Only the `jaic-cli` binary uses `jacket-config`, to suggest the project's entry when it is given a directory.
- `jaifmt` is written in Jai and reads `[format]` itself (`Jai_Format`'s `parse_jacket_config`); a change to the format keys is made in `stdlib/Extensions/Jai_Format/config.jai`.

## Configuration

Environment variables `jacket` reads: `CI` (suppresses migration writes). Everything else is jaic's, such as `JAIC_STDLIB` ([native linking](../native/native-linking.md)).

## Dependencies

`serde` and `toml` (reading), `toml_edit` (migration), `clap` (jacket's own commands). The tools it fronts are separate programs; nothing links against them.
