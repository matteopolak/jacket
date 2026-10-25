# Jai for Visual Studio Code

> [!NOTE]
> This project is unofficial and is not affiliated with or endorsed by Thekla, Inc. or the official Jai project.

Language support for Jai from [Jacket](https://github.com/matteopolak/jacket), the toolchain behind `jaic`: the `jailsp` language server, `jailint` lints and fixes, `jaifmt` formatting and syntax highlighting.

![Hover over a struct: its fields, size and alignment](https://raw.githubusercontent.com/matteopolak/jacket/main/editors/vscode/images/hover.png)

## Features

- **Errors as you type** from the real compiler, with hover (types, values, memory layout), completion, go to definition, references, rename, inlay hints, signature help, code lenses, `#insert`/`#run`/macro expansions and *Add `#import`* fixes.

  ![A type mismatch reported inline](https://raw.githubusercontent.com/matteopolak/jacket/main/editors/vscode/images/error.png)

- **Completion** of members, procedures, modules and directives, with auto-import: `prin` offers `print` and adds `#import "Basic";` when accepted (likewise for your own modules, and `#load` for your project's other files; `jai.completion.autoImport`). A `jacket.toml` at the project root can name its entry files and module folders (`[project]`).

  ![Completing the fields of a struct](https://raw.githubusercontent.com/matteopolak/jacket/main/editors/vscode/images/completion.png)

- **Inline assembly help**: inside `#asm` blocks, completion and hover for every instruction jaic accepts, with its operand forms and the CPU feature it needs, and signature help for its operands.

  ![Completing an AVX2 instruction in an #asm block](https://raw.githubusercontent.com/matteopolak/jacket/main/editors/vscode/images/asm-completion.png)

- **Refactorings** in the lightbulb menu: extract the selected expression into a variable, extract statements into a procedure (parameters and results worked out; declined when it cannot be done safely), inline a variable, add the missing cases of an `if x == {` on an enum, add the missing fields of a `Type.{...}` literal, and convert between `ifx` and `if`/`else`. **Call hierarchy** (incoming and outgoing calls) and **expand selection** work too.
- **Lints** from `jailint` with quick fixes; `source.fixAll.jailint` applies every safe fix, on demand or on save. Unsaved edits to `jacket.toml` apply at once.

  ![A jailint warning and its documentation link](https://raw.githubusercontent.com/matteopolak/jacket/main/editors/vscode/images/lint.png)

  ![The quick fix for it](https://raw.githubusercontent.com/matteopolak/jacket/main/editors/vscode/images/quick-fix.png)

- **Formatting** with `jaifmt`, honouring the `[format]` table of `jacket.toml` (including its `ignore` globs); works with `editor.formatOnSave`.

  ![A file before and after Format Document](https://raw.githubusercontent.com/matteopolak/jacket/main/editors/vscode/images/formatting.png)

- **Doc comments**: the comment above a declaration is shown as Markdown in hover, completion and signature help, with per-parameter docs (`- name: text` in the comment) and `[print]` / `[Type.field]` / `[text](print)` links you can click or go to the definition of. The Markdown in comments is highlighted, and fenced code in them as Jai.
- **Highlighting**: a TextMate grammar for keywords, directives, here-strings, numbers (`0x`, `0b`, `0h`, `_`), nested comments, declarations and `$T`/`$$x` polymorph variables, refined by the server's semantic tokens.
- **Embedded languages**: a here-string whose terminator names a language is highlighted as that language: `#string WGSL`, `GLSL`, `HLSL`, `SQL`, `JSON`, `HTML`, `CSS`, `JS`, `TS`, `PY`, `SH`, `C`, `CPP`, `RUST`, `YAML`, `TOML`, `JAI` and more (ignoring case). A WGSL grammar is bundled; GLSL and Metal need an extension that provides their grammar. Other terminators (`END`) stay plain strings.
- **Commands**: *Jai: Run File*, *Build File* and *Check File* (`jaic file.jai [-interp|-check]` in a terminal), *Restart Language Server*, *Download Toolchain*.
- **`jacket.toml` completion, hover docs and validation** with [Even Better TOML](https://marketplace.visualstudio.com/items?itemName=tamasfe.even-better-toml) installed: this extension ships a schema for `jacket.toml`, and Even Better TOML picks them up automatically.
- Snippets, bracket and comment configuration, and a file icon.

## Getting the toolchain

The extension looks for `jailsp` in `jai.server.path`, then on `PATH`, then next to `jai.compiler.path` or the `jaic` on `PATH`. If none is found it **asks** before downloading the compiler release pinned into this extension (`jaic`, `jailsp`, `jailint`, `jaifmt` and the standard library) from [github.com/matteopolak/jacket/releases](https://github.com/matteopolak/jacket/releases), and checks it against SHA-256 checksums pinned into the extension. Set `jai.toolchain.autoDownload` to `never` to turn this off, or `always` to skip the question. Prebuilt toolchains exist for macOS, Linux and Windows, each on x86-64 and arm64; elsewhere, build from source and set the paths.

## Settings

```jsonc
{
  "jai.server.path": "",            // jailsp
  "jai.compiler.path": "",          // jaic (Run/Build/Check)
  "jai.formatter.path": "",         // jaifmt
  "jai.stdlib.path": "",            // passed as JAIC_STDLIB
  "jai.stdlib.prelude": "",         // JAIC_PRELUDE; relative to stdlib, or absolute
  "jai.toolchain.autoDownload": "prompt",
  "jai.run.arguments": [],          // extra jaic arguments, such as "-O2"
  "jai.completion.autoImport": true, // offer names from modules and files not imported yet
  "jai.completion.callSnippets": "addParentheses", // or "fillArguments" / "none"
  "jai.inlayHints.closingBraceMinLines": 25, // "// name" after a long procedure's "}"; 0 turns it off
  "[jai]": {
    "editor.formatOnSave": true,
    "editor.codeActionsOnSave": { "source.fixAll.jailint": "explicit" }
  }
}
```

See [the extension's documentation](https://github.com/matteopolak/jacket/blob/main/docs/tools/vscode-extension.md) for how it works and how to change it.

The prelude initializes built-in types and context before each program. `jai.stdlib.prelude` overrides detection (`prelude/module.jai` by default); `~` expands to the home folder. Empty settings inherit `JAIC_PRELUDE`. See [prelude selection](../../docs/compiler/prelude-selection.md).
