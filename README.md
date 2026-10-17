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

Every install gives you four commands plus the standard library:

| Command | What it does |
| --- | --- |
| `jaic` | compiles, type-checks and interprets Jai programs |
| `jailsp` | language server for your editor |
| `jaifmt` | formats Jai source |
| `jailint` | finds likely mistakes, and fixes the safe ones |

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
jaic hello.jai              # build a native executable next to the file
./hello                     # Hello, world!

jaic hello.jai -interp      # or run it straight away in the interpreter
jaic hello.jai -check       # or only type-check it
```

Common options:

```sh
jaic hello.jai -o                   # optimized build
jaic hello.jai -exe build/hello     # choose the output path
jaic hello.jai -os windows          # cross-build for another OS (linux, macos, windows, wasm)
jaic hello.jai -cpu arm64           # ...or another CPU (x64, arm64)
jaic hello.jai -I vendor/modules    # also look for modules in a folder
jaic hello.jai -interp -- a b c     # pass arguments to the program
```

Run `jaic -help` for the full list.

## Editor setup

- **VS Code** (and Open VSX editors such as VSCodium and Cursor): install the [Jacket extension](https://marketplace.visualstudio.com/items?itemName=matteopolak.jacket). It can download the toolchain for you. See its [settings and features](docs/tools/vscode-extension.md).
- **Neovim** 0.11+: [jacket.nvim](editors/nvim/README.md) sets up the language server, formatter and linter, and can download the toolchain too.
- **Any other editor**: run `jailsp` as a language server over stdio.

You get diagnostics as you type, hover, go to definition, find references, rename, completion with auto-imports, inlay hints and code actions. The VS Code and Neovim plugins also format with `jaifmt` and lint with `jailint`.

## Projects

Build a program by passing its entry file to `jaic`. That file may also be a build metaprogram that sets up workspaces at compile time:

```sh
jaic src/main.jai
jaic build.jai
```

Modules are found in a `modules/` folder next to your entry file, then in folders given with `-I`, then in the standard library.

For the language server, put a `jai.toml` at the project root when your entry file isn't `build.jai`, `first.jai`, `main.jai` or `src/main.jai`, or when you keep modules elsewhere:

```toml
build_files = ["build.jai"]        # what you pass to jaic
import_path = ["vendor/modules"]   # extra module folders, like -I
```

## Formatting and linting

```sh
jaifmt src/              # format every .jai file under src/ in place
jaifmt --check src/      # list files that would change (exit 1 if any)

jailint src/             # lint every .jai file under src/
jailint src/ --fix       # apply the safe fixes
```

Lint rules can be configured with a `jailint.toml`; see the [linter docs](docs/tools/jailint.md). Formatter details are in the [jaifmt docs](docs/tools/jaifmt.md).

## Documentation

- [Features](docs/features.md): what works, performance, and what's missing
- [Compatibility with real projects](docs/tools/upstream-corpus.md#project-status)
- [Changelog](CHANGELOG.md)
- [All docs](docs/README.md)

## Feedback

Bug reports and feature requests are welcome as [issues](https://github.com/matteopolak/jacket/issues/new/choose). Pull requests are not accepted; see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[AGPL-3.0-or-later](LICENSE), with a [runtime library exception](LICENSE-EXCEPTION): programs you compile with Jacket are yours to license however you like. See [docs/license.md](docs/license.md).
