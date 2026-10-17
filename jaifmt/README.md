# jaifmt

> [!NOTE]
> This project is unofficial and is not affiliated with or endorsed by Thekla, Inc. or the official Jai project.

A code formatter for [Jai](https://en.wikipedia.org/wiki/Jai_(programming_language)), written in Jai. It is part of the [Jacket](../README.md) toolchain.

`jaifmt` produces canonical output: one statement per line, block bodies on their own lines, braces joined to their headers, computed indentation and zero or one space between tokens, whatever the input looked like. Formatting twice changes nothing. Comments, blank lines (up to a limit) and the line breaks inside expressions are kept; long lines are not re-wrapped.

Before it writes anything it lexes its own output and compares it with the input. If the tokens or comments differ, it refuses, so it cannot change what a program means.

```sh
jaifmt src tests                 # rewrite files in place (directories are searched for *.jai)
jaifmt --check src tests         # list the files that would change; exit status 1 (for CI)
jaifmt --stdin < in.jai > out.jai   # editor integration
```

## Install

`jaifmt` ships in every toolchain install, next to `jaic`, `jailsp` and `jailint`:

```sh
brew install matteopolak/tap/jacket        # macOS and Linux
nix profile install github:matteopolak/jacket
```

There are also the `install.sh` and `install.ps1` scripts and prebuilt archives on the [releases page](https://github.com/matteopolak/jacket/releases); see the [main README](../README.md#install) and [package managers](../docs/tools/releases.md).

The [VS Code extension](../docs/tools/vscode-extension.md) uses `jaifmt` as the document formatter. Other editors can run `jaifmt --stdin` as an external formatter.

To build it from a checkout (it needs `jaic` with LLVM; see the [LLVM setup](../docs/tools/llvm-setup.md)):

```sh
jaic jaifmt/build.jai             # target/jaifmt (native, optimised)
jaic jaifmt/build.jai - wasm      # target/jaifmt.wasm (WASI)
```

## Features

- Runs natively, under `jaic -interp` and in the browser (the playground formats with the same code, compiled to a `jaifmt.wasm` of about 283 KB).
- Token-equivalence check on every file; a file whose brackets do not balance or that does not lex is left unchanged, with an error.
- Blank lines between file-level items, spaced as the dominant style of public Jai code.
- `// jaifmt: off` and `// jaifmt: on` fence a region that must stay as written; string literals, here-strings and `#asm` blocks are always kept byte for byte.
- Exit status: 0 success, 1 `--check` found files to change, 2 errors.
- Command line parsed by `Extensions/Args`: every flag has a `--no-` form, a mistyped option gets a suggestion, `--help` is generated from the declarations in `main.jai`.

## Configuration

Settings are read from the nearest `jaifmt.toml` above each file, or from `--config <file>`:

```toml
indent_width = 4
max_blank_lines = 2
max_width = 100            # only reported by --verbose
brace_style = "same_line"  # or "preserve"
ignore = ["generated/**"]
```

Options: `--check`, `--stdin`, `--config <file>`, `-v`/`--verbose`, `--color auto|always|never`, `-h`/`--help`.

## Layout of this folder

| File | What it is |
| --- | --- |
| `main.jai` | the command-line program: arguments, directory walk, config lookup, `--check` |
| `build.jai` | a metaprogram that builds the optimised native binary or the wasm module |
| `playground.jai`, `wasm.jai` | the drivers for the browser engine and the WASI build |

The formatter itself is the library `stdlib/Extensions/Jai_Format`, which the program here wraps.

## Documentation

The docs are the source of truth for rules, options and internals:

- [jaifmt (Jai formatter)](../docs/tools/jaifmt.md): spacing and line-structure rules, configuration, the module API, tests and how to change a rule
- [Code formatting](../docs/tools/development.md): how this repository checks formatting
- [Package managers](../docs/tools/releases.md) and the [VS Code extension](../docs/tools/vscode-extension.md)
