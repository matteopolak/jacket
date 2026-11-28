<div align="center">
  <img src="docs/images/banner.svg" alt="Jacket" width="420">
  <p>
    <a href="https://github.com/matteopolak/jacket/actions/workflows/ci.yml"><img src="https://github.com/matteopolak/jacket/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
    <a href="https://github.com/matteopolak/jacket/releases/latest"><img src="https://img.shields.io/github/v/release/matteopolak/jacket" alt="Latest release"></a>
    <a href="https://jacket.zulipchat.com/register/"><img src="https://img.shields.io/badge/chat-on%20Zulip-blue?logo=zulip" alt="Chat on Zulip"></a>
  </p>
</div>

Jacket is a toolchain for the [Jai](https://en.wikipedia.org/wiki/Jai_(programming_language)) programming language, written from scratch. It has a compiler (`jaic`), a language server (`jailsp`), a formatter (`jaifmt`), a linter (`jailint`), and `jacket`, which runs all of them from one `jacket.toml` and adds tests, benchmarks, API docs and git dependencies. Its standard library is written in Jai and adds [extension modules](docs/extensions/README.md) such as JSON, HTTP, Postgres, Regex, TOML and WebGPU.

> [!NOTE]
> Jacket is unofficial and is not affiliated with or endorsed by Thekla, Inc. or the official Jai project.

### Documentation

- [API reference](https://jacket.matteopolak.com) for the standard library, with runnable examples
- [Guides](docs/README.md#guides): testing, benchmarking, documenting a library, memory safety, other targets, dependencies
- [Command-line reference](docs/reference/tools.md) and [lint rules](docs/reference/lint-rules.md)
- [Features](docs/features.md): what works and what is missing
- [Playground](https://jacket.matteopolak.com/playground) and [changelog](CHANGELOG.md)

### Example

```jai
#import "Basic";
#import "Extensions/Json";

User :: struct {
    name: string;
    languages: [..] string;
}

main :: () {
    text := #string JSON
{"name": "Ada", "languages": ["Jai", "C"]}
JSON;
    user, error := from_json(User, text);
    assert(error.kind == .NONE, "%", error.message);
    defer free_json(*user);

    print("% writes %\n", user.name, user.languages);
    print("%\n", to_json(User.{ name = "Grace" }));
}
```

```console
$ jacket run users.jai
   Compiling users (users.jai)
    Finished debug [unoptimized] in 0.39s
     Running `users`
Ada writes ["Jai", "C"]
{"name":"Grace","languages":[]}
```

### Usage

Install on macOS or Linux with Homebrew or the install script:

```sh
brew install matteopolak/tap/jacket
curl -fsSL https://jacket.matteopolak.com/install.sh | sh
```

On Windows, in PowerShell:

```powershell
irm https://jacket.matteopolak.com/install.ps1 | iex
```

`jacket upgrade` updates an installation. Nix, prebuilt archives and CI setup are in [install channels](docs/tools/releases.md#install-channels).

Start a project and run it:

```console
$ jacket new hello
created program hello in hello: jacket.toml, build.jai, main.jai
$ cd hello
$ jacket run
   Compiling hello (build.jai)
    Finished debug [unoptimized] in 0.19s
     Running `hello`
Hello from hello!
```

Tests are procedures with a `@test` note, in files under `tests/`:

```jai
// hello/tests/greeting.jai
#import "Basic";
#import "Extensions/Testing";

greets_by_name :: () {
    greeting := tprint("Hello from %!", "hello");
    expect_equal(greeting, "Hello from hello!");
} @test
```

```console
$ jacket test
    Starting 1 test
        PASS [   0.069s] greets_by_name
     Summary [   0.197s] 1 test run: 1 passed
```

`jacket.toml` holds the settings of every tool, so the editor, the build and CI agree:

```toml
[project]
entry = ["build.jai"]
import-path = ["src"]          # folders `#import` searches

[format]                       # jaifmt
indent-width = 4
max-width = 100

[lint.rules]                   # jailint
unused_variable = "deny"

[test]                         # jacket test
timeout = 30

[target.pi]                    # jacket run --target pi
triple = "armv7-unknown-linux-gnueabihf"
runner = "qemu-arm -L {sysroot} {exe}"
```

For editors, install the [VS Code extension](https://marketplace.visualstudio.com/items?itemName=matteopolak.jacket) (also on Open VSX) or [jacket.nvim](editors/nvim/README.md) for Neovim 0.11+. Any other editor can run `jailsp` over stdio.

### Tools

| Tool | What it does |
| --- | --- |
| `jaic` | Compiles Jai to native executables and libraries through LLVM, or runs it in its interpreter (`-interp`). |
| `jailsp` | Language server: errors and lints as you type, hover with types and layout, completion with auto-import, go to definition, rename, inlay hints, code actions. |
| `jailint` | Reports mistakes, [memory bugs](docs/guides/memory-safety.md) and performance pitfalls; `jacket lint --fix` applies the safe fixes. |
| `jaifmt` | Formats Jai code ([rules](docs/tools/jaifmt-rules.md)). |
| `jacket` | Runs the tools, [tests](docs/guides/testing.md), [benchmarks](docs/guides/benchmarking.md), [API docs](docs/guides/documenting.md), [dependencies](docs/guides/dependencies.md) and `migrate` for older Jai code. |

### Platform support

Jacket runs on macOS (arm64, x86-64), Linux (x86-64, arm64) and Windows (x86-64, arm64). Programs build for those, for WebAssembly, for 32-bit and big-endian Linux, and for boards with no operating system down to the 8-bit AVR ([other targets](docs/guides/other-targets.md)). Cross targets are experimental.

### Contributing

Bug reports and feature requests are welcome as [issues](https://github.com/matteopolak/jacket/issues/new/choose). Pull requests are not accepted; see [CONTRIBUTING.md](CONTRIBUTING.md). Jacket is a clean-room implementation: contributors must not read the official Jai sources.

### License

[AGPL-3.0-or-later](LICENSE), with a [runtime library exception](LICENSE-EXCEPTION): programs you compile with Jacket can use any license. The standard library ([`stdlib/`](stdlib/LICENSE.md)) is also available under MIT OR Apache-2.0, and the specifications ([`docs/specs/`](docs/specs/LICENSE.md)) are public domain (CC0).
