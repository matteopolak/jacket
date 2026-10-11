# `jai{c,lsp,fmt,lint}`

[![Compiler checks](https://github.com/matteopolak/jai/actions/workflows/ci.yml/badge.svg)](https://github.com/matteopolak/jai/actions/workflows/ci.yml)

An independent toolchain for the [Jai](https://en.wikipedia.org/wiki/Jai_(programming_language)) programming language:

- `jaic`, a compiler that type-checks, interprets and natively builds real Jai programs and libraries, with its own standard library;
- `jailsp`, a language server;
- `jaifmt`, a formatter;
- `jailint`, a linter.

It is an independently implemented Jai compiler. It builds native executables for macOS, Linux and Windows (x86-64 and arm64) and WebAssembly, and real projects such as the Focus editor compile and run.

**[Try it in the browser →](https://matteopolak.com/playground/jai)**

## Install

Each install includes `jaic`, `jailsp`, `jailint`, `jaifmt` and the standard library.

```sh
brew install matteopolak/tap/jai               # macOS, Linux
winget install matteopolak.jai                 # Windows
nix profile install github:matteopolak/jai     # Nix
curl -fsSL https://raw.githubusercontent.com/matteopolak/jai/main/install.sh | sh
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
jaic check examples/compile-time-record.jai        # type-check only
jaic run examples/compile-time-record.jai          # run in the interpreter
jaic build examples/compile-time-record.jai -O2    # native executable (-os windows to cross-build)
jaic check broken.jai --color always               # force coloured errors (or NO_COLOR=1 to turn them off)
jaic run examples/tour/main.jai                    # the language tour the playground opens with
jailint src/                                       # lint every .jai file under src/
jailint src/ --fix                                 # apply the safe fixes
```

## Editors

- **VS Code** and Open VSX editors: the [Jai extension](https://marketplace.visualstudio.com/items?itemName=matteopolak.jai-toolchain) ([docs](docs/tools/vscode-extension.md)).
- **Neovim** 0.11+: [jai.nvim](editors/nvim/README.md) (language server, formatter, linter, toolchain download).
- **Anything else**: run `jailsp` as a language server over stdio ([language server](docs/compiler/language-server.md)).

## Documentation

- [Features](docs/features.md): what works, performance, and what is missing
- [Language server](docs/compiler/language-server.md) · [Formatter](docs/tools/jaifmt.md) · [Linter](docs/tools/jailint.md) · [Browser build](docs/browser/playground.md)
- [Compatibility with real projects](docs/tools/upstream-corpus.md#project-status) · [Changelog](CHANGELOG.md) · [All docs](docs/README.md)

## Contributing

Pull requests are not accepted. Bug reports and feature requests are welcome as [issues](https://github.com/matteopolak/jai/issues/new/choose); see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[AGPL-3.0-or-later](LICENSE), with a [runtime library exception](LICENSE-EXCEPTION): programs you compile are yours to license however you like. See [docs/license.md](docs/license.md).
