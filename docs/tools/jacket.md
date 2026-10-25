# jacket

## What it is

`jacket` is one front door to the toolchain: it runs `jaic`, `jailint`, `jaifmt` and `jailsp`, and it owns `jacket.toml`, the single project config file those tools share. Source: `crates/jacket` (the dispatcher) and `crates/jacket-config` (schema, lookup, migration, entry inference).

## The rule: a drop-in for the tools

`jacket build <args>` is `jaic <args>`. The arguments are forwarded byte for byte, single-dash Jai flags included; clap never parses, translates or reorders them. `jacket build -help` prints the compiler's own help.

```
jacket build main.jai -o -exe app -import_dir vendor      # jaic main.jai -o -exe app -import_dir vendor
jacket check                                              # jaic -check <entry>
jacket run -- --port 8080                                 # jaic <entry>, then the executable it wrote, with --port 8080
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
jacket test    Build the tests once and run each in its own process (jacket-test.md)
jacket bench   Reserved for benchmarks; `--list` lists them
jacket lint    Lint with `jailint` (`--fix`, `-D warnings`, paths)
jacket fmt     Format with `jaifmt` (`--check`, paths; the project by default)
jacket lsp     Run the language server over stdio (`jailsp`)
jacket new     Create a project in a new directory        (--lib for a library)
jacket init    Create a project in the current directory  (--lib for a library)
jacket config  show
jacket clean   Remove the executable `jacket build` writes
jacket version Print the version of jacket and of every tool it fronts
jacket upgrade Install the latest release over this installation [--version TAG] [--check] [--yes]
jacket jaic|jailint|jailsp|jaifmt <args>   The tool with exactly these arguments
jacket <other> ...                         Runs `jacket-<other>` from PATH, as cargo does
```

`new`, `init`, `config`, `clean`, `version` and `upgrade` are `jacket`'s own commands and use `--long` flags. Tools are found next to the `jacket` executable (symlinks are followed, so a `~/.local/bin/jacket` link finds the installed release), else on `PATH`.

## Build status

`jacket build`, `check` and `run` show jaic's [build status](cli.md#build-status): on a terminal, a `Compiling` line, a status line redrawn in place while the compiler works, and a `Finished` line. `jacket` names the project after its directory (it passes `JACKET_PROJECT` to jaic), and `jacket run` adds a line of its own before starting the program:

```
   Compiling demo (src/main.jai)
    Finished debug [unoptimized] in 0.42s
     Running `demo`
```

Pipes and logs get none of it. `-quiet`/`--quiet`, `NO_COLOR`, `-no_color` and `JAIC_PROGRESS` work as they do for jaic; `crates/jacket/src/status.rs` repeats jaic's rules for the `Running` line, so change both together.

## Upgrading

`jacket upgrade` installs the latest GitHub release over the installation that holds the running `jacket`: the whole folder (`jaic`, `jailsp`, `jailint`, `jaifmt`, `jacket`, `stdlib/`, `native-libs/`), not only the binary. Source: `crates/jacket/src/upgrade/`.

```
jacket upgrade                  # latest release; asks first when stdin is a terminal
jacket upgrade --check          # only report whether an update exists (exit 0 either way)
jacket upgrade --version 0.20.0 # a specific release, newer or older
jacket upgrade --yes            # no confirmation prompt
```

How it works:

1. The install is the folder beside the canonical `jacket` (symlinks followed). It must hold `jaic` and `stdlib/`, so a `cargo build` binary is refused.
2. A Homebrew Cellar path, a `/nix/store` path or the VS Code extension's download folder is refused with the command that does own the update (`brew upgrade jacket`, `nix profile upgrade jacket`, the extension). `--check` still reports, and prints the same advice.
3. The releases API (`/releases/latest` or `/releases/tags/v<version>`) names the release. The same or an older latest release prints `already up to date`; an explicit `--version` may go back.
4. This platform's archive (`jacket-<platform>.tar.gz`, `.zip` on Windows; the names in [releases](releases.md#install-channels) and `tests/installers/vectors.json`) and `SHA256SUMS` are downloaded into `<install>.jacket-new/` next to the install. The archive's SHA-256 must match its `SHA256SUMS` line, and the unpacked tree must hold `jaic`, `jacket` and `stdlib/`. Any failure removes the staging folder and leaves the install as it was, with the error on stderr and exit code 2.
5. The swap. Unix renames the install folder to `<install>.jacket-old` and the new tree into its place, undoing the first rename if the second fails; the running process keeps its open file. Windows cannot rename a folder holding a running `.exe`, so it moves each top-level entry into `<install>\.jacket-old` and the new entries in, rolling every move back on a failure. Whatever could not be deleted (the old, still-running `jacket.exe`) is removed by the next `jacket upgrade`.

The install folder is named for the version `install.sh` put there (`~/.local/share/jai/0.21.0`) and keeps that name after an upgrade, so the `~/.local/bin` symlinks and the Windows `current` junction stay valid. The parent folder must be writable by you.

Downloads are made by `curl` and unpacked by `tar` (both ship with macOS, nearly every Linux and Windows 10 and later, where `tar.exe` also reads zip), the way `install.sh` does it, so no HTTP or TLS stack is linked into `jacket` and `HTTPS_PROXY`, `NO_PROXY` and the system certificates work as for any `curl`. `GITHUB_TOKEN`, if set, is sent to the API only (through `curl`'s stdin config, never the command line), to lift the anonymous rate limit; a 403 or 429 suggests it.

To change it: the platform-to-archive mapping is `asset_name` in `upgrade/mod.rs` (a test reads the installer vectors, so add a platform to the vectors first); package-manager detection is `managed.rs`; the swap is `swap.rs`; the network and `tar` are behind the `Source` trait in `net.rs`. Tests (`upgrade/tests.rs`, plus per-file unit tests) run against a fake release folder and never touch the network. Windows (`Mode::Entries`) is tested on every platform at the swap level, but the real `curl.exe`/`tar.exe` path has only run on macOS.

## jacket.toml

The nearest directory above the working directory (or the open file, for `jailsp`) that holds a `jacket.toml` is the project. Every section and key is optional.

In VS Code, install Even Better TOML (`tamasfe.even-better-toml`) for key completion, hover docs and validation: the Jai extension registers `editors/vscode/schemas/jacket.schema.json` through `tomlValidation`, which that extension reads. Other editors can point Taplo at the same schema.

In the editor, config changes apply on save. Incomplete edits do not trigger parse warnings; see [saved editor configuration](../compiler/saved-configuration.md).

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

[[lint.overrides]]              # levels for some paths; a later entry wins
paths = ["tests/**"]
rules = { unused_variable = "warn" }

[format]                        # every jaifmt key, see jaifmt.md
indent_width = 4
ignore = ["gen/**"]

[test]                          # jacket test, see jacket-test.md
paths = ["libs/parser"]         # folders with a tests/ directory
timeout = 120                   # seconds per test

[test.profile.ci]               # jacket test --profile ci
retries = 2

[postgres]                      # read by Extensions/Postgres, see docs/postgres/managed.md
managed = true                  # describe typed queries in a managed server when no URL is set
managed_version = "16.15.1"     # a pinned release from MANAGED_ARCHIVES

[experimental]                  # may change between releases; each is a jaic flag
parse_threads = 4               # -parse_threads
serial_parse = false            # -serial_parse
codegen_units = 4               # -codegen_units
```

Precedence: command-line flags, then `jacket.toml`, then defaults. Unknown sections and keys are never fatal and never read: parsing drops them (`serde_ignored` collects their paths) and every tool prints the same warning, `unknown setting `lint.max_nestng` in jacket.toml (ignored); did you mean `lint.max_nesting`?` (the suggestion appears when a known key is close). A top-level key that belongs in a section gets that instead: `unknown setting `import_path` in jacket.toml (ignored); it belongs under `[project]``, and likely spellings point at the real key (`build_files` → ``use `entry` under `[project]` ``); the aliases live in `schema::ALIASES`. `jailint` still rejects unknown rule names and levels inside `[lint.rules]`, whose keys are rule names, not schema keys. A file that is not valid TOML, or has a wrongly typed value, is an error naming the file and line.

`jacket config show` prints the effective settings and the file they came from.

## How to change it

- A new `jaic`-flag-backed setting: add the field to `Experimental` in `crates/jacket-config/src/schema.rs`, forward it in `defaults` in `crates/jacket/src/compile.rs`, and add a case to the `compile.rs` tests; extend `editors/vscode/scripts/build.ts` and run it.
- A new section read by a tool: add a typed struct in `schema.rs` (no catch-all map: unknown keys are collected by `parse_noting`, and the `known_keys` list there feeds the did-you-mean, which a test keeps in step with the struct), then read it from `Loaded`.
- Entry inference is `jacket_config::first_entry` and `INFERRED_ENTRIES`; `jaic-cli` and `jailsp` both use it, so keep one order.
- The compiler library (`crates/jaic`) never reads `jacket.toml` and stays dependency-free; `jaic` takes its settings as flags from `jacket`. Only the `jaic-cli` binary uses `jacket-config`, to suggest the project's entry when it is given a directory.
- `jaifmt` is written in Jai and reads `[format]` itself (`Jai_Format`'s `parse_config`); a change to the format keys is made in `stdlib/Extensions/Jai_Format/config.jai`.

## Configuration

Environment variables `jacket` reads: `CI` (makes the build status plain); `JAIC_PROGRESS`, `NO_COLOR` and `TERM` (the `Running` line); for `upgrade`, `GITHUB_TOKEN` and the proxy variables `curl` reads. Everything else is jaic's, such as `JAIC_STDLIB` ([native linking](../native/native-linking.md)).

## Dependencies

`serde` and `toml` (reading), `toml_edit` (migration), `clap` (jacket's own commands), `serde_json` (`upgrade` reads the releases API). `upgrade` also needs `curl` and `tar` on `PATH`. The tools it fronts are separate programs; nothing links against them.
