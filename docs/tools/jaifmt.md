# jaifmt (Jai formatter)

## What it is

`jaifmt` formats Jai source and is itself written in Jai. The formatter is the jaic extension module `Extensions/Jai_Format` (text in, text out, no file access), so the same code runs natively, under `jaic -interp` and in the browser; `jaifmt/main.jai` is the command-line front end. Output is canonical: it depends on the input's tokens, comments and blank lines, not its spacing. Lines are never re-wrapped. Output whose token stream differs from the input is refused, so formatting cannot change what a program means.

```sh
jaic jaifmt/build.jai                      # optimised build -> target/jaifmt (add `- wasm` for jaifmt.wasm)
target/jaifmt stdlib tests                 # rewrite in place
target/jaifmt --check stdlib tests         # CI: list files that would change, exit 1
target/jaifmt --stdin < in.jai > out.jai   # editors
jaic jaifmt/main.jai -interp -- --check "$PWD/stdlib"   # interpreter: use absolute paths
```

Flags are declared at the top of `jaifmt/main.jai` and parsed by [`Extensions/Args`](../extensions/args.md) (`--help` is generated from them). Paths are files or directories (searched for `*.jai`, skipping dot-directories and symlinked directories). Exit status: 0 success, 1 `--check` found files to change, 2 errors (usage, unreadable file, input that does not lex, unbalanced brackets, failed token check).

`jaifmt/build.jai` is a [metaprogram](../metaprogramming/compiler-module.md) building with `set_optimization(.OPTIMIZED)` (no bounds/null/cast checks). It needs real `jaic`, not `-check`. The default output is `target/` at the repository root; a relative `-o` is relative to `jaifmt/`. CI, Nix and the tests use `jaic jaifmt/main.jai -O2`, which keeps the checks.

## How it works

`stdlib/Extensions/Jai_Format/` holds:

- `lexer.jai`: same token boundaries as `crates/jaic/src/lexer.rs`, but comments are tokens and each token records the whitespace before it.
- `format.jai`: `plan_lines` decides line breaks (`breaks[i]`), then the emit loop writes each line with indentation from a stack of `Frame`s and zero or one space between tokens (`spacing_rule`).
- `config.jai`: `jaifmt.toml` parsing and ignore globs.

Behaviour in short:

- **Lines.** Non-empty `{ ... }` bodies go on their own lines; with `brace_style = "same_line"` a `{` on its own line joins its header (except after `;`, `{`, `}`, `,`, `(`, `#scope_*`, where it starts a block). `;`-separated statements are split, except one simple statement after a `case` label. Kept as written: `.{ }`/`.[ ]` literals, braceless bodies, continued expressions, trailing comments.
- **Blank lines.** Trailing whitespace removed, blank runs capped at `max_blank_lines`, one final newline, CRLF kept if mostly CRLF. Blank lines are only added where a break already exists, never removed, and comments stay attached to the item below.
  - `blank_lines_between_items` (default on): a blank between file-level items when either spans several lines or one is an import and the other is not; runs of imports and runs of one-line items stay together.
  - Opt-in code-block options (all off by default): `blank_line_after_declarations`, `blank_lines_around_blocks`, `blank_line_between_cases`, `blank_line_before_comments`, `blank_line_after_guards`, `blank_line_before_return`. Short bodies (four lines or fewer) and tiny single-line-body block runs are left compact.
- **Indentation** follows bracket nesting. Continuation lines inside brackets keep their offset from the bracket (aligned argument lists stay aligned); other continuation lines keep their offset from the statement's first line. This is the one place input layout still shows.
- **Spacing** is none or one space, with no alignment. `spacing_rule` (first match wins) follows the dominant style of `stdlib/`. Binary versus prefix is decided from the previous code token. `.PRESENCE` (keep the source's choice) is used only where the parser checks adjacency (`#library,system` versus `#library, system`, `#location(x)`).
- **Verbatim:** strings, here-strings, `#asm { }` bodies and everything between `// jaifmt: off` and `// jaifmt: on`.
- **Safety check** (`check_equivalent`): the output is re-lexed and must have the same tokens, comments and line breaks before the same tokens, because the parser is line-sensitive in a few places (`}` then a line starting with `*`, a directive on the next line, a note after `;`). Only breaks the parser ignores may differ (`break_change_allowed`). On mismatch the file is left unchanged, exit 2, "internal error" naming the token.

## Configuration

`jaifmt.toml` (small TOML subset: integers, quoted strings, one-line string arrays; unknown keys are errors). The nearest one up the tree applies per file; `--config` overrides.

```toml
indent_width = 4          # 1-16
case_indent = 4           # default indent_width
case_body_indent = 4      # default indent_width
max_blank_lines = 2
max_width = 100           # only reported by --verbose
brace_style = "same_line" # or "preserve"
blank_lines_between_items = true
blank_line_after_declarations = false   # plus the five other opt-in options above
ignore = ["tests/corpus/**", "generated/*.jai"]   # globs relative to the file; `**` crosses directories
```

The repository's config ignores pinned corpus fixtures, the formatter's golden inputs and `tests/compiler/native/debug-info/**` (fixed line numbers), and enables four of the opt-in blank-line options with `max_blank_lines = 1`. Colour follows `NO_COLOR`, `FORCE_COLOR`, `CLICOLOR_FORCE` as in jaic. CI builds jaifmt and runs `--check stdlib tests benchmarks tools jaifmt examples`. Format-only commits go in `.git-blame-ignore-revs` (see [development](development.md)).

## Module API

```jai
#import "Extensions/Jai_Format";
config, ok, error := parse_config(toml_text);
formatted, ok, error := format_source(source, config);   // free `formatted`; on failure it is ""
```

Also `format_tokens_equivalent`, `format_is_ignored(config, path)` (set `config.root` first), `format_glob_match`, `format_count_long_lines`. `Format_Config` fields mirror the TOML keys.

## Browser

- `jaifmt/playground.jai` runs in the wasm engine against `/workspace` (`TARGET` constant names the file); exit 0 means stdout is the formatted file, 1 means stderr has one `jaifmt: ...` line. `tools/build_scripting_wasm.py` stages it as `jaifmt-playground.jai` in every browser bundle.
- `jaifmt/wasm.jai` is a WASI command: `jaic jaifmt/wasm.jai -os wasm -O2 -no_debug_info -exe jaifmt.wasm`, run with `node --no-warnings tools/wasi_run.mjs jaifmt.wasm --config "indent_width = 2" < in.jai`. Source on stdin, config from `--config` or `JAIFMT_CONFIG`, same exit contract. It is about 35 times faster than the engine path but needs Memory64 (Chrome 133, Firefox 134, node 24, not Safari); pages fall back to the engine driver when `WebAssembly.validate` rejects it. Its hash is recorded as `jaifmt_wasm_sha256` in the release bundle's `build-metadata.json`. `node tools/check_jaifmt_wasm.mjs jaifmt.wasm target/jaifmt` and `tools/check_jai_format_wasm.mjs` compare it with native on every golden case.

## How to change it

- **Spacing:** add to `spacing_rule` above the rules it overrides, plus a line in `tests/cases/spacing.in.jai`. Return `.NONE` or `.SPACE`; `.PRESENCE` only where the parser looks at adjacency.
- **Blank-line rule for code blocks:** `statement_wants_blank` (one `if` per option), a field in `Format_Config`, a key in `parse_config_into`, and a `stmt-*` golden case. It may only call `add_blank_line`. File-level items: `separate_pair`, `finish_item`, `block_ends_item`, `is_item_container`.
- **Line structure:** `plan_lines`; any break it adds or removes must be accepted by `break_change_allowed`, which must accept only breaks the parser ignores.
- **Indentation:** `line_indent`, `track`, `starts_statement`.
- **Lexer:** mirror `crates/jaic/src/lexer.rs`. If the parser starts depending on whitespace somewhere new (look for `newline_before` in `crates/jaic/src/parser/`), teach `check_equivalent` and keep that spacing.
- **Golden tests:** `stdlib/Extensions/Jai_Format/tests/fixtures/<name>.in.jai` must format to `<name>.out.jai` (optional `<name>.toml`). Run `jaic stdlib/Extensions/Jai_Format/tests/golden.jai -interp`; regenerate with `-- --bless` and review the diff. `stdlib/Extensions/Jai_Format/tests/api.jai` covers the API in memory so it also runs in the browser engine. `crates/jaic-cli/tests/native.rs` covers the CLI and checks idempotence over the repository's Jai files.
- The strongest test is semantic: format a scratch copy of `stdlib/`, `tests/` and the upstream corpus and run the sweep.
- Layout-sensitive files: add to `ignore` or wrap in `// jaifmt: off`/`on`. Tests comparing `#location` lines must not rely on two statements sharing a line.
- Moving `jaifmt/`: CI, `nix/jaifmt.nix`, `tools/build_scripting_wasm.py`, `tools/build_pgo.py`, the two wasm check scripts, the jaic-cli tests and `SOURCES` in `tests/golden.jai` name its files.
- Keep the module free of file access, threads and `#foreign` calls: the playground cannot run them.

## Dependencies

`Jai_Format` uses `Basic` and `String` only. The CLI adds `File`, `File_Utilities`, `Sort` and `POSIX`/`Windows` for stdin. The browser path needs `crates/jai-wasm/js/engine.mjs`; `jaifmt.wasm` needs a native `jaic` with `wasm-ld` and `Wasi_Runtime`.
