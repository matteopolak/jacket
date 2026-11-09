<div align="center">
  <img src="docs/images/logo.svg" alt="" width="128" height="128">
  <h1>Jacket</h1>
  <p>A compiler, standard library, language server, formatter and linter for the <a href="https://en.wikipedia.org/wiki/Jai_(programming_language)">Jai</a> programming language.<br>It builds for macOS, Linux, Windows and WebAssembly.</p>
  <p>
    <a href="https://github.com/matteopolak/jacket/actions/workflows/ci.yml"><img src="https://github.com/matteopolak/jacket/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
    <a href="https://github.com/matteopolak/jacket/releases/latest"><img src="https://img.shields.io/github/v/release/matteopolak/jacket?color=d8a03f" alt="Latest release"></a>
    <a href="https://jacket.zulipchat.com/join/rzi2dto5ll5s3fhjc3rk6sgl/"><img src="https://img.shields.io/badge/chat-on%20Zulip-d8a03f?logo=zulip" alt="Chat on Zulip"></a>
  </p>
  <p>
    <a href="https://jacket.matteopolak.com"><b>API docs</b></a> ·
    <a href="docs/README.md"><b>Guides and reference</b></a> ·
    <a href="https://matteopolak.com/playground/jacket">Playground</a> ·
    <a href="https://jacket.zulipchat.com/join/rzi2dto5ll5s3fhjc3rk6sgl/">Chat (Zulip)</a> ·
    <a href="CHANGELOG.md">Changelog</a>
  </p>
</div>

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

`jacket build`, `check` and `run` take the same flags as the compiler, `jaic`, and also read your project's `jacket.toml`. `jaic hello.jai -o -os windows` is the direct form; run `jaic -help` for every option. See [jacket](docs/tools/jacket.md) and the [command-line reference](docs/reference/tools.md#compiling).

## Contributing

Bug reports and feature requests are welcome as [issues](https://github.com/matteopolak/jacket/issues/new/choose). Pull requests are not accepted; see [CONTRIBUTING.md](CONTRIBUTING.md).

Jacket is a clean-room implementation: contributors must not read the official Jai sources at all.

## License

[AGPL-3.0-or-later](LICENSE), with a [runtime library exception](LICENSE-EXCEPTION): programs you compile with Jacket can use any license you choose.
