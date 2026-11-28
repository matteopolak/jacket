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
jacket migrate                  # rewrite old Jai code (prints the diff; --apply writes)
jacket upgrade                  # install the latest release
```

What `jacket` adds is the project:

- With no `.jai` file named, the entry file goes first: `[project] entry`, else the first of `build.jai`, `first.jai`, `main.jai`, `src/main.jai`, `module.jai`.
- `[project] import_path` becomes `-import_dir`, before your own flags.
- `[experimental]` settings become the matching jaic flags unless you passed them.
- `jacket fmt` with no path formats the whole project.

`run` is `jaic <args>` followed by starting the executable ([where it goes](cli.md#where-the-executable-goes)); arguments after `--` go to the program. With `-interp`, `jaic` runs the program itself.

## Commands

`build`, `check`, `run`, `lint`, `fmt` and `lsp` front the tool of the same name. `test`, `bench`, `doc` and `migrate` are [jacket test](jacket-test.md), [jacket bench](jacket-bench.md), [jacket doc](jacket-doc.md) and [jacket migrate](jacket-migrate.md). `dep` manages [git dependencies](dependencies.md). `new`, `init`, `config`, `toolchain`, `clean`, `explain`, `version` and `upgrade` are `jacket`'s own and use `--long` flags. `jacket jaic|jailint|jailsp|jaifmt <args>` runs that tool verbatim, and any other command runs `jacket-<command>` from `PATH`. Every flag is in the [tools reference](../reference/tools.md).

### Targets

`jacket build|check|run --target NAME` (also `-target NAME`) builds for a named target. `NAME` is a `[target.NAME]` table of `jacket.toml` or a built-in board; an LLVM triple still goes to jaic unchanged. `jacket toolchain` lists every name with its triple, board, and the linker, libraries and runner this machine has for it; `jacket toolchain check [--tool NAME]... [TARGET]...` exits non-zero and says how to get what is missing. An unknown name fails and suggests the nearest one (`unknown target `pic`; did you mean `pico`?`).

```toml
[target.pico]
triple = "thumbv6m-none-eabi"
runner = "flash {exe}"          # `{exe}` is the built program; `jacket run` starts this instead

[target.dev]
board = "Qemu_Virt_Aarch64"     # builds with Extensions/Embedded/Boards/<board> as a plugin
```

A board adds `+Extensions/Embedded/Boards/<board>` to jaic's command line, so the board's `configure` sets the OS, CPU, triple and linker script. Built-in boards carry a runner (QEMU for `Qemu_Virt_Aarch64`); a table of the same name replaces it. `jacket run --target NAME` builds, then starts the runner, and refuses program arguments. `jacket test --target NAME` builds the tests for the target and runs each through the runner; on a board it adds the plugin option `tests`, which exports the reset function that runs the tests ([tests on a board](jacket-test.md#tests-on-a-board)). In a runner, `{exe}` is the program (put after the runner's words when absent) and `{args}` the test arguments as one word, for an emulator that takes them through an option (`-append {args}`); without `{args}` they follow the program. Each board module states its triple and runner as the string constants `triple` and `runner`; `jacket` embeds the modules listed in `BOARD_SOURCES` (`crates/jacket/src/target.rs`) and reads them, and a test keeps the list in step with `stdlib/Extensions/Embedded/Boards`.

Tools are found next to the `jacket` executable (symlinks followed), else on `PATH`.

`jacket explain` reads the catalog in `crates/jaic-syntax/src/diagnostics/catalog.rs`; see [diagnostics](../compiler/diagnostics.md). `build`, `check` and `run` show jaic's [build status](cli.md#output), and `jacket run` adds a `Running` line (`crates/jacket/src/status.rs`; keep it in step with jaic's rules). `jacket run` and `jacket test` start the executable jaic reports writing through `JAIC_REPORT_EXE` (a metaprogram can rename it); a build that reports none is the error `jaic did not report the executable it wrote`.

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
version = "0.19.1"              # another jacket release refuses to build this project

[lint]
exclude = ["tests/negative/**"]
max_nesting = 5

[lint.rules]
unused_variable = "deny"        # allow | warn | deny

[[lint.overrides]]              # levels for some paths; a later entry wins
paths = ["tests/**"]
rules = { unused_variable = "warn", leniencies = "allow" }   # compiler warnings too: a code, `leniencies` or `compiler`

[format]                        # every jaifmt key, see jaifmt.md
indent_width = 4
ignore = ["gen/**"]

[test]                          # see jacket-test.md
paths = ["libs/parser"]         # folders with a tests/ directory
timeout = 120                   # seconds per test

[test.profile.ci]               # jacket test --profile ci
retries = 2

[target.pico]                   # jacket build --target pico; see Targets above
triple = "thumbv6m-none-eabi"
runner = "flash {exe}"

[doc]                           # jacket doc's site, see jacket-doc.md
path = "src/my_lib"
repo = "https://github.com/me/my_lib"

[dependencies]                  # jacket dep add/update/sync; see dependencies.md
library = { git = "someone/library", version = "^1.2" }

[resolver]                      # how jacket dep picks versions
min_release_age = "3d"
seed = "k3v9q2xh"

[package]                       # what this repository says when others depend on it
name = "my_lib"
requires_jacket = ">=0.40"

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
- `crates/jaic` never reads `jacket.toml`; `jaic` takes settings as flags from `jacket`, including `-package_import_dir` for each dependency's own `import_path`.
- **Dependencies:** `crates/jacket/src/dep/` holds the commands (see [dependencies](dependencies.md)); `jacket_config::packages` reads what dependencies add to the module search and is shared by `jacket`, `jailint` and the compiler tests.

## Dependencies

`serde`, `toml`, `toml_edit`, `clap`, `serde_json`. `upgrade` needs `curl` and `tar` on `PATH`. The tools it fronts are separate programs. Environment variables are in the [reference](../reference/tools.md#environment-variables).
