# jaifmt

> [!NOTE]
> This project is unofficial and is not affiliated with or endorsed by Thekla, Inc. or the official Jai project.

The code formatter for [Jai](https://en.wikipedia.org/wiki/Jai_(programming_language)) in the [Jacket](../README.md) toolchain. It is written in Jai.

```sh
jaifmt src/                    # format every .jai file under src/, in place
jaifmt --check src/            # only list the files that would change (exit 1 if any)
jaifmt --stdin < in.jai        # format standard input, for editors
```

## What it does

jaifmt gives the same output however the input was spaced. It puts one statement on each line and each block body on its own lines, joins `{` to the line it opens, fixes indentation, and leaves either no space or one space between tokens. Running it twice changes nothing.

It keeps what you chose on purpose: your comments, your blank lines (up to a limit) and where you broke a long expression. It never re-wraps long lines.

It can't change what your program means. Before writing a file, jaifmt lexes its own output and compares it with the input. If any token or comment differs, it leaves the file alone and reports an error. A file that doesn't lex or has unbalanced brackets is also left alone.

To keep a region exactly as written, put it between `// jaifmt: off` and `// jaifmt: on`. Strings, here-strings and `#asm` blocks are always kept byte for byte.

## Install

jaifmt comes with every Jacket install, next to `jaic`, `jailsp` and `jailint`:

```sh
brew install matteopolak/tap/jacket
```

The [main README](../README.md#install) lists the other ways to install.

**Editors:** the [VS Code extension](../docs/tools/vscode-extension.md) and [jacket.nvim](../editors/nvim/README.md) format with jaifmt. In any other editor, set `jaifmt --stdin` as the external formatter.

## Options

| Option | Effect |
| --- | --- |
| `--check` | write nothing; list files that would change and exit with 1 |
| `--stdin` | format standard input to standard output |
| `--config <file>` | use this file instead of the nearest `jaifmt.toml` |
| `-v`, `--verbose` | report every file, a summary, and lines longer than `max_width` |
| `--color auto\|always\|never` | colour in error messages (`NO_COLOR` is respected) |

Every flag also has a `--no-` form. Directories are searched for `*.jai` files.

The exit status is 0 on success, 1 when `--check` found files to change, and 2 on an error.

## Configuration

jaifmt reads the nearest `jaifmt.toml` above each file:

```toml
indent_width = 4
max_blank_lines = 2
brace_style = "same_line"   # or "preserve"
ignore = ["generated/**"]
```

A few more options add blank lines in specific places (between `case`s, before a `return`, ...). They are all off by default. The [full reference](../docs/tools/jaifmt.md#configuration) lists them. An unknown key is an error, so a typo can't go unnoticed.

## Building from source

Building jaifmt needs `jaic` with LLVM; see the [LLVM setup](../docs/tools/llvm-setup.md):

```sh
jaic jaifmt/build.jai           # writes target/jaifmt
jaic jaifmt/build.jai - wasm    # writes target/jaifmt.wasm, used by the playground
```

The formatting itself lives in the `Extensions/Jai_Format` module, which works on strings and has no file access. This folder only holds the command-line program (`main.jai`), the build script (`build.jai`) and the two browser drivers (`playground.jai`, `wasm.jai`).

## More

- [jaifmt reference](../docs/tools/jaifmt.md): every rule, the module API, the browser build, and how to change a rule
