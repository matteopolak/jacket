# Jacket

[![Compiler checks](https://github.com/matteopolak/jacket/actions/workflows/ci.yml/badge.svg)](https://github.com/matteopolak/jacket/actions/workflows/ci.yml)

Jacket ("Jai kit") is an independently implemented toolchain for the [Jai](https://en.wikipedia.org/wiki/Jai_(programming_language)) programming language: a compiler with its own standard library, a language server, a formatter and a linter. It builds native programs for macOS, Linux and Windows (x86-64 and arm64) and WebAssembly, and compiles real projects such as the Focus editor.

**[▶ Try it in the browser](https://matteopolak.com/playground/jacket)**, no install needed.

> [!NOTE]
> This project is unofficial and is not affiliated with or endorsed by Thekla, Inc. or the official Jai project.

## Contents

- [Install](#install)
- [Quick start](#quick-start)
- [Editor setup](#editor-setup)
- [Projects](#projects)
- [Formatting and linting](#formatting-and-linting)
- [Documentation](#documentation)
- [Feedback](#feedback)
- [License](#license)

## Install

| Platform | Command |
| --- | --- |
| macOS, Linux (Homebrew) | `brew install matteopolak/tap/jacket` |
| macOS, Linux (script) | `curl -fsSL https://raw.githubusercontent.com/matteopolak/jacket/main/install.sh \| sh` |
| Windows (PowerShell) | `irm https://raw.githubusercontent.com/matteopolak/jacket/main/install.ps1 \| iex` |
| Nix | `nix profile install github:matteopolak/jacket` |

Every install gives you five commands plus the standard library:

| Command | What it does |
| --- | --- |
| `jacket` | runs the tools below with your project's `jacket.toml`: `build`, `check`, `run`, `lint`, `fmt`, `lsp`, `new` |
| `jaic` | compiles, type-checks and interprets Jai programs |
| `jailsp` | language server for your editor |
| `jaifmt` | formats Jai source |
| `jailint` | finds likely mistakes, and fixes the safe ones |

To update a script or PowerShell install, run `jacket upgrade` (`--check` only looks; Homebrew and Nix installs update through `brew upgrade jacket` and Nix).

Prebuilt archives are on the [releases page](https://github.com/matteopolak/jacket/releases). For other ways to install (CI, building from source), see [installation](docs/tools/releases.md#install-channels).

## Quick start

Save this as `hello.jai`:

```jai
#import "Basic";

main :: () {
    name := "world";
    print("Hello, %!\n", name);
}
```

Then:

```sh
jacket run hello.jai            # build a native executable next to the file and run it: Hello, world!
jacket run hello.jai -interp    # or run it straight away in the interpreter
jacket check hello.jai          # or only type-check it
jacket build hello.jai          # or only build it
```

Common options:

```sh
jacket build hello.jai -o                   # optimized build
jacket build hello.jai -exe build/hello     # choose the output path
jacket build hello.jai -os windows          # cross-build for another OS (linux, macos, windows, wasm)
jacket build hello.jai -cpu arm64           # ...or another CPU (x64, arm64)
jacket build hello.jai -I vendor/modules    # also look for modules in a folder
jacket run hello.jai -- a b c               # pass arguments to the program
```

`jacket build` takes exactly the compiler's arguments, so `jacket build hello.jai -o` is `jaic hello.jai -o`. Run `jacket build -help` for the full list.

## Editor setup

- **VS Code** (and Open VSX editors such as VSCodium and Cursor): install the [Jacket extension](https://marketplace.visualstudio.com/items?itemName=matteopolak.jacket). It can download the toolchain for you. See its [settings and features](docs/tools/vscode-extension.md).
- **Neovim** 0.11+: [jacket.nvim](editors/nvim/README.md) sets up the language server, formatter and linter, and can download the toolchain too.
- **Any other editor**: run `jailsp` as a language server over stdio.

You get diagnostics as you type, hover, go to definition, find references, rename, completion with auto-imports, inlay hints and code actions. The VS Code and Neovim plugins also format with `jaifmt` and lint with `jailint`.

## Projects

Build a program by passing its entry file to `jacket build`. That file may also be a build metaprogram that sets up workspaces at compile time:

```sh
jacket build src/main.jai
jacket build build.jai
jacket build              # the project's entry file: build.jai, first.jai, main.jai or src/main.jai
```

Modules are found in a `modules/` folder next to your entry file, then in folders given with `-I`, then in the standard library.

Put a `jacket.toml` at the project root when your entry file isn't `build.jai`, `first.jai`, `main.jai` or `src/main.jai`, or when you keep modules elsewhere:

```toml
[project]
entry = ["build.jai"]              # what `jacket build` compiles
import_path = ["vendor/modules"]   # extra module folders, like -I
```

`jacket` is the front door: `jacket build`, `check`, `run`, `lint`, `fmt` and `lsp` take exactly the arguments of `jaic`, `jailint`, `jaifmt` and `jailsp`, and add this file's settings; see [jacket](docs/tools/jacket.md). The old `jai.toml`, `jailint.toml` and `jaifmt.toml` still work and `jacket config migrate` folds them into one `jacket.toml`.

## Formatting and linting

```sh
jacket fmt               # format every .jai file in the project in place
jacket fmt --check src/  # list files under src/ that would change (exit 1 if any)

jacket lint src/         # lint every .jai file under src/
jacket lint --fix src/   # apply the safe fixes
```

Lint rules (`[lint]`) and formatter settings (`[format]`) live in `jacket.toml`; see the [linter docs](docs/tools/jailint.md). Formatter details are in the [jaifmt docs](docs/tools/jaifmt.md).

## Documentation

- [Features](docs/features.md): what works, performance, and what's missing
- [Compatibility with real projects](docs/tools/upstream-corpus.md#project-status)
- [Changelog](CHANGELOG.md)
- [All docs](docs/README.md)

## Feedback

Bug reports and feature requests are welcome as [issues](https://github.com/matteopolak/jacket/issues/new/choose). Pull requests are not accepted; see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[AGPL-3.0-or-later](LICENSE), with a [runtime library exception](LICENSE-EXCEPTION): programs you compile with Jacket are yours to license however you like. See [docs/license.md](docs/license.md).
