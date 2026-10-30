# Lint rules

Generated from the rule table in `crates/jailint/src/rules/` by `tools/generators/generate_reference_docs.py`; edit a rule's `INFO` there. [jailint](../tools/jailint.md) explains each rule and how to configure them.

## Categories

A rule's category sets its default level. Name a category wherever a rule name works (`[lint.rules]`, `-W`, `// jailint: allow(...)`).

| Category | Default | What |
| --- | --- | --- |
| `correctness` | deny | code that is wrong: it does not do what it says |
| `suspicious` | warn | code that is most likely wrong, or does nothing |
| `complexity` | warn | code that says something in a roundabout way |
| `style` | warn | code that works but is not written the usual way |
| `perf` | warn | code that could be faster |
| `pedantic` | allow | stricter checks with more false positives |
| `restriction` | allow | checks that forbid things a project may choose to avoid |

## Rules

104 rules.

| Rule | Category | Default | Finds |
| --- | --- | --- | --- |
| [`absurd_comparison`](../tools/jailint.md#absurd_comparison) | correctness | deny | a comparison an unsigned value always passes or fails (`u >= 0`) |
| [`almost_swapped`](../tools/jailint.md#almost_swapped) | correctness | deny | `a = b; b = a;`, a swap that sets both to `b` |
| [`approx_constant`](../tools/jailint.md#approx_constant) | style | warn | a hand-typed `3.14159` or `1.4142` where `Math` has the constant |
| [`assert_constant`](../tools/jailint.md#assert_constant) | style | warn | `assert(true)` or `#assert true`, which can never fail |
| [`autocast_in_arg`](../tools/jailint.md#autocast_in_arg) | restriction | allow | `xx`, the automatic cast, written as an argument of a call |
| [`autocast_used`](../tools/jailint.md#autocast_used) | restriction | allow | `xx`, the automatic cast, outside a call's arguments |
| [`bad_bit_mask`](../tools/jailint.md#bad_bit_mask) | correctness | deny | a mask test that can never match (`(x & 4) == 1`, `(x \| 2) == 0`) |
| [`banner_comment`](../tools/jailint.md#banner_comment) | style | warn | a comment that is only a line of `-`, `=` or `*` fencing off a section |
| [`bitwise_precedence`](../tools/jailint.md#bitwise_precedence) | suspicious | warn | a bitwise operator that binds differently than in C (`1 << n - 1`) |
| [`bool_comparison`](../tools/jailint.md#bool_comparison) | complexity | warn | comparing a bool with `true` or `false` |
| [`bool_to_int_with_if`](../tools/jailint.md#bool_to_int_with_if) | pedantic | allow | `ifx c then 1 else 0`, a `bool` turned into an integer with a branch |
| [`broken_doc_link`](../tools/jailint.md#broken_doc_link) | style | warn | a link in a doc comment (`[name]`) that names no declaration |
| [`cognitive_complexity`](../tools/jailint.md#cognitive_complexity) | pedantic | allow | a procedure whose branching scores above `max_complexity` (default 25) |
| [`constant_condition`](../tools/jailint.md#constant_condition) | suspicious | warn | an `if` or `while` whose condition the compiler shows is always true or false |
| [`defer_frees_returned`](../tools/jailint.md#defer_frees_returned) | correctness | deny | `defer free(p);` followed by `return p;`, which hands back freed memory |
| [`defer_in_loop`](../tools/jailint.md#defer_in_loop) | suspicious | warn | a `defer` in a loop body that cleans up something from outside the loop |
| [`detached_doc_comment`](../tools/jailint.md#detached_doc_comment) | pedantic | allow | a comment that reads as a declaration's docs but has a blank line between it and the declaration |
| [`double_free`](../tools/jailint.md#double_free) | correctness | deny | two `@FreesArgument` calls on the same variable with nothing in between |
| [`dropped_result`](../tools/jailint.md#dropped_result) | suspicious | warn | a call drops the `bool` that follows the values it keeps (`n := to_integer(s);`) |
| [`duplicate_condition`](../tools/jailint.md#duplicate_condition) | suspicious | warn | an `else if` that repeats an earlier condition |
| [`element_count_as_bytes`](../tools/jailint.md#element_count_as_bytes) | correctness | deny | `memcpy(d, xs.data, xs.count)` with elements wider than a byte |
| [`enum_member_prefix`](../tools/jailint.md#enum_member_prefix) | style | warn | every member of an enum starts with the enum's name |
| [`erasing_op`](../tools/jailint.md#erasing_op) | suspicious | warn | an operation that is always `0` (`x * 0`, `x & 0`) |
| [`excessive_bools`](../tools/jailint.md#excessive_bools) | restriction | allow | a procedure with more `bool` parameters than `max_bools` (default 3) |
| [`excessive_nesting`](../tools/jailint.md#excessive_nesting) | pedantic | allow | blocks nested deeper than `max_nesting` (default 6) |
| [`exit_in_module`](../tools/jailint.md#exit_in_module) | restriction | allow | a module ends the process |
| [`field_reassign_with_default`](../tools/jailint.md#field_reassign_with_default) | pedantic | allow | `x: T; x.a = 1; x.b = 2;` where a struct literal `x := T.{ a = 1, b = 2 }` sets the fields at the declaration |
| [`float_equality`](../tools/jailint.md#float_equality) | pedantic | allow | `==` or `!=` between two computed floats |
| [`float_loop_counter`](../tools/jailint.md#float_loop_counter) | pedantic | allow | a `while` loop counted in floats (`while t < 1.0 { t += 0.1; }`) |
| [`format_push_string`](../tools/jailint.md#format_push_string) | perf | warn | `append(*b, tprint(...))`, which formats into a new string to add it to a builder |
| [`identical_branches`](../tools/jailint.md#identical_branches) | suspicious | warn | an `if`/`else` or `ifx` whose branches are the same |
| [`identical_case_bodies`](../tools/jailint.md#identical_case_bodies) | pedantic | allow | two cases of a `switch` with the same body, which could list their values together |
| [`identical_operands`](../tools/jailint.md#identical_operands) | suspicious | warn | an operator with the same expression on both sides (`a == a`) |
| [`identity_op`](../tools/jailint.md#identity_op) | complexity | warn | an operation that changes nothing (`x + 0`, `x * 1`) |
| [`if_not_else`](../tools/jailint.md#if_not_else) | pedantic | allow | an `if` or `ifx` with a negated condition and an `else` |
| [`impossible_comparison`](../tools/jailint.md#impossible_comparison) | correctness | deny | comparisons of one integer that never pass together (`x > 5 && x < 3`) |
| [`incomplete_enum_switch`](../tools/jailint.md#incomplete_enum_switch) | pedantic | allow | a `switch` on an enum that neither covers every member nor has a default case |
| [`index_only_loop`](../tools/jailint.md#index_only_loop) | complexity | warn | `for i: 0..xs.count-1` where `i` only indexes `xs` |
| [`infinite_loop`](../tools/jailint.md#infinite_loop) | suspicious | warn | a `while` whose condition nothing in the loop changes |
| [`int_plus_one`](../tools/jailint.md#int_plus_one) | complexity | warn | `x >= y + 1`, which is `x > y` |
| [`integer_division_in_float`](../tools/jailint.md#integer_division_in_float) | suspicious | warn | an integer division whose result is used as a float |
| [`large_stack_array`](../tools/jailint.md#large_stack_array) | pedantic | allow | a local fixed array bigger than `max_stack_array_bytes` (default 64 KiB) |
| [`leaked_allocation`](../tools/jailint.md#leaked_allocation) | pedantic | allow | a local holding memory from an `@Allocates` call that is never freed, returned or handed on |
| [`leaked_temporary`](../tools/jailint.md#leaked_temporary) | suspicious | warn | an `@Allocates` result that is only compared, measured, indexed or printed, so nothing can free it |
| [`loop_with_break_condition`](../tools/jailint.md#loop_with_break_condition) | complexity | warn | `while true { if !c break; ... }`, a loop whose condition is its first statement |
| [`lossy_float_literal`](../tools/jailint.md#lossy_float_literal) | style | warn | a `float32` literal with more digits than a `float32` keeps (`31415.92653`) |
| [`lossy_xx`](../tools/jailint.md#lossy_xx) | pedantic | allow | `xx` that narrows a number to a smaller type |
| [`magic_number`](../tools/jailint.md#magic_number) | pedantic | allow | an unexplained number in arithmetic, a comparison or an assignment |
| [`manual_array_find`](../tools/jailint.md#manual_array_find) | complexity | warn | a loop that searches an array and breaks with a found flag or index, which `array_find` does |
| [`manual_assign_op`](../tools/jailint.md#manual_assign_op) | complexity | warn | `a = a + b` instead of `a += b` |
| [`manual_clamp`](../tools/jailint.md#manual_clamp) | complexity | warn | a value kept between two constant bounds by hand instead of with `clamp` |
| [`manual_index_counter`](../tools/jailint.md#manual_index_counter) | complexity | warn | a counter incremented in a `for` loop that always equals `it_index` |
| [`manual_memcpy`](../tools/jailint.md#manual_memcpy) | perf | warn | `for i: 0..n-1 { dst[i] = src[i]; }`, an element-by-element copy `memcpy` does at once |
| [`min_max`](../tools/jailint.md#min_max) | correctness | deny | `min(0, max(100, x))`, a clamp that is always one bound |
| [`misrefactored_assign_op`](../tools/jailint.md#misrefactored_assign_op) | correctness | deny | `a += a + b`, an update that applies the operator to `a` twice |
| [`missing_doc`](../tools/jailint.md#missing_doc) | restriction | allow | an exported procedure, struct or enum with no doc comment |
| [`mixed_case_hex_literals`](../tools/jailint.md#mixed_case_hex_literals) | style | warn | a hex literal with both upper and lower case digits (`0xDeadbeef`) |
| [`modified_range_bound`](../tools/jailint.md#modified_range_bound) | suspicious | warn | a variable bounding a `for` range assigned in the loop, which does not change it |
| [`needless_bool`](../tools/jailint.md#needless_bool) | complexity | warn | `ifx c then true else false` and `if c return true; else return false;` |
| [`needless_continue`](../tools/jailint.md#needless_continue) | style | warn | a `continue` as the last thing a loop body does |
| [`neg_cmp_float`](../tools/jailint.md#neg_cmp_float) | pedantic | allow | `!(a < b)` on floats, which differs from `a >= b` when a value is NaN |
| [`neg_multiply`](../tools/jailint.md#neg_multiply) | style | warn | `x * -1`, which is `-x` |
| [`no_effect`](../tools/jailint.md#no_effect) | suspicious | warn | a statement that computes a value and drops it (`x == 5;`) |
| [`platform_leak`](../tools/jailint.md#platform_leak) | restriction | allow | portable code imports something that asserts an operating system it is not built for |
| [`pointer_into_resizable`](../tools/jailint.md#pointer_into_resizable) | suspicious | warn | a pointer into a resizable array used after a `@GrowsArray` call may have moved the array (`p := *xs[i]; array_add(*xs, v); use(p);`) |
| [`print_in_module`](../tools/jailint.md#print_in_module) | restriction | allow | a module writes to the program's output |
| [`push_in_static_if`](../tools/jailint.md#push_in_static_if) | suspicious | warn | a scoped push directly in an `#if` arm, which lasts past the arm |
| [`range_past_count`](../tools/jailint.md#range_past_count) | correctness | deny | `for i: 0..xs.count` indexing `xs[i]`, one past the end |
| [`redundant_alias`](../tools/jailint.md#redundant_alias) | complexity | warn | a local that only exists to be copied into another local or returned |
| [`redundant_cast`](../tools/jailint.md#redundant_cast) | complexity | warn | a cast to the type the value already has |
| [`redundant_deref`](../tools/jailint.md#redundant_deref) | complexity | warn | `p.*.field` where `p.field` means the same |
| [`remove_in_for`](../tools/jailint.md#remove_in_for) | correctness | deny | removing from an array while a `for` walks it forward |
| [`returned_local_address`](../tools/jailint.md#returned_local_address) | correctness | deny | `return *x;` with `x` a local, a pointer into a stack frame that is gone |
| [`reversed_range`](../tools/jailint.md#reversed_range) | correctness | deny | a range whose start is above its end, which never runs |
| [`self_assignment`](../tools/jailint.md#self_assignment) | suspicious | warn | a value assigned to itself (`x = x;`) |
| [`shadowed_it`](../tools/jailint.md#shadowed_it) | suspicious | warn | a nested `for` hides an `it` the enclosing loop uses |
| [`shadowed_local`](../tools/jailint.md#shadowed_local) | pedantic | allow | a local variable named like a parameter or another local in an enclosing scope of the same procedure |
| [`shift_overflow`](../tools/jailint.md#shift_overflow) | suspicious | warn | a shift by at least the bit width of its operand (`x << 40` with `x: s32`) |
| [`single_case_switch`](../tools/jailint.md#single_case_switch) | style | warn | a `switch` with a single case, which is an `if` |
| [`string_literal_write`](../tools/jailint.md#string_literal_write) | correctness | deny | a write into a string literal, which is read-only (`s := "abc"; s[0] = x;`) |
| [`temp_storage_escape`](../tools/jailint.md#temp_storage_escape) | pedantic | allow | a `@TempAllocates` result (`tprint`, `talloc`) stored in a global or through a pointer, or returned |
| [`todo_marker`](../tools/jailint.md#todo_marker) | restriction | allow | a comment with a `TODO`, `FIXME`, `XXX` or `HACK` marker |
| [`too_many_arguments`](../tools/jailint.md#too_many_arguments) | restriction | allow | a procedure with more parameters than `max_arguments` (default 7) |
| [`unchecked_directive`](../tools/jailint.md#unchecked_directive) | restriction | allow | `#no_abc`, `#no_aoc` or `#asm` with no comment saying why |
| [`unclosed_resource`](../tools/jailint.md#unclosed_resource) | suspicious | warn | a local opened by an `@Opens` call (`file_open`, `append(*builder, ..)`) that nothing closes |
| [`uninit_read`](../tools/jailint.md#uninit_read) | correctness | deny | a variable declared `= ---` read before anything is written to it |
| [`unknown_allow`](../tools/jailint.md#unknown_allow) | suspicious | warn | a `// jailint: allow(...)` comment naming a rule that does not exist |
| [`unnecessary_force_cast`](../tools/jailint.md#unnecessary_force_cast) | complexity | warn | `cast,force` where the value already has the type or the conversion widens |
| [`unnecessary_semicolon`](../tools/jailint.md#unnecessary_semicolon) | style | warn | a `;` that ends nothing (`x = 1;;`, `if c { ... };`) |
| [`unpaired_lock`](../tools/jailint.md#unpaired_lock) | suspicious | warn | a `@Locks` call with a path that returns or ends without a matching `@Unlocks` |
| [`unreadable_literal`](../tools/jailint.md#unreadable_literal) | pedantic | allow | a long number without `_` separators (`1000000`, `0xdeadbeef`) |
| [`unsigned_range_end_minus_one`](../tools/jailint.md#unsigned_range_end_minus_one) | suspicious | warn | `for 0..n-1` with an unsigned `n`, which wraps around when `n` is 0 |
| [`unused_import`](../tools/jailint.md#unused_import) | style | warn | an `#import` nothing in its scope uses |
| [`unused_parameter`](../tools/jailint.md#unused_parameter) | style | warn | a parameter the procedure never uses |
| [`unused_private_declaration`](../tools/jailint.md#unused_private_declaration) | suspicious | warn | a `#scope_file` declaration that nothing in its file uses |
| [`unused_result`](../tools/jailint.md#unused_result) | suspicious | warn | a call to a `@UseResult` procedure, as a statement |
| [`unused_suppression`](../tools/jailint.md#unused_suppression) | suspicious | warn | a name in a `// jailint: allow(...)` comment or `@jailint_allow` note that silences nothing |
| [`unused_variable`](../tools/jailint.md#unused_variable) | style | warn | a local variable that is never used |
| [`use_after_free`](../tools/jailint.md#use_after_free) | correctness | deny | a variable read after a `@FreesArgument` call freed it (`free(p); use(p);`) |
| [`useless_format`](../tools/jailint.md#useless_format) | complexity | warn | `tprint("%", s)` or `sprint("%", s)` with a string `s`, which only copies it |
| [`useless_resizable`](../tools/jailint.md#useless_resizable) | perf | warn | a local `[..] T` that is only filled with a known number of `array_add` calls and never resized |
| [`using_pollution`](../tools/jailint.md#using_pollution) | pedantic | allow | a `using` of a struct whose members share names with parameters, locals or globals already in scope |
| [`wrapping_constant`](../tools/jailint.md#wrapping_constant) | correctness | deny | a constant that wraps to the type the other operand gives it (`(0xffff_ffff - 40) / h`) |
| [`zero_ptr`](../tools/jailint.md#zero_ptr) | style | warn | `cast(*T) 0`, which is `null` |
