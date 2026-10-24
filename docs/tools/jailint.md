# jailint (Jai linter)

## What it is

`jailint` reports common mistakes and unidiomatic code in Jai programs and fixes the ones it can. Rules run on the program after `jaic` has type-checked it, so they know each expression's type and what each name refers to. `jailsp` shows the same findings in editors as diagnostics with quick fixes.

```sh
cargo build -p jailint --release
jacket lint src/                  # lint every .jai file under src/
jacket lint main.jai --fix        # apply the safe fixes
jacket lint -D warnings stdlib    # fail on any finding (CI)
jacket lint --list                # rules and default levels
```

`jacket lint` runs `jailint` with exactly these arguments (see [jacket](jacket.md)); the standalone `jailint` binary takes the same ones.

Other options (see `--help`): `-A`/`-W`/`-D <rule>` (allow, warn, deny; `all`; `-D warnings`), `--config`, `--max-nesting <n>` (blocks `excessive_nesting` allows), `-I <dir>`, `-j <n>` (programs compiled at once; default one per CPU, at most 8), `--add <source>` (extra program-scope source, like jaic), `--color`, `-v`. Exit status: 0 clean, 1 a denied finding or incomplete compilation (not with `--allow-incomplete`, for a tree whose platform-specific modules only compile on their own OS; it prints how many programs did not compile completely, and `-v` lists them), 2 usage or configuration error. Output goes through the shared `jaic::render`, so `--color`, `NO_COLOR` and `JAIC_DIAGNOSTICS` apply as in the compiler.

## How it works

1. **Plan** (`driver.rs`). Paths become `.jai` files minus `exclude`. `module.jai` and files directly in an import directory are modules, compiled by importing them from an empty program so every procedure is checked. A file that a module (or a nearby program) `#load`s is compiled as part of that module or program (`enclosing_module`, `enclosing_program`); otherwise `unused_parameter` would flag a module's exported interface. Only the listed files are reported.
2. **Compile** with lint facts on (`IdeFacts::lint`): per-expression types and constness, cast types, every entity a name resolved to, and which import answered each lookup. Compile-time code runs in the sandboxed host with a block budget; workspaces a metaprogram creates are linted too (`jaic::build::WorkspaceObserver`).
3. **Lint** (`lib.rs`). Re-parse, build suppressions, collect procedure bodies and whether every instance checked fully, then run each enabled rule. Where the compiler did not look (an `#if` branch left out, a failed body) rules fall back to text and stay quiet.
4. **Report.** `--fix` applies non-overlapping machine-applicable fixes and re-lints, up to 5 passes (`fix.rs`). The language server's `source.fixAll.jailint` applies one pass. A file reached from several roots is reported once.

Editors: `jailsp` lints each open document using the cached compile that hover uses. Diagnostics carry the rule as `code` and a link to its `### <rule>` section below, and `deny` rules are errors. Settings come from the `[lint]` table of the nearest `jacket.toml`, else `jailint.toml` (the browser build sends it as an open document). See [the language server](../compiler/language-server.md#lints-and-quick-fixes).

## Target checking

```sh
jacket lint -os windows -cpu arm64 -D warnings path/to/module
jacket lint -target wasm64-unknown-wasi -D warnings path/to/module
jacket lint -os windows --manifest target/windows-lint.json path/to/module
jacket lint --context app/main.jai path/to/parameterized-module
```

`-os linux|windows|macos|wasm`, `-cpu x64|arm64` and `-target <triple>` select real compiler options; they do not rewrite OS/CPU constants. Windows from another OS defaults to the GNU ABI unless a `*-pc-windows-msvc` triple is given. Linker and SDK availability are not checked.

`--context <program>` checks a module inside a real enclosing program (so module parameters are real) while reporting only the requested paths; an absent requested file fails the check. `--manifest <file>` writes JSON listing each compiler context (OS/CPU, `complete`, `visited_files`), `unvisited_files` (target-inactive conditional loads) and incomplete roots with their first error. A file visited in an incomplete context is not coverage; run one invocation per target and context group and merge the manifests. Do not lint a parameter-required module as an unbound root.

## Declaration notes

Several rules decide what a call means from a note on the procedure's declaration, never from its name. Write the note after the body (`} @UseResult`) or, for a bodiless declaration, after the semicolon. A call counts only when every procedure the callee resolved to carries the note, so notes work the same on your own procedures as on the library's, and a procedure that merely shares a name with a library one is left alone. The library's `Basic`, `String` and `Math` procedures are annotated where they are declared (`stdlib/internal/`).

| Note | Rule | Meaning |
| --- | --- | --- |
| `@UseResult` | `unused_result` | calling it as a statement is a mistake: it only computes its result and leaves its arguments alone |
| `@ReturnsSmaller` | `min_max` | returns the smaller of its two arguments (the library's `min`) |
| `@ReturnsLarger` | `min_max` | returns the larger of its two arguments (the library's `max`) |
| `@RemovesFromArray` | `remove_in_for` | removes elements from the array its first argument points to |
| `@FreesArgument` | `defer_in_loop`, `use_after_free`, `double_free` | after the call the *first* argument's memory or resource is gone: freed, closed or destroyed (`free`, `array_free`, `array_reset`, closers and `deinit`s). Not for a procedure whose first argument is only an allocator, heap or owner |
| `@Allocates` | `leaked_allocation` | the result is newly allocated and the caller owns it (`alloc`, `New`, `copy_string`, `sprint`, `join`, ...) |
| `@TakesByteCount` | `element_count_as_bytes` | the last argument is a number of bytes in the memory the pointer arguments point at (`memcpy`, `memset`, `memcmp`) |
| `@Locks` | `unpaired_lock` | takes the lock its first argument names (`lock(*mutex)`); a procedure with this note is a wrapper and is not checked itself |
| `@Unlocks` | `unpaired_lock` | gives back the lock its first argument names |
| `@AssertsArgument` | `assert_constant` | stops the program unless its first argument is true (the library's `assert`) |

```jai
shout :: (s: string) -> string {
    return to_upper_copy(s);
} @UseResult
```

To change what a rule recognises, annotate the declaration; `jailint` has no name lists, so adding a note needs no change to the linter.

## Rules

Every rule belongs to one category, and the category sets its default level, as in clippy:

| Category | Default | What |
| --- | --- | --- |
| `correctness` | deny | code that is wrong: it does not do what it says |
| `suspicious` | warn | code that is most likely wrong, or does nothing |
| `complexity` | warn | code that says something in a roundabout way |
| `style` | warn | code that works but is not written the usual way |
| `perf` | warn | code that could be faster |
| `pedantic` | allow | stricter checks with more false positives |
| `restriction` | allow | checks that forbid things a project may choose to avoid |

A category name works wherever a rule name does: `[lint.rules] pedantic = "warn"`, `-W pedantic`, `// jailint: allow(suspicious)`. In `[lint.rules]` a rule's own entry wins over its category's; on the command line later flags win (`-W pedantic -A magic_number`). `deny` findings are errors: the run exits 1 and editors show them as errors. `jailint --list` prints each rule's category and default.

Columns: `yes` means `--fix` applies the fix; `editor` means the fix may change behaviour, so it is only a quick fix in the editor; `no` means none. "Side-effect-free" below means names, member paths, literals, indexing, casts and operators: no calls. Rules that compare expressions compare them as written, ignoring whitespace. A rule is removed once `jaic` rejects what it found; a `jailint.toml` naming a removed rule fails with `unknown rule`.

Each rule's source (`crates/jailint/src/rules/<rule>.rs`) opens with a doc comment saying exactly when it fires, and `tests/lint/<rule>/` has `bad.jai` (+ `bad.expected`, `bad.fixed.jai`) and `good.jai`. The `every_rule_is_documented` test requires a table row and a `### <rule>` section here, because editors link to the anchor.

| Rule | Category | Default | Fix | Finds |
| --- | --- | --- | --- | --- |
| `absurd_comparison` | correctness | deny | editor | `u >= 0`, `u < 0` with an unsigned `u`: always true or false |
| `almost_swapped` | correctness | deny | editor | `a = b; b = a;`, a swap that sets both to `b` |
| `approx_constant` | style | warn | editor | `3.14159`, `6.2832`, `1.4142`: a hand-typed approximation of a constant `Math` has (`PI`, `TAU`, `SQRT2`, ...) |
| `assert_constant` | style | warn | yes | `assert(true, "...")`, `#assert true;`: an assertion whose condition is the literal `true`. `assert(false, ...)` is a deliberate failure and is not reported |
| `banner_comment` | style | warn | yes | a comment that is only a line of `-`, `=` or `*` fencing off a section |
| `bitwise_precedence` | suspicious | warn | yes | `1 << n - 1`, `flags \| 1 << 3`: bitwise operators that group unlike C |
| `bool_comparison` | complexity | warn | yes | `x == true`, `x != false`, `false == x` |
| `broken_doc_link` | style | warn | editor | a doc-comment link (`[prnt]`, `[Type.fild]`) that names no declaration |
| `cognitive_complexity` | pedantic | allow | no | a procedure whose branching, weighted by how deeply it nests, scores above `max_complexity` (default 25) |
| `constant_condition` | suspicious | warn | no | an `if` or `while` whose condition is always true or always false, by the compiler's value ranges |
| `defer_frees_returned` | correctness | deny | editor | `defer free(p);` then `return p;`: the caller gets memory the `defer` already released |
| `defer_in_loop` | suspicious | warn | no | a `defer` in a loop body that cleans up something from outside the loop |
| `double_free` | correctness | deny | editor | `free(p); free(p);`: a variable freed twice by the same procedure with nothing in between |
| `dropped_result` | suspicious | warn | editor | `n := to_integer(s);`: a call that keeps fewer values than it returns and whose next value is a `bool`, usually "did it work" |
| `duplicate_condition` | suspicious | warn | editor | an `else if` that repeats an earlier condition and can never run |
| `element_count_as_bytes` | correctness | deny | yes | `memcpy(d, xs.data, xs.count)` with `xs: [] s32`: an element count passed where a `@TakesByteCount` procedure wants bytes |
| `erasing_op` | suspicious | warn | editor | `x * 0`, `x & 0`, `x % 1`: always `0` |
| `excessive_nesting` | pedantic | allow | no | blocks nested deeper than `max_nesting` (default 6) |
| `float_equality` | pedantic | allow | no | `==` or `!=` between two computed floats |
| `identical_branches` | suspicious | warn | editor | `if c { A } else { A }`, `ifx c then a else a` |
| `identical_case_bodies` | pedantic | allow | editor | `case .PNG; return "image"; case .JPEG; return "image";`: cases that do the same thing and could be `case .PNG, .JPEG;` |
| `identical_operands` | suspicious | warn | no | `a == a`, `x - x`, `ok && ok` |
| `identity_op` | complexity | warn | yes | `x + 0`, `x * 1`, `x / 1` |
| `impossible_comparison` | correctness | deny | editor | `x > 5 && x < 3`, `x != 1 \|\| x != 2`: comparisons of one integer that never pass together, or always pass between them |
| `index_only_loop` | complexity | warn | yes | `for i: 0..xs.count-1` where `i` only indexes the array or string `xs` |
| `infinite_loop` | suspicious | warn | no | a `while` whose condition nothing in the loop changes |
| `int_plus_one` | complexity | warn | editor | `x >= y + 1`, `x - 1 >= y`, `x + 1 <= y`, `x <= y - 1`: an integer comparison with `1` added or taken off, which is a strict comparison |
| `integer_division_in_float` | suspicious | warn | editor | `cast(float)(a / b)`, `1 / 2 * w`: an integer quotient used as a float |
| `leaked_allocation` | pedantic | allow | editor | `p := New(T);` that is never freed, returned or handed on |
| `loop_with_break_condition` | complexity | warn | yes | `while true { if i >= n break; ... }`: a loop that opens by leaving when a test fails, which is the `while` condition |
| `lossy_xx` | pedantic | allow | yes | `xx` that narrows a number to a smaller type |
| `magic_number` | pedantic | allow | no | `if x > 37`, `y * 1.0472`, `timeout = 8640`: an unexplained number where a named constant would read better |
| `manual_assign_op` | complexity | warn | yes | `a = a + b` to `a += b` |
| `manual_clamp` | complexity | warn | editor | `max(0, min(100, x))`, `if x < 0 { x = 0; } else if x > 100 { x = 100; }`, `ifx v < lo then lo else ifx v > hi then hi else v`: a hand-written `clamp` |
| `manual_index_counter` | complexity | warn | yes | a counter kept next to a `for` loop that always equals `it_index` |
| `min_max` | correctness | deny | editor | `min(0, max(100, x))`: a clamp with swapped bounds |
| `misrefactored_assign_op` | correctness | deny | editor | `total += total + x;`: a compound assignment whose right side repeats the target and the operator, which was probably `total = total + x;` rewritten halfway |
| `modified_range_bound` | suspicious | warn | no | `for 0..n-1 { ...; n -= 1; }`: the range was fixed before the first iteration, so the change does not reach the loop |
| `needless_bool` | complexity | warn | yes | `ifx c then true else false`, `if c return true; else return false;` |
| `neg_multiply` | style | warn | yes | `x * -1`, `-1 * x`, `f * -1.0`: a multiplication by minus one, which is a negation |
| `no_effect` | suspicious | warn | editor | `x == 5;`, `count + 1;`, `flush;`: a statement that does nothing |
| `range_past_count` | correctness | deny | editor | `for i: 0..xs.count` indexing `xs[i]`: one past the end |
| `redundant_alias` | complexity | warn | yes | a local only copied into another local (`a := f(); b := a;`) or returned right away |
| `redundant_cast` | complexity | warn | yes | a cast to the type the value already has |
| `remove_in_for` | correctness | deny | editor | `array_*_remove_*` on the array a `for` is walking |
| `returned_local_address` | correctness | deny | no | `return *x;`, `return *x.field;`, `return buffer.data;` with `x` or `buffer` a local: a pointer into a stack frame that is gone |
| `reversed_range` | correctness | deny | editor | `for i: 10..0`: a range that never runs |
| `self_assignment` | suspicious | warn | editor | `x = x;` |
| `shadowed_it` | suspicious | warn | no | a nested `for` hides an `it` the enclosing loop still uses |
| `single_case_switch` | style | warn | editor | `if mode == { case .VERBOSE; ...; }`, with at most a default case: an `if` written as a `switch` |
| `string_literal_write` | correctness | deny | editor | `s := "abc"; s[0] = x;`: a write into a string literal, which is read-only |
| `uninit_read` | correctness | deny | editor | `total: int = ---; for xs total += it;`: a variable read before anything is written to it |
| `unknown_allow` | suspicious | warn | editor | a `// jailint: allow(...)` comment naming a rule that does not exist |
| `unpaired_lock` | suspicious | warn | editor | `lock(*m);` with a `return`, or the end of the procedure, reached before the matching unlock |
| `unused_import` | style | warn | yes | an `#import` nothing in its scope uses |
| `unused_parameter` | style | warn | editor | a parameter the procedure never uses |
| `unused_result` | suspicious | warn | editor | `trim(line);`: a call to a `@UseResult` procedure, as a statement |
| `unused_variable` | style | warn | yes | a local variable that is never used |
| `use_after_free` | correctness | deny | no | `free(p); print("%", p.x);`: a variable read after a `@FreesArgument` call freed it |
| `wrapping_constant` | correctness | deny | editor | `(0xffff_ffff - 40) / h`, `h < 0x8000_0000` with `h: s32`: a constant that wraps |
| `zero_ptr` | style | warn | editor | `cast(*u8) 0`: a null pointer written as a cast of zero |

### absurd_comparison
An unsigned value is never below zero, so counting down an unsigned `i` with `while i >= 0` never stops. Only the bottom of a type is checked: aliases like `c_ulong` change width per platform.
### almost_swapped
Two consecutive statements with side-effect-free sides of the same type. A real swap is `a, b = b, a;`. The editor offers the swap as a quick fix (not applied by `--fix`; it skips a pair with a comment between).
### approx_constant
Needs at least four significant digits (`3.14` is a rough value on purpose) and a match where the literal stops, rounded or cut off. Knows π, 2π, π/2, π/4, √2 and √½. Constant declarations are left alone. A test that checks a value on purpose takes `// jailint: allow(approx_constant)`. The editor offers the replacement (`PI64` family for a `float64` literal, a quotient in parentheses); the `Math` import is yours to add.
### assert_constant
The call must be to a procedure with the `@AssertsArgument` note (the library's `assert`). Macros are skipped. The fix removes the statement when it stands in a block and its other arguments have no side effects.
### banner_comment
A comment of three or more of `-=*#~_` and nothing else, line or block (`// ------`). Name the section in one plain `// Names` line instead, or split the file. Rust and Python files are held to the same rule by `tools/ci/check_source_style.py`. Generated files are skipped. `--fix` removes the line; a section name on a line of its own stays.
### bitwise_precedence
Jai gives all bitwise and shift operators one level above `*`, grouped left to right, so `1 << n - 1` is `(1 << n) - 1`. The fix adds the parentheses Jai already implies; move them if the C reading was meant.
### bool_comparison
Only when the other side is a non-constant `bool`.
### broken_doc_link
Only comments directly above a declaration (or trailing a field or parameter) are read, and links resolve like the language server's. Whether bracketed text counts as a link is decided by `jaic::docs` (see [doc comments](../compiler/doc-comments.md)): `[x]`, `[TODO]`, `a[i]`, plain lowercase words and URL targets are not. A single close misspelling is offered as a quick fix. `// jailint: allow(broken_doc_link)` inside the comment silences it.
### cognitive_complexity
Off by default. Scores how hard a procedure is to follow. Each `if`, `while`, `for`, `switch` and `ifx` adds 1, plus 1 for every one of those it sits inside, so nested branching costs more than the same branching in sequence. Each `else if` and `else` adds 1 with no nesting cost, each run of `&&` or `||` adds 1 (`a && b && c` is one, `a && b || c` two), and each labelled `break` or `continue` adds 1. `#if` and `#run` add nothing, and a procedure written inside a body is scored on its own. A procedure scoring above `[lint] max_complexity` (default 25) is reported on its name, with its score. On the standard library, 342 procedures score above 25 (the median of those is 39). Fix by splitting it into smaller procedures, returning early, or naming parts of long conditions. Generated files are skipped.
### constant_condition
Reads the compiler's reachability facts (`jaic::sema::reachability` and `jaic::ide::values`, the code the editor fades): reports the condition behind a branch that never runs, such as `if count < 5` inside `if count > 10`, or an `else` after a test that always holds. No analysis of its own, so it is as sure as the fading: unsure means silent. Compile-time constants (`if DEBUG`, `while true`) and `&&`/`||` with one as an operand (`if flag && TARGET_OS != .WINDOWS`, constant only on some targets), tests inside code that is dead anyway, and comparisons a type's range decides (left to `absurd_comparison`) are skipped.
### defer_frees_returned
The `defer` must be a single `@FreesArgument` call on a local. Quiet when anything after it assigns the variable or takes its address (`p = null;` hands ownership on), and in macros. Returning a field (`return name.count;`) is fine: it is read before the `defer` runs. The editor offers to remove the `defer`.
### defer_in_loop
The loop body's scope ends every iteration, so `defer free(buffer)` frees an outer buffer after the first pass. Fires only for calls of `@FreesArgument` procedures (the library's `free`, `array_free`, `deinit`s, closers and destroyers carry it) on values declared before the loop and not used earlier in the body.
### double_free
The same walk as `use_after_free`: a statement-level `@FreesArgument` call on a local or parameter, then, as the next thing that mentions it in the block, a statement-level call of the *same* procedure on it (`free(p); free(p);`, `close(f); close(f);`). Calls of different procedures are not paired, because `deinit(p); free(p);` releases the contents and then the memory. Silent when anything else mentions the variable in between, when a `return`/`break`/`continue` comes first, for a `defer` or a free nested in a branch, and for the cases listed under `use_after_free`. The editor offers to remove the second free; `--fix` does not, since the first one may be the wrong call.
### dropped_result
A declaration or assignment whose right side is one call that returns more values than the left side names, where the first dropped value, the one right after the last kept, is a `bool`: `value := parse(text);` quietly turns a failed parse into `0`. Receive the flag (`value, ok := parse(text);`) or write `value, _ := parse(text);` to say it is ignored on purpose. A success or found flag sits right after the value it describes, so a `bool` that comes later, after other results (status extras of a wide API such as `pressed, state, released := button(...)`), and dropped values of other types (counts, remainders) are fine, as are `_ := f();`, calls used as statements, compound assignments and calls whose procedure type is not known. Calls through procedure values count. To make dropping a result a hard compile error, mark it `#must` where the procedure is declared (see [directives and notes](../language/directives-and-notes.md)). The editor offers to append `, _` (declarations without a type, and assignments).
### duplicate_condition
Conditions must be side-effect-free; a call in between restarts the comparison. A repeated `case` value is not checked here: the compiler rejects it. The editor offers to remove the dead `else if` branch (not when a comment sits in it).
### element_count_as_bytes
Only when `xs.data` is another argument of the same call, so the count clearly describes that array. Elements of one byte (`u8`, `s8`, `bool`) and strings are left alone. The fix appends `* size_of(Elem)`; it is not applied by `--fix`, since the element type may need its module's name.
### erasing_op
Left alone inside an index or array literal, where a `0 *` lines up with other rows. When the other operand is side-effect-free the editor offers to write the `0`.
### excessive_nesting
Off by default. A level is a braced block or a `switch`; `else if` chains and `#if` add none. A procedure written inside a block starts counting from that block's depth, so a nested procedure cannot hide the nesting. Reports the first block past the limit on each path, on its `{` (or the `switch` header). The limit is `[lint] max_nesting` in `jacket.toml` or `--max-nesting <n>`, default 6; enable it with `-W excessive_nesting` or `[lint.rules]`. On the standard library, a limit of 6 finds nothing and 5 finds five blocks. Fix by returning or `continue`ing early, or by moving the inner part into a procedure. Generated files are skipped.
### float_equality
Off by default: exact comparison is often deliberate. Constants, `x != x` and `operator ==` bodies are not reported.
### identical_branches
Compared by tokens. Empty branches, `#if`, and the last link of an `else if` chain are left alone. When the condition is side-effect-free, the editor offers to keep one copy of the block (or the value of an `ifx`) and drop the test.
### identical_case_bodies
Off by default, since one case per row is sometimes the clearer table. Compares the tokens of a case's body with those of every earlier case of the same `switch`; default cases, empty bodies, `#through` cases and `#if` switches are skipped. The editor merges the later case into the earlier one's value list; `--fix` leaves it.
### identical_operands
Float `==`, `!=` and `-` are left alone (NaN tests), as are operator procedures.
### identity_op
`0 + x`, operations inside indexes/array literals and shifts by `0` are left alone (they line up with neighbours).
### impossible_comparison
Only integer expressions compared with integer literals, within one run of `&&` or `||`. Floats are left alone (`x > 1 && x < 2` holds for `1.5`). Redundant pairs such as `x < 3 && x < 5` are not reported. For a lone pair of comparisons the editor offers to swap `&&` and `||`, when the swapped test is not impossible too.
### index_only_loop
Becomes `for xs` (or `for *xs` when elements are written), other uses of `i` become `it_index`. Quiet when the loop may change `xs`, walks a grid, indexes other arrays or `xs[i + 1]`, or when `xs` is a constant. When the rewrite would make `it` name an inner loop's element, the fix is withheld and a named iterator is suggested.
### infinite_loop
Condition reads only locals, parameters and literals; skipped for bodies with `break`, `return`, `#insert`, `#asm`, `using`, macro calls or taken addresses.
### int_plus_one
Integers only, and only the literal `1`. The editor offers the strict comparison; `--fix` leaves it, since where `y + 1` wraps (the top of the type) the two forms differ.
### integer_division_in_float
Operands are typed before the cast, so the integers divide first. A variable divided by a power-of-two literal is taken as deliberate. The editor offers to convert the dividend to a float (`cast(float) done / total`); `--fix` does not, since it changes the value.
### leaked_allocation
Off by default: ownership conventions differ, and a program that allocates once for its whole run may leak on purpose. A local declared from a call of an `@Allocates` procedure (the library's `alloc`, `New`, `NewArray`, `copy_string`, `sprint`, `join`, `array_copy`, `to_upper_copy`, `read_entire_file`, ...; a cast around the call is fine) whose every later mention is harmless: reading or writing a number, bool or enum through it (`p.count`, `p.depth = 1`), comparing it, testing it in an `if`/`while`, or a bare statement. Passing it (or anything reached through it) to any call, freeing it (also by `defer`), returning it, assigning it elsewhere, taking its address, putting it in a literal or reassigning it all count as handling the memory and keep the rule silent. It also stays silent when it cannot tell which allocator is in force: a procedure that uses `push_context`, assigns `context`, calls a macro (`push_allocator` is one) or has `#insert`/`using`, an allocating call with `,, allocator = temp`, or a name mentioned where the walk cannot follow. For a pointer, the editor offers `defer free(x);` on the next line.
### loop_with_break_condition
A `while true` (macros aside) whose body starts with an `if` without an `else` that only does an unlabelled `break`, with or without braces. The fix puts the opposite test in the header (`!c` becomes `c`, `a >= b` on numbers becomes `a < b`) and removes the `if`; a comment inside the `if`, or before it in the loop, withholds it.
### lossy_xx
Off by default: narrowing with `xx` is common Jai style. The fix spells out `cast(T)`.
### magic_number
Off by default. Reports a decimal literal written directly as an operand of `+ - * / %` or a comparison, or as the value of an assignment (`x = 37;`, `x += 37;`). Arguments, indexes, array and struct literals, `return` values, `case` labels and declarations (`x := 37;` names the number already) are left alone, as is anything inside a constant declaration, a type (`[48] u8`), a `cast`, `#run` or `#asm`. Bitwise and shift operators are skipped: masks and shift counts are their own idiom. Never reported: whole numbers below `magic_number_min` in magnitude (default 3, so `-2` to `2`), powers of two, all-ones masks (`255`, `1023`), hex, octal and binary literals, floats with an allowed whole value (`2.0`) or a fraction in eighths (`0.5`, `0.25`, `1.5`), `10`, `100`, `1000`, and anything in `magic_number_allow`. Tune it under `[lint]`:

```toml
[lint]
magic_number_allow = [60, 24, 3.14159]   # never reported (negatives too)
magic_number_min = 10                    # whole numbers below 10 are fine
magic_number_in_tests = true             # also check files in `tests`/`test` folders

[lint.rules]
magic_number = "warn"
```

Files in a `tests` or `test` folder are skipped unless `magic_number_in_tests` is set, since test data is full of expected values. A printable character, tab, newline or carriage-return code compared with a byte of a `string` (`text[i]`, a local set from one, or `it` in `for text`) gets a `--fix` to `#char "a"`. In `<`, `<=`, `>` and `>=` only range ends are rewritten (digits, `a z A Z f F`, space, `~`), so `header[2] > 88` in a binary buffer stays a number. Other bytes may be table entries, so in their `==` and `!=` the help only suggests `#char`. Generated files are skipped. On the standard library outside its tests, the defaults report about 1,400 numbers: mostly character codes, calendar arithmetic and file-format offsets. Fix by naming the number: `RETRY_AFTER :: 8640;`.
### manual_assign_op
Target must be side-effect-free and the operation must keep its type.
### manual_clamp
Numbers with constant bounds, the lower not above the upper (a swapped pair is `min_max`'s). Three shapes: nested `@ReturnsLarger`/`@ReturnsSmaller` calls (the library's `max` and `min`), an `if` and `else if` that assign a side-effect-free target its bounds, and a nested `ifx`. The editor offers `clamp(x, lo, hi)`; `--fix` leaves it, since a statement inside an `else if` chain would need braces.
### manual_index_counter
`n := 0` right before a forward `for` (in a block, a `case` body or a `#if` branch), `n += 1` at the end of the body (or `defer n += 1`), `n` unused after the loop.
### min_max
Literal bounds only, with `@ReturnsSmaller`/`@ReturnsLarger` procedures (the library's `min`/`max`). Use `clamp`. The editor offers `clamp(value, lo, hi)` with the two bounds in order.
### misrefactored_assign_op
The same operator on both sides, a side-effect-free target of a number type spelled the same in both places (`a op= a op b`), and for `+ * & | ^` on integers also `a op= b op a`. The editor offers `a op= b`; `--fix` leaves it, since the other reading (`a = a op b`) is as likely to be the one meant.
### modified_range_bound
Only locals named directly in a range end (`n` in `0..n-1`); `xs.count` bounds are left alone. Quiet when a `break` or `return` follows the assignment, or the statement holding it, in the same block (`n = it; break;` records a result).
### needless_bool
A trailing `if c return true; return false;` is left alone when an earlier guard in the block also returns a literal. Comments inside the `if` withhold the fix.
### neg_multiply
Signed integers and floats only, and only the literals `-1` and `-1.0`. The fix writes `-x`, and `-(a + b)` when the other side is not a single term.
### no_effect
An expression statement containing no call. The editor offers the likely intent: `==` becomes `=`, `x + 1;` becomes `x += 1;`, a procedure name gets `()`, and any other statement is removed. `--fix` applies none, since the typo may be in either place.
### range_past_count
Ranges include their end. Quiet when the body mentions `xs.count` (it may guard the index). The `- 1` fix changes behaviour, so it is editor-only.
### redundant_alias
Style rule, deliberately narrow so the fix never changes behaviour: the two statements are adjacent with nothing (not even a comment) between, the first name appears exactly twice in the body, the types are identical, and no `#insert`, `using`, notes or flags are involved. Parameters, loop variables, globals, macros and generated files are skipped.
### redundant_cast
Aliases like `c_long` differ per platform, so the value must be declared with the same type spelling. Polymorphic bodies and macros are skipped.
### remove_in_for
The loop steps past the element moved into the removed slot. Use `remove it;`. Fires on `@RemovesFromArray` procedures (`Basic`'s `array_*_remove_*`). Quiet for backwards loops and when `break`/`return` follows. When the call is a statement removing the element being visited (`array_unordered_remove_by_index(*xs, it_index);`), the editor offers `remove it;`. It is not applied by `--fix`, since the call may keep the order and `remove` does not.
### returned_local_address
Only variables declared in the body: parameters are left alone. A path that goes through a pointer, view or resizable array (`*p.field` with `p: *T`) is fine, and so is indexing a struct with an `operator *[]`.
### reversed_range
Counting down is `for < i: 0..10`. The fix changes behaviour, so it is editor-only.
### self_assignment
Not auto-fixed: the slip may be on the other side. After `using info;`, `width = width;` assigns a field to itself; names found only through `using` that also exist as earlier locals are not reported.
### shadowed_it
`row := it;` before the inner loop is the usual idiom and is not reported.
### single_case_switch
One case of one value, plus at most a default case. `#complete` switches, `#through` cases, cases listing several values and `#if` switches are skipped. The editor offers `if x == v { ... }` (with the default as the `else`); the bodies keep their text, so `jaifmt` fixes the indentation, and `--fix` leaves it.
### string_literal_write
Literals are read-only and equal literals share storage, so the write is a runtime error (interpreter) or a crash (native). Fires on `s[i] = x`, `s[i] += x`, `s.data[i] = x` and `s.data.* = x` when `s` was declared from a literal (`s := "abc";`, `s: string = "abc";`) in the same statement list and nothing since could have changed it: any assignment to `s`, taking `*s`, or redeclaring the name (in any nested scope) ends the check, so `s = copy_string(s);` silences later writes. Writes in nested blocks and in macros are not reported. The editor offers to wrap the literal in `copy_string(...)` where the variable is declared (the copy is then yours to free).
### uninit_read
The first mention after the declaration, in the same block, decides; a write anywhere (even inside an `if`) ends the check, so only straight-line reads are reported. Taking the address (`init(*x)`, `for *x`), `.data`/`.count` of a fixed-size array and compiler built-ins (`type_of(x)`) are not reads. Procedures with `using` are skipped. The editor offers to drop the `= ---` so the variable starts out zeroed.
### unknown_allow
A name in a line comment's `allow(...)` or `allow-file(...)` list that is not a rule, a category or `all`. The comment allows nothing, so a typo leaves the finding in place, and a rule that was renamed leaves a comment that covers nothing. Suggests the closest rule name. `@jailint_allow(...)` notes are not checked. The editor offers to swap in the suggested name.
### unpaired_lock
A statement-level call of a `@Locks` procedure with an argument (`lock(*state.mutex);`) at the top level of a procedure body, followed by a path that reaches a `return`, or the end of the body, without a call of an `@Unlocks` procedure on the same first argument (compared as written, ignoring whitespace) or a `defer` of one. A `defer` earlier in the body counts. Paths made of statements in sequence, `if`/`else` and blocks are followed; anything else is silent: a loop or `switch` that mentions the unlock (one with a `return` and no unlock is reported at the `return`), `break`/`continue`, an `if` whose branches disagree about unlocking, `#insert`, `using`, macro calls. A procedure that is itself `@Locks` is a wrapper and is not checked, and a lock taken inside a nested block is not followed. A lock whose result reports failure (`err := lock(...)`) is not a statement-level call and is left alone. When nothing below unlocks by hand and the call is spelled `lock(...)`, the editor offers `defer unlock(...);` right after it (it assumes the paired procedure is called `unlock`).
### unused_import
Unnamed imports count as used when any lookup went through them, or (guard for unchecked code) when the module's files mention a name it exports. Not reported: imports with module parameters, modules with `#program_export`, programs that did not compile fully, generated files. Files whose first 10 lines say a tool generated them (`Automatically generated`, `DO NOT EDIT`, ...) are skipped by every `unused_*` rule.
### unused_parameter
Only for procedures whose every caller is in view: named, only called (never taken as a value), called at least once, not overloaded or exported, no `#c_call`/`#foreign`/`#expand`/notes. Also skipped when a variant exists (same name and parameter names, e.g. another platform's definition in an `#if` branch or sibling `.jai` file the program does not load; those files are parsed, not text-searched). Prefix a name with `_` to keep it. The editor offers the `_` prefix; `--fix` does not, since a caller may pass the argument by name.
### unused_result
Procedures declared with `@UseResult` (many in `Basic`/`String`/`Math`), so a program's own `trim` is left alone unless it carries the note. Calls with a pointer argument (works in place) are skipped. When the first argument is a plain name of the result's type, the editor offers `x = f(x);`.
### unused_variable
A name that appears anywhere else in its block is never reported (covers `#if`-excluded code and `#asm`). Skipped: backtick declarations, `using`, `_` names, blocks with `#insert`, macros. Fix removes the declaration, drops trailing call results, or renames to `_`.
### use_after_free
A local or parameter that is the first argument of a statement-level `@FreesArgument` call (`free(p);`, not `free(p.data)` or `array_free(*xs)`) and is read by a later statement of the same block before anything gives it a new value. Deliberately narrow: the first mention after the free decides, so `p = null;`, `p == null`, a `return`/`break`/`continue`, a second free (see `double_free`), `#insert`, `using`, a macro call or a taken address all end the check silently. A `defer`red free and a free nested in a branch are not followed. What counts as a read follows the callee's first parameter: for a raw pointer, string, array or handle (`free(p)`, `close(fd)`) any read does; for a pointer to a struct (`deinit(*table)`) only reaching into it (`t.field`, `t.*`) does, since the struct itself may be started over (`t.* = .{}`) or handed to the allocator. So a `@FreesArgument` note must be on a procedure whose *first* argument is what it releases.
### wrapping_constant
A constant takes the other operand's type, so the first divides `-41` by `h`. Only `/ % < <= > >=` next to a non-constant integer; `+ - *`, `==`, bitwise operators, named constants and casts are not reported. Suggested fix is computing in `s64`.

### zero_ptr
Only `cast(*T) 0` (also `0x0`) without cast modifiers. The editor offers `null`; `--fix` leaves it, since in `p := cast(*u8) 0;` the cast also gives `p` its type. Macros and generated files are skipped.
### Left out on purpose

`never_loop` (`for t { first = it; break; }` is the first-entry idiom), the style lints `collapsible_if`, `needless_return`, `redundant_else`, `manual_swap`, `len_zero`, `cmp_null`, `nonminimal_bool` (idiomatic or no better in Jai), `cast_possible_truncation`/`cast_sign_loss` (covered by `lossy_xx`), `out_of_bounds_indexing` (the compiler checks constant indices), `suspicious_assignment_formatting` (`jaifmt` normalizes it).

## Suppression

```jai
// jailint: allow(unused_variable)        covers the next line with code
x := compute(); // jailint: allow(all)     after code: its own line
// jailint: allow-file(float_equality)     whole file

callback :: (event: int, data: *void) {
    print("%\n", event);
} @jailint_allow(unused_parameter)        // after a declaration: all of it
```

Rules are comma-separated; `all` means every rule. A name that is neither a rule, a category nor `all` is reported by `unknown_allow`. A `_` prefix marks an unused variable or parameter as intentional.

## Configuration

The `[lint]` table of `jacket.toml` ([jacket](jacket.md)), found by walking up from the first path (or `--config`). A directory with only a legacy `jailint.toml` is read the same way (the same keys, without the `[lint]` header) and the tool suggests `jacket config migrate`. Unknown settings are dropped with a warning (`unknown setting ... (ignored)`); unknown rules or levels are errors so a typo cannot leave a rule on. `-A`/`-W`/`-D` override the file, overrides included.

```toml
[lint]
exclude = ["tests/corpus/negative/**", "generated/**"]   # globs relative to this file
max_nesting = 6                                          # blocks `excessive_nesting` allows
max_complexity = 25                                      # score `cognitive_complexity` allows
magic_number_allow = [60, 24]                            # numbers `magic_number` accepts
magic_number_min = 3                                     # whole numbers below this are fine

[lint.rules]
excessive_nesting = "warn"
float_equality = "warn"
unused_parameter = "allow"

[[lint.overrides]]                                       # levels for some paths only
paths = ["tests/**"]
rules = { float_equality = "allow", magic_number = "allow" }
```

Both tools match globs the same way: `*` and `?` stay within one path component, `**` crosses components (`**/x` also matches `x` at the top), a trailing `/` is dropped, an empty pattern matches nothing, and a pattern that matches a directory covers everything in it. The definition is `glob` in `crates/jailint/src/config.rs` and `glob_path_match` in `stdlib/Extensions/Jai_Format/config.jai`; both run every case in `tests/glob/cases.txt` (`pattern<TAB>path<TAB>match|nomatch`), so add a case there when changing either.

`[[lint.overrides]]` entries set rule or category levels for the files their `paths` globs match. For a file, the last matching entry that names the rule (or, failing that, its category) wins over `[lint.rules]`; `-A`/`-W`/`-D` win over both. `Config::level_at` resolves it, and the editor schema reuses the `[lint.rules]` schema for each entry's `rules`. The other `[lint]` settings (`max_complexity`, `magic_number_*`) are not per path.

The formatter is written in Jai and reads the `[format]` table itself; both tables live in one file. The repository's `jacket.toml` excludes negative compiler cases, `tests/lint` bad cases, formatter golden inputs and fuzzer seeds. CI runs `jailint -D warnings -j <cores> prelude stdlib examples tools jaifmt tests benchmarks`; deliberate findings in tests carry an `allow` with the reason. It also holds the repository to `magic_number` (time units allowed) and, under `stdlib/`, to `float_equality`; `[[lint.overrides]]` turn `magic_number` off for benchmarks, examples and binary-format, numeric and OS-binding code, and `float_equality` off in stdlib tests.

| Setting | Effect |
| --- | --- |
| `JAIC_STDLIB` | stdlib directory (default: next to the executable, else the checkout's) |
| `JAILINT_BLESS=1` | with `cargo test -p jailint`, rewrite `bad.expected` and `bad.fixed.jai` |
| `BLOCK_BUDGET` (`driver.rs`) | basic blocks compile-time code may run per root |

## How to change it

**Add a rule:**

1. Create `crates/jailint/src/rules/<name>.rs` with a doc comment (what, why, exactly when) and `pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>)`.
2. Register it in `rules/mod.rs` (`mod` line, `RULES` entry with default level and summary).
3. Add `tests/lint/<name>/bad.jai` and `good.jai` (whole programs with `main`), then `JAILINT_BLESS=1 cargo test -p jailint` and read the output.
4. Run it over the repository and the upstream corpus with `-W <name>` and check every hit; a rule that is sometimes wrong costs more trust than it earns.
5. Add the table row and a `### <name>` section here.

Inside a rule: `cx.procs` lists bodies (trust facts only for `p.clean` / `p.typed()`); `cx.ty`, `cx.constant`, `cx.entities(span)` and `cx.compiler.is_used` read facts; `cx.mentions` is the textual guard for unchecked code; `syntax::walk` does not enter nested procedures or struct bodies. A fix is byte edits; mark it `machine_applicable` only if it cannot change behaviour and the result still compiles (`fixed_cases_are_clean` re-lints `bad.fixed.jai`). New facts go in `IdeFacts` (`crates/jaic/src/ide/facts.rs`) behind `lint`.

Gotchas:

- Facts are recorded only for files under the root's directory, so module files reached from a program are not linted from it.
- `ide_check_all` checks bodies nothing called; a body that failed in any polymorph instance is not clean. Language-server sketches come after `IdeFacts::sketch_from` and rules never see them.
- A metaprogram needing a foreign call the sandbox lacks stops there; only what compiled is linted (`-v` shows the root).

## Dependencies

`crates/jaic` (parser, checker with lint facts, workspace observer, sandboxed host). `crates/jai-language-server` depends on `jailint` and shares `jailint::format_string`.
