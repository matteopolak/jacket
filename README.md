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

**Compiler (`jaic`).** Debug builds are a little faster than the official compiler's, and optimized builds often take a fraction of the time ([numbers](docs/features.md#performance)).

- `#run` and `jaic -interp` call real C libraries, and C can call back into Jai, even from its own threads.
- `#asm` covers SSE through AVX2, FMA, AES, SHA and common AVX-512, and runs on any CPU.
- Runtime checks for overflow, bounds, narrowing casts and `#complete` switches, plus `-sanitize address,undefined`.
- Targets beyond desktop: WebAssembly (WASI and the browser), 32-bit x86 and ARM, big-endian s390x and PowerPC64, and bare-metal Cortex-M, AArch64 and AVR boards ([other targets](docs/guides/other-targets.md); the last three groups are experimental).

**Language server (`jailsp`).** Its answers come from the compiler itself, compile-time code included ([README](crates/jai-language-server/README.md)).

- Hover shows what a macro or `#insert` expanded to, the value a `#run` produced, and a struct's size and field offsets.
- Completion offers names from modules you haven't imported yet and adds the `#import`.
- Code actions replace a `#run` with its value, fill in missing `case`s and struct fields, and extract procedures.
- The same server runs in the [online playground](https://jacket.matteopolak.com/playground), compiled to WebAssembly.

**Linter (`jailint`).** Rules run on the type-checked program, so they know every expression's type ([README](crates/jailint/README.md)).

- Follows pointers through the program to find [use after free, double frees, temporary storage read after a reset, pointers into arrays that grew, and leaks](docs/guides/memory-safety.md).
- `jacket lint --fix` applies the fixes that are safe.

**Formatter (`jaifmt`).** It refuses any output whose tokens differ from the input, so formatting can't change what a program means ([docs](docs/tools/jaifmt.md)). It is written in Jai and is also a library, `Jai_Format`.

**Project tool (`jacket`).** It shares one `jacket.toml` across all the tools.

- `jacket test` runs unit, property, golden-output, compile-fail and snapshot tests, plus the examples in doc comments, and reviews changed snapshots interactively ([testing guide](docs/guides/testing.md)).
- `jacket bench` reports time per iteration with a confidence interval and compares against a saved baseline with a p-value ([benchmarking guide](docs/guides/benchmarking.md)).
- `jacket doc` writes an API site where every example has a Run button that compiles it in the browser ([jacket doc](docs/tools/jacket-doc.md)).
- `jacket migrate` rewrites code written for an older Jai, and `jacket dep` pins other people's modules as git submodules ([dependencies](docs/guides/dependencies.md)).

The [standard library](https://jacket.matteopolak.com) adds [extensions](docs/extensions/README.md) for HTTP/1–2, gRPC, JSON, Postgres, WebGPU, regex, images and more. [Features](docs/features.md) lists what works and what is missing.

## Contributing

Bug reports and feature requests are welcome as [issues](https://github.com/matteopolak/jacket/issues/new/choose). Pull requests are not accepted; see [CONTRIBUTING.md](CONTRIBUTING.md).

Jacket is a clean-room implementation: contributors must not read the official Jai sources at all.

## License

[AGPL-3.0-or-later](LICENSE), with a [runtime library exception](LICENSE-EXCEPTION): programs you compile with Jacket can use any license you choose. The standard library ([`stdlib/`](stdlib/LICENSE.md)) is also available under MIT OR Apache-2.0, so you can copy modules into your own projects, and the specifications ([`docs/specs/`](docs/specs/LICENSE.md)) are public domain (CC0).
