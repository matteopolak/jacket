# jaifmt (Jai formatter)

`jaifmt` formats Jai source and is itself written in Jai. The formatter is the extension module `Extensions/Jai_Format` (text in, text out, no file access), so the same code runs natively, under `jaic -interp` and in the browser; `jaifmt/main.jai` is the command-line front end. Output depends on the input's tokens, comments and blank lines, not its spacing. Lines are never re-wrapped. Output whose token stream differs from the input is refused, so formatting can't change what a program means.

```sh
jacket fmt                                 # format the whole project
jacket fmt --check                         # CI: list files that would change, exit 1
jaic jaifmt/build.jai                      # build target/jaifmt (add `- wasm` for jaifmt.wasm)
target/jaifmt stdlib tests                 # rewrite in place
target/jaifmt --stdin < in.jai > out.jai   # editors
target/jaifmt --stdin --range 120:240 < in.jai > out.jai   # only the statements bytes 120..240 touch
```

`jacket fmt` runs `jaifmt` with the arguments as written; there is no other path. For each file `jaifmt` finds the nearest `jacket.toml` (read once per directory), validates the complete file with [`Extensions/Toml`](../extensions/toml.md), reads the `[format]` and `[doc.format]` schemas and drops files its `ignore` patterns match. With no path it formats the directory of the nearest `jacket.toml` to the working directory, else the working directory. `--no-config` skips the file and `--set KEY=VALUE` overrides one key.

Paths are files or directories (searched for `*.jai`, skipping dot-directories and symlinks). Flags are declared at the top of `jaifmt/main.jai` and parsed by [`Extensions/Args`](../extensions/args.md). Exit status: 0 success, 1 `--check` found files to change, 2 errors (usage, unreadable file, input that doesn't lex, failed token check). `--check` lists `path:line` on stdout and, when anything would change, a hint with the rules page (`RULES_URL` in `jaifmt/main.jai`) on stderr; `-v` links the `max-width` page next to long lines.

Files are formatted in parallel, one worker per logical core, or `-j N` workers (`-j 1` stays on one thread). Each file is read, formatted and written independently by `format_batch` (`jaifmt/batch.jai`); the main thread then reports the results in sorted path order, so output, totals and exit status are the same for any `-j`. Config files are resolved before the workers start, because the config cache and its warnings are not thread-safe. `jaifmt/tests/batch-order.jai` checks that a parallel run gives the same results as a serial one. The wasm build formats one input from stdin and uses no threads.

`jaifmt/build.jai` builds optimized and needs a real `jaic`, not `-check`. CI, Nix and the tests use `jaic jaifmt/main.jai -O2`, which keeps the checks.

[Format rules](jaifmt-rules.md) shows every rule and setting with the code before and after. Its ```` ```jai after ```` blocks are checked: the golden test (`Jai_Format/tests/golden.jai`) formats each ```` ```jai before ```` block with the settings named after `after` and fails if the output differs, so change the page when you change the output.

## How it works

`stdlib/Extensions/Jai_Format/` holds:

- `lexer.jai`: the same token boundaries as `crates/jaic-syntax/src/lexer.rs`, but comments are tokens and each token records the whitespace before it.
- `format.jai`: `plan_lines` decides line breaks, then the emit loop writes each line with indentation from a stack of frames and zero or one space between tokens (`spacing_rule`).
- `config.jai`: `[format]` parsing and ignore globs, for standalone use and the browser. `Format_Config` carries `@toml(...)` notes (range, error message, skipped fields) and the file is read with [typed TOML decoding](../extensions/toml-typed.md), so a new setting is one field with a note plus a line in `move_setting` and `FORMAT_KEYS`.
- `project.jai`: the upward search for the governing `jacket.toml`.
- `range.jai`: `--range` / `format_range`. The file is lexed and cut into top-level statements (ending at a `;`, at a `}` that closes a body without `;`, `else` or a note after it, or at a `#scope_*` line). Statements the byte range overlaps are formatted as a text of their own and spliced back; everything else is copied byte for byte. The region grows to take statements that share a line with it, and an empty range picks the statement it sits in. A range in no statement, or after a `// jaifmt: off`, changes nothing. Blank lines between the region and its neighbours are kept as written, and an error is reported against the region, not the file.

Behaviour in short:

- **Lines.** Non-empty `{ ... }` bodies go on their own lines; `brace-style = "same_line"` joins a lone `{` to its header. `;`-separated statements are split. Kept as written: `.{ }`/`.[ ]` literals, braceless bodies, continued expressions, trailing comments.
- **Blank lines.** Trailing whitespace removed, runs capped at `max-blank-lines`, one final newline. Blank lines are only added where a break already exists, never removed. `blank-lines-between-items` (default on) separates file-level items; the `blank_line_*` options are opt-in.
- **Indentation** follows bracket nesting; continuation lines keep their offset from the bracket or statement start.
- **Spacing** is none or one space, with no alignment, following the dominant style of `stdlib/`.
- **Verbatim:** strings, here-strings, `#asm { }` bodies and everything between `// jaifmt: off` and `// jaifmt: on`.
- **Doc examples** (`format-doc-examples`, `format_doc_examples_in`): after the main pass, the `jai` blocks in `//` comment runs are formatted as separate texts (a file if the block declares `main`, else a procedure body that is unwrapped again) and written back with the same prefix. `max-width` is reduced by the prefix width. A block is kept as written unless it formats, keeps its tokens (`check_equivalent`) and formats to itself; the file as a whole must keep its non-comment tokens (`same_code_tokens`), which catches fence-looking lines inside here-strings and block comments.
- **Doc example settings** (`[doc.format]`): `parse_config` checks each key like a `[format]` one and records it in `Format_Config.doc_format` (`key`, `value` as written); `doc_example_config` applies them in file order to a copy of the config, and `format_doc_examples_in` formats with that copy. Unknown keys are dropped and warned about as `doc.format.<key>`. The file parser, `--set`, the browser formatter and the language server all go through `parse_config` / `format_apply_setting`.
- **Safety check** (`check_equivalent`): the output is re-lexed and must have the same tokens, comments and line breaks before the same tokens (the parser is line-sensitive in a few places). On mismatch the file is left unchanged, exit 2, naming the token.

## Configuration

The `[format]` table of the nearest `jacket.toml` ([jacket](jacket.md)). `--config` overrides it. `jaifmt --is-ignored <file>` answers the ignore question (exit 0 ignored, 1 not, 2 error); the VS Code and Neovim plugins use it.

```toml
[format]
indent-width = 4          # 1-16
case-indent = 4           # default indent_width
case-body-indent = 4      # default indent_width
max-blank-lines = 2
max-width = 100           # only reported by --verbose
brace-style = "same_line" # or "preserve"
blank-lines-between-items = true
blank-line-after-declarations = false   # plus blank_lines_around_blocks, blank_line_between_cases,
                                        # blank_line_before_comments, blank_line_after_guards, blank_line_before_return
format-doc-examples = true              # format the jai blocks in // comments
ignore = ["tests/corpus/**", "generated/*.jai"]   # globs relative to the file

# Overrides for the examples in doc comments only; any [format] key but ignore and
# format_doc_examples. A key left out is inherited from [format].
[doc.format]
blank-line-after-declarations = false
```

Ignore globs follow the same rules as lint paths ([jailint](jailint.md#configuration)); `PathPattern` in `crates/jacket-config/src/pattern.rs` and `glob_path_match` in `Jai_Format/config.jai` both run `tests/glob/cases.txt`. Colour follows `NO_COLOR`, `FORCE_COLOR` and `CLICOLOR_FORCE`. CI runs `--check stdlib tests benchmarks tools jaifmt examples`.

## Module API

```jai
#import "Extensions/Jai_Format";
config, ok, error := parse_config(toml_text);
formatted, ok, error := format_source(source, config);   // free `formatted`; on failure it is ""
```

`format_range(source, from, to, config)` formats only the statements a byte range touches. Also `format_find_config`, `format_is_ignored` (set `config.root` first), `format_tokens_equivalent` and `format_glob_match`. `Format_Config` fields mirror the TOML keys.

## Browser

- `jaifmt/playground.jai` runs in the wasm engine; `tools/browser/build_scripting_wasm.py` stages it in every browser bundle.
- `jaifmt/wasm.jai` is a WASI command: `jaic jaifmt/wasm.jai -os wasm -O2 -no_debug_info -exe jaifmt.wasm`, run with `node --no-warnings tools/wasi_run.ts jaifmt.wasm --config-text $'[format]\nindent_width = 2' < in.jai`. It is much faster than the engine path but needs Memory64 (Chrome 133, Firefox 134, node 24, not Safari); pages fall back to the engine otherwise. `node tools/browser/check_jaifmt_wasm.ts wasi jaifmt.wasm target/jaifmt` compares it with native.

## How to change it

- **Spacing:** add to `spacing_rule` above the rules it overrides, plus a line in `tests/cases/spacing.in.jai`.
- **A blank-line option:** `statement_wants_blank`, a field in `Format_Config`, an `apply_key` arm and `FORMAT_KEYS` entry, the matching entry in `FORMAT_SETTINGS` and `Format` in `crates/jacket-config/src/schema.rs` (the sync test `jai_format_sync.rs` fails until they agree), and a `stmt-*` golden case.
- **Line structure:** `plan_lines`; any break it adds or removes must be accepted by `break_change_allowed`, which may accept only breaks the parser ignores.
- **Indentation:** `line_indent`, `track`, `starts_statement`.
- **Lexer:** mirror `crates/jaic-syntax/src/lexer.rs` (`crates/jaic/tests/formatter_lexer.rs` fails on drift). If the parser starts depending on whitespace somewhere new, teach `check_equivalent`.
- **Golden tests:** `Jai_Format/tests/fixtures/<name>.in.jai` must format to `<name>.out.jai`. Run `jaic stdlib/Extensions/Jai_Format/tests/golden.jai -interp`; regenerate with `-- --bless` and review the diff.
- Layout-sensitive files: add to `ignore` or wrap in `// jaifmt: off`/`on`.
- Moving `jaifmt/`: CI, `nix/jaifmt.nix`, `tools/browser/build_scripting_wasm.py`, `tools/dev/build_pgo.py`, the wasm check scripts and the jaic-cli tests name its files.
- Keep the module free of file access, threads and `#foreign` calls: the playground can't run them.

## Dependencies

`Jai_Format` uses `Basic` and `String` only. The CLI adds `File`, `File_Utilities`, `Sort` and `POSIX`/`Windows`. `jaifmt.wasm` needs a native `jaic` with `wasm-ld` and `Wasi_Runtime`.

## TOML validation

Standalone and browser configuration parsing requires a complete valid TOML 1.1 document, including unrelated sections; malformed statements, repeated keys and dotted-key or table redefinitions are line-specific errors. Unknown valid settings are ignored and warned about. Settings consume decoded values, so integer spellings such as `indent-width = 0x2`, escaped strings and TOML 1.1 syntax work, and `--set` converts the same way. `[doc.format]` values keep their value spans and source order. File discovery and ignore-glob matching stay in the formatter adapter. The Rust `jacket-config` parser is an independent implementation.
