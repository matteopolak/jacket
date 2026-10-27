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

Other options (see `--help`): `--compile-time-budget <blocks>` (blocks one compile-time evaluation may run, default 200000000, `0` for no limit), `--check-time-limit <seconds>` (default 10, `0` for no limit; it applies to each program checked, so every program gets the whole limit and a run over many programs has no overall limit; the limits are the command line's, never `jacket.toml`'s, so a project cannot raise them), `-A`/`-W`/`-D <rule>` (allow, warn, deny; `all`; `-D warnings`), `--config`, `--max-nesting <n>` (blocks `excessive_nesting` allows), `-I <dir>`, `-j <n>` (programs compiled at once; default one per CPU, at most 8), `--add <source>` (extra program-scope source, like jaic), `--color`, `-v`, and `--os`/`--cpu`/`--target` (the system the programs are linted for). The compile and target flags, `-j`/`--jobs` and `--color` are the table in `jaic::flags`, the same one `jaic` parses (so `jailint -j 4` and `jaic -j 4` mean the same); only jailint's own flags are in `crates/jailint/src/cli.rs`, and `--help` and the did-you-mean suggestions are generated from both tables. Exit status: 0 clean, 1 a denied finding or incomplete compilation (not with `--allow-incomplete`, for a tree whose platform-specific modules only compile on their own OS; it prints how many programs did not compile completely, and `-v` lists them), 2 usage or configuration error. Output goes through the shared `jaic::render`, so `--color`, `NO_COLOR` and `JAIC_DIAGNOSTICS` apply as in the compiler.

## How it works

1. **Plan** (`driver/plan.rs`, with `driver/claims.rs` for which root owns a file). Paths become `.jai` files minus `exclude`. `module.jai` and files directly in an import directory are modules, compiled by importing them from an empty program so every procedure is checked. A file that a module (or a nearby program) `#load`s is compiled as part of that module or program (`enclosing_module`, `enclosing_program`); otherwise `unused_parameter` would flag a module's exported interface. Only the listed files are reported. Each root is compiled with `jaic::project::program_for`'s import folders ([project](../compiler/project.md#import-paths)): the root's `modules/`, the `import_path` of the nearest `jacket.toml` (as `jacket` passes it to `jaic`), `-I` folders, the stdlib.
2. **Compile** (`driver/compile.rs`, over `jaic::ide::LintCompile`, the same lint compile the language server uses, so both lint one compile) with lint facts on (`IdeFacts::lint`): per-expression types and constness, cast types, every entity a name resolved to, and which import answered each lookup. Compile-time code runs in the sandboxed host with a block budget per evaluation and a wall-clock limit per root (`--compile-time-budget`, `--check-time-limit`, [limits](../compiler/compile-time-limits.md); a stopped root is reported as incomplete); workspaces a metaprogram creates are linted too (`jaic::build::WorkspaceObserver`).
3. **Lint** (`lib.rs`, driven from `driver/lint_root.rs`). Re-parse, build suppressions, collect procedure bodies and whether every instance checked fully, then run each enabled rule. Where the compiler did not look (an `#if` branch left out, a failed body) rules fall back to text and stay quiet.
4. **Leak pass** (`driver/leak.rs`, only when `platform_leak` is on). Each root whose entry file is portable is compiled again for every target system with `Compiler::keep_going_past_asserts`; `jaic` notes each `#import`/`#load` it meets and each failing `#assert` (`sema/import_trace.rs`). `rules::platform_leak::analyze` turns those into findings. The failed asserts, with the chain of import sites to each, are also in `Outcome::failed_asserts`.
5. **Report.** `--fix` applies non-overlapping machine-applicable fixes and re-lints, up to 5 passes (`fix.rs`). The language server's `source.fixAll.jailint` applies one pass. A file reached from several roots is reported once.

Editors: `jailsp` lints each open document using the cached compile that hover uses. Diagnostics carry the rule as `code` and a link to its `### <rule>` section below, and `deny` rules are errors. Settings come from the `[lint]` table of the nearest `jacket.toml` (the browser build sends it as an open document). See [the language server](../compiler/language-server.md#lints-and-quick-fixes).

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
| `@GrowsArray` | `pointer_into_resizable` | may reallocate the resizable array its first argument points to, moving its elements (`array_add`, `array_insert_at`, `array_reserve`, `array_resize`) |
| `@AddsToArray` | `useless_resizable` | adds one element to the end of the resizable array its first argument points to (`array_add(*xs, item)`) |
| `@FreesArgument` | `defer_in_loop`, `use_after_free`, `double_free` | after the call the *first* argument's memory or resource is gone: freed, closed or destroyed (`free`, `array_free`, `array_reset`, closers and `deinit`s). Not for a procedure whose first argument is only an allocator, heap or owner |
| `@Allocates` | `leaked_allocation`, `leaked_temporary` | the result is newly allocated and the caller owns it (`alloc`, `New`, `copy_string`, `sprint`, `join`, ...) |
| `@TakesByteCount` | `element_count_as_bytes` | the last argument is a number of bytes in the memory the pointer arguments point at (`memcpy`, `memset`, `memcmp`) |
| `@Locks` | `unpaired_lock` | takes the lock its first argument names (`lock(*mutex)`); a procedure with this note is a wrapper and is not checked itself |
| `@Unlocks` | `unpaired_lock` | gives back the lock its first argument names |
| `@AssertsArgument` | `assert_constant` | stops the program unless its first argument is true (the library's `assert`) |
| `@PushesContext` | `push_in_static_if` | a macro that changes the context and restores it when the caller's scope ends (`push_allocator`) |
| `@WritesOutput` | `print_in_module` | writes to the program's standard output or error (`print`, `write_string`, `write_strings`, `write_builder`) |
| `@Exits` | `exit_in_module` | ends the process (`exit`) |
| `@PrintLike` | `leaked_temporary` | its `string` parameter is a format string, followed by the values it formats (`print`, `log`, `sprint`, `tprint`) |
| `@TempAllocates` | `temp_storage_escape` | the result lives in temporary storage, which the program resets (`tprint`, `talloc`, `talloc_string`, `copy_temporary_string`, `temp_c_string`); a procedure with this note is a wrapper whose own `return`s are not checked |
| `@Opens` | `unclosed_resource` | acquires something that has to be given back: it returns the resource (`file_open`) or starts holding memory in the local its first argument points to (`init_string_builder`, `append`, `print_to_builder`) |
| `@Closes` | `unclosed_resource` | gives back what `@Opens` acquired from its first argument (`file_close`, `free_buffers`, `reset`, `builder_to_string`, `write_builder`); a `@FreesArgument` procedure closes as well |
| `@FormatsToString` | `useless_format`, `format_push_string` | its first argument is a format string and the rest are the values; it returns the formatted text as a new string (`sprint`, `tprint`) |
| `@AppendsString` | `format_push_string` | adds the string in its second argument to the builder in its first (`append(*builder, s)`) |

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

Columns: `yes` means `--fix` applies the fix; `editor` means the fix may change behaviour, so it is only a quick fix in the editor; `no` means none. "Side-effect-free" below means names, member paths, literals, indexing, casts and operators: no calls. Rules that compare expressions compare them as written, ignoring whitespace. A rule is removed once `jaic` rejects what it found; a `[lint.rules]` table naming a removed rule fails with `unknown rule`.

Each rule's source (`crates/jailint/src/rules/<rule>.rs`) opens with a doc comment saying exactly when it fires, and `tests/lint/<rule>/` has `bad.jai` (+ `bad.expected`, `bad.fixed.jai`) and `good.jai`. The `every_rule_is_documented` test requires a table row and a `### <rule>` section here, because editors link to the anchor.

| Rule | Category | Default | Fix | Finds |
| --- | --- | --- | --- | --- |
| `absurd_comparison` | correctness | deny | editor | `u >= 0`, `u < 0` with an unsigned `u`: always true or false |
| `almost_swapped` | correctness | deny | editor | `a = b; b = a;`, a swap that sets both to `b` |
| `approx_constant` | style | warn | editor | `3.14159`, `6.2832`, `1.4142`: a hand-typed approximation of a constant `Math` has (`PI`, `TAU`, `SQRT2`, ...) |
| `assert_constant` | style | warn | yes | `assert(true, "...")`, `#assert true;`: an assertion whose condition is the literal `true`. `assert(false, ...)` is a deliberate failure and is not reported |
| `autocast_in_arg` | restriction | allow | editor | `draw(xx width, xx height)`: the callee's parameter types decide the conversions |
| `autocast_used` | restriction | allow | editor | `small: u8 = xx big;`, `return xx count;`: a conversion whose target type is left to the context |
| `bad_bit_mask` | correctness | deny | editor | `(x & 4) == 1`, `(x | 2) == 0`: a comparison of a masked value with a constant the mask makes unreachable |
| `banner_comment` | style | warn | yes | a comment that is only a line of `-`, `=` or `*` fencing off a section |
| `bitwise_precedence` | suspicious | warn | yes | `1 << n - 1`, `flags \| 1 << 3`: bitwise operators that group unlike C |
| `bool_comparison` | complexity | warn | yes | `x == true`, `x != false`, `false == x` |
| `bool_to_int_with_if` | pedantic | allow | editor | `ifx flag then 1 else 0`, `if flag n = 1; else n = 0;`: a `bool` converted to an integer by a branch rather than `cast(int) flag` |
| `broken_doc_link` | style | warn | editor | a doc-comment link (`[prnt]`, `[Type.fild]`) that names no declaration |
| `cognitive_complexity` | pedantic | allow | no | a procedure whose branching, weighted by how deeply it nests, scores above `max_complexity` (default 25) |
| `constant_condition` | suspicious | warn | no | an `if` or `while` whose condition is always true or always false, by the compiler's value ranges |
| `defer_frees_returned` | correctness | deny | editor | `defer free(p);` then `return p;`: the caller gets memory the `defer` already released |
| `defer_in_loop` | suspicious | warn | no | a `defer` in a loop body that cleans up something from outside the loop |
| `detached_doc_comment` | pedantic | allow | editor | `// Adds one to n.` and a blank line before `bump :: (n: int) -> int`: a comment that reads as docs but is not attached |
| `double_free` | correctness | deny | editor | `free(p); free(p);`: a variable freed twice by the same procedure with nothing in between |
| `dropped_result` | suspicious | warn | editor | `n := to_integer(s);`: a call that keeps fewer values than it returns and whose next value is a `bool`, usually "did it work" |
| `duplicate_condition` | suspicious | warn | editor | an `else if` that repeats an earlier condition and can never run |
| `element_count_as_bytes` | correctness | deny | yes | `memcpy(d, xs.data, xs.count)` with `xs: [] s32`: an element count passed where a `@TakesByteCount` procedure wants bytes |
| `enum_member_prefix` | style | warn | no | `Color :: enum { COLOR_RED; COLOR_GREEN; }`: members that repeat the enum's name, which `Color.RED` already says |
| `erasing_op` | suspicious | warn | editor | `x * 0`, `x & 0`, `x % 1`: always `0` |
| `excessive_bools` | restriction | allow | no | `open :: (path: string, read: bool, write: bool, create: bool, truncate: bool)` |
| `excessive_nesting` | pedantic | allow | no | blocks nested deeper than `max_nesting` (default 6) |
| `exit_in_module` | restriction | allow | no | `exit(1)` in a file that belongs to a module |
| `field_reassign_with_default` | pedantic | allow | editor | `config: Config; config.width = 80; config.height = 24;` |
| `float_equality` | pedantic | allow | no | `==` or `!=` between two computed floats |
| `float_loop_counter` | pedantic | allow | no | `while t < 1.0 { ...; t += 0.1; }`: rounding error decides how many times it runs |
| `format_push_string` | perf | warn | editor | `append(*builder, tprint("%: %\n", name, value))`: `print_to_builder` writes into the builder directly |
| `identical_branches` | suspicious | warn | editor | `if c { A } else { A }`, `ifx c then a else a` |
| `identical_case_bodies` | pedantic | allow | editor | `case .PNG; return "image"; case .JPEG; return "image";`: cases that do the same thing and could be `case .PNG, .JPEG;` |
| `identical_operands` | suspicious | warn | no | `a == a`, `x - x`, `ok && ok` |
| `identity_op` | complexity | warn | yes | `x + 0`, `x * 1`, `x / 1` |
| `if_not_else` | pedantic | allow | editor | `if !ready { ... } else { ... }`, `if a != b { ... } else { ... }`, `ifx !c then x else y`: a negation the swapped branches would not need |
| `impossible_comparison` | correctness | deny | editor | `x > 5 && x < 3`, `x != 1 \|\| x != 2`: comparisons of one integer that never pass together, or always pass between them |
| `incomplete_enum_switch` | pedantic | allow | no | `if kind == { case .CIRCLE; ...; case .SQUARE; ...; }` with `TRIANGLE` left out and no `#complete` or `case;` |
| `index_only_loop` | complexity | warn | yes | `for i: 0..xs.count-1` where `i` only indexes the array or string `xs` |
| `infinite_loop` | suspicious | warn | no | a `while` whose condition nothing in the loop changes |
| `int_plus_one` | complexity | warn | editor | `x >= y + 1`, `x - 1 >= y`, `x + 1 <= y`, `x <= y - 1`: an integer comparison with `1` added or taken off, which is a strict comparison |
| `integer_division_in_float` | suspicious | warn | editor | `cast(float)(a / b)`, `1 / 2 * w`: an integer quotient used as a float |
| `large_stack_array` | pedantic | allow | no | `scratch: [1 << 20] u8;` as a local: a megabyte of stack |
| `leaked_allocation` | pedantic | allow | editor | `p := New(T);` that is never freed, returned or handed on |
| `loop_with_break_condition` | complexity | warn | yes | `while true { if i >= n break; ... }`: a loop that opens by leaving when a test fails, which is the `while` condition |
| `leaked_temporary` | suspicious | warn | no | `if sprint("%", n) == "7"`, `print("%", sprint(...))`: an `@Allocates` result that is compared, measured, indexed or printed and nothing can free |
| `lossy_float_literal` | style | warn | yes | `pi: float32 = 31415.92653;`: digits a `float32` cannot hold |
| `lossy_xx` | pedantic | allow | yes | `xx` that narrows a number to a smaller type |
| `magic_number` | pedantic | allow | no | `if x > 37`, `y * 1.0472`, `timeout = 8640`: an unexplained number where a named constant would read better |
| `manual_array_find` | complexity | warn | no | `for xs { if it == wanted { found = true; break; } }` |
| `manual_assign_op` | complexity | warn | yes | `a = a + b` to `a += b` |
| `manual_clamp` | complexity | warn | editor | `max(0, min(100, x))`, `if x < 0 { x = 0; } else if x > 100 { x = 100; }`, `ifx v < lo then lo else ifx v > hi then hi else v`: a hand-written `clamp` |
| `manual_index_counter` | complexity | warn | yes | a counter kept next to a `for` loop that always equals `it_index` |
| `manual_memcpy` | perf | warn | no | `for i: 0..count-1 { dst[i] = src[i]; }` |
| `min_max` | correctness | deny | editor | `min(0, max(100, x))`: a clamp with swapped bounds |
| `misrefactored_assign_op` | correctness | deny | editor | `total += total + x;`: a compound assignment whose right side repeats the target and the operator, which was probably `total = total + x;` rewritten halfway |
| `missing_doc` | restriction | allow | no | `parse :: (text: string) -> Tree { ... }` with no comment above it |
| `mixed_case_hex_literals` | style | warn | editor | `0xDeadbeef`, `0xFFfF`: a hex literal whose letters are in both cases |
| `modified_range_bound` | suspicious | warn | no | `for 0..n-1 { ...; n -= 1; }`: the range was fixed before the first iteration, so the change does not reach the loop |
| `needless_bool` | complexity | warn | yes | `ifx c then true else false`, `if c return true; else return false;` |
| `needless_continue` | style | warn | yes | `for xs { work(it); continue; }`, `if c { work(); continue; }` ending a loop: a `continue` the loop would have done anyway |
| `neg_cmp_float` | pedantic | allow | editor | `!(x < limit)` with floats: not the same as `x >= limit` for NaN |
| `neg_multiply` | style | warn | yes | `x * -1`, `-1 * x`, `f * -1.0`: a multiplication by minus one, which is a negation |
| `no_effect` | suspicious | warn | editor | `x == 5;`, `count + 1;`, `flush;`: a statement that does nothing |
| `platform_leak` | restriction | allow | no | `#import "POSIX";` in code held to every operating system: it fails to compile on Windows |
| `pointer_into_resizable` | suspicious | warn | no | `p := *xs[i]; array_add(*xs, v); use(p);`: a pointer into a resizable array used after the array may have moved |
| `print_in_module` | restriction | allow | no | `print(...)` or `write_string(...)` in a file that belongs to a module |
| `push_in_static_if` | suspicious | warn | no | `push_allocator(a);` directly in an `#if` arm: it lasts to the end of the procedure, not the arm |
| `range_past_count` | correctness | deny | editor | `for i: 0..xs.count` indexing `xs[i]`: one past the end |
| `redundant_alias` | complexity | warn | yes | a local only copied into another local (`a := f(); b := a;`) or returned right away |
| `redundant_cast` | complexity | warn | yes | a cast to the type the value already has |
| `redundant_deref` | complexity | warn | yes | `p.*.field` with `p: *Point`: member access already looks through a pointer to a struct |
| `remove_in_for` | correctness | deny | editor | `array_*_remove_*` on the array a `for` is walking |
| `returned_local_address` | correctness | deny | no | `return *x;`, `return *x.field;`, `return buffer.data;` with `x` or `buffer` a local: a pointer into a stack frame that is gone |
| `reversed_range` | correctness | deny | editor | `for i: 10..0`: a range that never runs |
| `self_assignment` | suspicious | warn | editor | `x = x;` |
| `shadowed_it` | suspicious | warn | no | a nested `for` hides an `it` the enclosing loop still uses |
| `shadowed_local` | pedantic | allow | no | `count := 10;` inside an `if` in a procedure with a parameter `count`: a local that hides another of the same procedure |
| `shift_overflow` | suspicious | warn | no | `x << 40` with `x: s32`, `small << 8` with `small: u8`: a shift that moves every bit out |
| `single_case_switch` | style | warn | editor | `if mode == { case .VERBOSE; ...; }`, with at most a default case: an `if` written as a `switch` |
| `string_literal_write` | correctness | deny | editor | `s := "abc"; s[0] = x;`: a write into a string literal, which is read-only |
| `temp_storage_escape` | pedantic | allow | no | `label = tprint("item %", id);` with `label` a global: temporary-storage memory kept past the next reset |
| `todo_marker` | restriction | allow | no | `// TODO: handle the empty list` |
| `too_many_arguments` | restriction | allow | no | `draw :: (x: int, y: int, w: int, h: int, r: u8, g: u8, b: u8, a: u8)` |
| `unchecked_directive` | restriction | allow | no | `f :: () #no_abc { ... }` or `#asm { ... }` with no `//` comment on the line or the line above |
| `unclosed_resource` | suspicious | warn | no | `builder: String_Builder; append(*builder, "x");` or `file, ok := file_open(p);` with nothing that closes it |
| `uninit_read` | correctness | deny | editor | `total: int = ---; for xs total += it;`: a variable read before anything is written to it |
| `unknown_allow` | suspicious | warn | editor | a `// jailint: allow(...)` comment naming a rule that does not exist |
| `unnecessary_force_cast` | complexity | warn | yes | `cast,force(s64) a` with `a: s32`: nothing is reinterpreted and there is no range to check |
| `unnecessary_semicolon` | style | warn | yes | `x = 1;;`, `if ready { ... };`: an empty statement |
| `unpaired_lock` | suspicious | warn | editor | `lock(*m);` with a `return`, or the end of the procedure, reached before the matching unlock |
| `unreadable_literal` | pedantic | allow | yes | `1000000`, `0xdeadbeef`, `1234567.5`: a long literal with no `_` between its digits |
| `unsigned_range_end_minus_one` | suspicious | warn | no | `for i: 0..n-1` with `n: u32`: when `n` is `0`, `n - 1` wraps to the largest value |
| `unused_import` | style | warn | yes | an `#import` nothing in its scope uses |
| `unused_parameter` | style | warn | editor | a parameter the procedure never uses |
| `unused_private_declaration` | suspicious | warn | no | `#scope_file` followed by `helper :: () { ... }` that nothing in the file mentions |
| `unused_result` | suspicious | warn | editor | `trim(line);`: a call to a `@UseResult` procedure, as a statement |
| `unused_suppression` | suspicious | warn | no | `x := 1; // jailint: allow(float_equality)`: a name in an allow comment or note that silences nothing |
| `unused_variable` | style | warn | yes | a local variable that is never used |
| `use_after_free` | correctness | deny | no | `free(p); print("%", p.x);`: a variable read after a `@FreesArgument` call freed it |
| `useless_format` | complexity | warn | editor | `tprint("%", name)`, `sprint("%", name)` with `name: string`: a copy made through a string builder |
| `useless_resizable` | perf | warn | no | `parts: [..] string;` followed by `array_add(*parts, "a"); array_add(*parts, "b");` and only reads |
| `using_pollution` | pedantic | allow | no | `draw :: (count: int, using style: Style)` where `Style` has a `count` member: a `using` that brings in a name already in scope |
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
### autocast_in_arg
The call site then depends on the parameter types: changing `draw` to take a `float` instead of an `s32` changes what `xx` does at every call, and nothing there says so. Fires on an `xx` that is directly an argument of a call (other uses are `autocast_used`). When the compiler resolved the cast the message names both types and the editor offers `cast(T)` for plain `xx`; `--fix` does not apply it. Off by default: `xx` in arguments is idiomatic Jai, especially for C APIs.
### autocast_used
`xx` takes its target type from the context, so the line does not say what the value becomes, and changing the type on the left silently changes the conversion on the right. `cast(u8) big` says it where it happens. Fires on every `xx` that is not directly an argument of a call (those are `autocast_in_arg`). When the compiler resolved the cast the message names both types and the editor offers `cast(T)` for plain `xx`; `--fix` does not apply it. Off by default: `xx` is idiomatic Jai.
### bad_bit_mask
`x & m` has only bits of `m`, so a constant with a bit outside `m` is never equal to it; `x | m` always has the bits of `m`, so a constant missing one never equals it. `!=` is then always true. Needs an integer literal on each side of the comparison and of the mask, non-negative, and a non-constant `x`. Usually the constant is wrong (`== 4` or `!= 0` was meant). The editor offers to write `false` or `true`.
### banner_comment
A comment of three or more of `-=*#~_` and nothing else, line or block (`// ------`). Name the section in one plain `// Names` line instead, or split the file. Rust and Python files are held to the same rule by `tools/ci/check_source_style.py`. Generated files are skipped. `--fix` removes the line; a section name on a line of its own stays.
### bitwise_precedence
Jai gives all bitwise and shift operators one level above `*`, grouped left to right, so `1 << n - 1` is `(1 << n) - 1`. The fix adds the parentheses Jai already implies; move them if the C reading was meant.
### bool_comparison
Only when the other side is a non-constant `bool`.
### bool_to_int_with_if
Off by default, since many read the `ifx` more easily than the cast. A `bool` condition and an integer result, with the literals `1` and `0` (`0` and `1` gives `cast(int) !flag`); the `if`/`else` form needs both branches to assign the same side-effect-free target. The editor offers the cast; `--fix` leaves it.
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
### detached_doc_comment
A top-level procedure, struct or enum with a blank line between its declaration and a comment above that reads as a sentence (three or more words, starting with a capital letter or a backtick and ending in `.`, `!` or `?`: section headings, commented-out code, `TODO` markers and `jailint:` directives are left alone). Hover and the generated docs only see comments directly above a declaration, so the gap hides the text. A comment before the file's first code is a header and is left alone. The editor offers to close the gap; `--fix` does not, since the comment may be a section heading. Only a blank line is looked at, not code between the comment and the declaration. Off by default.
### double_free
The same walk as `use_after_free`: a statement-level `@FreesArgument` call on a local or parameter, then, as the next thing that mentions it in the block, a statement-level call of the *same* procedure on it (`free(p); free(p);`, `close(f); close(f);`). Calls of different procedures are not paired, because `deinit(p); free(p);` releases the contents and then the memory. Silent when anything else mentions the variable in between, when a `return`/`break`/`continue` comes first, for a `defer` or a free nested in a branch, and for the cases listed under `use_after_free`. The editor offers to remove the second free; `--fix` does not, since the first one may be the wrong call.
### dropped_result
A declaration or assignment whose right side is one call that returns more values than the left side names, where the first dropped value, the one right after the last kept, is a `bool`: `value := parse(text);` quietly turns a failed parse into `0`. Receive the flag (`value, ok := parse(text);`) or write `value, _ := parse(text);` to say it is ignored on purpose. A success or found flag sits right after the value it describes, so a `bool` that comes later, after other results (status extras of a wide API such as `pressed, state, released := button(...)`), and dropped values of other types (counts, remainders) are fine, as are `_ := f();`, calls used as statements, compound assignments and calls whose procedure type is not known. Calls through procedure values count. To make dropping a result a hard compile error, mark it `#must` where the procedure is declared (see [directives and notes](../language/directives-and-notes.md)). The editor offers to append `, _` (declarations without a type, and assignments).
### duplicate_condition
Conditions must be side-effect-free; a call in between restarts the comparison. A repeated `case` value is not checked here: the compiler rejects it. The editor offers to remove the dead `else if` branch (not when a comment sits in it).
### element_count_as_bytes
Only when `xs.data` is another argument of the same call, so the count clearly describes that array. Elements of one byte (`u8`, `s8`, `bool`) and strings are left alone. The fix appends `* size_of(Elem)`; it is not applied by `--fix`, since the element type may need its module's name.
### enum_member_prefix
Two or more members, all beginning with the enum's name once case and `_` are ignored (`Token_Kind` and `TOKEN_KIND_IDENT`) and each with something left after it. Enums that add members with `#insert` or `using`, enums declared `using` (their members are used without the enum's name, so the prefix is what tells them apart), and generated files are skipped. No fix: renaming members changes every use of them.
### erasing_op
Left alone inside an index or array literal, where a `0 *` lines up with other rows. When the other operand is side-effect-free the editor offers to write the `0`.
### excessive_bools
A parameter counts when its type is written `bool`, or it has no type and defaults to `true` or `false`. A procedure with more than `[lint] max_bools` (default 3) is reported on its name, with its count. At the call `open(path, true, false, true, false)` does not say which flag is which: use an enum, an `enum_flags`, or split the procedure. `#c_call` and foreign procedures and generated files are skipped. No fix. Off by default.
### excessive_nesting
Off by default. A level is a braced block or a `switch`; `else if` chains and `#if` add none. A procedure written inside a block starts counting from that block's depth, so a nested procedure cannot hide the nesting. Reports the first block past the limit on each path, on its `{` (or the `switch` header). The limit is `[lint] max_nesting` in `jacket.toml` or `--max-nesting <n>`, default 6; enable it with `-W excessive_nesting` or `[lint.rules]`. On the standard library, a limit of 6 finds nothing and 5 finds five blocks. Fix by returning or `continue`ing early, or by moving the inner part into a procedure. Generated files are skipped.
### exit_in_module
A file belongs to a module when it or a directory above it holds a `module.jai`, with no `tests` or `test` folder in between (those hold programs that use the module). A call is reported when the procedure it calls has the `@Exits` note (the library's `exit`). A library that exits skips the program's cleanup and cannot be tested: return an error. Programs, macros and generated files are not checked. No fix.
### field_reassign_with_default
The fields not named in the literal keep their defaults, as in the declaration, and the values are set where the variable is made instead of in statements a later edit can separate from it. Fires on a local `name: T;` of a struct type with no value, directly followed by one or more plain assignments `name.field = value;` to distinct fields, whose values do not mention `name`. Unions, `+=` and nested members are left alone. The editor offers the literal when no comment sits among the statements; `--fix` does not apply it. Off by default: declaring and then assigning is common, readable Jai (over 500 places in this repository).
### float_equality
Off by default: exact comparison is often deliberate. Constants, `x != x` and `operator ==` bodies are not reported.
### float_loop_counter
Adding `0.1` ten times does not give `1.0`, so whether the loop runs ten or eleven times depends on rounding error, which grows with the iteration count. Count with an integer and compute the float from it (`for i: 0..9 { t := i * 0.1; }`). `for` loops cannot count in floats (the compiler rejects a float range), so this looks at `while`: the condition compares a float variable with an expression that does not mention it, and the body adds a step to it or takes one from it (`t += dt`, `t = t - dt`) that does not mention it either. Convergence loops and doubling loops are left alone. Off by default: a loop that drains a float budget reads fine this way. No fix.
### format_push_string
Fires on a call of a procedure noted `@AppendsString` (the library's `append` of a string) whose string is a call of one noted `@FormatsToString` (`tprint`, `sprint`). `print_to_builder(*builder, ...)` takes the same format and values and writes into the builder without building a second string first; a `sprint` result is also never freed by this form. The editor offers the rewritten call; `--fix` leaves it alone because a formatting failure leaves the builder with the part that was written instead of nothing.
### identical_branches
Compared by tokens. Empty branches, `#if`, and the last link of an `else if` chain are left alone. When the condition is side-effect-free, the editor offers to keep one copy of the block (or the value of an `ifx`) and drop the test.
### identical_case_bodies
Off by default, since one case per row is sometimes the clearer table. Compares the tokens of a case's body with those of every earlier case of the same `switch`; default cases, empty bodies, `#through` cases and `#if` switches are skipped. The editor merges the later case into the earlier one's value list; `--fix` leaves it.
### identical_operands
Float `==`, `!=` and `-` are left alone (NaN tests), as are operator procedures.
### identity_op
`0 + x`, operations inside indexes/array literals and shifts by `0` are left alone (they line up with neighbours).
### if_not_else
Off by default, since putting the exceptional or shorter branch first is often right. A condition `!c`, or `a != b` of numbers, pointers, enums or strings, with a plain `else` (an `else if` chain is skipped). The editor swaps the branches when both are blocks, or for an `ifx`; `--fix` leaves it.
### impossible_comparison
Only integer expressions compared with integer literals, within one run of `&&` or `||`. Floats are left alone (`x > 1 && x < 2` holds for `1.5`). Redundant pairs such as `x < 3 && x < 5` are not reported. For a lone pair of comparisons the editor offers to swap `&&` and `||`, when the swapped test is not impossible too.
### incomplete_enum_switch
Lists the members no case mentions. Mark the switch `#complete` so the compiler keeps the list current, or add `case;` to name the fallback. Plain enums only (not `enum_flags`); a case value that is not a plain member name (`case Color.RED + 1;`) hides the coverage, so such switches are skipped. Members with the same value as a listed one count as covered. Off by default: handling two members of ten is often deliberate. No fix.
### index_only_loop
Becomes `for xs` (or `for *xs` when elements are written), other uses of `i` become `it_index`. Quiet when the loop may change `xs`, walks a grid, indexes other arrays or `xs[i + 1]`, or when `xs` is a constant. When the rewrite would make `it` name an inner loop's element, the fix is withheld and a named iterator is suggested.
### infinite_loop
Condition reads only locals, parameters and literals; skipped for bodies with `break`, `return`, `#insert`, `#asm`, `using`, macro calls or taken addresses.
### int_plus_one
Integers only, and only the literal `1`. The editor offers the strict comparison; `--fix` leaves it, since where `y + 1` wraps (the top of the type) the two forms differ.
### integer_division_in_float
Operands are typed before the cast, so the integers divide first. A variable divided by a power-of-two literal is taken as deliberate. The editor offers to convert the dividend to a float (`cast(float) done / total`); `--fix` does not, since it changes the value.
### large_stack_array
Locals live on the stack, which is small (a thread's is often 1 MiB or less) and shared with deep recursion, so a local of a megabyte can overflow it with no message beyond a crash. Fires on a local variable (not a parameter) whose type is a fixed array larger than `[lint] max_stack_array_bytes` (64 KiB by default); arrays of structs count by total size. Locals of polymorphic procedures and macros, and generated files, are skipped. Allocate large buffers with `NewArray` or `alloc`, use a resizable array, or keep them in a global. No fix. Off by default.
### leaked_allocation
Off by default: ownership conventions differ, and a program that allocates once for its whole run may leak on purpose. A local declared from a call of an `@Allocates` procedure (the library's `alloc`, `New`, `NewArray`, `copy_string`, `sprint`, `join`, `array_copy`, `to_upper_copy`, `read_entire_file`, ...; a cast around the call is fine) whose every later mention is harmless: reading or writing a number, bool or enum through it (`p.count`, `p.depth = 1`), comparing it, testing it in an `if`/`while`, or a bare statement. Passing it (or anything reached through it) to any call, freeing it (also by `defer`), returning it, assigning it elsewhere, taking its address, putting it in a literal or reassigning it all count as handling the memory and keep the rule silent. It also stays silent when it cannot tell which allocator is in force: a procedure that uses `push_context`, assigns `context`, calls a macro (`push_allocator` is one) or has `#insert`/`using`, an allocating call with `,, allocator = temp`, or a name mentioned where the walk cannot follow. For a pointer, the editor offers `defer free(x);` on the next line.
### leaked_temporary
A call of an `@Allocates` procedure (`sprint`, `join`, `copy_string`, ...) whose result is used where it is read and dropped at once: an operand of a comparison, the receiver of `.count` or of an index, or a value argument of a `@PrintLike` call. With no name holding it, nothing can free the memory, so it leaks on every run. Anything else (storing, passing to another call, returning) may hand the memory on and is not reported. It stays quiet in a procedure that changes the allocator (`push_context`, assigning `context`, `push_allocator` and other macros, `#insert`) and for a call with a `,,` context change. Keep the value in a local and `defer free` it, or use `tprint`. No fix.
### loop_with_break_condition
A `while true` (macros aside) whose body starts with an `if` without an `else` that only does an unlabelled `break`, with or without braces. The fix puts the opposite test in the header (`!c` becomes `c`, `a >= b` on numbers becomes `a < b`) and removes the `if`; a comment inside the `if`, or before it in the loop, withholds it.
### lossy_float_literal
A `float32` keeps about seven decimal digits. Digits past the shortest spelling that reads back as the same `float32` change nothing and suggest a precision the program does not have. Fires on a float literal that becomes a `float32` where it stands (the value of a declaration of type `float32`, the right side of an assignment to a `float32`, the operand beside a `float32` value) and whose significant digits (trailing zeros not counted) outnumber those of the shortest spelling of the same value. `--fix` writes that spelling, which stores the same `float32`. The compiler already rejects a `float32` literal with more than 8 digits after the point, and types one with exactly 8 as `float64` (so it is not seen here); this finds what is left, such as `123456.789012`, where the digits before the point use up the precision. Literals passed as arguments or cast, `float64` literals and hexadecimal floats are not looked at.
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
### manual_array_find
Fires on a `for` over an array whose body is one `if` without an `else`: it compares the loop's value (`it`, or the name given to it) with a side-effect-free expression of the same type that does not mention the loop's names, and its body is assignments of `true`/`false` or the index to variables outside the loop, then `break`. `array_find(xs, wanted)` returns the flag and the index (and searches long integer arrays a vector at a time). Loops over a pointer, in reverse or with a custom iterator, and searches that compare a member (`it.id == key`), are left alone. No fix.
### manual_assign_op
Target must be side-effect-free and the operation must keep its type.
### manual_clamp
Numbers with constant bounds, the lower not above the upper (a swapped pair is `min_max`'s). Three shapes: nested `@ReturnsLarger`/`@ReturnsSmaller` calls (the library's `max` and `min`), an `if` and `else if` that assign a side-effect-free target its bounds, and a nested `ifx`. The editor offers `clamp(x, lo, hi)`; `--fix` leaves it, since a statement inside an `else if` chain would need braces.
### manual_index_counter
`n := 0` right before a forward `for` (in a block, a `case` body or a `#if` branch), `n += 1` at the end of the body (or `defer n += 1`), `n` unused after the loop.
### manual_memcpy
Fires on a `for` over a range starting at `0` whose only statement is `dst[i] = src[i];` with `i` the loop's variable, `dst` and `src` different names or member paths, both arrays or pointers, and the same element type on both sides. The suggestion is `memcpy(dst.data, src.data, count * size_of(T))`, or assigning a fixed array whole. `memcpy` assumes the ranges do not overlap, which pointers and slices can break, so there is no fix.
### min_max
Literal bounds only, with `@ReturnsSmaller`/`@ReturnsLarger` procedures (the library's `min`/`max`). Use `clamp`. The editor offers `clamp(value, lo, hi)` with the two bounds in order.
### misrefactored_assign_op
The same operator on both sides, a side-effect-free target of a number type spelled the same in both places (`a op= a op b`), and for `+ * & | ^` on integers also `a op= b op a`. The editor offers `a op= b`; `--fix` leaves it, since the other reading (`a = a op b`) is as likely to be the one meant.
### missing_doc
A procedure, struct, enum or union declared at the top level of a file while the scope is exported (the default, until a `#scope_file` or `#scope_module`) needs a doc: the comment directly above it. `main`, constants, aliases, declarations that bind several names, and generated files are skipped. No fix: the text is the author's to write.
### mixed_case_hex_literals
Read from the tokens, so it needs no types; literals in macros and generated files are skipped. The editor offers the literal in the case most of its letters use (lower on a tie); which case to use is a matter of taste, so `--fix` leaves it.
### modified_range_bound
Only locals named directly in a range end (`n` in `0..n-1`); `xs.count` bounds are left alone. Quiet when a `break` or `return` follows the assignment, or the statement holding it, in the same block (`n = it; break;` records a result).
### needless_bool
A trailing `if c return true; return false;` is left alone when an earlier guard in the block also returns a literal. Comments inside the `if` withhold the fix.
### needless_continue
An unlabelled `continue` that ends a `for` or `while` body, or ends a branch of an `if` that is the body's last statement. The fix removes the statement when other statements share its block; an `if c continue;` that closes the loop does nothing at all, so it is reported without a fix.
### neg_cmp_float
Every comparison with a NaN is false, so `!(a < b)` is true for NaN where `a >= b` is false. Fires on a negated `<`, `<=`, `>` or `>=` with a float operand; `!(a == b)` is exactly `a != b` and is left alone. The editor offers the inverted comparison, which changes the result for NaN, so `--fix` does not apply it. Off by default: `!(x < limit)` is often written to stay on the safe side of NaN on purpose.
### neg_multiply
Signed integers and floats only, and only the literals `-1` and `-1.0`. The fix writes `-x`, and `-(a + b)` when the other side is not a single term.
### no_effect
An expression statement containing no call. The editor offers the likely intent: `==` becomes `=`, `x + 1;` becomes `x += 1;`, a procedure name gets `()`, and any other statement is removed. `--fix` applies none, since the typo may be in either place.
### platform_leak
Code meant to build everywhere that imports a module asserting an operating system it is not built for (`#assert OS == .LINUX || OS == .MACOS` in `POSIX`), found only when someone builds for that system. The rule compiles each portable root once per target (Linux, macOS and Windows on x64, from any host) with failing `#assert`s recorded instead of stopping the compile, and reports every `#import` or `#load` in portable code that brings in a file the settings say a target does not build, when an `#assert` about the operating system fails in that file or below it. The label is on the import and a note names the assert. Reports are by import site and system, so two systems at one import are two findings. An `#assert` that fails in a portable file with nothing above it to blame is reported on the assert: the file needs a `platforms` entry. Only asserts that name `OS` (or are `false`) count; CPU, size and build-setting asserts do not. Needs `[lint.portability]` (see Configuration); off by default, and `// jailint: allow(platform_leak)` covers an import. A compile that stops on another error before the asserts run finds nothing for that root. No fix.
### pointer_into_resizable
A resizable array that outgrows its storage gets a new block and frees the old one, so a pointer taken into the old block dangles. Fires on a local initialised with the address of an element (`p := *xs[i];`, `*xs[i].field`) or with the pointer a growing call returns (`p := array_add(*xs);`) when a later statement of the same block calls a `@GrowsArray` procedure on the same array and a statement after it mentions `p`; and on the iterator of `for *xs` when a statement of the body grows `xs` and a later one mentions the iterator. Arrays are compared as written. Growing inside a by-value `for xs` is fine (each iteration reads the count and data again), and the rule is quiet when the growing statement can leave the block (`return`, `break`, `continue`, `remove`) or the pointer is assigned a new value. A pointer that is still valid because the array was reserved first is reported too; take it again after growing, or keep the index. No fix.
### print_in_module
A file belongs to a module when it or a directory above it holds a `module.jai`, with no `tests` or `test` folder in between (those hold programs that use the module). A call is reported when the procedure it calls has the `@WritesOutput` note (the library's `print`, `write_string`, `write_strings` and `write_builder`). A library does not know where output belongs: return the text or log it, and let the program print. Programs, macros and generated files are not checked. No fix.
### push_in_static_if
`#if` adds no scope, so `push_allocator(a);` directly in an `#if` or `#else` arm lasts to the end of the enclosing block or procedure, not the arm. Fires on a call of a `@PushesContext` procedure (a macro that changes the context and restores it with a backtick `defer`; the library's `push_allocator` carries it) and on `push_context,defer_pop`. A push inside a nested `{ }` block of the arm, or outside any `#if`, is fine. Macros are not checked. No fix: the block must cover the code the push was meant for.
### range_past_count
Ranges include their end. Quiet when the body mentions `xs.count` (it may guard the index). The `- 1` fix changes behaviour, so it is editor-only.
### redundant_alias
Style rule, deliberately narrow so the fix never changes behaviour: the two statements are adjacent with nothing (not even a comment) between, the first name appears exactly twice in the body, the types are identical, and no `#insert`, `using`, notes or flags are involved. Parameters, loop variables, globals, macros and generated files are skipped.
### redundant_cast
Aliases like `c_long` differ per platform, so the value must be declared with the same type spelling. Polymorphic bodies and macros are skipped.
### redundant_deref
Fires when a member is taken from the dereference of an expression whose type is a pointer to a struct or union. A pointer to a pointer or to an array is left alone, where the `.*` changes the meaning. `--fix` removes the `.*` written after the pointer; the prefix form `(<<p).field` is reported without a fix.
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
### shadowed_local
A variable declaration (`name := ...`, `name: T`) in a nested block, loop body or `case` body whose name is a parameter or a variable declared earlier in an enclosing block of the same procedure. Not reported: loop variables, implicit (`it`, `it_index`) or named, and `while name := cond` bindings; names starting with `_`; a variable that starts from the one it hides (`n := n;`, `n := n + 1;`); macros (`#expand`); and code inside a nested procedure, which cannot see the outer locals. A name declared later in the outer block does not count, nor does a variable in a sibling block. No fix. Off by default.
### shift_overflow
The result is `0` (or `-1` for `>>` on a negative signed value) whatever the operand holds, and the compiler accepts it. Fires on `<<`, `>>` and `<<=`, `>>=` when the count is an integer literal no less than the bit width of the left operand's integer type. Constant left operands (`1 << 40`, typed by where they are used) and rotates are skipped. The usual cause is an operand narrower than meant: widen it first in parentheses, `(cast(u64) x) << 40` (`cast(u64) x << 40` shifts before it casts). No fix.
### single_case_switch
One case of one value, plus at most a default case. `#complete` switches, `#through` cases, cases listing several values and `#if` switches are skipped. The editor offers `if x == v { ... }` (with the default as the `else`); the bodies keep their text, so `jaifmt` fixes the indentation, and `--fix` leaves it.
### string_literal_write
Literals are read-only and equal literals share storage, so the write is a runtime error (interpreter) or a crash (native). Fires on `s[i] = x`, `s[i] += x`, `s.data[i] = x` and `s.data.* = x` when `s` was declared from a literal (`s := "abc";`, `s: string = "abc";`) in the same statement list and nothing since could have changed it: any assignment to `s`, taking `*s`, or redeclaring the name (in any nested scope) ends the check, so `s = copy_string(s);` silences later writes. Writes in nested blocks and in macros are not reported. The editor offers to wrap the literal in `copy_string(...)` where the variable is declared (the copy is then yours to free).
### temp_storage_escape
Off by default: handing back a temporary string from a helper is a common, documented convention. A call of a `@TempAllocates` procedure (or a local initialised with one) is reported when it is assigned to a global, to a member reached through a pointer or `.*`, added to such an array with an `@AddsToArray` call, declared as a file-scope variable, or returned from a procedure that is not itself noted `@TempAllocates`. A member of a local struct and plain locals are not reported. Copy the string first (`copy_string`), or note the returning procedure `@TempAllocates` so callers know to copy. No fix.
### todo_marker
A marker is `TODO`, `FIXME`, `XXX` or `HACK` in capitals, standing alone (`TODO:` and `(FIXME)` count; `todo`, `TODOS` and `AUTODOC` do not). Each comment is reported once, for its first marker. Strings are not looked at, and generated files are skipped. No fix. For a tree that must carry no unfinished-work markers, or a project that tracks them elsewhere.
### too_many_arguments
Every parameter counts, defaulted, baked (`$T`) and variadic ones included. A procedure with more than `[lint] max_arguments` (default 7) is reported on its name, with its count. Group parameters that belong together in a struct. `#c_call` and foreign procedures, whose signature C fixes, and generated files are skipped. No fix. Off by default.
### unchecked_directive
Each `#no_abc`, `#no_aoc` and `#asm` needs a comment on its own line or the line just above. These take away bounds and overflow checks or leave the language, so the reason belongs next to them. Generated files are skipped. No fix: the reason is the author's to write.
### unclosed_resource
A local that an `@Opens` call acquired something for, and that nothing closes. The opener is either assigned (`file, ok := file_open(path);`: the first name is the resource) or a statement on the address of a local declared in the procedure (`append(*builder, s);`, `init_string_builder(*builder);`). It fires when, afterwards, every mention of the local is another `@Opens` call, a comparison, a test in an `if` or `while`, or a number, bool or enum member read or written. Passing the local (or its address) to any other call, `defer`red or not, returning it, assigning it elsewhere, putting it in a literal, or a mention the scan cannot follow (a nested procedure, a code block) counts as closing or handing it on and keeps the rule quiet. It does not check that every path closes. `builder_to_string` and `write_builder` close a builder because they reset it unless `do_reset` is false. No fix.
### uninit_read
The first mention after the declaration, in the same block, decides; a write anywhere (even inside an `if`) ends the check, so only straight-line reads are reported. Taking the address (`init(*x)`, `for *x`), `.data`/`.count` of a fixed-size array and compiler built-ins (`type_of(x)`) are not reads. Procedures with `using` are skipped. The editor offers to drop the `= ---` so the variable starts out zeroed.
### unknown_allow
A name in a line comment's `allow(...)` or `allow-file(...)` list that is not a rule, a category or `all`. The comment allows nothing, so a typo leaves the finding in place, and a rule that was renamed leaves a comment that covers nothing. Suggests the closest rule name. `@jailint_allow(...)` notes are not checked. The editor offers to swap in the suggested name.
### unnecessary_force_cast
`,force` reinterprets the bits of a number as a float (or back) or of an aggregate as another of the same size, and also skips the range check a plain cast makes when it narrows an integer. When the value already has the target type or the conversion widens (`s32` to `s64`, `u8` to `u32`, `float32` to `float64`) neither applies, so the flag only warns the reader of something that is not there. Fires on a `cast,force` with no other modifier and a non-constant operand. `--fix` removes `,force`; the result is the same value, since the plain cast has nothing to check.
### unnecessary_semicolon
An empty statement in a block, a `case`, a procedure body or at the top level of a file. A `;` that is the whole body of a loop or `if` (`while poll() ;`) is how an empty body is written and is not reported. Macros and generated files are skipped. The fix removes the `;`, and its line when it stands alone.
### unpaired_lock
A statement-level call of a `@Locks` procedure with an argument (`lock(*state.mutex);`) at the top level of a procedure body, followed by a path that reaches a `return`, or the end of the body, without a call of an `@Unlocks` procedure on the same first argument (compared as written, ignoring whitespace) or a `defer` of one. A `defer` earlier in the body counts. Paths made of statements in sequence, `if`/`else` and blocks are followed; anything else is silent: a loop or `switch` that mentions the unlock (one with a `return` and no unlock is reported at the `return`), `break`/`continue`, an `if` whose branches disagree about unlocking, `#insert`, `using`, macro calls. A procedure that is itself `@Locks` is a wrapper and is not checked, and a lock taken inside a nested block is not followed. A lock whose result reports failure (`err := lock(...)`) is not a statement-level call and is left alone. When nothing below unlocks by hand and the call is spelled `lock(...)`, the editor offers `defer unlock(...);` right after it (it assumes the paired procedure is called `unlock`).
### unreadable_literal
Off by default: the standard library alone has thousands of such constants. Six or more decimal digits (groups of three), eight or more hex digits and nine or more binary digits (groups of four), or six or more digits before the point of a float. A literal that has any `_` is taken as grouped on purpose; octal literals and floats with an exponent are skipped, and so are macros and generated files. The fix adds the separators.
### unsigned_range_end_minus_one
Ranges include their end, so `0..n-1` visits the `n` indices below `n`, but with an unsigned `n` of `0` the end wraps around (or stops the program with overflow checks on) instead of giving an empty loop. Fires on a `for` over a range ending in `n - 1`, where `n` is a non-constant unsigned integer. Check for `0` first or count with a signed integer. No fix.
### unused_import
Unnamed imports count as used when any lookup went through them, or (guard for unchecked code) when the module's files mention a name it exports. Not reported: imports with module parameters, modules with `#program_export`, programs that did not compile fully, generated files. Files whose first 10 lines say a tool generated them (`Automatically generated`, `DO NOT EDIT`, ...) are skipped by every `unused_*` rule.
### unused_parameter
Only for procedures whose every caller is in view: named, only called (never taken as a value), called at least once, not overloaded or exported, no `#c_call`/`#foreign`/`#expand`/notes. Also skipped when a variant exists (same name and parameter names, e.g. another platform's definition in an `#if` branch or sibling `.jai` file the program does not load; those files are parsed, not text-searched). Prefix a name with `_` to keep it. The editor offers the `_` prefix; `--fix` does not, since a caller may pass the argument by name.
### unused_private_declaration
A procedure, struct, enum, constant or variable declared after `#scope_file` whose name appears nowhere else in its file (mentions in its own declaration do not count, so a procedure that only calls itself is unused; mentions in code an `#if` left out do). Nothing outside the file can see it, so it is dead code. Files with an `#insert` or `#placeholder` are skipped, as are `main`, the entry point, `build`, `get_plugin`, `for_expansion`, names starting with `__`, `_`, `using` and multi-name declarations, and declarations with notes, `#c_call`, `#foreign`, `#program_export`, `#compiler`, `#expand` or operator forms. No fix.
### unused_result
Procedures declared with `@UseResult` (many in `Basic`/`String`/`Math`), so a program's own `trim` is left alone unless it carries the note. Calls with a pointer argument (works in place) are skipped. When the first argument is a plain name of the result's type, the editor offers `x = f(x);`.
### unused_suppression
A name in a `// jailint: allow(...)`, `allow-file(...)` or `@jailint_allow(...)` that no finding was silenced by, reported on the name (or on the whole note). It is judged only when it proves something: every rule it names (every rule of a category for a category name, every rule for `all`) ran at an enabled level on the file, so `all` and category names with an off rule in them are left alone; the compile finished (not under `--allow-incomplete` with errors, and not in a file whose bodies failed to check); it does not sit inside an `#if` or `#if ==` block, whose other branches may be the ones that need it; and it names a real rule (`unknown_allow` reports the rest). `[[lint.overrides]]` entries in `jacket.toml` are not checked. No fix.
### unused_variable
A name that appears anywhere else in its block is never reported (covers `#if`-excluded code and `#asm`). Skipped: backtick declarations, `using`, `_` names, blocks with `#insert`, macros. Fix removes the declaration, drops trailing call results, or renames to `_`.
### use_after_free
A local or parameter that is the first argument of a statement-level `@FreesArgument` call (`free(p);`, not `free(p.data)` or `array_free(*xs)`) and is read by a later statement of the same block before anything gives it a new value. Deliberately narrow: the first mention after the free decides, so `p = null;`, `p == null`, a `return`/`break`/`continue`, a second free (see `double_free`), `#insert`, `using`, a macro call or a taken address all end the check silently. A `defer`red free and a free nested in a branch are not followed. What counts as a read follows the callee's first parameter: for a raw pointer, string, array or handle (`free(p)`, `close(fd)`) any read does; for a pointer to a struct (`deinit(*table)`) only reaching into it (`t.field`, `t.*`) does, since the struct itself may be started over (`t.* = .{}`) or handed to the allocator. So a `@FreesArgument` note must be on a procedure whose *first* argument is what it releases.
### useless_format
Fires on a call of a procedure noted `@FormatsToString` (the library's `tprint` and `sprint`) whose format is exactly `"%"` and whose single value is a `string`. The editor offers the string itself for `tprint` and `copy_string(s)` for `sprint`. They are not applied by `--fix`: a `tprint` result lives in temporary storage and a `sprint` result belongs to the caller, so replacing either changes who may free what.
### useless_resizable
A resizable array allocates and has to be freed; a handful of elements known up front fit in a fixed array on the stack (`parts: [2] string`). The rule is deliberately narrow. It fires on a local `name: [..] T;` without a value when every later use in the procedure is a statement of the same block that is just `array_add(*name, item)` (a procedure noted `@AddsToArray`), an index, `.count` or `.data`, the collection of a `for`, or a call that frees it (`@FreesArgument`). Passing the array anywhere else may need the `[..]` type, so those locals are left alone, as are adds made in loops, branches or several at once. No fix.
### using_pollution
A `using x;` statement, `using x: T;` declaration or `using` parameter whose struct (its own members and those of structs it `using`s, through a pointer too) has members named like a parameter or local visible at that point, a variable or constant at the top of the file, or a member an earlier `using` in the procedure brought in. The finding names up to four of them. `using,only(...)` and `using,except(...)` narrow the members; other filters are skipped. Procedures, types, macros (`#expand`), `_` and unnamed members are not counted. No fix. Off by default.
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

Rules are comma-separated; `all` means every rule. A name that is neither a rule, a category nor `all` is reported by `unknown_allow`, and one that silences nothing by `unused_suppression`. A `_` prefix marks an unused variable or parameter as intentional.

## Configuration

The `[lint]` table of `jacket.toml` ([jacket](jacket.md)), found by walking up from the first path (or `--config`, which then holds for every file). Each other file listed or found under a listed directory uses the nearest `jacket.toml` above it, so `jailint stdlib examples` applies `examples/jacket.toml` to the examples; `-A`/`-W`/`-D` and `--max-nesting` apply on top of every one, and a file the first root's `exclude` names stays excluded. `platform_leak`'s `[lint.portability]` is the first root's alone. Unknown settings are dropped with a warning (`unknown setting ... (ignored)`); unknown rules or levels are errors so a typo cannot leave a rule on. `-A`/`-W`/`-D` override the file, overrides included.

```toml
[lint]
exclude = ["tests/corpus/negative/**", "generated/**"]   # patterns relative to this file
max_nesting = 6                                          # blocks `excessive_nesting` allows
max_complexity = 25                                      # score `cognitive_complexity` allows
max_arguments = 7                                        # parameters `too_many_arguments` allows
max_bools = 3                                            # `bool` parameters `excessive_bools` allows
max_stack_array_bytes = 65536                            # bytes a local array may take for `large_stack_array`
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

`[lint.portability]` tells `platform_leak` which code is portable. A file is checked for a system unless the most specific `platforms` glob matching it (the most characters besides `*` and `?`) leaves the system out, or a `// jailint: platforms(linux, macos)` comment before its first line of code does; `[]` means no checked system builds it.

```toml
[lint.portability]
targets = ["linux", "macos", "windows"]                  # the default
portable = ["stdlib/**"]                                 # patterns that must build on every target

[lint.portability.platforms]                             # files limited to some systems
"stdlib/POSIX/**" = ["linux", "macos"]
"stdlib/Windows/**" = ["windows"]
"stdlib/Android/**" = []                                 # none of the three
```

The repository's `jacket.toml` holds `stdlib/` to it: OS modules and the per-system halves of modules (`internal/files/filesystem-windows.jai`, `internal/process/windows.jai`) are listed, stdlib tests are not checked (`"stdlib/**/tests/**" = []`), and CI runs the rule in its own step on Linux (a `just lint --os` run leaves it out, since it compiles every target itself). When the rule reports a stdlib import, guard it with `#if OS ==` (or move it into the per-system file) if portable code should not reach it; list the file under `platforms` if it really is for one system.

Path patterns (`exclude`, override `paths`, `portable`, `platforms` keys) are compiled when the file is read, and one that is malformed (an unclosed `[` or `{`, a trailing `\`) is a configuration error at its line. `*` and `?` stay within one path component, `**` crosses components (`**/x` also matches `x` at the top), `[abc]`, `[a-z]` and `[!abc]` match one character, `{a,b}` is one of its alternatives (they nest, and expand to at most 256 patterns), and `\` makes the next character literal. A trailing `/` is dropped, an empty pattern matches nothing, and a pattern that matches a directory covers everything in it. Matching takes time proportional to the path times the pattern, so many `**` stay fast. Paths are matched as `/` separated text; a `\` is a separator only where the platform uses it. The definition is `PathPattern` in `crates/jacket-config/src/pattern.rs`; `jaifmt` matches the forms without `[]`, `{}` and `\` in `glob_path_match` in `stdlib/Extensions/Jai_Format/config.jai`. Both run every case in `tests/glob/cases.txt` (`pattern<TAB>path<TAB>match|nomatch`), so add a case there when changing either.

`[[lint.overrides]]` entries set rule or category levels for the files their `paths` patterns match. For a file, the last matching entry that names the rule (or, failing that, its category) wins over `[lint.rules]`; `-A`/`-W`/`-D` win over both. `Config::level_at` resolves it, and the editor schema reuses the `[lint.rules]` schema for each entry's `rules`. The other `[lint]` settings (`max_complexity`, `max_arguments`, `max_bools`, `max_stack_array_bytes`, `magic_number_*`) are not per path.

The formatter is written in Jai and reads the `[format]` table itself; both tables live in one file. The repository's `jacket.toml` excludes negative compiler cases, `tests/lint` bad cases, formatter golden inputs and fuzzer seeds. CI runs `just lint` (`jailint -D warnings -j <cores> --allow-incomplete stdlib examples tools jaifmt tests benchmarks`); deliberate findings in tests carry an `allow` with the reason. It also holds the repository to `magic_number` (time units allowed) and, under `stdlib/`, to `float_equality`; `[[lint.overrides]]` turn `magic_number` off for benchmarks, examples and binary-format, numeric and OS-binding code, and `float_equality` off in stdlib tests.

| Setting | Effect |
| --- | --- |
| `JAIC_STDLIB` | stdlib directory (default: next to the executable, else the checkout's) |
| `JAILINT_BLESS=1` | with `cargo test -p jailint`, rewrite `bad.expected` and `bad.fixed.jai` |
| `--compile-time-budget`, `--check-time-limit` (`jaic::ide::lint_compile::CheckLimits`) | blocks per evaluation and seconds per program checked (each compile has its own clock, the leak pass's extra compiles too); the language server takes the same limits from the editor's settings |

## How to change it

**Add a rule:**

1. Create `crates/jailint/src/rules/<name>.rs` with a doc comment (what, why, exactly when), `pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>)` and `pub(crate) const INFO: super::RuleInfo` (name, default level, summary; the file name is the rule name, which a test checks).
2. Add `<name>` to the `rules!` list in `rules/mod.rs`; that one list declares the module, adds `INFO` to `RULES` and the module to the tests. The schema build (`editors/vscode/scripts/build.ts schemas`) reads the same list and each module's `INFO`.
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
