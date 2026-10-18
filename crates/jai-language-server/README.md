# jailsp

> [!NOTE]
> This project is unofficial and is not affiliated with or endorsed by Thekla, Inc. or the official Jai project.

The language server for [Jai](https://en.wikipedia.org/wiki/Jai_(programming_language)) in the [Jacket](../../README.md) toolchain.

jailsp checks your code with the real compiler, `jaic`, including the code that runs at compile time: `#run`, `#insert`, macros and build metaprograms. So the types in a hover, the members offered in completion and the errors you see are the compiler's own answers, not a guess from a separate parser.

The same server, compiled to WebAssembly, runs in the [online playground](https://matteopolak.com/playground/jacket).

## Install

jailsp comes with every Jacket install, next to `jaic`, `jailint` and `jaifmt`:

```sh
brew install matteopolak/tap/jacket
```

The [main README](../../README.md#install) lists the other ways to install.

- **VS Code** (and VSCodium, Cursor and other Open VSX editors): install the [Jacket extension](https://marketplace.visualstudio.com/items?itemName=matteopolak.jacket). It finds jailsp on your `PATH`, or offers to download the toolchain.
- **Neovim** 0.11+: [jacket.nvim](../../editors/nvim/README.md) sets it up.
- **Any other editor:** run `jailsp` as the language server for `.jai` files. It talks over stdin and stdout. For formatting, use `jaifmt --stdin`.

## What you get

- **Errors as you type:** syntax errors, type errors (several per file, even in code nothing calls yet), and [jailint](../jailint/README.md)'s warnings with quick fixes.
- **Hover:** a value's type, every overload of a procedure, a struct's size and field offsets, what a macro or `#insert` expands to, and the value a `#run` produced.
- **Completion:** names in scope, members by type, enum members from context (`color: Color = .`), directives and `#import` paths. It also completes names from modules you haven't imported yet and adds the `#import` for you.
- **Navigation:** go to definition (into modules and the standard library too), find references, call hierarchy, and document and workspace symbols.
- **Editing help:** rename, signature help, inlay hints for inferred types and parameter names, and semantic highlighting.
- **Code actions:** add a missing `#import`, add missing `case`s or struct fields, extract a variable or procedure, inline a variable, and replace a `#run` with its value.

It doesn't format code (use `jaifmt`), and it can't report type errors inside a polymorphic procedure nobody calls yet.

The [full feature list](../../docs/compiler/language-server.md#feature-list) names the LSP request behind each feature.

## Opening code you don't trust

Compile-time code is real code: opening a project and type-checking it runs its `#run` blocks and build metaprograms. jailsp runs them in a sandbox with a time budget, but if you don't trust a folder, start the server with `--untrusted`. It then checks everything that doesn't depend on compile-time results and runs nothing. VS Code does this automatically for folders you haven't trusted. See [workspace trust](../../docs/tools/workspace-trust.md).

## Configuration

Most projects need none: jailsp finds the entry file (`build.jai`, `first.jai`, `main.jai` or `src/main.jai`) and `modules/` folders on its own. If yours is laid out differently, add a `jai.toml` at the project root:

```toml
build_files = ["src/app.jai"]   # what you would pass to jaic
import_path = ["vendor"]        # extra module folders, like jaic -I
```

Lint levels come from the nearest `jailint.toml`. `JAIC_STDLIB` points the server at a different standard library.

The editor can send one setting: `completion.autoImport` (on by default) turns auto-import completion on or off.

## Compared with Jails

[Jails](https://github.com/SogoCZE/Jails) is the established Jai language server, written in Jai. The two work differently. Jails parses your files itself, and gets its errors by running a separately installed Jai compiler on your build file. jailsp has the type checker built in, so it needs no other compiler, and it also offers hover, find references, rename, inlay hints, code actions and lints. Jails' own feature set is go to definition, completion, signature help and compiler errors.

The [benchmarks page](../../docs/tools/benchmarks.md) explains how to time a server with `python3 tools/bench/main.py lsp`, including another server given with `--server NAME=COMMAND`.

## Building from source

```sh
cargo build -p jai-language-server --release   # target/release/jailsp
```

## More

- [Language server reference](../../docs/compiler/language-server.md): how analysis, caching and completion work, and their limits
- [VS Code extension](../../docs/tools/vscode-extension.md) and [browser build](../../docs/browser/playground.md)
