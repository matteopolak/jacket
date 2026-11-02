# Jacket

A compiler, standard library, language server, formatter and linter for the Jai programming language. Programs build for macOS, Linux, Windows and WebAssembly. Jacket is unofficial and is not affiliated with or endorsed by Thekla, Inc. or the official Jai project.

## Install

On macOS and Linux:

```sh
brew install matteopolak/tap/jacket
```

Without Homebrew, `curl -fsSL https://raw.githubusercontent.com/matteopolak/jacket/main/install.sh | sh`. On Windows, in PowerShell:

```sh
irm https://raw.githubusercontent.com/matteopolak/jacket/main/install.ps1 | iex
```

`jacket upgrade` updates it. Editors: the [VS Code extension](../tools/vscode-extension.md), [jacket.nvim](../../editors/nvim/README.md) for Neovim, or `jailsp` as a language server anywhere else.

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

`jacket new app` starts a project with a `jacket.toml`; then `jacket run`, `jacket test`, `jacket fmt` and `jacket lint` work from anywhere inside it.

## Where to start

- [Basic](../../stdlib/Basic): printing, formatting, arrays, strings and allocation. Nearly every program imports it.
- [String](../../stdlib/String) and [Hash_Table](../../stdlib/Hash_Table) for text and lookups; [File](../../stdlib/File) and [File_Utilities](../../stdlib/File_Utilities) for the file system; [Process](../../stdlib/Process) and [Thread](../../stdlib/Thread) for running work.
- Extensions add what the standard library lacks: [Json](../../stdlib/Extensions/Json), [Http](../../stdlib/Extensions/Http), [Postgres](../../stdlib/Extensions/Postgres), [Args](../../stdlib/Extensions/Args), [Regex](../../stdlib/Extensions/Regex), [UI](../../stdlib/Extensions/UI) and [Testing](../../stdlib/Extensions/Testing). They are marked with a badge; the official compiler does not ship them.

Every module is in the sidebar. Press <kbd>/</kbd> to search by name, or by signature: `string -> int` finds procedures that take a string and return an int.

## Guides

- [Testing](../guides/testing.md): `@test` procedures, golden files and checked doc examples.
- [Benchmarking](../guides/benchmarking.md): `@bench` procedures and comparing runs against a baseline.
- [Documenting a library](../guides/documenting.md): doc comments, runnable examples and building a site like this one.
- [Tools](tools.md), [lint rules](lint-rules.md) and [format rules](../tools/jaifmt-rules.md) list every command, check and setting.
