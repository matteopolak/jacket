# Jacket

A compiler, standard library, language server, formatter and linter for the [Jai](https://en.wikipedia.org/wiki/Jai_(programming_language)) programming language. It builds for macOS, Linux, Windows and WebAssembly.

**[API docs](https://jacket.matteopolak.com)** · **[Guides and reference](docs/README.md)** · [Playground](https://matteopolak.com/playground/jacket) · [Changelog](CHANGELOG.md)

> [!NOTE]
> Jacket is unofficial and is not affiliated with or endorsed by Thekla, Inc. or the official Jai project.

## Install

```sh
brew install matteopolak/tap/jacket                                                      # macOS, Linux
curl -fsSL https://raw.githubusercontent.com/matteopolak/jacket/main/install.sh | sh     # macOS, Linux
irm https://raw.githubusercontent.com/matteopolak/jacket/main/install.ps1 | iex          # Windows (PowerShell)
```

Update with `jacket upgrade`. Other channels (Nix, prebuilt archives, CI, building from source) are in [installation](docs/tools/releases.md#install-channels).

Editors:

- **VS Code** and Open VSX editors (VSCodium, Cursor): the [Jacket extension](https://marketplace.visualstudio.com/items?itemName=matteopolak.jacket). It can download the toolchain. See [its docs](docs/tools/vscode-extension.md).
- **Neovim** 0.11+: [jacket.nvim](editors/nvim/README.md).
- **Anything else**: run `jailsp` as a language server over stdio.

## Hello, world

```jai
#import "Basic";

main :: () {
    print("Hello, world!\n");
}
```

```sh
jacket run hello.jai
```

## Everyday commands

```sh
jacket new app           # start a project
jacket build             # build the project (or: jacket build hello.jai)
jacket run               # build and run
jacket test              # run tests and doc examples
jacket check             # type-check only
jacket fmt               # format
jacket lint --fix        # lint, applying safe fixes
jacket doc               # build the API docs site
```

`jacket build`, `check` and `run` take the same flags as the compiler, `jaic`, and also read your project's `jacket.toml`. `jaic hello.jai -o -os windows` is the direct form; run `jaic -help` for every option. See [jacket](docs/tools/jacket.md) and the [command-line reference](docs/reference/command-line.md).

## Contributing

Bug reports and feature requests are welcome as [issues](https://github.com/matteopolak/jacket/issues/new/choose). Pull requests are not accepted; see [CONTRIBUTING.md](CONTRIBUTING.md).

Jacket is a clean-room implementation: contributors must not read the official Jai sources at all.

## License

[AGPL-3.0-or-later](LICENSE), with a [runtime library exception](LICENSE-EXCEPTION): programs you compile with Jacket can use any license you choose.
