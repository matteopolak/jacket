# jailint (Jai linter)

`jailint` reports mistakes and unidiomatic code in Jai programs and fixes the ones it safely can. Rules run on the program after `jaic` has type-checked it, so they know each expression's type and what each name refers to. `jailsp` shows the same findings in editors, with quick fixes.

```sh
jacket lint src/                  # lint every .jai file under src/
jacket lint main.jai --fix        # apply the safe fixes
jacket lint -D warnings stdlib    # fail on any finding (CI)
jacket lint --list                # rules and default levels
```

`jacket lint` runs `jailint` with the same arguments ([jacket](jacket.md)). Every flag is in the [command-line reference](../reference/tools.md#linting); the useful ones:

- `-A` / `-W` / `-D <rule>` allow, warn or deny a rule or category (`-D warnings` denies all warnings).
- `--config <file>` use this `jacket.toml`; `--max-nesting <n>`; `-I <dir>`; `-j <n>` programs compiled at once.
- `--compile-time-budget <steps>` and `--check-time-limit <seconds>` cap compile-time code per program. They are command-line only, so a project cannot raise them.
- `-os`, `-cpu`, `-target` pick the system to lint for.

Exit status: 0 clean, 1 a denied finding or a program that did not compile fully (`--allow-incomplete` skips that), 2 usage or configuration error.

## How it works

- **Plan** (`driver/plan.rs`): paths become `.jai` files minus `exclude`. A module is compiled by importing it from an empty program so every procedure is checked; a file that a module `#load`s is compiled as part of it. Only the listed files are reported.
- **Compile** (`driver/compile.rs`): the same lint compile the language server uses, with lint facts on (`IdeFacts::lint`): expression types, constness, cast types and what each name resolved to. Compile-time code runs sandboxed ([limits](../compiler/compile-time-limits.md)).
- **Lint** (`lib.rs`, `driver/lint_root.rs`): re-parse, build suppressions, run each enabled rule. Where the compiler did not look (an `#if` branch left out, a failed body) rules fall back to text and stay quiet.
- **Leak pass** (`driver/leak.rs`, only with `platform_leak`): portable roots are compiled again for every target system and the imports and failed asserts are traced.
- **Fix** (`fix.rs`): `--fix` applies non-overlapping machine-applicable fixes and re-lints, up to 5 passes.

In editors, diagnostics carry the rule as `code` and link to its `### <rule>` section below; `deny` rules are errors. Settings come from the `[lint]` table of the nearest `jacket.toml` ([language server](../compiler/language-server.md#lints-and-quick-fixes)).

## Target checking

```sh
jacket lint -os windows -cpu arm64 -D warnings path/to/module
jacket lint -target wasm64-unknown-wasi -D warnings path/to/module
jacket lint -os windows --manifest target/windows-lint.json path/to/module
jacket lint --context app/main.jai path/to/parameterized-module
```

`-os`, `-cpu` and `-target` select real compiler options; they do not rewrite OS/CPU constants. Linker and SDK availability are not checked.

`--context <program>` checks a module inside a real enclosing program (so module parameters are real) and reports only the requested paths. `--manifest <file>` writes JSON listing each compiler context, `unvisited_files` (target-inactive conditional loads) and incomplete roots. A file seen only in an incomplete context is not coverage, so run one invocation per target and merge the manifests.

## Declaration notes

Several rules decide what a call means from a note on the procedure's declaration, never from its name. Write the note after the body (`} @UseResult`) or, for a bodiless declaration, after the semicolon. A call counts only when every procedure the callee resolved to carries the note. The library's `Basic`, `String` and `Math` procedures are annotated where they are declared (`stdlib/internal/`).

```jai
shout :: (s: string) -> string {
    return to_upper_copy(s);
} @UseResult
```

To change what a rule recognises, annotate the declaration; the linter has no name lists.

| Note | Rule | Meaning |
| --- | --- | --- |
| `@UseResult` | `unused_result` | calling it as a statement is a mistake |
| `@ReturnsSmaller`, `@ReturnsLarger` | `min_max` | returns the smaller or larger of two arguments (bodiless only) |
| `@RemovesFromArray` | `remove_in_for`, lifetime rules | removes elements; `(ordered,1)`/`(unordered,1)` name the index argument, `(last)` pops |
| `@GrowsArray` | `pointer_into_resizable` | may reallocate the resizable array its first argument points to |
| `@AddsToArray` | `useless_resizable` | adds one element to that array |
| `@FreesArgument` | `defer_in_loop`, lifetime rules | an argument's memory is gone after the call (the first, or `@FreesArgument(1)`); needed only when the body frees part of it or keeps it reusable |
| `@Allocates` | `leaked_allocation`, `leaked_temporary` | the result is newly allocated and the caller owns it (bodiless only) |
| `@TakesByteCount` | `element_count_as_bytes` | the last argument is a byte count |
| `@Locks`, `@Unlocks` | `unpaired_lock` | takes or gives back the lock its first argument names; `@Locks` also declares an intentional wrapper |
| `@AssertsArgument` | `assert_constant` | stops the program unless its first argument is true |
| `@WritesOutput` | `print_in_module` | writes to standard output or error (bodiless only) |
| `@Exits` | `exit_in_module` | ends the process (bodiless only) |
| `@PrintLike`, `@FormatsToString` | `leaked_temporary`, `useless_format`, `format_push_string` | first string argument is a format string; the latter returns the text |
| `@AppendsString` | `format_push_string` | adds its second argument to the builder in its first |
| `@TempAllocates` | `temp_storage_escape` | the result lives in temporary storage, where the body can't show it |
| `@Opens`, `@Closes` | `unclosed_resource` | acquires or gives back a resource |

Notes marked "bodiless only" are read only on a declaration without a body, such as a `#foreign` one. Otherwise the linter works the effect out from the checked body and the bodies it calls: a procedure that returns `alloc`'s result allocates, one whose every path reaches `exit` exits, a macro that pushes a context or assigns into `context` changes it, and `a < b` choosing between its two arguments makes a minimum or maximum. A note on a procedure with a body is ignored for these, so the declaration and the body can't disagree.

Results are followed through struct fields, so a string or array view whose `data` came from `tprint` or `alloc` is temporary or owned like the pointer itself. A global that briefly holds temporary storage is fine when every path clears it before temporary storage is reset or the procedure returns:

```jai
saved_text: string;

ok :: () {
    defer saved_text = "";
    saved_text = tprint("%", 1);  // cleared on every path: no finding
}
```

Most lifetime effects are inferred from procedure bodies, so the notes below are only needed where a body can't say it: foreign procedures, sorts and other "may" effects, and logical meaning a body hides. They are read by the [lifetime rules](#lifetime-rules).

| Note | Meaning |
| --- | --- |
| `@MovesElements(insert,2)`, `(swap,i,j)`, `(reverse)`, `(sort)` | moves elements inside the array argument; pointers to moved slots now see other elements |
| `@SwapsArguments` | swaps what its two pointer arguments point to |
| `@RemovesElement(k)`, `@ResetsElements` | removes the entry with key argument `k`, or all entries (tables, pools) |
| `@BorrowsElement(c,k,r)`, `@ElementResult(c,k,r)` | the result (or result `r`) points at the entry for key `k` (or `unknown`) in container `c` |
| `@TemporaryStorageMark`, `@RestoresTemporaryStorage`, `@ResetsTemporaryStorage` | saves, restores or resets the temporary-storage watermark |
| `@NoLifetimeEffects` | a foreign or opaque procedure that frees, moves and keeps nothing it is given |
| `@PreservesUnpassedStorage` | a fixed, zero-argument C foreign declaration returning `s32`; preserves each allocation instance separately proved fresh and unreachable by the call, without publishing an alias |
| `@ValidUntilNextCall` | the returned pointer dies at the next call of the same procedure |
| `@RegistersBorrower(r,cb,data)`, `@StartsBorrower(r)`, `@CompletesBorrower(r)`, `@EndsBorrower(r)` | a worker `r` keeps `data` for callback `cb` from start until it completes or ends |

## Rules

Each rule has a category that sets its default level: `correctness` denies; `suspicious`, `complexity`, `style` and `perf` warn; `pedantic` and `restriction` are off. [The lint reference](../reference/lint-rules.md) lists every rule with its category, default and summary, followed by its `### <rule>` section from this page (generated by `lint_reference`; the docs site gives each rule a page). Every finding ends with a link to that page (`jailint::rule_url`), and the language server sends it as the diagnostic's `codeDescription`, which editors make clickable. Write those sections so they read alone: anchors to other rules stay working, other anchors point back here. A category name works wherever a rule name does (`-W pedantic`, `[lint.rules] pedantic = "warn"`); a rule's own entry beats its category, and later flags beat earlier ones.

Generated example fences declare which source and diagnostic belong together: `jai jacket-example/1 {"kind":"example"}` followed by `diagnostic jacket-result/1 {"kind":"diagnostic"}`. Corrected source uses `jai jacket-example/1 {"kind":"fixed"}`; an optional `note` records when a rule is off by default. Settings precede the example. These markers carry presentation metadata; the source files and rendered diagnostic bytes remain unchanged.

Change the emission in `crates/jailint/examples/lint_reference.rs` and the docs site's semantic fence consumer together. Regenerate the lint reference with `cargo run --locked -q -p jailint --example lint_reference > docs/reference/lint-rules.md` after refreshing the rule snapshots. The generator depends on `jailint::RULES`, this page, and `tests/lint/<rule>/example*`; it has no extra configuration.

Each rule's source (`crates/jailint/src/rules/<rule>.rs`) opens with a doc comment saying exactly when it fires, and `tests/lint/<rule>/` has its `bad.jai` and `good.jai`. A test requires a `### <rule>` section here, because editors link to the anchor.

### absurd_comparison
An unsigned value is never below zero, so counting down an unsigned `i` with `while i >= 0` never stops.

### almost_swapped
Two consecutive statements with side-effect-free sides of the same type.

### approx_constant
Needs at least four significant digits (`3.14` is a rough value on purpose) and a match where the literal stops, rounded or cut off.

### assert_constant
The call must be to a procedure with the `@AssertsArgument` note (the library's `assert`).

### autocast_in_arg
The call site then depends on the parameter types: changing `draw` to take a `float` instead of an `s32` changes what `xx` does at every call, and nothing there says so.

### autocast_used
`xx` takes its target type from the context, so the line does not say what the value becomes, and changing the type on the left silently changes the conversion on the right.

### bad_bit_mask
`x & m` has only bits of `m`, so a constant with a bit outside `m` is never equal to it; `x | m` always has the bits of `m`, so a constant missing one never equals it.

### banner_comment
A comment of three or more of `-=*#~_` and nothing else, line or block (`// ------`).

### bitwise_precedence
Jai gives all bitwise and shift operators one level above `*`, grouped left to right, so `1 << n - 1` is `(1 << n) - 1`.

### bool_comparison
Only when the other side is a non-constant `bool`.

### bool_to_int_with_if
Off by default, since many read the `ifx` more easily than the cast.

### broken_doc_link
Only comments directly above a declaration (or trailing a field or parameter) are read, and links resolve like the language server's.

### cognitive_complexity
Off by default. Scores how hard a procedure is to follow.

### constant_condition
Reads the compiler's reachability facts (`jaic::sema::reachability` and `jaic::ide::values`, the code the editor fades): reports the condition behind a branch that never runs, such as `if count < 5` inside `if count > 10`, or an `else` after a test that always holds.

### defer_frees_returned
The `defer` must be a single `@FreesArgument` call on a local.

### defer_in_loop
The loop body's scope ends every iteration, so `defer free(buffer)` frees an outer buffer after the first pass.

### detached_doc_comment
A top-level procedure, struct or enum with a blank line between its declaration and a comment above that reads as a sentence (three or more words.

### double_free
A value freed or closed on every path, then freed or closed again (`free(p); free(p);`, `close(f); close(f);`), through copies and wrappers. See [lifetime rules](#lifetime-rules).

### dropped_result
A declaration or assignment whose right side is one call that returns more values than the left side names, where the first dropped value, the one right after the last kept, is a `bool`: `value := parse(text);` quietly turns a failed parse into `0`.

### duplicate_condition
Conditions must be side-effect-free; a call in between restarts the comparison.

### element_count_as_bytes
Only when `xs.data` is another argument of the same call, so the count clearly describes that array.

### enum_member_prefix
Two or more members, all beginning with the enum's name once case and `_` are ignored (`Token_Kind` and `TOKEN_KIND_IDENT`) and each with something left after it.

### erasing_op
Left alone inside an index or array literal, where a `0 *` lines up with other rows.

### excessive_bools
A parameter counts when its type is written `bool`, or it has no type and defaults to `true` or `false`.

### excessive_nesting
Off by default. A level is a braced block or a `switch`; `else if` chains and `#if` add none.

### exit_in_module
A file belongs to a module when it or a directory above it holds a `module.jai`, with no `tests` or `test` folder in between (those hold programs that use the module).

### field_reassign_with_default
The fields not named in the literal keep their defaults, as in the declaration, and the values are set where the variable is made instead of in statements a later edit can separate from it.

### float_equality
Off by default: exact comparison is often deliberate.

### float_loop_counter
Adding `0.1` ten times does not give `1.0`, so whether the loop runs ten or eleven times depends on rounding error, which grows with the iteration count.

### format_push_string
Fires on a call of a procedure noted `@AppendsString` (the library's `append` of a string) whose string is a call of one noted `@FormatsToString` (`tprint`, `sprint`).

### identical_branches
Compared by tokens. Empty branches, `#if`, and the last link of an `else if` chain are left alone.

### identical_case_bodies
Off by default, since one case per row is sometimes the clearer table.

### identical_operands
Float `==`, `!=` and `-` are left alone (NaN tests), as are operator procedures.

### identity_op
`0 + x`, operations inside indexes/array literals and shifts by `0` are left alone (they line up with neighbours).

### if_not_else
Off by default, since putting the exceptional or shorter branch first is often right.

### impossible_comparison
Only integer expressions compared with integer literals, within one run of `&&` or `||`.

### incomplete_enum_switch
Lists the members no case mentions. Mark the switch `#complete` so the compiler keeps the list current, or add `case;` to name the fallback.

### index_only_loop
Becomes `for xs` (or `for *xs` when elements are written), other uses of `i` become `it_index`.

### infinite_loop
Condition reads only locals, parameters and literals; skipped for bodies with `break`, `return`, `#insert`, `#asm`, `using`, macro calls or taken addresses.

### int_plus_one
Integers only, and only the literal `1`. The editor offers the strict comparison; `--fix` leaves it, since where `y + 1` wraps (the top of the type) the two forms differ.

### integer_division_in_float
Operands are typed before the cast, so the integers divide first.

### large_stack_array
Locals live on the stack, which is small (a thread's is often 1 MiB or less) and shared with deep recursion, so a local of a megabyte can overflow it with no message beyond a crash.

### leaked_allocation
Off by default: ownership conventions differ, and a program that allocates once for its whole run may leak on purpose.

### leaked_temporary
A call of a procedure that allocates its result (`sprint`, `join`, `copy_string`, ...) whose result is used where it is read and dropped at once: an operand of a comparison, the receiver of `.count` or of an index, or a value argument of a `@PrintLike` call.

### loop_with_break_condition
A `while true` (macros aside) whose body starts with an `if` without an `else` that only does an unlabelled `break`, with or without braces.

### lossy_float_literal
A `float32` keeps about seven decimal digits. Digits past the shortest spelling that reads back as the same `float32` change nothing and suggest a precision the program does not have.

### lossy_xx
Off by default: narrowing with `xx` is common Jai style.

### magic_number
Off by default. Reports a decimal literal written directly as an operand of `+ - * / %` or a comparison, or as the value of an assignment (`x = 37;`, `x += 37;`).

### manual_array_find
Fires on a `for` over an array whose body is one `if` without an `else`: it compares the loop's value (`it`, or the name given to.

### manual_assign_op
Target must be side-effect-free and the operation must keep its type.

### manual_clamp
Numbers with constant bounds, the lower not above the upper (a swapped pair is `min_max`'s).

### manual_index_counter
`n := 0` right before a forward `for` (in a block, a `case` body or a `#if` branch), `n += 1` at the end of the body (or `defer n += 1`), `n` unused after the loop.

### manual_memcpy
Fires on a `for` over a range starting at `0` whose only statement is `dst[i] = src[i];` with `i` the loop's variable, `dst` and `src` different names or member paths, both arrays or pointers, and the same element type on both sides.

### min_max
Literal bounds only, with procedures that select the smaller or larger argument (the library's `min`/`max`).

### misrefactored_assign_op
The same operator on both sides, a side-effect-free target of a number type spelled the same in both places (`a op= a op b`), and for `+ * & | ^` on integers also `a op= b op a`.

### missing_doc
A procedure, struct, enum or union declared at the top level of a file while the scope is exported (the default, until a `#scope_file` or `#scope_module`) needs a doc: the comment directly above it.

### mixed_case_hex_literals
Read from the tokens, so it needs no types; literals in macros and generated files are skipped.

### modified_range_bound
Only locals named directly in a range end (`n` in `0..n-1`); `xs.count` bounds are left alone.

### needless_bool
A trailing `if c return true; return false;` is left alone when an earlier guard in the block also returns a literal.

### needless_continue
An unlabelled `continue` that ends a `for` or `while` body, or ends a branch of an `if` that is the body's last statement.

### neg_cmp_float
Every comparison with a NaN is false, so `!(a < b)` is true for NaN where `a >= b` is false.

### neg_multiply
Signed integers and floats only, and only the literals `-1` and `-1.0`.

### no_effect
An expression statement containing no call. The editor offers the likely intent: `==` becomes `=`, `x + 1;` becomes `x += 1;`, a procedure name gets `()`, and any other statement is removed.

### platform_leak
Code meant to build everywhere that imports a module asserting an operating system it is not built for (`#assert OS == .LINUX || OS == .MACOS` in `POSIX`), found only when someone builds for that system.

### pointer_into_resizable
A resizable array that outgrows its storage gets a new block and frees the old one, so a pointer taken into the old block dangles. Uses the [lifetime engine](#lifetime-rules); `pointer_lifetime` also covers removals and moves.

By default it reports growth that happens on every path. Strict mode also reports growth on one branch:

```jai
first := *names[0];
if verbose  array_add(*names, "grace");
print("%\n", first.*);   // strict: `first` may outlive the storage it refers to
```

See [Must and strict](../guides/memory-safety.md#must-and-strict) for the full example and output.

### pointer_lifetime
@experimental. Off by default (`-W pointer_lifetime` turns it on). The successor to `pointer_into_resizable`: it follows the checked control flow, so it sees every branch, loop, `defer`, labelled jump, `#if` choice and macro expansion the compiler took, per polymorphic instance.

It fires only when every path to a use of `p := *xs[i]` passes a call that grows `xs` after `p` was taken. What a procedure does to its arguments is inferred from its body and the bodies it calls (through allocator calls, wrappers and recursion), so `push :: (s: *Stack, v: int) { array_add(*s.items, v); }` counts as growing `s.items` with no note. A call grows only if it does so on every path that returns; one that grows only when out of capacity is a "may", which strict mode will report. `@GrowsArray` still works as a stated contract. When the growth is several calls deep, the finding lists the chain (up to seven calls, eliding the middle). A path that re-takes the pointer, replaces `xs`, or runs inline assembly makes the use unknown, and unknown is quiet. The finding points at the use, with labels at the derivation and at the last invalidating call on each path:

```
12 |     p := *xs[0];
   |          ------ pointer derived here
14 |     grow(*xs);
   |     --------- `xs` storage invalidated here
15 |     sink(p);
   |          ^ pointer used here after its storage was invalidated
```

The pointer is followed through copies: other locals, struct fields, pointer arithmetic, casts, and views or strings made from the array (`v: []int = xs;` then `v.data`). `*outer[0].inner[0]` points into the inner array, so growing `outer` leaves it alone; `*outer[0].inner` points into `outer` and is invalidated. A `@GrowsArray` call that returns a pointer into its argument (`p := array_add(*xs);`) counts as a fresh derivation. Reading `v.count` or replacing `v.data` after growth is not a use.

Anything reached through a pointer the lint can't resolve (a heap field a callee could rebind, an overlapping union field) is unknown, so it stays quiet. A callee that keeps or writes through a pointer argument in a way the lint can't follow makes only that argument unknown. `JAILINT_LIFETIME_STATS=1` prints per-file counts of bodies analysed and why others were skipped.

`@PreservesUnpassedStorage` is a conditional external boundary, not a purity
promise. Protected storage must be live, fresh and unpublished at the actual call;
absence from the argument list alone is insufficient. The declaration promises
that its complete transitive execution, including callbacks, preserves those
same allocation instances and their bytes and does not expose aliases to them.
It may still change its own globals, TLS and allocations. Independent release,
reset or other resource contracts prevent use of this promise. For example:

```jai
initialize_native :: () -> s32 #foreign native_library "initialize"; @PreservesUnpassedStorage
```

The checked declaration, selected library/symbol and lowered C ABI are retained
and revalidated before use. This does not select an allocator or turn an unknown
incoming context into the default context. To extend admission, update the
external-contract classifier and its stale-declaration/ABI tests together with
the consuming object-specific proof; do not treat it as an empty memory frame.

### Lifetime rules

`use_after_free`, `double_free`, `returned_local_address`, `temp_storage_escape`, `pointer_into_resizable`, `remove_in_for` and `pointer_lifetime` share one engine over the checked control flow. They differ only in which storage they watch: heap blocks, stack frames, temporary storage, array buffers, element slots, workers. Each finding labels where the pointer came from and what invalidated it.

**Must and strict.** By default (`must`) a rule fires only when every path invalidates. `strict` also reports "may" effects (growth only when full, conditional frees, sorts) and anything a foreign or unknown procedure could do to an argument it was given:

```toml
[lint.lifetimes]
mode = "strict"     # default "must"
```

`--lifetime-mode strict` overrides the file. Strict findings show one shortest path with its branch choices and how many other paths lead there ("infinitely many" around a loop).

**Indirect calls.** A procedure value with a known set of targets (a local, a constant, a factory's result) counts as calling all of them: an effect is certain only if every target has it. A parameter or a mutable global could be anything, so in must mode it proves nothing.

**Threads.** A worker given data with `thread_init`-style calls keeps it until it completes or is deinitialized. Freeing that data first, or letting it go out of scope, is a finding. Races are out of scope.

### print_in_module
A file belongs to a module when it or a directory above it holds a `module.jai`, with no `tests` or `test` folder in between (those hold programs that use the module).

### push_in_static_if
`#if` adds no scope, so `push_allocator(a);` directly in an `#if` or `#else` arm lasts to the end of the enclosing block or procedure, not the arm.

### range_past_count
Ranges include their end. Quiet when the body mentions `xs.count` (it may guard the index).

### redundant_alias
Style rule, deliberately narrow so the fix never changes behaviour: the two statements are adjacent with nothing (not even a comment) between, the first name appears exactly twice in the body, the types are identical, and no `#insert`, `using`, notes or flags are involved.

### redundant_cast
Aliases like `c_long` differ per platform, so the value must be declared with the same type spelling.

### redundant_deref
Fires when a member is taken from the dereference of an expression whose type is a pointer to a struct or union.

### remove_in_for
The loop steps past the element moved into the removed slot. Fires when a removal at the cursor reaches the loop's step on every path; `break`, `return` or writing the index yourself stops it. Removal inferred through a wrapper counts.

### returned_local_address
Covers locals, by-value parameters (the copy is the callee's), `for` variables and views of local fixed arrays, through copies, fields and `Any`. Literals the compiler keeps alive are fine.

### reversed_range
Counting down is `for < i: 0..10`. The fix changes behaviour, so it is editor-only.

### self_assignment
Not auto-fixed: the slip may be on the other side.

### shadowed_it
`row := it;` before the inner loop is the usual idiom and is not reported.

### shadowed_local
A variable declaration (`name := ...`, `name: T`) in a nested block, loop body or `case` body whose name is a parameter or a variable declared earlier in an enclosing block of the same procedure.

### shift_overflow
The result is `0` (or `-1` for `>>` on a negative signed value) whatever the operand holds, and the compiler accepts it.

### single_case_switch
One case of one value, plus at most a default case.

### string_literal_write
Literals are read-only and equal literals share storage, so the write is a runtime error (interpreter) or a crash (native).

### temp_storage_escape
Off by default: handing back a temporary string from a helper is a common, documented convention. Fires when temporary memory is kept past a reset or a restored mark (`auto_release_temp`) that frees it. Values baked at compile time are copied out, so a `tprint` in a constant is fine.

### todo_marker
A marker is `TODO`, `FIXME`, `XXX` or `HACK` in capitals, standing alone (`TODO:` and `(FIXME)` count; `todo`, `TODOS` and `AUTODOC` do not).

### too_many_arguments
Every parameter counts, defaulted, baked (`$T`) and variadic ones included.

### unchecked_directive
Each `#no_abc`, `#no_aoc` and `#asm` needs a comment on its own line or the line just above.

### unclosed_resource
A local that an `@Opens` call acquired something for, and that nothing closes.

### uninit_read
The first mention after the declaration, in the same block, decides; a write anywhere (even inside an `if`) ends the check, so only straight-line reads are reported.

### unknown_allow
A name in a line comment's `allow(...)` or `allow-file(...)` list that is not a rule, a category or `all`.

### unnecessary_force_cast
`,force` reinterprets the bits of a number as a float (or back) or of an aggregate as another of the same size, and also skips the range check a plain cast makes when it narrows an integer.

### unnecessary_semicolon
An empty statement in a block, a `case`, a procedure body or at the top level of a file.

### unpaired_lock
A top-level acquisition whose following paths leave recursive lock levels held. Native-backed wrappers can supply inferred depth effects; `@Locks` and `@Unlocks` cover other protocols. Each acquired level needs a release or proven deferred release. `@Locks` declares an intentional wrapper exemption; an inferred acquisition describes an effect but does not exempt the function. Unknown calls receiving the same argument remain uncertain.

### unreadable_literal
Off by default: the standard library alone has thousands of such constants.

### unsigned_range_end_minus_one
Ranges include their end, so `0..n-1` visits the `n` indices below `n`, but with an unsigned `n` of `0` the end wraps around (or stops the program with overflow checks on) instead of giving an empty loop.

### unused_import
Unnamed imports count as used when any lookup went through them, or (guard for unchecked code) when the module's files mention a name it exports.

### unused_parameter
Only for procedures whose every caller is in view: named, only called (never taken as a value), called at least once, not overloaded or exported, no `#c_call`/`#foreign`/`#expand`/notes.

### unused_private_declaration
A procedure, struct, enum, constant or variable declared after `#scope_file` whose name appears nowhere else in its file (mentions in its own declaration do not count, so a procedure that only calls itself is unused; mentions in code an `#if` left out do).

### unused_result
Procedures declared with `@UseResult` (many in `Basic`/`String`/`Math`), so a program's own `trim` is left alone unless it carries the note.

### unused_suppression
A name in a `// jailint: allow(...)`, `allow-file(...)` or `@jailint_allow(...)` that no finding was silenced by, reported on the name (or on the whole note).

### unused_variable
A name that appears anywhere else in its block is never reported (covers `#if`-excluded code and `#asm`).

### use_after_free
A pointer read after every path to it freed its storage: `free(p)`, `array_free(*xs)` then `xs.data`, a wrapper that frees, a `defer` that already ran. See [lifetime rules](#lifetime-rules).

### useless_format
Fires on a call of a procedure noted `@FormatsToString` (the library's `tprint` and `sprint`) whose format is exactly `"%"` and whose single value is a `string`.

### useless_resizable
A resizable array allocates and has to be freed; a handful of elements known up front fit in a fixed array on the stack (`parts: [2] string`).

### using_pollution
A `using x;` statement, `using x: T;` declaration or `using` parameter whose struct (its own members and those of structs it `using`s, through a pointer too).

### wrapping_constant
A constant takes the other operand's type, so the first divides `-41` by `h`.

### zero_ptr
Only `cast(*T) 0` (also `0x0`) without cast modifiers.

### Left out on purpose

`never_loop`, `collapsible_if`, `needless_return`, `redundant_else`, `manual_swap`, `len_zero`, `cmp_null`, `nonminimal_bool` (idiomatic or no better in Jai), `cast_possible_truncation` and `cast_sign_loss` (covered by `lossy_xx`), `out_of_bounds_indexing` (the compiler checks constant indices), `suspicious_assignment_formatting` (`jaifmt` normalizes it).

## Suppression

```jai
// jailint: allow(unused_variable)        covers the next line with code
x := compute(); // jailint: allow(all)     after code: its own line
// jailint: allow-file(float_equality)     whole file

callback :: (event: int, data: *void) {
    print("%\n", event);
} @jailint_allow(unused_parameter)        // after a declaration: all of it
```

Rules are comma-separated; `all` means every rule. An unknown name is reported by `unknown_allow`, one that silences nothing by `unused_suppression`. A `_` prefix marks an unused variable or parameter as intentional.

## Configuration

The `[lint]` table of `jacket.toml`, found by walking up from the first path (or `--config`). Files from other listed roots use their own nearest `jacket.toml`. `-A`/`-W`/`-D` override the file. Unknown settings warn; unknown rules or levels are errors so a typo cannot leave a rule on.

```toml
[lint]
exclude = ["tests/corpus/negative/**", "generated/**"]   # patterns relative to this file
max_nesting = 6                                          # blocks `excessive_nesting` allows
max_complexity = 25                                      # score `cognitive_complexity` allows
max_arguments = 7                                        # parameters `too_many_arguments` allows
max_bools = 3                                            # `bool` parameters `excessive_bools` allows
max_stack_array_bytes = 65536                            # bytes a local array may take
magic_number_allow = [60, 24]                            # numbers `magic_number` accepts
magic_number_min = 3                                     # whole numbers below this are fine
magic_number_in_tests = false                            # also check `tests`/`test` folders

[lint.rules]
excessive_nesting = "warn"
float_equality = "warn"
unused_parameter = "allow"

[[lint.overrides]]                                       # levels for some paths only
paths = ["tests/**"]
rules = { float_equality = "allow", magic_number = "allow" }
```

The last matching override naming the rule (or its category) beats `[lint.rules]`; command-line flags beat both. Other settings are not per path.

`[lint.portability]` tells `platform_leak` which code must build everywhere. A file is checked for a system unless the most specific `platforms` glob matching it leaves the system out, or a `// jailint: platforms(linux, macos)` comment before its first code line does.

```toml
[lint.portability]
targets = ["linux", "macos", "windows"]                  # the default
portable = ["stdlib/**"]                                 # patterns that must build on every target

[lint.portability.platforms]                             # files limited to some systems
"stdlib/POSIX/**" = ["linux", "macos"]
"stdlib/Windows/**" = ["windows"]
"stdlib/Android/**" = []                                 # none of the three
```

Path patterns: `*` and `?` stay within a path component, `**` crosses components, `[abc]`, `[a-z]`, `[!abc]` match one character, `{a,b}` alternates, `\` escapes. A malformed pattern is a configuration error. The definition is `PathPattern` in `crates/jacket-config/src/pattern.rs`; the formatter has its own matcher in `stdlib/Extensions/Jai_Format/config.jai`. Both run `tests/glob/cases.txt` (`pattern<TAB>path<TAB>match|nomatch`), so add a case there when changing either.

The repository's own `jacket.toml` excludes negative compiler cases, lint `bad` cases and fuzzer seeds, and CI runs `just lint` over `stdlib examples tools jaifmt tests benchmarks`. Deliberate findings in tests carry an `allow` with the reason.

## How to change it

**Add a rule:**

1. Create `crates/jailint/src/rules/<name>.rs` with a doc comment (what, why, exactly when), `pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>)` and `pub(crate) const INFO: super::RuleInfo` (name, default level, summary).
2. Add `<name>` to the `rules!` list in `rules/mod.rs`. The editor schema build reads the same list.
3. Add `tests/lint/<name>/bad.jai` and `good.jai` (whole programs with `main`), then run `INSTA_UPDATE=always cargo test -p jailint` and read the output.
   Also add `example.jai`, the 5 to 15 line program the rule's page shows: realistic code with exactly one finding of the rule (the test enforces it, and writes `example.snap`). `example-fixed.jai` is the same code fixed, and must produce nothing. An `example.toml` (a `jacket.toml`) holds settings the example needs, such as a lower `max_complexity`. The page embeds the code and `example.snap` when the reference is generated, so the diagnostic is never typed by hand; an off-by-default rule is linted with itself enabled.
4. Run it over the repository and the upstream corpus with `-W <name>` and check every hit. A rule that is sometimes wrong costs more trust than it earns.
5. Add a `### <name>` section here, then regenerate the reference: `PYTHONPATH=tools python3 tools/generators/generate_reference_docs.py`.

Inside a rule: `cx.procs` lists bodies (trust facts only for `p.clean` / `p.typed()`); `cx.ty`, `cx.constant`, `cx.entities(span)` and friends read facts ([IDE facts](../compiler/ide-facts.md)); `cx.mentions` is the textual guard for unchecked code. A fix is byte edits; mark it `machine_applicable` only if it cannot change behaviour and the result still compiles. New facts go in `IdeFacts` (`crates/jaic/src/ide/facts.rs`) behind `lint`.

Gotchas:

- Facts are recorded only for files under the root's directory.
- A body that failed in any polymorph instance is not clean.
- A metaprogram needing a foreign call the sandbox lacks stops there; only what compiled is linted (`-v` shows the root).

## Dependencies

`crates/jaic` (parser, checker with lint facts, sandboxed host). `crates/jai-language-server` depends on `jailint`.

Lifetime implementation and profiling: [lifetime analysis](../development/lifetime-analysis.md). Receiver-field cleanup follows the allocation loaded at the call: replacing a field before freeing it does not free the field's original storage. Conditional replacement retains possible origins without claiming that either one is always freed.
