# Lint rules

The 105 rules `jailint` checks, by category. [jailint](../tools/jailint.md) says how to run it, configure rules and silence a finding.

A rule's category sets its default level. Name a category wherever a rule name works (`[lint.rules]`, `-W`, `// jailint: allow(...)`).

| Category | Default | What |
| --- | --- | --- |
| [`correctness`](#correctness) | deny | code that is wrong: it does not do what it says |
| [`suspicious`](#suspicious) | warn | code that is most likely wrong, or does nothing |
| [`complexity`](#complexity) | warn | code that says something in a roundabout way |
| [`style`](#style) | warn | code that works but is not written the usual way |
| [`perf`](#perf) | warn | code that could be faster |
| [`pedantic`](#pedantic) | allow | stricter checks with more false positives |
| [`restriction`](#restriction) | allow | checks that forbid things a project may choose to avoid |

## Correctness

Code that is wrong: it does not do what it says. Default: `deny`.

### absurd_comparison

Finds a comparison an unsigned value always passes or fails (`u >= 0`).

Category `correctness`, `deny` by default.

An unsigned value is never below zero, so counting down an unsigned `i` with `while i >= 0` never stops.

### almost_swapped

Finds `a = b; b = a;`, a swap that sets both to `b`.

Category `correctness`, `deny` by default.

Two consecutive statements with side-effect-free sides of the same type.

### bad_bit_mask

Finds a mask test that can never match (`(x & 4) == 1`, `(x | 2) == 0`).

Category `correctness`, `deny` by default.

`x & m` has only bits of `m`, so a constant with a bit outside `m` is never equal to it; `x | m` always has the bits of `m`, so a constant missing one never equals it.

### defer_frees_returned

Finds `defer free(p);` followed by `return p;`, which hands back freed memory.

Category `correctness`, `deny` by default.

The `defer` must be a single `@FreesArgument` call on a local.

### double_free

Finds two `@FreesArgument` calls on the same variable with nothing in between.

Category `correctness`, `deny` by default.

A value freed or closed on every path, then freed or closed again (`free(p); free(p);`, `close(f); close(f);`), through copies and wrappers. See [lifetime rules](../tools/jailint.md#lifetime-rules).

### element_count_as_bytes

Finds `memcpy(d, xs.data, xs.count)` with elements wider than a byte.

Category `correctness`, `deny` by default.

Only when `xs.data` is another argument of the same call, so the count clearly describes that array.

### impossible_comparison

Finds comparisons of one integer that never pass together (`x > 5 && x < 3`).

Category `correctness`, `deny` by default.

Only integer expressions compared with integer literals, within one run of `&&` or `||`.

### min_max

Finds `min(0, max(100, x))`, a clamp that is always one bound.

Category `correctness`, `deny` by default.

Literal bounds only, with `@ReturnsSmaller`/`@ReturnsLarger` procedures (the library's `min`/`max`).

### misrefactored_assign_op

Finds `a += a + b`, an update that applies the operator to `a` twice.

Category `correctness`, `deny` by default.

The same operator on both sides, a side-effect-free target of a number type spelled the same in both places (`a op= a op b`), and for `+ * & | ^` on integers also `a op= b op a`.

### range_past_count

Finds `for i: 0..xs.count` indexing `xs[i]`, one past the end.

Category `correctness`, `deny` by default.

Ranges include their end. Quiet when the body mentions `xs.count` (it may guard the index).

### remove_in_for

Finds removing from an array while a `for` walks it forward.

Category `correctness`, `deny` by default.

The loop steps past the element moved into the removed slot. Fires when a removal at the cursor reaches the loop's step on every path; `break`, `return` or writing the index yourself stops it. Removal inferred through a wrapper counts.

### returned_local_address

Finds `return *x;` with `x` a local, a pointer into a stack frame that is gone.

Category `correctness`, `deny` by default.

Covers locals, by-value parameters (the copy is the callee's), `for` variables and views of local fixed arrays, through copies, fields and `Any`. Literals the compiler keeps alive are fine.

### reversed_range

Finds a range whose start is above its end, which never runs.

Category `correctness`, `deny` by default.

Counting down is `for < i: 0..10`. The fix changes behaviour, so it is editor-only.

### string_literal_write

Finds a write into a string literal, which is read-only (`s := "abc"; s[0] = x;`).

Category `correctness`, `deny` by default.

Literals are read-only and equal literals share storage, so the write is a runtime error (interpreter) or a crash (native).

### uninit_read

Finds a variable declared `= ---` read before anything is written to it.

Category `correctness`, `deny` by default.

The first mention after the declaration, in the same block, decides; a write anywhere (even inside an `if`) ends the check, so only straight-line reads are reported.

### use_after_free

Finds a variable read after a `@FreesArgument` call freed it (`free(p); use(p);`).

Category `correctness`, `deny` by default.

A pointer read after every path to it freed its storage: `free(p)`, `array_free(*xs)` then `xs.data`, a wrapper that frees, a `defer` that already ran. See [lifetime rules](../tools/jailint.md#lifetime-rules).

### wrapping_constant

Finds a constant that wraps to the type the other operand gives it (`(0xffff_ffff - 40) / h`).

Category `correctness`, `deny` by default.

A constant takes the other operand's type, so the first divides `-41` by `h`.

## Suspicious

Code that is most likely wrong, or does nothing. Default: `warn`.

### bitwise_precedence

Finds a bitwise operator that binds differently than in C (`1 << n - 1`).

Category `suspicious`, `warn` by default.

Jai gives all bitwise and shift operators one level above `*`, grouped left to right, so `1 << n - 1` is `(1 << n) - 1`.

### constant_condition

Finds an `if` or `while` whose condition the compiler shows is always true or false.

Category `suspicious`, `warn` by default.

Reads the compiler's reachability facts (`jaic::sema::reachability` and `jaic::ide::values`, the code the editor fades): reports the condition behind a branch that never runs, such as `if count < 5` inside `if count > 10`, or an `else` after a test that always holds.

### defer_in_loop

Finds a `defer` in a loop body that cleans up something from outside the loop.

Category `suspicious`, `warn` by default.

The loop body's scope ends every iteration, so `defer free(buffer)` frees an outer buffer after the first pass.

### dropped_result

Finds a call drops the `bool` that follows the values it keeps (`n := to_integer(s);`).

Category `suspicious`, `warn` by default.

A declaration or assignment whose right side is one call that returns more values than the left side names, where the first dropped value, the one right after the last kept, is a `bool`: `value := parse(text);` quietly turns a failed parse into `0`.

### duplicate_condition

Finds an `else if` that repeats an earlier condition.

Category `suspicious`, `warn` by default.

Conditions must be side-effect-free; a call in between restarts the comparison.

### erasing_op

Finds an operation that is always `0` (`x * 0`, `x & 0`).

Category `suspicious`, `warn` by default.

Left alone inside an index or array literal, where a `0 *` lines up with other rows.

### identical_branches

Finds an `if`/`else` or `ifx` whose branches are the same.

Category `suspicious`, `warn` by default.

Compared by tokens. Empty branches, `#if`, and the last link of an `else if` chain are left alone.

### identical_operands

Finds an operator with the same expression on both sides (`a == a`).

Category `suspicious`, `warn` by default.

Float `==`, `!=` and `-` are left alone (NaN tests), as are operator procedures.

### infinite_loop

Finds a `while` whose condition nothing in the loop changes.

Category `suspicious`, `warn` by default.

Condition reads only locals, parameters and literals; skipped for bodies with `break`, `return`, `#insert`, `#asm`, `using`, macro calls or taken addresses.

### integer_division_in_float

Finds an integer division whose result is used as a float.

Category `suspicious`, `warn` by default.

Operands are typed before the cast, so the integers divide first.

### leaked_temporary

Finds an `@Allocates` result that is only compared, measured, indexed or printed, so nothing can free it.

Category `suspicious`, `warn` by default.

A call of an `@Allocates` procedure (`sprint`, `join`, `copy_string`, ...) whose result is used where it is read and dropped at once: an operand of a comparison, the receiver of `.count` or of an index, or a value argument of a `@PrintLike` call.

### modified_range_bound

Finds a variable bounding a `for` range assigned in the loop, which does not change it.

Category `suspicious`, `warn` by default.

Only locals named directly in a range end (`n` in `0..n-1`); `xs.count` bounds are left alone.

### no_effect

Finds a statement that computes a value and drops it (`x == 5;`).

Category `suspicious`, `warn` by default.

An expression statement containing no call. The editor offers the likely intent: `==` becomes `=`, `x + 1;` becomes `x += 1;`, a procedure name gets `()`, and any other statement is removed.

### pointer_into_resizable

Finds a pointer into a resizable array used after a `@GrowsArray` call may have moved the array (`p := *xs[i]; array_add(*xs, v); use(p);`).

Category `suspicious`, `warn` by default.

A resizable array that outgrows its storage gets a new block and frees the old one, so a pointer taken into the old block dangles. Uses the [lifetime engine](../tools/jailint.md#lifetime-rules); `pointer_lifetime` also covers removals and moves.

### push_in_static_if

Finds a scoped push directly in an `#if` arm, which lasts past the arm.

Category `suspicious`, `warn` by default.

`#if` adds no scope, so `push_allocator(a);` directly in an `#if` or `#else` arm lasts to the end of the enclosing block or procedure, not the arm.

### self_assignment

Finds a value assigned to itself (`x = x;`).

Category `suspicious`, `warn` by default.

Not auto-fixed: the slip may be on the other side.

### shadowed_it

Finds a nested `for` hides an `it` the enclosing loop uses.

Category `suspicious`, `warn` by default.

`row := it;` before the inner loop is the usual idiom and is not reported.

### shift_overflow

Finds a shift by at least the bit width of its operand (`x << 40` with `x: s32`).

Category `suspicious`, `warn` by default.

The result is `0` (or `-1` for `>>` on a negative signed value) whatever the operand holds, and the compiler accepts it.

### unclosed_resource

Finds a local opened by an `@Opens` call (`file_open`, `append(*builder, ..)`) that nothing closes.

Category `suspicious`, `warn` by default.

A local that an `@Opens` call acquired something for, and that nothing closes.

### unknown_allow

Finds a `// jailint: allow(...)` comment naming a rule that does not exist.

Category `suspicious`, `warn` by default.

A name in a line comment's `allow(...)` or `allow-file(...)` list that is not a rule, a category or `all`.

### unpaired_lock

Finds a `@Locks` call with a path that returns or ends without a matching `@Unlocks`.

Category `suspicious`, `warn` by default.

A statement-level call of a `@Locks` procedure with an argument (`lock(*state.mutex);`) at the top level of a procedure body, followed by a path that reaches a.

### unsigned_range_end_minus_one

Finds `for 0..n-1` with an unsigned `n`, which wraps around when `n` is 0.

Category `suspicious`, `warn` by default.

Ranges include their end, so `0..n-1` visits the `n` indices below `n`, but with an unsigned `n` of `0` the end wraps around (or stops the program with overflow checks on) instead of giving an empty loop.

### unused_private_declaration

Finds a `#scope_file` declaration that nothing in its file uses.

Category `suspicious`, `warn` by default.

A procedure, struct, enum, constant or variable declared after `#scope_file` whose name appears nowhere else in its file (mentions in its own declaration do not count, so a procedure that only calls itself is unused; mentions in code an `#if` left out do).

### unused_result

Finds a call to a `@UseResult` procedure, as a statement.

Category `suspicious`, `warn` by default.

Procedures declared with `@UseResult` (many in `Basic`/`String`/`Math`), so a program's own `trim` is left alone unless it carries the note.

### unused_suppression

Finds a name in a `// jailint: allow(...)` comment or `@jailint_allow` note that silences nothing.

Category `suspicious`, `warn` by default.

A name in a `// jailint: allow(...)`, `allow-file(...)` or `@jailint_allow(...)` that no finding was silenced by, reported on the name (or on the whole note).

## Complexity

Code that says something in a roundabout way. Default: `warn`.

### bool_comparison

Finds comparing a bool with `true` or `false`.

Category `complexity`, `warn` by default.

Only when the other side is a non-constant `bool`.

### identity_op

Finds an operation that changes nothing (`x + 0`, `x * 1`).

Category `complexity`, `warn` by default.

`0 + x`, operations inside indexes/array literals and shifts by `0` are left alone (they line up with neighbours).

### index_only_loop

Finds `for i: 0..xs.count-1` where `i` only indexes `xs`.

Category `complexity`, `warn` by default.

Becomes `for xs` (or `for *xs` when elements are written), other uses of `i` become `it_index`.

### int_plus_one

Finds `x >= y + 1`, which is `x > y`.

Category `complexity`, `warn` by default.

Integers only, and only the literal `1`. The editor offers the strict comparison; `--fix` leaves it, since where `y + 1` wraps (the top of the type) the two forms differ.

### loop_with_break_condition

Finds `while true { if !c break; ... }`, a loop whose condition is its first statement.

Category `complexity`, `warn` by default.

A `while true` (macros aside) whose body starts with an `if` without an `else` that only does an unlabelled `break`, with or without braces.

### manual_array_find

Finds a loop that searches an array and breaks with a found flag or index, which `array_find` does.

Category `complexity`, `warn` by default.

Fires on a `for` over an array whose body is one `if` without an `else`: it compares the loop's value (`it`, or the name given to.

### manual_assign_op

Finds `a = a + b` instead of `a += b`.

Category `complexity`, `warn` by default.

Target must be side-effect-free and the operation must keep its type.

### manual_clamp

Finds a value kept between two constant bounds by hand instead of with `clamp`.

Category `complexity`, `warn` by default.

Numbers with constant bounds, the lower not above the upper (a swapped pair is `min_max`'s).

### manual_index_counter

Finds a counter incremented in a `for` loop that always equals `it_index`.

Category `complexity`, `warn` by default.

`n := 0` right before a forward `for` (in a block, a `case` body or a `#if` branch), `n += 1` at the end of the body (or `defer n += 1`), `n` unused after the loop.

### needless_bool

Finds `ifx c then true else false` and `if c return true; else return false;`.

Category `complexity`, `warn` by default.

A trailing `if c return true; return false;` is left alone when an earlier guard in the block also returns a literal.

### redundant_alias

Finds a local that only exists to be copied into another local or returned.

Category `complexity`, `warn` by default.

Style rule, deliberately narrow so the fix never changes behaviour: the two statements are adjacent with nothing (not even a comment) between, the first name appears exactly twice in the body, the types are identical, and no `#insert`, `using`, notes or flags are involved.

### redundant_cast

Finds a cast to the type the value already has.

Category `complexity`, `warn` by default.

Aliases like `c_long` differ per platform, so the value must be declared with the same type spelling.

### redundant_deref

Finds `p.*.field` where `p.field` means the same.

Category `complexity`, `warn` by default.

Fires when a member is taken from the dereference of an expression whose type is a pointer to a struct or union.

### unnecessary_force_cast

Finds `cast,force` where the value already has the type or the conversion widens.

Category `complexity`, `warn` by default.

`,force` reinterprets the bits of a number as a float (or back) or of an aggregate as another of the same size, and also skips the range check a plain cast makes when it narrows an integer.

### useless_format

Finds `tprint("%", s)` or `sprint("%", s)` with a string `s`, which only copies it.

Category `complexity`, `warn` by default.

Fires on a call of a procedure noted `@FormatsToString` (the library's `tprint` and `sprint`) whose format is exactly `"%"` and whose single value is a `string`.

## Style

Code that works but is not written the usual way. Default: `warn`.

### approx_constant

Finds a hand-typed `3.14159` or `1.4142` where `Math` has the constant.

Category `style`, `warn` by default.

Needs at least four significant digits (`3.14` is a rough value on purpose) and a match where the literal stops, rounded or cut off.

### assert_constant

Finds `assert(true)` or `#assert true`, which can never fail.

Category `style`, `warn` by default.

The call must be to a procedure with the `@AssertsArgument` note (the library's `assert`).

### banner_comment

Finds a comment that is only a line of `-`, `=` or `*` fencing off a section.

Category `style`, `warn` by default.

A comment of three or more of `-=*#~_` and nothing else, line or block (`// ------`).

### broken_doc_link

Finds a link in a doc comment (`[name]`) that names no declaration.

Category `style`, `warn` by default.

Only comments directly above a declaration (or trailing a field or parameter) are read, and links resolve like the language server's.

### enum_member_prefix

Finds every member of an enum starts with the enum's name.

Category `style`, `warn` by default.

Two or more members, all beginning with the enum's name once case and `_` are ignored (`Token_Kind` and `TOKEN_KIND_IDENT`) and each with something left after it.

### lossy_float_literal

Finds a `float32` literal with more digits than a `float32` keeps (`31415.92653`).

Category `style`, `warn` by default.

A `float32` keeps about seven decimal digits. Digits past the shortest spelling that reads back as the same `float32` change nothing and suggest a precision the program does not have.

### mixed_case_hex_literals

Finds a hex literal with both upper and lower case digits (`0xDeadbeef`).

Category `style`, `warn` by default.

Read from the tokens, so it needs no types; literals in macros and generated files are skipped.

### needless_continue

Finds a `continue` as the last thing a loop body does.

Category `style`, `warn` by default.

An unlabelled `continue` that ends a `for` or `while` body, or ends a branch of an `if` that is the body's last statement.

### neg_multiply

Finds `x * -1`, which is `-x`.

Category `style`, `warn` by default.

Signed integers and floats only, and only the literals `-1` and `-1.0`.

### single_case_switch

Finds a `switch` with a single case, which is an `if`.

Category `style`, `warn` by default.

One case of one value, plus at most a default case.

### unnecessary_semicolon

Finds a `;` that ends nothing (`x = 1;;`, `if c { ... };`).

Category `style`, `warn` by default.

An empty statement in a block, a `case`, a procedure body or at the top level of a file.

### unused_import

Finds an `#import` nothing in its scope uses.

Category `style`, `warn` by default.

Unnamed imports count as used when any lookup went through them, or (guard for unchecked code) when the module's files mention a name it exports.

### unused_parameter

Finds a parameter the procedure never uses.

Category `style`, `warn` by default.

Only for procedures whose every caller is in view: named, only called (never taken as a value), called at least once, not overloaded or exported, no `#c_call`/`#foreign`/`#expand`/notes.

### unused_variable

Finds a local variable that is never used.

Category `style`, `warn` by default.

A name that appears anywhere else in its block is never reported (covers `#if`-excluded code and `#asm`).

### zero_ptr

Finds `cast(*T) 0`, which is `null`.

Category `style`, `warn` by default.

Only `cast(*T) 0` (also `0x0`) without cast modifiers.

## Perf

Code that could be faster. Default: `warn`.

### format_push_string

Finds `append(*b, tprint(...))`, which formats into a new string to add it to a builder.

Category `perf`, `warn` by default.

Fires on a call of a procedure noted `@AppendsString` (the library's `append` of a string) whose string is a call of one noted `@FormatsToString` (`tprint`, `sprint`).

### manual_memcpy

Finds `for i: 0..n-1 { dst[i] = src[i]; }`, an element-by-element copy `memcpy` does at once.

Category `perf`, `warn` by default.

Fires on a `for` over a range starting at `0` whose only statement is `dst[i] = src[i];` with `i` the loop's variable, `dst` and `src` different names or member paths, both arrays or pointers, and the same element type on both sides.

### useless_resizable

Finds a local `[..] T` that is only filled with a known number of `array_add` calls and never resized.

Category `perf`, `warn` by default.

A resizable array allocates and has to be freed; a handful of elements known up front fit in a fixed array on the stack (`parts: [2] string`).

## Pedantic

Stricter checks with more false positives. Default: `allow`.

### bool_to_int_with_if

Finds `ifx c then 1 else 0`, a `bool` turned into an integer with a branch.

Category `pedantic`, `allow` by default.

Off by default, since many read the `ifx` more easily than the cast.

### cognitive_complexity

Finds a procedure whose branching scores above `max_complexity` (default 25).

Category `pedantic`, `allow` by default.

Off by default. Scores how hard a procedure is to follow.

### detached_doc_comment

Finds a comment that reads as a declaration's docs but has a blank line between it and the declaration.

Category `pedantic`, `allow` by default.

A top-level procedure, struct or enum with a blank line between its declaration and a comment above that reads as a sentence (three or more words.

### excessive_nesting

Finds blocks nested deeper than `max_nesting` (default 6).

Category `pedantic`, `allow` by default.

Off by default. A level is a braced block or a `switch`; `else if` chains and `#if` add none.

### field_reassign_with_default

Finds `x: T; x.a = 1; x.b = 2;` where a struct literal `x := T.{ a = 1, b = 2 }` sets the fields at the declaration.

Category `pedantic`, `allow` by default.

The fields not named in the literal keep their defaults, as in the declaration, and the values are set where the variable is made instead of in statements a later edit can separate from it.

### float_equality

Finds `==` or `!=` between two computed floats.

Category `pedantic`, `allow` by default.

Off by default: exact comparison is often deliberate.

### float_loop_counter

Finds a `while` loop counted in floats (`while t < 1.0 { t += 0.1; }`).

Category `pedantic`, `allow` by default.

Adding `0.1` ten times does not give `1.0`, so whether the loop runs ten or eleven times depends on rounding error, which grows with the iteration count.

### identical_case_bodies

Finds two cases of a `switch` with the same body, which could list their values together.

Category `pedantic`, `allow` by default.

Off by default, since one case per row is sometimes the clearer table.

### if_not_else

Finds an `if` or `ifx` with a negated condition and an `else`.

Category `pedantic`, `allow` by default.

Off by default, since putting the exceptional or shorter branch first is often right.

### incomplete_enum_switch

Finds a `switch` on an enum that neither covers every member nor has a default case.

Category `pedantic`, `allow` by default.

Lists the members no case mentions. Mark the switch `#complete` so the compiler keeps the list current, or add `case;` to name the fallback.

### large_stack_array

Finds a local fixed array bigger than `max_stack_array_bytes` (default 64 KiB).

Category `pedantic`, `allow` by default.

Locals live on the stack, which is small (a thread's is often 1 MiB or less) and shared with deep recursion, so a local of a megabyte can overflow it with no message beyond a crash.

### leaked_allocation

Finds a local holding memory from an `@Allocates` call that is never freed, returned or handed on.

Category `pedantic`, `allow` by default.

Off by default: ownership conventions differ, and a program that allocates once for its whole run may leak on purpose.

### lossy_xx

Finds `xx` that narrows a number to a smaller type.

Category `pedantic`, `allow` by default.

Off by default: narrowing with `xx` is common Jai style.

### magic_number

Finds an unexplained number in arithmetic, a comparison or an assignment.

Category `pedantic`, `allow` by default.

Off by default. Reports a decimal literal written directly as an operand of `+ - * / %` or a comparison, or as the value of an assignment (`x = 37;`, `x += 37;`).

### neg_cmp_float

Finds `!(a < b)` on floats, which differs from `a >= b` when a value is NaN.

Category `pedantic`, `allow` by default.

Every comparison with a NaN is false, so `!(a < b)` is true for NaN where `a >= b` is false.

### shadowed_local

Finds a local variable named like a parameter or another local in an enclosing scope of the same procedure.

Category `pedantic`, `allow` by default.

A variable declaration (`name := ...`, `name: T`) in a nested block, loop body or `case` body whose name is a parameter or a variable declared earlier in an enclosing block of the same procedure.

### temp_storage_escape

Finds a `@TempAllocates` result (`tprint`, `talloc`) stored in a global or through a pointer, or returned.

Category `pedantic`, `allow` by default.

Off by default: handing back a temporary string from a helper is a common, documented convention. Fires when temporary memory is kept past a reset or a restored mark (`auto_release_temp`) that frees it. Values baked at compile time are copied out, so a `tprint` in a constant is fine.

### unreadable_literal

Finds a long number without `_` separators (`1000000`, `0xdeadbeef`).

Category `pedantic`, `allow` by default.

Off by default: the standard library alone has thousands of such constants.

### using_pollution

Finds a `using` of a struct whose members share names with parameters, locals or globals already in scope.

Category `pedantic`, `allow` by default.

A `using x;` statement, `using x: T;` declaration or `using` parameter whose struct (its own members and those of structs it `using`s, through a pointer too).

## Restriction

Checks that forbid things a project may choose to avoid. Default: `allow`.

### autocast_in_arg

Finds `xx`, the automatic cast, written as an argument of a call.

Category `restriction`, `allow` by default.

The call site then depends on the parameter types: changing `draw` to take a `float` instead of an `s32` changes what `xx` does at every call, and nothing there says so.

### autocast_used

Finds `xx`, the automatic cast, outside a call's arguments.

Category `restriction`, `allow` by default.

`xx` takes its target type from the context, so the line does not say what the value becomes, and changing the type on the left silently changes the conversion on the right.

### excessive_bools

Finds a procedure with more `bool` parameters than `max_bools` (default 3).

Category `restriction`, `allow` by default.

A parameter counts when its type is written `bool`, or it has no type and defaults to `true` or `false`.

### exit_in_module

Finds a module ends the process.

Category `restriction`, `allow` by default.

A file belongs to a module when it or a directory above it holds a `module.jai`, with no `tests` or `test` folder in between (those hold programs that use the module).

### missing_doc

Finds an exported procedure, struct or enum with no doc comment.

Category `restriction`, `allow` by default.

A procedure, struct, enum or union declared at the top level of a file while the scope is exported (the default, until a `#scope_file` or `#scope_module`) needs a doc: the comment directly above it.

### platform_leak

Finds portable code imports something that asserts an operating system it is not built for.

Category `restriction`, `allow` by default.

Code meant to build everywhere that imports a module asserting an operating system it is not built for (`#assert OS == .LINUX || OS == .MACOS` in `POSIX`), found only when someone builds for that system.

### pointer_lifetime

Finds a pointer used after its storage moves or its element becomes invalid.

Category `restriction`, `allow` by default.

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

### print_in_module

Finds a module writes to the program's output.

Category `restriction`, `allow` by default.

A file belongs to a module when it or a directory above it holds a `module.jai`, with no `tests` or `test` folder in between (those hold programs that use the module).

### todo_marker

Finds a comment with a `TODO`, `FIXME`, `XXX` or `HACK` marker.

Category `restriction`, `allow` by default.

A marker is `TODO`, `FIXME`, `XXX` or `HACK` in capitals, standing alone (`TODO:` and `(FIXME)` count; `todo`, `TODOS` and `AUTODOC` do not).

### too_many_arguments

Finds a procedure with more parameters than `max_arguments` (default 7).

Category `restriction`, `allow` by default.

Every parameter counts, defaulted, baked (`$T`) and variadic ones included.

### unchecked_directive

Finds `#no_abc`, `#no_aoc` or `#asm` with no comment saying why.

Category `restriction`, `allow` by default.

Each `#no_abc`, `#no_aoc` and `#asm` needs a comment on its own line or the line just above.
