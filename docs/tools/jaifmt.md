# jaifmt (Jai formatter)

## What it is

`jaifmt` is a code formatter for Jai, written in Jai. The formatter itself is the stdlib module `Jai_Format` (text in, text out, no file access), so the same code runs natively, under `jaic run` and in the browser playground; `tools/jaifmt/main.jai` is the command-line front end. It normalizes indentation, spacing and brace placement while keeping comments, blank lines and the line structure of the source. It refuses to write any output whose token stream differs from the input, so it cannot change what a program means.

```sh
target/debug/jaic build tools/jaifmt/main.jai -O2 -o target/jaifmt
target/jaifmt stdlib tests                 # rewrite in place
target/jaifmt --check stdlib tests         # CI: list files that would change, exit 1
target/jaifmt --stdin < in.jai > out.jai   # editor integration
```

Options: `--check`, `--stdin`, `--config <file>`, `--verbose`/`-v` (summary, files formatted, lines over `max_width`). Paths are files or directories (searched recursively for `*.jai`, skipping dot-directories and not following symlinked directories). Exit status: 0 success, 1 `--check` found files to change, 2 errors (unreadable files, input that does not lex, unbalanced brackets, a failed token check). `--check` prints `path:line` with the first line that would change.

Under the interpreter: `jaic run tools/jaifmt/main.jai -- --check "$PWD/stdlib"`. `jaic run` starts programs in the main file's directory, so pass absolute paths.

## How it works

`stdlib/Jai_Format/module.jai` holds the public API (below) and loads three files:

- `lexer.jai`: a tokenizer with the same token boundaries as `crates/jaic/src/lexer.rs` (same punctuation table, here-strings, `@` notes, `\` identifier separators, `.5` floats), but comments are tokens and every token records the whitespace before it (`ws_start`, `newlines`, `column`).
- `format.jai`: re-emits the tokens. It never splits a line. Each line's indentation and each gap between two tokens on a line is decided separately.
- `config.jai`: `jaifmt.toml` parsing and ignore globs (the caller reads the file).

`tools/jaifmt/main.jai` adds the file side: arguments, the directory walk, upward discovery of `jaifmt.toml` (cached per directory), `--check` reporting and writing files. `tools/jaifmt/playground.jai` is the browser's driver (see [Browser playground](#browser-playground)).

**Indentation** follows bracket nesting (a stack of `Frame`s):

- A `{` that ends its line opens a block: its lines are indented one level from the statement that opened it (so a multi-line procedure header still indents the body by one level), and the `}` returns to that level.
- Inside `(`, `[`, or a `{` with code after it on the same line, continuation lines keep their original offset from the line that opened the bracket. Aligned argument lists stay aligned and move with their statement.
- A line that does not start a statement (the previous code token is not `;`, `{`, `}`, a `#scope_*` directive or a note after `}`/`;`) also keeps its offset from the statement's first line. This covers braceless `if x` bodies, `#if` bodies, and expressions continued over lines.
- In `if x == {` (and `if #complete x == {`) blocks, `case` lines are indented by `case_indent` and the statements under a case by `case_body_indent` more. A comment line takes the indentation of the code that follows it.
- Block comments that start a line move as a unit: continuation lines shift by the same amount (dedenting only if every line has room).

**Spacing** between two tokens on a line is one of: none, one space, or the original whitespace. The rules (`spacing_rule`) are deliberately narrow:

| Pair | Result |
| --- | --- |
| before `,` `;` `)` `]`, after `(` `[` `.[` | none |
| after a directive and after its flags (`#library,system`, `#type, distinct`, `#location(x)`) | kept: the parser checks adjacency there |
| after `,` (not a cast flag comma like `cast,no_check`) and after `;` on the same line | one space |
| before `{`, before `//` | one space |
| after `{` / `.{`, before `}` | kept (`{a}` and `{ a }` both stay) |
| `x: int` | the colon attaches to declared names; a typed constant's second `:` keeps its spacing |
| `=` `:=` `::` `==` `!=` `<=` `>=` `&&` `\|\|` compound assignments `->` `=>` | spaced |
| `+ - * / % & \| ^ << >>` used as binary operators | tight on both sides stays tight (`n-1`, `a*b + c`), anything else is spaced |
| prefix `-x` `*T` `<<p` `!ok` | attached |
| `if(` `while(` `return(` ... | one space |
| everything else (`.`, `..`, `$T`, `xx`, after directives, after `}` ...) | kept |

Binary versus prefix is decided from the previous code token: an operand (identifier, literal, `)` of a call or group, `]` of an index, `.*`) means binary; an operator, keyword, `(`, `)` of `cast(T)`, or `]` of an array type (`[4] *int`) means prefix; `}`, directives and notes mean unknown, and the spacing is kept. Whenever a "space" is required and the source already has two or more spaces, those are kept: that is how alignment (`A   :: 1;`, `case .X;  stmt;`, aligned trailing comments) survives.

**Line structure**: trailing whitespace is removed, runs of blank lines are capped at `max_blank_lines`, leading blank lines are dropped and the file ends with exactly one newline. Files that mostly use CRLF keep CRLF. With `brace_style = "same_line"` two joins happen: a `{` alone on its line after a header (`if x` / `) -> int` / `else` / a directive) moves up, and `else` on the line after a `}` that closes a multi-line block moves up. Nothing else crosses a line.

**Verbatim regions**: string literals, here-strings (`#string TAG ... TAG`, body and terminator line byte for byte), the inside of `#asm { }`, and everything between `// jaifmt: off` and `// jaifmt: on` (or the end of file).

**Long lines** are not wrapped. `--verbose` counts lines over `max_width`.

**Safety check** (`check_equivalent`): after formatting, the output is lexed again and compared with the input: same tokens (kind and text) in the same order, the same comments (up to whitespace), and a line break before the same tokens, because the parser is line-sensitive in a few places (`}` then a line starting with `*`/`-` begins a new statement; a directive on the next line ends a declaration). The only allowed line-break difference is before a joined `{` or `else`. It also requires the same adjacency after directives and their flags (`directive_zone`): `#library,system` is a flag but `#library, system` is an argument, and `#location(x)` takes an operand that `#location (x)` does not. On a mismatch the file is left unchanged and jaifmt exits with 2 and an "internal error" naming the token.

## How to change it

- New spacing rule: add it to `spacing_rule` in `stdlib/Jai_Format/format.jai`, above the more general rules it should override, and add a line to `stdlib/Jai_Format/tests/cases/spacing.in.jai`. Prefer returning `.KEEP` when the right answer depends on context the formatter does not track.
- New indentation behavior: `line_indent` (where a line starts) and `track` (what a token does to the frame stack). `starts_statement` decides statement versus continuation lines.
- Lexer changes must mirror `crates/jaic/src/lexer.rs`; if the two disagree about where a token ends, the safety check can accept output that `jaic` lexes differently. Likewise, if the parser starts to depend on whitespace somewhere new (look for `newline_before` and `span.start == ...end` in `crates/jaic/src/parser/`), teach `check_equivalent` about it and make the formatter keep that spacing.
- The strongest test is semantic: format a scratch copy of `stdlib/`, `tests/` and `corpus/upstream` (point a worktree's `corpus/upstream` symlink at the copy) and run the sweep. This is how the directive-flag adjacency rule was found.
- Golden tests: `stdlib/Jai_Format/tests/cases/<name>.in.jai` must format to `<name>.out.jai` (with `<name>.toml` as config if present). Run `jaic run stdlib/Jai_Format/tests/golden.jai` (part of the sweep's `modules` set); after an intended change, regenerate with `jaic run stdlib/Jai_Format/tests/golden.jai -- --bless` and review the diff. The same test checks idempotence, the refusal of malformed input, the safety check, config parsing and globs, and that the formatter's own sources are formatted.
- `tests/stdlib/jai-format-api.jai` exercises the public API on in-memory text only, so it also runs in the playground's stdlib pass set (`tools/playground_stdlib_expected.json`). `node tools/check_jai_format_wasm.mjs <jai_wasm.wasm | staged-dir>` runs the browser driver through the wasm engine on every golden case (CI runs it on the debug wasm).
- Keep the module free of file access, threads and `#foreign` calls: the playground cannot run them.
- `crates/jaic-cli/tests/native.rs` (`jaifmt_builds_and_formats`) builds the tool natively and checks the CLI: `--stdin`, `--check`, in-place rewrites, ignore globs and exit codes.
- A file whose layout matters (generated tables, test fixtures with recorded positions): add it to `ignore`, or wrap the region in `// jaifmt: off` / `// jaifmt: on`.

## Configuration

`jaifmt.toml`, one `key = value` per line with `#` comments (a small TOML subset: integers, quoted strings and one-line string arrays; unknown keys and bad values are errors reported as `line N: ...`). The CLI uses the one in the file's directory or the nearest parent (per file, like rustfmt; `--config` overrides it for all files):

```toml
indent_width = 4          # 1-16
case_indent = 4           # `case` lines in `if x == {`; default indent_width
case_body_indent = 4      # statements under a case; default indent_width
max_blank_lines = 2       # consecutive blank lines kept
max_width = 100           # only reported by --verbose; lines are never wrapped
brace_style = "same_line" # or "preserve"
ignore = ["tests/corpus/**", "generated/*.jai"]
```

Ignore globs are relative to the config file's directory: `*` and `?` stay within a path component, `**` crosses components, and a glob that matches a directory ignores everything below it. Explicitly named files are ignored too. The repository's root `jaifmt.toml` ignores `tests/corpus/**` (fixtures pinned by `sha256` in `tests/corpus/manifest.json`, plus deliberately malformed negative cases) and `stdlib/Jai_Format/tests/cases/**` (golden inputs). The defaults match the dominant style of `stdlib/` and the public Jai modules: 4 spaces, braces on the same line, `case` one level in and its body one more.

**CI**: the `test` job in `.github/workflows/ci.yml` builds `jaifmt` with the debug `jaic` and runs `jaifmt --check stdlib tests benchmarks tools examples`. The step is advisory (`continue-on-error`, not in the enforcement step) until the repository has been formatted once; then add `steps.jai-format.outcome == 'failure'` to the final step's condition.

**Speed** (Apple M5, including the safety check and compiling the program where it applies):

| Run | Time |
| --- | --- |
| native `-O2`, ~650 files / 180k lines | 0.4 s (building the tool: about 1 s) |
| `jaic run` (release `jaic`), whole repository | 30 s |
| `jaic run`, one 327-line file / one 814-line file | 0.14 s / 0.21 s |
| browser engine (release wasm), 327-line file | 75 ms (a golden case: ~25 ms) |
| browser engine (debug wasm, as in CI), 327-line file | 375 ms |

## Module API

```jai
#import "Jai_Format";

config, ok, error := parse_config(toml_text);       // Format_Config, bool, string
formatted, ok, error := format_source(source, config); // config defaults to .{}
```

- `format_source(text, config = .{}) -> result: string, ok: bool, error: string`: on success `result` is newly allocated (`free` it); on failure it is `""` and `error` (temporary storage) says why, usually as `line:column: message`: text that does not lex, unbalanced brackets, or a failed token check. Input is never partially formatted.
- `parse_config(text) -> config: Format_Config, ok: bool, error: string`: parses `jaifmt.toml` text. `config.root` is left empty; set it to the config's directory if you use `format_is_ignored`.
- `Format_Config` fields: `indent_width` (4), `case_indent` and `case_body_indent` (-1: `indent_width`), `max_blank_lines` (2), `max_width` (100, reporting only), `brace_style` (`.SAME_LINE` or `.PRESERVE`), `ignore`, `root`.
- Also: `format_tokens_equivalent(before, after) -> ok, why` (the safety check), `format_is_ignored(config, path)`, `format_glob_match(pattern, text)`, `format_count_long_lines(text, max_width)`, `FORMAT_CONFIG_FILE_NAME` (`"jaifmt.toml"`).

## Browser playground

The playground runs `tools/jaifmt/playground.jai` in the wasm engine against its virtual `/workspace`. It formats `/workspace/main.jai` (the `TARGET` constant; replace that line to format another file), with the nearest `jaifmt.toml` between the file's directory and `/workspace`:

```js
const driver = await (await fetch("jaifmt-playground.jai")).text();   // tools/jaifmt/playground.jai
const files = { ...workspaceFiles, "__jaifmt__.jai": driver.replace(/^TARGET :: ".*";$/m, `TARGET :: ${JSON.stringify(path)};`) };
const result = engine.play(files, "__jaifmt__.jai");
if (result.exitCode === 0) editor.setText(result.stdout);   // the formatted file
else showError(result.stderr);                                // "jaifmt: main.jai:3:7: unbalanced ..."
```

Exit code 0 means stdout is the whole formatted file; 1 means stdout is empty and stderr has one `jaifmt: ...` line (bad config, unreadable file, or input that cannot be formatted safely). To let users configure it, create `/workspace/jaifmt.toml`, for example `indent_width = 2`. The page can instead call the module from its own driver: `#import "Jai_Format"` is bundled with the rest of `stdlib/`.

## Dependencies

`Jai_Format` uses only `Basic` and `String`. The CLI adds `File`, `File_Utilities`, `POSIX` (stdin/stdout/stderr descriptors) and `Sort`; the playground driver adds `File`. The browser path needs the wasm engine (`web/scripting-runtime/engine.mjs`, [`tools/build_scripting_wasm.py`](../../tools/build_scripting_wasm.py)). Running it under the interpreter needs `jaic run ... -- args` ([interpreter](../compiler/interpreter.md)). The token rules come from `crates/jaic/src/lexer.rs`; the line-sensitive parser spots from `crates/jaic/src/parser/expr.rs` and `decl.rs`.
