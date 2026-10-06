# jaifmt (Jai formatter)

## What it is

`jaifmt` is a code formatter for Jai, written in Jai. The formatter itself is the stdlib module `Jai_Format` (text in, text out, no file access), so the same code runs natively, under `jaic run` and in the browser playground; `jaifmt/main.jai` is the command-line front end. It produces canonical output, like rustfmt: one statement per line, block bodies on their own lines, braces joined to their headers, computed indentation and exactly zero or one space between tokens, whatever the input's spacing. Comments, blank lines (up to `max_blank_lines`) and the breaks inside expressions are kept; lines are not re-wrapped. It refuses to write any output whose token stream differs from the input, so it cannot change what a program means.

```sh
target/debug/jaic build jaifmt/build.jai   # or: jaic build jaifmt/main.jai -O2 -o target/jaifmt
target/jaifmt stdlib tests                 # rewrite in place
target/jaifmt --check stdlib tests         # CI: list files that would change, exit 1
target/jaifmt --stdin < in.jai > out.jai   # editor integration
```

Options: `--check`, `--stdin`, `--config <file>`, `--verbose`/`-v` (summary, files formatted, lines over `max_width`), `--color auto|always|never` (`auto` colours on a terminal unless `NO_COLOR` is set; `FORCE_COLOR`/`CLICOLOR_FORCE` turn it on), `--help`/`-h`. Paths are files or directories (searched recursively for `*.jai`, skipping dot-directories and not following symlinked directories). Exit status: 0 success, 1 `--check` found files to change, 2 errors (unreadable files, input that does not lex, unbalanced brackets, a failed token check). `--check` prints `path:line` with the first line that would change; paths inside the current directory are shown relative to it, as in its error messages.

Errors use the compilers' layout, a lowercase message and a `help:` line saying what to do; paths are relative to the current directory:

```
src/bad.jai:1:20: error: unbalanced `}`
help: the file was left unchanged; jaifmt formats only code whose brackets balance
error: `missing.jai` does not exist
error: unknown option `--chek`
help: `jaifmt --help` lists the options
```

A bad `jaifmt.toml` is reported as ``in `path`, line N: ...`` with the fix.

Under the interpreter: `jaic run jaifmt/main.jai -- --check "$PWD/stdlib"`. `jaic run` starts programs in the main file's directory, so pass absolute paths.

### Building with the metaprogram (`jaifmt/build.jai`)

The program lives in the top-level `jaifmt/` directory (`main.jai`, the browser drivers `playground.jai` and `wasm.jai`, and `build.jai`); `tools/` is for repository-maintenance scripts. `build.jai` is a [Compiler module](../metaprogramming/compiler-module.md) metaprogram that builds an optimised jaifmt in a workspace of its own:

```sh
jaic build jaifmt/build.jai                     # target/jaifmt (native, optimised, no debug info)
jaic build jaifmt/build.jai - wasm              # target/jaifmt.wasm (WASI, same as -os wasm)
jaic build jaifmt/build.jai - -o /abs/path/fmt  # choose the output file
```

- It must be `jaic build` (or `jaic run`): `jaic check` has no output backend, so the workspace is only type-checked ([workspaces](../metaprogramming/workspaces.md#output)).
- The default output is `target/` at the repository root (anchored on `#filepath`), wherever `jaic` was started. A relative `-o` path is relative to `jaifmt/`, because `jaic`, like `jai`, runs from the main file's directory. The default is not `jaifmt` at the root because that is the source directory.
- `set_optimization(.OPTIMIZED)` means `-O2` code with bounds, null and cast checks off, as a shipping build. The plain `jaic build jaifmt/main.jai -O2` (used by CI, the Nix package and the tests) keeps the checks. Both pass the token-equivalence check, which is what guards the output.
- The wasm build sets `os_target = .WASM`, `cpu_target = .CUSTOM` and the triple `wasm64-unknown-wasi`, which makes `jaic` link `Wasi_Runtime` exactly as `-os wasm` does.

## How it works

The output is canonical: it depends on the tokens, comments and blank lines of the input, not on how the input was spaced. Two inputs that differ only in spacing between tokens, in where braces and `else` sit, or in how many statements share a line format to the same text (like rustfmt). Formatting is idempotent.

`stdlib/Jai_Format/module.jai` holds the public API (below) and loads three files:

- `lexer.jai`: a tokenizer with the same token boundaries as `crates/jaic/src/lexer.rs` (same punctuation table, here-strings, `@` notes, `\` identifier separators, `.5` floats), but comments are tokens and every token records the whitespace before it (`ws_start`, `newlines`, `column`).
- `format.jai`: two passes over the tokens. `plan_lines` decides the line structure (`breaks[i]`: line breaks before token i). The emit loop then writes each line: indentation from a stack of `Frame`s, and zero or one space between neighbors (`spacing_rule`).
- `config.jai`: `jaifmt.toml` parsing and ignore globs (the caller reads the file).

`jaifmt/main.jai` adds the file side: arguments, the directory walk, upward discovery of `jaifmt.toml` (cached per directory), `--check` reporting and writing files. `jaifmt/playground.jai` is the browser engine's driver and `jaifmt/wasm.jai` the WASI driver compiled to `jaifmt.wasm` (see [Browser playground](#browser-playground)).

**Line structure** (`plan_lines`). The source's line breaks are the starting point; then:

- With `brace_style = "same_line"`, a `{` on its own line joins the code before it, whatever the header is: `main :: ()`, `-> int`, `struct`, `enum u8`, `if s == "a"`, `if x ==` (a switch), `for arr`, `else`, `#run`, `#if X`, a note, blank lines in between. Not after `;`, `{`, `}`, `,`, `(`, `#scope_*`, `#load`/`#import`: there the `{` starts a block statement. `else` on a line after `}` joins it (`} else {`).
- Every non-empty `{ ... }` (procedure bodies, struct/enum bodies, control flow, `#code`, `#run` and procedure literals in expressions) has its body on lines of its own: a break after `{` and before `}`. `{}` stays, and `{` `}` with only whitespace between become `{}`. A statement written after a block's `}` on the same line moves down (`{ a(); } b();`).
- Statements separated by `;` inside a block (or at file level) go on separate lines: `x:=1;   if x>0 {...}` becomes two statements. Exception: a `case` label keeps one simple statement after it when written on the same line (`case .A; return 1;`, `case 2; #through;`), the common Jai idiom; `case 2; a(); b();` is split.
- Unchanged: `.{ }` and `.[ ]` literals keep their line structure, braceless bodies (`if x return;`), `#run f()` without a block, expressions continued over lines, trailing comments (`x := 1; // why`). A `{` with a comment between it and its header stays on its own line (joining would reorder the comment).
- Trailing whitespace is removed, runs of blank lines are capped at `max_blank_lines`, leading blank lines are dropped and the file ends with exactly one newline. Files that mostly use CRLF keep CRLF.

**Indentation** follows bracket nesting:

- A `{` that ends its output line opens a block: its lines are indented one level from the statement that opened it (so a multi-line procedure header still indents the body by one level), and the `}` returns to that level.
- Inside `(`, `[`, or a `.{ }` literal spanning lines, a continuation line written further right than the opening bracket keeps its offset from the bracket (aligned argument lists stay aligned even when spacing earlier on the line changes); otherwise it keeps its offset from the line that holds the bracket.
- A line that does not start a statement (the previous code token is not `;`, `{`, `}`, a `#scope_*` directive or a note after `}`/`;`) keeps its offset from the statement's first line, but is never left of it. This covers braceless `if x` bodies, `#if` bodies, and expressions continued over lines.
- In `if x == {` (and `if #complete x == {`) blocks, `case` lines are indented by `case_indent` and the statements under a case by `case_body_indent` more. A comment line takes the indentation of the code that follows it.
- Block comments that start a line move as a unit: continuation lines shift by the same amount (dedenting only if every line has room).

These continuation offsets are the one place where the input's layout still shows in the output: lines are never re-wrapped, so a hand-broken argument list keeps its shape.

**Spacing** between two tokens on a line is exactly none or one space. There is no alignment: runs of spaces are not kept, aligned `::` constants and aligned trailing comments are not preserved. The rules (`spacing_rule`, first match wins), chosen to match the dominant style of `corpus/upstream` and `stdlib/`:

| Pair | Result |
| --- | --- |
| before a line comment (`x := 1; // c`) | one space |
| after a directive and inside its flags (`#library,system`, `#type, distinct`, `#location(x)`), and between `operator` and `::` | one space if the source had any whitespace, else none: the parser checks adjacency there |
| before `,` `;`, after `(` `[` `.[`, before `)` `]` | none: `f(a, b)`, `.[1, 2, 3]` |
| after `,` (not a cast flag comma `cast,no_check`, not `,,` context arguments) and after `;` on a line | one space |
| `.{` literals | spaced inside: `.{ x = 1, y = 2 }`; empty `.{}` |
| `{` literals without the dot | like `.{`: `f({ .A, 1 })`, `return { w, h };`. A `{` after `(`, `,`, `=`, `:=`, `[` or `return` that holds no `;` is a literal (`opens_dotless_literal`); any other `{` is a block |
| `{` `}` | one space around (`if x {`); `{}` when empty |
| `x: int`, `N: int : 5` | the first `:` of a declaration attaches to its names, a second `:` is spaced |
| `=` `:=` `::` `==` `!=` `<=` `>=` `&&` `\|\|` compound assignments `->` `=>` | spaced |
| `/ % \| ^ >> <<< >>>`, and `+ - * & << < >` used as binary operators | spaced: `a * b`, `n - 1`, `x << 2` |
| prefix `-x` `+x` `*T` `<<p` `!ok` `~x` `$T` `` `x ``, `**T` | attached |
| `$T/Interface` | tight |
| `a.b`, `x.*`, `Vec.{`, `int.[`, `0..n`, `..rest`, `[..]` | tight; `.A` after an operator or keyword is spaced (`x == .A`, `return .{}`) |
| `foo(`, `a[i]`, `cast(T)`, `size_of(T)`, `struct(T: Type)` | tight |
| `if (` `while (` `return (` `xx (` `case (` and other keywords before `(` / `[` | one space |
| after `cast(T)` and after an array type `[4] int`, `[..] *u8`, `[] u8` | one space |
| `for < i: 0..n` | spaced |
| everything else (words, literals, notes, directives) | one space |

Binary versus prefix is decided from the previous code token: an operand (identifier, literal, `)` of a call or group, `]` of an index, `}` of a `.{ }` literal, `.*`, value directives like `#line`) means binary; an operator, keyword, `(`, `)` of `cast(T)`, `]` of an array type, the `}` of a block or another directive means prefix; after a note it is unknown and whether the source had a space is kept. A "none" that would glue two tokens into a different one (`- -x` would lex as `---`, `a / *p` as a comment, a number followed by `.`) becomes one space (`would_merge`).

**Verbatim regions**: string literals, here-strings (`#string TAG ... TAG`, body and terminator line byte for byte), the inside of `#asm { }`, and everything between `// jaifmt: off` and `// jaifmt: on` (or the end of file).

**Long lines** are not wrapped. `--verbose` counts lines over `max_width`.

**Safety check** (`check_equivalent`): after formatting, the output is lexed again and compared with the input: same tokens (kind and text) in the same order, the same comments (up to whitespace), and a line break before the same tokens, because the parser is line-sensitive in a few places (`}` then a line starting with `*`/`-`/`+`/`<<` begins a new statement; a directive on the next line ends a declaration's flags; a note after `;` attaches to the declaration). Line breaks may differ only where the parser never looks (`break_change_allowed`): after a block's `{`, after `;` (except before a note), before `}`, between `}` and a word (`} else`, a statement after a block), after a note, and before a `{` joined to its header (removal only). It also requires the same adjacency after directives and their flags (`directive_zone`): `#library,system` is a flag but `#library, system` is an argument, and `#location(x)` takes an operand that `#location (x)` does not. On a mismatch the file is left unchanged and jaifmt exits with 2 and an "internal error" naming the token.

## How to change it

- New spacing rule: add it to `spacing_rule` in `stdlib/Jai_Format/format.jai`, above the more general rules it should override, and add a line to `stdlib/Jai_Format/tests/cases/spacing.in.jai`. Return `.PRESENCE` only where the parser itself looks at adjacency; everything else must be `.NONE` or `.SPACE` so the output stays canonical.
- New line-structure rule: `plan_lines` (where breaks are added or removed). Any break it adds or removes must be one `break_change_allowed` accepts, and that function must only accept breaks the parser ignores.
- New indentation behavior: `line_indent` (where a line starts) and `track` (what a token does to the frame stack). `starts_statement` decides statement versus continuation lines.
- Lexer changes must mirror `crates/jaic/src/lexer.rs`; if the two disagree about where a token ends, the safety check can accept output that `jaic` lexes differently. Likewise, if the parser starts to depend on whitespace somewhere new (look for `newline_before` and `span.start == ...end` in `crates/jaic/src/parser/`), teach `check_equivalent` about it and make the formatter keep that spacing.
- The strongest test is semantic: format a scratch copy of `stdlib/`, `tests/` and `corpus/upstream` (point a worktree's `corpus/upstream` symlink at the copy) and run the sweep. This is how the directive-flag adjacency rule was found.
- Golden tests: `stdlib/Jai_Format/tests/cases/<name>.in.jai` must format to `<name>.out.jai` (with `<name>.toml` as config if present). Run `jaic run stdlib/Jai_Format/tests/golden.jai` (part of the sweep's `modules` set); after an intended change, regenerate with `jaic run stdlib/Jai_Format/tests/golden.jai -- --bless` and review the diff. The same test checks idempotence, the refusal of malformed input, the safety check, config parsing and globs, and that the formatter's own sources are formatted.
- `tests/stdlib/jai-format-api.jai` exercises the public API on in-memory text only, so it runs unchanged in the browser engine (`tools/check_playground_stdlib.mjs`). `node tools/check_jai_format_wasm.mjs <jai_wasm.wasm | staged-dir>` runs the browser driver through the wasm engine on every golden case (CI runs it on the debug wasm).
- Moving or renaming `jaifmt/`: CI (`ci.yml`), `nix/jaifmt.nix`, `tools/build_scripting_wasm.py` (and its test), `tools/build_pgo.py`, `tools/check_jaifmt_wasm.mjs`, `tools/check_jai_format_wasm.mjs`, the jaic-cli tests and the last entry of `SOURCES` in `stdlib/Jai_Format/tests/golden.jai` all name its files. The browser bundle's names (`jaifmt-playground.jai`, `jaifmt.wasm`) are what pages fetch, so they do not follow the source path.
- Keep the module free of file access, threads and `#foreign` calls: the playground cannot run them.
- `crates/jaic-cli/tests/native.rs` (`jaifmt_builds_and_formats`) builds the tool natively and checks the CLI: `--stdin`, `--check`, in-place rewrites, ignore globs and exit codes. `jaifmt_build_metaprogram` (and `jaifmt_wasm_from_the_build_metaprogram` in `tests/wasm_target`) builds it through `jaifmt/build.jai`. `jaifmt_is_idempotent_on_the_repository` formats a copy of every `.jai` file under `stdlib/`, `tests/` (minus `tests/corpus`), `tools/`, `jaifmt/`, `benchmarks/` and `examples/` and checks that a second run changes nothing.
- A file whose layout matters (generated tables, test fixtures with recorded line numbers such as `tests/native/debug-info`): add it to `ignore`, or wrap the region in `// jaifmt: off` / `// jaifmt: on`. Tests that compare `#location` lines must not rely on two statements sharing a line, since they will be split.

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

Ignore globs are relative to the config file's directory: `*` and `?` stay within a path component, `**` crosses components, and a glob that matches a directory ignores everything below it. Explicitly named files are ignored too. The repository's root `jaifmt.toml` ignores `tests/corpus/**` (fixtures pinned by `sha256` in `tests/corpus/manifest.json`, plus deliberately malformed negative cases), `stdlib/Jai_Format/tests/cases/**` (golden inputs) and `tests/native/debug-info/**` (breakpoints at fixed line numbers). The defaults match the dominant style of `stdlib/` and `corpus/upstream`: 4 spaces, braces on the same line, `case` one level in and its body one more.

**CI**: the `test` job in `.github/workflows/ci.yml` builds `jaifmt` with the debug `jaic` and runs `jaifmt --check prelude stdlib tests benchmarks tools jaifmt examples`. Like the other checks it is recorded with `continue-on-error` and enforced by the job's last step, so an unformatted file fails CI. Format-only commits go in `.git-blame-ignore-revs`.

**Speed** (Apple M5, including the safety check and compiling the program where it applies):

| Run | Time |
| --- | --- |
| native `-O2`, ~650 files / 180k lines | 0.4 s (building the tool: about 1 s) |
| `jaic run` (release `jaic`), whole repository | 30 s |
| `jaic run`, one 327-line file / one 814-line file | 0.14 s / 0.21 s |
| browser engine (release wasm), 327-line file | 75 ms (a golden case: ~25 ms) |
| browser engine (debug wasm, as in CI), 327-line file | 375 ms |
| `jaifmt.wasm` under node 24, 327-line file / a golden case | 2.2 ms / 0.7 ms (a fresh instance each run; compiling the module: under 1 ms) |

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

The [hosted playground](https://matteopolak.com/playground/jai) runs `jaifmt/playground.jai` in the wasm engine against its virtual `/workspace`. It formats `/workspace/main.jai` (the `TARGET` constant; replace that line to format another file), with the nearest `jaifmt.toml` between the file's directory and `/workspace`:

```js
const driver = await (await fetch("jaifmt-playground.jai")).text();   // jaifmt/playground.jai
const files = { ...workspaceFiles, "__jaifmt__.jai": driver.replace(/^TARGET :: ".*";$/m, `TARGET :: ${JSON.stringify(path)};`) };
const result = engine.play(files, "__jaifmt__.jai");
if (result.exitCode === 0) editor.setText(result.stdout);   // the formatted file
else showError(result.stderr);                                // "jaifmt: main.jai:3:7: unbalanced ..."
```

`tools/build_scripting_wasm.py` stages the driver as `jaifmt-playground.jai` next to `jai_wasm.wasm`, so it is part of every browser release bundle (`tools/package_browser_release.py`, [browser compiler](../browser/playground.md)); the portfolio serves it from `/jai/<commit>/jaifmt-playground.jai`. Exit code 0 means stdout is the whole formatted file; 1 means stdout is empty and stderr has one `jaifmt: ...` line (bad config, unreadable file, or input that cannot be formatted safely). To let users configure it, create `/workspace/jaifmt.toml`, for example `indent_width = 2`. The page can instead call the module from its own driver: `#import "Jai_Format"` is bundled with the rest of `stdlib/`.

### WebAssembly build (`jaifmt.wasm`)

`jaifmt/wasm.jai` is jaifmt as a WASI command, compiled by a native `jaic` ([wasm target](../native/wasm-target.md)):

```sh
jaic build jaifmt/wasm.jai -os wasm -O2 --no-debug-info -o jaifmt.wasm
node --no-warnings tools/wasi_run.mjs jaifmt.wasm --config "indent_width = 2" < in.jai > out.jai
node --no-warnings tools/check_jaifmt_wasm.mjs jaifmt.wasm target/jaifmt   # golden cases, byte-identical to native
```

It reads the source from stdin and writes the formatted file to stdout with exit 0. The `jaifmt.toml` text comes from `--config <text>`, else the `JAIFMT_CONFIG` environment variable, else the defaults. `--name <file>` sets the name used in messages (default `main.jai`). On any error stdout is empty, stderr has one `jaifmt: ...` line and the exit status is 1, the same contract as the engine driver.

It is about 35 times faster than interpreting `jaifmt-playground.jai` in the engine, and does not need the engine loaded:

| | `jaifmt-playground.jai` in the engine (release) | `jaifmt.wasm` |
| --- | --- | --- |
| download | `jai_wasm.wasm` (shared with the playground) | 214 KB (61 KB gzip, 48 KB brotli) |
| 327-line file | 75 ms | 2.2 ms |
| golden case | ~25 ms | 0.7 ms |
| runtimes | any wasm32 browser | Memory64: Chrome 133, Firefox 134, node 24; not Safari yet |

(Apple M5, LLVM 23, `-O2 --no-debug-info`, no `wasm-opt`; node 24.12.)

`tools/build_scripting_wasm.py --jaic <native jaic>` stages it as `jaifmt.wasm` and records `jaifmt_wasm_sha256` in `build-metadata.json`; release bundles require it ([compiler releases](../browser/compiler-releases.md)). To use it, a page:

1. Fetches `/jai/<commit>/jaifmt.wasm` and compiles it once (`WebAssembly.compile`).
2. Per run, instantiates it with a WASI preview 1 shim providing `fd_read`, `fd_write`, `args_sizes_get`, `args_get`, `environ_sizes_get`, `environ_get`, `clock_time_get` and `proc_exit` from `wasi_snapshot_preview1`. Every pointer argument is a 32-bit offset (`i32`) into the module's 64-bit memory, so index the memory's `ArrayBuffer` with `Number(ptr)`; `memory.buffer` must be read again after calls, since it can grow.
3. Passes `["jaifmt.wasm", "--config", tomlText]` as arguments, serves the source to `fd_read(0, ...)`, collects `fd_write(1/2, ...)`, and calls `_start`. `proc_exit(code)` ends the run: throw from the shim and catch it around `_start`.
4. On exit 0 replaces the editor text with stdout; on 1 shows stderr.
5. Where `WebAssembly.validate` rejects the module (no Memory64, Safari today), falls back to `jaifmt-playground.jai` in the engine.

## Dependencies

`Jai_Format` uses only `Basic` and `String`. The CLI adds `File`, `File_Utilities` and `Sort`, plus `POSIX` (or `Windows` on Windows) to read standard input; it keeps paths with forward slashes, so it runs on Windows too (release archives ship `jaifmt.exe`); the playground driver adds `File`. The browser path needs the wasm engine (`crates/jai-wasm/js/engine.mjs`, [`tools/build_scripting_wasm.py`](../../tools/build_scripting_wasm.py)), or, for `jaifmt.wasm`, a native `jaic` with `wasm-ld` to build it, `Wasi_Runtime`, and a Memory64 runtime. Running it under the interpreter needs `jaic run ... -- args` ([interpreter](../compiler/interpreter.md)). The token rules come from `crates/jaic/src/lexer.rs`; the line-sensitive parser spots from `crates/jaic/src/parser/expr.rs` and `decl.rs`.
