# jailint

A linter for [Jai](https://en.wikipedia.org/wiki/Jai_(programming_language)), part of the [`jai{c,lsp,fmt,lint}`](../../README.md) toolchain. It reports common mistakes and unidiomatic code, like clippy does for Rust, and fixes the ones it safely can.

The rules run on the program after `jaic` has type-checked it, so they know each expression's type and what every name refers to. The same findings appear in editors through [`jailsp`](../jai-language-server/README.md), as diagnostics with quick fixes.

```sh
jailint src/                  # lint every .jai file under src/
jailint main.jai --fix        # apply the fixes that are safe to apply
jailint -D warnings stdlib    # fail on any finding (CI)
jailint --list                # the rules and their default levels
```

Output follows rustc's layout:

```text
warning[index_only_loop]: `i` counts through `names` to index it
  --> list.jai:21:5
   |
21 |     for i: 0..names.count - 1 {
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: loop over the elements: `for names`, with `it` and `it_index`
   = note: `index_only_loop` is `warn` by default
```

## Install

`jailint` ships in every toolchain install, next to `jaic`, `jailsp` and `jaifmt`:

```sh
brew install matteopolak/tap/jai        # macOS and Linux
winget install matteopolak.jai          # Windows
nix profile install github:matteopolak/jai
```

There are also the `install.sh` and `install.ps1` scripts and prebuilt archives on the [releases page](https://github.com/matteopolak/jai/releases); see the [main README](../../README.md#install) and [package managers](../../docs/tools/package-managers.md). The [VS Code extension](../../docs/tools/vscode-extension.md) shows jailint's findings and fixes through `jailsp`.

From a checkout: `cargo build -p jailint --release --locked` (the binary is `target/release/jailint`).

## Features

- 32 rules (`jailint --list`): unused variables, parameters and imports; loops that only index; hand-kept counters; redundant casts; `x == true`; a shadowed `it`; a `defer` in a loop; likely bugs such as `u >= 0` on an unsigned value, `1 << n - 1`, identical branches or operands, and a `while` whose condition nothing changes; unidiomatic code such as `x = x + 1`; constants that silently wrap to a narrower type; and opt-in checks for exact float comparison and narrowing `xx`.
- `--fix` applies the machine-applicable fixes; in an editor each fixable finding is a quick fix, and there is a fix-all action.
- Diagnostics in the compilers' layout, with colour following `--color`, `NO_COLOR` and `JAIC_DIAGNOSTICS`.
- Exit status: 0 when nothing at level `deny` was found, 1 when something was, 2 on a usage or configuration error.
- Options: `--fix`, `--config <file>`, `-A`/`-W`/`-D <rule>` (allow, warn, deny; `all` names every rule, `-D warnings` denies every warning), `-I <dir>`, `-j <n>`, `--list`, `--color`, `-v`.

## Configuration

`jailint.toml`, found by walking up from the first path (or `--config`):

```toml
exclude = ["tests/corpus/negative/**", "generated/**"]   # globs relative to the file

[rules]
float_equality = "warn"
unused_parameter = "allow"
```

An unknown key, rule or level is an error. Command-line `-A`/`-W`/`-D` override the file. To silence a finding in source:

```jai
// jailint: allow(unused_variable)        covers the next line with code
// jailint: allow-file(float_equality)     covers the whole file
```

A variable or parameter whose name starts with `_` is "unused on purpose".

## Documentation

The docs are the source of truth for the rules and internals:

- [jailint (Jai linter)](../../docs/tools/jailint.md): every rule with an example, suppression, configuration, how it runs and how to add a rule
- [Language server](../../docs/compiler/language-server.md#lints-and-quick-fixes): lints and quick fixes in editors
- [Diagnostics](../../docs/compiler/diagnostics.md): the shared renderer
