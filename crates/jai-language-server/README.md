# jailsp

> [!NOTE]
> This project is unofficial and is not affiliated with or endorsed by Thekla or the official Jai project.

The language server for [Jai](https://en.wikipedia.org/wiki/Jai_(programming_language)) in the [Jacket](../../README.md) toolchain. It speaks the Language Server Protocol over stdio. The same Rust session also runs in the browser, compiled to WebAssembly, behind the [online playground](https://matteopolak.com/playground/jai).

It type-checks your code with `jaic`, the toolchain's compiler, including compile-time code (`#run`, `#insert`, macros and metaprograms). Hover, completion and diagnostics therefore come from the real type checker, and not from a separate approximation of the language.

## Install

`jailsp` ships in every toolchain install, next to `jaic`, `jailint` and `jaifmt`:

```sh
brew install matteopolak/tap/jacket        # macOS and Linux
nix profile install github:matteopolak/jacket
```

There are also the `install.sh` and `install.ps1` scripts and prebuilt archives on the [releases page](https://github.com/matteopolak/jacket/releases); see the [main README](../../README.md#install) and [package managers](../../docs/tools/package-managers.md).

**VS Code and compatible editors:** install the [Jai extension](https://marketplace.visualstudio.com/items?itemName=matteopolak.jai-toolchain) (also on [Open VSX](https://open-vsx.org/extension/matteopolak/jai-toolchain)). It starts `jailsp` from your `PATH`, or offers to download the toolchain ([details](../../docs/tools/vscode-extension.md)).

**Other editors:** start `jailsp` with no arguments as the language server for `.jai` files, and `jaifmt --stdin` as the formatter. `jailsp --help` and `--version` work from a terminal.

From a checkout: `cargo build -p jai-language-server --release --locked` (the binary is `target/release/jailsp`).

## Features

Everything below is in the [feature list](../../docs/compiler/language-server.md#feature-list) with the LSP method behind it.

- **Diagnostics:** syntax errors, the type checker's errors (several per file, also in code nothing calls), and [jailint](../jailint/README.md)'s lints with quick fixes. Pull diagnostics when the client supports them, push otherwise.
- **Doc comments:** the comment above a declaration shows as Markdown in hover, completion and signature help (with per-parameter docs), and `[name]` links in it resolve.
- **Hover:** types, every overload, struct layout (size, alignment, padding, field offsets), the expansion of a macro call or `#insert`, the value of a `#run`, whether an `#if` held, and the arguments of a format string.
- **Completion:** scope-aware names, members, inferred enum members (`d: Color = .`), directives, `#load`/`#import` paths, `#asm` instructions and registers, and auto-import of names from stdlib modules, project modules and files not loaded yet (with the `#import` added for you).
- **Navigation:** go to definition (into modules and the stdlib too), type definition, references, document highlights, document and workspace symbols, call hierarchy, document links, folding and selection ranges.
- **Editing:** rename, signature help, inlay hints (inferred types, parameter names, `#run` values), semantic tokens, code lenses for polymorphs.
- **Code actions:** add a missing `#import`, show or inline an expansion, replace a `#run` with its value, lint fixes, and refactorings (extract variable or procedure, inline variable, add missing `case`s and struct fields, `ifx` rewrites).
- **Not supported:** formatting (use `jaifmt --stdin`), a type error inside a polymorphic procedure nobody instantiates, and several errors from one procedure body.

## Configuration

- `jai.toml` at the project root states the entry files and module folders; without it the server infers them (`modules/` folders next to the file and above it, up to the workspace folder):
  ```toml
  build_files = ["src/main.jai"]
  import_path = ["vendor"]
  ```
- `jailint.toml` (nearest above a document) sets lint levels and exclusions.
- Client settings (`initializationOptions` or the `jai` section of `workspace/didChangeConfiguration`): `completion.autoImport` (default `true`). VS Code's `jai.completion.autoImport` sends it.
- `JAIC_STDLIB` overrides the standard library directory the native server reads.

## Compared with Jails

[Jails](https://github.com/SogoCZE/Jails) by SogoCZE is the established language server for Jai, written in Jai, and the one many Jai programmers use today. This section compares the two as they behave on one machine. It is written from Jails' readme and source and from our own docs, and anything not checked is marked.

Jails was built from the revision pinned in our [upstream corpus](../../docs/tools/upstream-corpus.md) (`42fa76c8`, version string 0.2.22). It was compiled with **`jaic`, this project's compiler, and not with the official Jai compiler**, so the Jails numbers below reflect jaic's code generation (`-release`, which its build script maps to `VERY_OPTIMIZED`) and may differ from a build by Jai. Jails' diagnostics run a Jai compiler as a child process; none was configured here (see below), so those numbers say nothing about that path.

### Features

"Yes" for Jails means found in its source or readme at that revision. "Not run" means we read it but did not exercise it.

| Feature | jailsp | Jails |
| --- | --- | --- |
| Go to definition | yes (also fields, enum members, the resolved overload, `#load`/`#import` targets) | yes (parser-based) |
| Completion | yes: type-checker scopes, members, inferred enum members, directives, paths | yes: names in scope, members, module names for `#import`, optional `(` insertion |
| Auto-import completion | yes, adds the `#import`/`#load` | no (no additional edits in its source) |
| Hover | yes | no: the request is handled by commented-out code and `hoverProvider` is not advertised |
| Find references, document highlight | yes | no |
| Rename | yes | no |
| Signature help | yes (overload the call resolved to) | yes |
| Diagnostics: parse errors | yes, all of them in the open documents, while typing | not found in its source (it reports what the compiler reports) |
| Diagnostics: type-check errors | yes, in-process, several per file | by running a Jai compiler on the project's `build_root`; its source parses the first error and says warnings are a to-do. Not run here: no Jai compiler was configured |
| Diagnostics: lint | yes ([jailint](../jailint/README.md) rules and quick fixes) | no |
| Semantic tokens | yes | no (commented out) |
| Inlay hints | yes | no |
| Code actions, refactorings | yes | no |
| Formatting | no (use [jaifmt](../../jaifmt/README.md)) | no |
| Call hierarchy | yes | no |
| Document symbols | yes | yes |
| Workspace symbols | yes (open documents and project files) | yes (can include local modules) |
| `#asm` | yes: instruction, operand and register completion, hover, signature help | not found in source |
| Metaprograms and `#run` | the compile runs `#run` and a build metaprogram's workspaces in a sandbox; hovers show the `#run` value and expansions | does not evaluate compile-time code; its diagnostics run your build program through the Jai compiler, which you configure to produce no output |
| Macro and `#insert` expansion | yes (hover, code actions, expansion documents) | planned in its readme |
| Browser build | yes (WebAssembly, used by the playground) | not found |
| Project config | `jai.toml`, `jailint.toml`; inferred when absent | `jails.json`: `roots`, `local_modules`, `build_root`, `intermediate_path`, `auto_insert_parentheses`, `use_symbols_from_local_modules` |
| Needs an installed Jai compiler | no, the checker is built in | yes: it finds `jai` on `PATH` (or `-jai_path`) for its module directory and for diagnostics |
| Editor extension | VS Code, Open VSX | VS Code Marketplace (per its readme: x64 Windows and ARM64 macOS builds); other clients use the binary |

Jails describes itself as experimental and unstable, and its readme lists go to definition, completion, signature help and compiler errors as its feature set; the table follows that.

### Performance

`tools/lsp_bench.py` starts each server fresh for each session over stdio and times a fixed script of requests on real and generated projects ([method](../../docs/tools/lsp-benchmark.md)). Milliseconds are the median of the warm sessions (two, from `--repeat 3`); CPU is the server's total user plus system seconds for a session; RSS is the peak resident set. Both servers were driven as push-diagnostics clients.

Apple M5, 10 cores, 16 GiB, macOS 27.0, 9 October 2026. Both servers are release builds; jailsp is checkout `e15e3678` (`jailsp 0.6.0`), which includes the debounced push diagnostics.

| Workload | Server | First diagnostics | Hover | Edit, then hover | Typing completion (first / median) | CPU s | Peak RSS MiB |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| chess-jai (78 KiB) | jailsp | 241 | 0.6 | 18 | 4.9 / 3.4 | 0.26 | 82 |
| | Jails | none | n/a | n/a | 9.1 / 4.5 | 0.05 | 69 |
| Focus (`editors.jai`, 200 KiB) | jailsp | 522 | 1.2 | 280 | 10 / 7.4 | 1.92 | 273 |
| | Jails | none | n/a | n/a | 60 / 44 | 0.50 | 515 |
| gen-60k (12 files) | jailsp | 504 | 1.1 | 222 | 13 / 13 | 1.58 | 217 |
| | Jails | none | n/a | n/a | 27 / 20 | 0.12 | 206 |
| gen-240k (48 files) | jailsp | 2,046 | 2.4 | 1,565 | 20 / 16 | 10.10 | 646 |
| | Jails | none | n/a | n/a | 61 / 48 | 0.41 | 638 |
| large-100k (one 3.3 MB file) | jailsp | 1,102 | 8.9 | 400 | 239 / 228 | 3.79 | 617 |
| | Jails | none | n/a | n/a | 151 / 88 | 0.62 | 322 |

How to read it:

- **First diagnostics:** `none` means Jails pushed nothing within 3 s. Its diagnostics come from a Jai compiler child process, which this machine did not have configured, so no comparison is possible. jailsp's figure includes a full type check of the program, and with push diagnostics it also includes the short quiet period jailsp waits for before publishing.
- **Hover and edit-then-hover:** `n/a` means Jails does not advertise hover. jailsp's edit-then-hover includes re-checking the edited program.
- **Typing completion:** the two servers do different work. Jails completes from its own syntax tree; jailsp completes from the last compile that type-checked, which is what lets it offer members by type and auto-imports. In this run jailsp's median is lower than Jails' on four of the five workloads (chess-jai, Focus, gen-60k, gen-240k) and higher on large-100k (228 against 88 ms); its first keystroke after an edit is lower on the same four and higher on large-100k (239 against 151 ms). The Jails rows use a build with a one-line change (below).
- **Memory and CPU:** Jails' peak RSS is lower on four of the five workloads (by 13 MiB or less except on large-100k, 322 against 617 MiB) and higher on Focus (515 against 273 MiB); its CPU time is lower on all of them, with the caveat that it type-checks nothing.
- Jails as built from the pinned revision exited with a runtime crash during the typing step in six of the eight workloads in this run (`jails`, `focus-main`, `gen-60k`, `gen-240k`, `large-25k`, `large-100k`; see below). The numbers above are from the patched build. The as-built results are in the reproduction files.

**The crash and the patch.** `reset_file` in Jails' `server/program.jai` calls `table_reset` on the file's declaration table before releasing the memory pool that table's entries were allocated from. After the next parse the table points into memory the pool has given back, and our `Pool` frees released blocks, so the memory is reused. With a debug build the crash is an assertion or a null dereference inside `table_add` while re-parsing after an edit. Replacing the `table_reset(*declarations);` line with a reset of the table's fields (so it allocates anew) removes the crash in a replay of the recorded session; no other change was made. We believe this is a lifetime issue in Jails' code rather than in jaic, but we did not check it against another implementation, so it is possible that the behaviour depends on the allocator in use.

### Reproducing

Jails is not modified in `corpus/upstream`; the copy below goes to a scratch directory (`cp -RL` follows the corpus' symlinked submodules):

```sh
J=/tmp/jails-build
mkdir -p $J && cp -RL corpus/upstream/SogoCZE--Jails/. $J/ && rm -rf $J/bin/*
(cd $J && /path/to/jaic build.jai - -release)      # writes $J/bin/jails
# For the patched build, replace `table_reset(*declarations);` in $J/server/program.jai with:
#   declarations.entries = .{}; declarations.count = 0; declarations.allocated = 0; declarations.slots_filled = 0; declarations.allocator = .{};

# modules directory Jails reads for completion: the stdlib of this repository
mkdir -p /tmp/jai-root && ln -sfn "$PWD/stdlib" /tmp/jai-root/modules

# jailsp (both diagnostic styles, committed in benchmarks/results/lsp-apple-m5.*)
python3 tools/lsp_bench.py --repeat 3 --out lsp.json --markdown lsp.md
# Jails
python3 tools/lsp_bench.py --server "jails=$J/bin/jails -jai_path /tmp/jai-root" \
    --only jails --diagnostics push --push-wait 3 --repeat 3 --out jails.json --markdown jails.md
```

Jails ran with no `jails.json` in the workloads (nothing was written into the corpus), so it parsed only the open documents and what they load. A `jails.json` with `roots` and `local_modules` may improve its results on projects with modules; that was not tried. No other process was compiling during the timed runs (runs were repeated until a window with no other build running). The jailsp rows are from `benchmarks/results/lsp-apple-m5.json`; the full cross-server runs are not committed.

## Documentation

The docs under `docs/` are the source of truth:

- [Shared Jai language server](../../docs/compiler/language-server.md): feature list, how semantic analysis, the compile cache and completion work, configuration and limits
- [Language server refactorings](../../docs/compiler/language-server-refactorings.md)
- [Language server benchmark](../../docs/tools/lsp-benchmark.md) and [latest results](../../benchmarks/results/lsp-apple-m5.md)
- [VS Code extension](../../docs/tools/vscode-extension.md), [jailint](../../docs/tools/jailint.md), [browser build](../../docs/browser/playground.md)
