<div align="center">
  <img src="docs/images/banner.svg" alt="Jacket" width="420">
  <p>A compiler, standard library, language server, formatter and linter for the <a href="https://en.wikipedia.org/wiki/Jai_(programming_language)">Jai</a> programming language.</p>
  <p>
    <a href="https://github.com/matteopolak/jacket/actions/workflows/ci.yml"><img src="https://github.com/matteopolak/jacket/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
    <a href="https://github.com/matteopolak/jacket/releases/latest"><img src="https://img.shields.io/github/v/release/matteopolak/jacket" alt="Latest release"></a>
    <a href="https://jacket.zulipchat.com/register/"><img src="https://img.shields.io/badge/chat-on%20Zulip-blue?logo=zulip" alt="Chat on Zulip"></a>
  </p>
  <p>
    <a href="https://jacket.matteopolak.com"><b>API docs</b></a> ·
    <a href="docs/README.md"><b>Guides and reference</b></a> ·
    <a href="https://jacket.matteopolak.com/playground">Playground</a> ·
    <a href="https://jacket.zulipchat.com/register/">Chat (Zulip)</a> ·
    <a href="CHANGELOG.md">Changelog</a>
  </p>
</div>

> [!NOTE]
> Jacket is unofficial and is not affiliated with or endorsed by Thekla, Inc. or the official Jai project.

## Install

macOS and Linux, with Homebrew:

```sh
brew install matteopolak/tap/jacket
```

macOS and Linux, with the install script:

```sh
curl -fsSL https://jacket.matteopolak.com/install.sh | sh
```

Windows, in PowerShell:

```powershell
irm https://jacket.matteopolak.com/install.ps1 | iex
```

Update with `jacket upgrade`. Nix, prebuilt archives, CI setup and building from source are in [installation](docs/tools/releases.md#install-channels).

For editors, install the [VS Code extension](https://marketplace.visualstudio.com/items?itemName=matteopolak.jacket) (also on Open VSX for VSCodium and Cursor) or [jacket.nvim](editors/nvim/README.md) for Neovim 0.11+. Any other editor can run `jailsp` as a language server over stdio.

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

`jacket new app` starts a project. Run `jacket --help` for the rest, or read [jacket](docs/tools/jacket.md).

## What's inside

**`jaic`**
- Builds for 32-bit x86 and ARM, big-endian Linux, WebAssembly, and bare-metal boards down to the 8-bit AVR ([other targets](docs/guides/other-targets.md))
- [Extensions](docs/extensions/README.md) add modules such as HTTP, gRPC, Postgres, Regex, Toml and WebGPU; Json, Csv, Byte_Search and Hash_Map use SIMD

**`jailsp`** ([README](crates/jai-language-server/README.md))
- Errors and lint warnings as you type, with quick fixes
- Hover with types, struct layout, macro and `#insert` expansions, and `#run` results
- Completion with auto-import, enum members from context and `#import` paths
- Go to definition, find references, call hierarchy, symbols and rename
- Signature help, inlay hints and semantic highlighting
- Code actions: add missing `case`s or fields, extract or inline, replace a `#run` with its value

**`jailint`** ([README](crates/jailint/README.md))
- Reports common mistakes, [memory bugs](docs/guides/memory-safety.md) and performance pitfalls
- `jacket lint --fix` applies safe fixes

**`jaifmt`** ([docs](docs/tools/jaifmt.md))
- Formats code

**`jacket`**
- One `jacket.toml` for every tool
- [Tests](docs/guides/testing.md), [benchmarks](docs/guides/benchmarking.md) and [API docs](docs/tools/jacket-doc.md) with runnable examples
- Migrates code from older Jai and manages [dependencies](docs/guides/dependencies.md)

[Features](docs/features.md) lists what works and what's missing.

## Contributing

Bug reports and feature requests are welcome as [issues](https://github.com/matteopolak/jacket/issues/new/choose). Pull requests are not accepted; see [CONTRIBUTING.md](CONTRIBUTING.md).

Jacket is a clean-room implementation: contributors must not read the official Jai sources at all.

## License

[AGPL-3.0-or-later](LICENSE), with a [runtime library exception](LICENSE-EXCEPTION): programs you compile with Jacket can use any license you choose. The standard library ([`stdlib/`](stdlib/LICENSE.md)) is also available under MIT OR Apache-2.0, so you can copy modules into your own projects, and the specifications ([`docs/specs/`](docs/specs/LICENSE.md)) are public domain (CC0).
