# jacket

`jacket` is the front door to the toolchain. It runs `jaic`, `jailint`, `jaifmt` and `jailsp`, and owns `jacket.toml`, the one project config file they share. Source: `crates/jacket` (dispatcher) and `crates/jacket-config` (schema, lookup, entry inference).

## A drop-in for the tools

`jacket build <args>` is `jaic <args>`. Arguments are forwarded byte for byte, single-dash Jai flags included.

```sh
jacket new app                  # create a project
jacket build main.jai -o        # jaic main.jai -o
jacket check                    # jaic -check <entry>
jacket run -- --port 8080       # build <entry>, then run it with --port 8080
jacket test                     # jacket test (see jacket-test.md)
jacket lint --fix src           # jailint --fix src
jacket fmt --check              # jaifmt --check on the project's files
jacket doc                      # API reference site (see jacket-doc.md)
jacket explain type-mismatch    # what a diagnostic code means
jacket upgrade                  # install the latest release
```

What `jacket` adds is the project:

- With no `.jai` file named, the entry file goes first: `[project] entry`, else the first of `build.jai`, `first.jai`, `main.jai`, `src/main.jai`, `module.jai`.
- `[project] import_path` becomes `-import_dir`, before your own flags.
- `[experimental]` settings become the matching jaic flags unless you passed them.
- `jacket fmt` with no path formats the whole project.

`run` is `jaic <args>` followed by starting the executable ([where it goes](cli.md#where-the-executable-goes)); arguments after `--` go to the program. With `-interp`, `jaic` runs the program itself.

## Commands

`build`, `check`, `run`, `lint`, `fmt` and `lsp` front the tool of the same name. `test`, `bench` and `doc` are [jacket test](jacket-test.md), [jacket bench](jacket-bench.md) and [jacket doc](jacket-doc.md). `new`, `init`, `config`, `clean`, `explain`, `version` and `upgrade` are `jacket`'s own and use `--long` flags. `jacket jaic|jailint|jailsp|jaifmt <args>` runs that tool verbatim, and any other command runs `jacket-<command>` from `PATH`, as cargo does. Every flag is in the [command-line reference](../reference/command-line.md).

Tools are found next to the `jacket` executable (symlinks followed), else on `PATH`.

`jacket explain` reads the catalog in `crates/jaic-syntax/src/diagnostics/catalog.rs`; see [diagnostics](../compiler/diagnostics.md). `build`, `check` and `run` show jaic's [build status](cli.md#output), and `jacket run` adds a `Running` line (`crates/jacket/src/status.rs`; keep it in step with jaic's rules).

## Upgrading

`jacket upgrade` installs the latest GitHub release over the installation that holds the running `jacket` (the whole folder, `stdlib/` and `native-libs/` included).

```sh
jacket upgrade                  # latest release; asks first on a terminal
jacket upgrade --check          # only report whether an update exists
jacket upgrade --version 0.20.0 # a specific release, newer or older
jacket upgrade --yes            # no prompt
```

- The install must hold `jaic` and `stdlib/`, so a `cargo build` binary is refused. Homebrew, Nix and the VS Code extension's download folder are refused with the command that does own the update.
- The archive is downloaded with `curl` and unpacked with `tar` (so `HTTPS_PROXY` and system certificates work), checked against `SHA256SUMS`, and staged in `<install>.jacket-new/`. Any failure leaves the install untouched (exit 2).
- The swap renames folders on Unix and moves entries on Windows (a running `.exe` can't be renamed). Leftovers are removed by the next upgrade. The install folder keeps its name, so `~/.local/bin` symlinks stay valid.
- `GITHUB_TOKEN`, if set, is sent to the API only, to lift the rate limit.

To change it: `asset_name` in `upgrade/mod.rs` maps platform to archive (a test reads `tests/installers/vectors.json`, so add a platform there first); `managed.rs` detects package managers; `swap.rs` does the swap; `net.rs` is the `Source` trait the tests fake.

## jacket.toml

The nearest directory above the working directory (or the open file, for `jailsp`) holding a `jacket.toml` is the project. Every key is optional. Config changes apply on save in editors ([saved configuration](../compiler/saved-configuration.md)). In VS Code, Even Better TOML gives completion and validation through `editors/vscode/schemas/jacket.schema.json`; other editors can point Taplo at it.

```toml
[project]
entry = ["src/main.jai"]        # string or array; relative to this file
import_path = ["vendor"]       # more module folders (-import_dir)

[toolchain]
version = "0.19.1"              # jacket warns when its release differs

[lint]
exclude = ["tests/negative/**"]
max_nesting = 5

[lint.rules]
unused_variable = "deny"        # allow | warn | deny

[[lint.overrides]]              # levels for some paths; a later entry wins
paths = ["tests/**"]
rules = { unused_variable = "warn" }

[format]                        # every jaifmt key, see jaifmt.md
indent_width = 4
ignore = ["gen/**"]

[test]                          # see jacket-test.md
paths = ["libs/parser"]         # folders with a tests/ directory
timeout = 120                   # seconds per test

[test.profile.ci]               # jacket test --profile ci
retries = 2

[doc]                           # jacket doc's site, see jacket-doc.md
path = "src/my_lib"
repo = "https://github.com/me/my_lib"

[postgres]                      # read by Extensions/Postgres
managed = true

[experimental]                  # may change between releases; each is a jaic flag
parse_threads = 4
codegen_units = 4
```

Precedence: command-line flags, then `jacket.toml`, then defaults. Unknown keys are never fatal and never read; every tool warns the same way, with a did-you-mean (`unknown setting `lint.max_nestng` in jacket.toml (ignored); did you mean `lint.max_nesting`?`). `jailint` still rejects unknown rule names and levels. A file that isn't valid TOML, or has a wrongly typed value, is an error naming file and line. `jacket config show` prints the effective settings and where each came from.

## How to change it

- **A jaic-flag-backed setting:** add the field to `Experimental` in `crates/jacket-config/src/schema.rs`, forward it in `defaults` in `crates/jacket/src/compile.rs`, add a test case there, and rerun `editors/vscode/scripts/build.ts` for the schema.
- **A new section:** add a typed struct in `schema.rs` (no catch-all map; `parse_noting` collects unknown keys and `known_keys` feeds the did-you-mean, which a test keeps in step) and read it from `Loaded`. A tool that validates its own values passes a `Check` to `load_checked`.
- **Entry inference:** `jacket_config::first_entry` and `INFERRED_ENTRIES`. `jaic-cli` and `jailsp` share them, so keep one order.
- **`[format]`:** `FORMAT_SETTINGS` in `schema.rs` is the authority. `jacket fmt` (`crates/jacket/src/fmt.rs`) resolves them and passes each to `jaifmt` as `--no-config --set key=value`. Standalone `jaifmt` and the browser use the Jai TOML reader in `stdlib/Extensions/Jai_Format`, and `jai_format_sync.rs` fails when the two drift. To add a key: the table, `Format` in `schema.rs`, then `Format_Config`, `apply_key` and `FORMAT_KEYS` in `Jai_Format`.
- `crates/jaic` never reads `jacket.toml`; `jaic` takes settings as flags from `jacket`.

## Dependencies

`serde`, `toml`, `toml_edit`, `clap`, `serde_json`. `upgrade` needs `curl` and `tar` on `PATH`. The tools it fronts are separate programs. Environment variables are in the [reference](../reference/environment-variables.md).
