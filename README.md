# Jacket

Jacket ("Jai kit") is an independently implemented toolchain for the [Jai](https://en.wikipedia.org/wiki/Jai_(programming_language)) programming language.

> [!NOTE]
> This project is unofficial and is not affiliated with or endorsed by Thekla or the official Jai project.

[![Compiler checks](https://github.com/matteopolak/jacket/actions/workflows/ci.yml/badge.svg)](https://github.com/matteopolak/jacket/actions/workflows/ci.yml)

It includes:

- `jaic`, a compiler that type-checks, interprets and natively builds real Jai programs and libraries, with its own standard library;
- `jailsp`, a language server;
- `jaifmt`, a formatter;
- `jailint`, a linter.

`jaic` builds native executables for macOS, Linux and Windows (x86-64 and arm64) and WebAssembly, and real projects such as the Focus editor compile and run.

**[Try it in the browser →](https://matteopolak.com/playground/jacket)**

## Install

Each install includes `jaic`, `jailsp`, `jailint`, `jaifmt` and the standard library.

```sh
brew install matteopolak/tap/jacket               # macOS, Linux
nix profile install github:matteopolak/jacket     # Nix
curl -fsSL https://raw.githubusercontent.com/matteopolak/jacket/main/install.sh | sh
```

The last line is the install script for macOS and Linux. There are also a PowerShell script, prebuilt archives, a GitHub Action and [build-from-source steps](docs/tools/package-managers.md#other-ways-to-install).

## Usage

```jai
Node :: struct(T: Type) {
    next: *Node(T);
    value: T = 19;
}

answer :: () -> int { return 23; }

main :: () -> int {
    node: Node(int);
    return node.value + #run answer(); // Exit status: 42
}
```

```sh
jaic examples/compile-time-record.jai -check        # type-check only
jaic examples/compile-time-record.jai -interp       # run in the interpreter
jaic examples/compile-time-record.jai               # native executable (-o: optimized; -os windows to cross-build)
jaic broken.jai -check -no_color                    # plain errors (or FORCE_COLOR=1 to force colour into a pipe)
jaic examples/tour/main.jai -interp                 # the language tour the playground opens with
jailint src/                                       # lint every .jai file under src/
jailint src/ --fix                                 # apply the safe fixes
```

## Editors

- **VS Code** and Open VSX editors: the [Jai extension](https://marketplace.visualstudio.com/items?itemName=matteopolak.jacket) ([docs](docs/tools/vscode-extension.md)).
- **Neovim** 0.11+: [jacket.nvim](editors/nvim/README.md) (language server, formatter, linter, toolchain download).
- **Anything else**: run `jailsp` as a language server over stdio ([language server](docs/compiler/language-server.md)).

## Documentation

- [Features](docs/features.md): what works, performance, and what is missing
- [Language server](docs/compiler/language-server.md) · [Formatter](docs/tools/jaifmt.md) · [Linter](docs/tools/jailint.md) · [Browser build](docs/browser/playground.md)
- [Compatibility with real projects](docs/tools/upstream-corpus.md#project-status) · [Changelog](CHANGELOG.md) · [All docs](docs/README.md)

## Contributing

Pull requests are not accepted. Bug reports and feature requests are welcome as [issues](https://github.com/matteopolak/jacket/issues/new/choose); see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[AGPL-3.0-or-later](LICENSE), with a [runtime library exception](LICENSE-EXCEPTION): programs you compile are yours to license however you like. See [docs/license.md](docs/license.md).
