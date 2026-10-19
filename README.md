# Jacket

[![Compiler checks](https://github.com/matteopolak/jacket/actions/workflows/ci.yml/badge.svg)](https://github.com/matteopolak/jacket/actions/workflows/ci.yml)

Jacket is a compiler, standard library, language server, formatter and linter for the [Jai](https://en.wikipedia.org/wiki/Jai_(programming_language)) programming language, written independently of the official compiler. It builds programs for macOS, Linux and Windows (x86-64 and arm64) and for WebAssembly, and compiles real projects such as the Focus editor.

[Try it in the browser](https://matteopolak.com/playground/jacket) without installing anything.

> [!NOTE]
> Jacket is unofficial and is not affiliated with or endorsed by Thekla, Inc. or the official Jai project.

## Install

| Platform | Command |
| --- | --- |
| macOS, Linux (Homebrew) | `brew install matteopolak/tap/jacket` |
| macOS, Linux (script) | `curl -fsSL https://raw.githubusercontent.com/matteopolak/jacket/main/install.sh \| sh` |
| Windows (PowerShell) | `irm https://raw.githubusercontent.com/matteopolak/jacket/main/install.ps1 \| iex` |
| Nix | `nix profile install github:matteopolak/jacket` |

To update, run `jacket upgrade`, or `brew upgrade jacket` / `nix profile upgrade jacket` if you installed that way.

Prebuilt archives are on the [releases page](https://github.com/matteopolak/jacket/releases). Other options, such as CI setup and building from source, are in [installation](docs/tools/releases.md#install-channels).

## Usage

Save this as `hello.jai`:

```jai
#import "Basic";

main :: () {
    print("Hello, world!\n");
}
```

```sh
jacket run hello.jai            # build and run it
jacket run hello.jai -interp    # run it in the interpreter, without a native build
jacket check hello.jai          # type-check only
jacket build hello.jai          # build only
```

Common build options:

```sh
jacket build hello.jai -o                   # optimize
jacket build hello.jai -exe build/hello     # set the output path
jacket build hello.jai -os windows          # target another OS: linux, macos, windows, wasm
jacket build hello.jai -cpu arm64           # target another CPU: x64, arm64
jacket build hello.jai -I vendor/modules    # add a module folder
jacket run hello.jai -- a b c               # pass arguments to the program
```

Run `jacket build -help` for every option.

## Tools

| Command | Purpose |
| --- | --- |
| `jacket` | Project command: `new`, `build`, `check`, `run`, `fmt`, `lint`, `lsp`, `upgrade` |
| `jaic` | Compiler and interpreter |
| `jailsp` | Language server |
| `jaifmt` | Formatter |
| `jailint` | Linter, with automatic fixes |

`jacket build`, `check` and `run` take the same arguments as `jaic`, and `fmt`, `lint` and `lsp` take those of `jaifmt`, `jailint` and `jailsp`. The difference is that `jacket` also applies your project's `jacket.toml`. See [jacket](docs/tools/jacket.md).

## Projects

`jacket build` with no file builds the first of `build.jai`, `first.jai`, `main.jai` or `src/main.jai` it finds. The entry file can be a build metaprogram that sets up workspaces at compile time.

Modules are looked up in a `modules/` folder next to the entry file, then in `-I` folders, then in the standard library.

Add a `jacket.toml` at the project root to change these defaults:

```toml
[project]
entry = ["build.jai"]              # what `jacket build` compiles
import_path = ["vendor/modules"]   # extra module folders, like -I
```

The same file holds lint rules (`[lint]`) and formatter settings (`[format]`). If you have older `jai.toml`, `jailint.toml` or `jaifmt.toml` files, `jacket config migrate` merges them into one `jacket.toml`.

## Formatting and linting

```sh
jacket fmt               # format every .jai file in the project
jacket fmt --check src/  # list files under src/ that would change; exits 1 if any

jacket lint src/         # lint every .jai file under src/
jacket lint --fix src/   # apply the safe fixes
```

See the [linter](docs/tools/jailint.md) and [formatter](docs/tools/jaifmt.md) docs for rules and settings.

## Editors

- **VS Code**, and Open VSX editors such as VSCodium and Cursor: install the [Jacket extension](https://marketplace.visualstudio.com/items?itemName=matteopolak.jacket). It can download the toolchain for you. See its [features and settings](docs/tools/vscode-extension.md).
- **Neovim** 0.11+: [jacket.nvim](editors/nvim/README.md) sets up the language server, formatter and linter, and can also download the toolchain.
- **Other editors**: run `jailsp` as a language server over stdio.

The language server provides diagnostics, hover, go to definition, find references, rename, completion with auto-imports, inlay hints and code actions.

## More

- [Features](docs/features.md): what works, performance, and what's missing
- [Compatibility with real projects](docs/tools/upstream-corpus.md#project-status)
- [Changelog](CHANGELOG.md)
- [All docs](docs/README.md)

Bug reports and feature requests are welcome as [issues](https://github.com/matteopolak/jacket/issues/new/choose). Pull requests are not accepted; see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[AGPL-3.0-or-later](LICENSE), with a [runtime library exception](LICENSE-EXCEPTION): programs you compile with Jacket can use any license you choose. See [docs/license.md](docs/license.md).
