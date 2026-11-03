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

Example:

```jai
#import "Basic";

countdown :: (n: u32) {
    i := n;
    while i >= 0 {
        print("%\n", i);
        i -= 1;
    }
}

main :: () {
    countdown(3);
}
```

```diagnostic
error[absurd_comparison]: `i >= 0` is always `true`
  --> example.jai:5:11
   |
 5 |     while i >= 0 {
   |           ^^^^^^
   |
help: `u32` is unsigned and holds nothing below 0; compare with a signed value, or drop the check
   |
 5 ~     while true {
   |
   = note: `absurd_comparison` is `deny` by default
```

Fixed:

```jai
#import "Basic";

countdown :: (n: u32) {
    i := n;
    while i > 0 {
        print("%\n", i);
        i -= 1;
    }
}

main :: () {
    countdown(3);
}
```

An unsigned value is never below zero, so counting down an unsigned `i` with `while i >= 0` never stops.

### almost_swapped

Finds `a = b; b = a;`, a swap that sets both to `b`.

Category `correctness`, `deny` by default.

Example:

```jai
#import "Basic";

main :: () {
    a := 1;
    b := 2;

    a = b;
    b = a;

    print("% %\n", a, b);
}
```

```diagnostic
error[almost_swapped]: this sets both `a` and `b` to `b`
  --> example.jai:7:5
   |
 7 |     a = b;
   |     ^^^^^^
   |
help: to swap them, write `a, b = b, a;`
   |
 7 -     a = b;
 8 -     b = a;
 7 +     a, b = b, a;
   |
   = note: `almost_swapped` is `deny` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    a := 1;
    b := 2;

    tmp := a;

    a = b;
    b = tmp;

    print("% %\n", a, b);
}
```

Two consecutive statements with side-effect-free sides of the same type.

### bad_bit_mask

Finds a mask test that can never match (`(x & 4) == 1`, `(x | 2) == 0`).

Category `correctness`, `deny` by default.

Example:

```jai
#import "Basic";

can_write :: (flags: u32) -> bool {
    return (flags & 4) == 1;
}

main :: () {
    print("%\n", can_write(5));
}
```

```diagnostic
error[bad_bit_mask]: `(flags & 4) == 1` is always `false`: `& 4` leaves only the bits of 4, and 1 has others
  --> example.jai:4:13
   |
 4 |     return (flags & 4) == 1;
   |             ^^^^^^^^^^^^^^^
   |
help: compare with the mask itself (`(x & m) == m`) or test for non-zero
   |
 4 ~     return false;
   |
   = note: `bad_bit_mask` is `deny` by default
```

Fixed:

```jai
#import "Basic";

can_write :: (flags: u32) -> bool {
    return (flags & 4) != 0;
}

main :: () {
    print("%\n", can_write(5));
}
```

`x & m` has only bits of `m`, so a constant with a bit outside `m` is never equal to it; `x | m` always has the bits of `m`, so a constant missing one never equals it.

### defer_frees_returned

Finds `defer free(p);` followed by `return p;`, which hands back freed memory.

Category `correctness`, `deny` by default.

Example:

```jai
#import "Basic";
#import "String";

full_name :: (first: string) -> string {
    name := join(first, " Smith");
    defer free(name);
    return name;
}

main :: () {
    name := full_name("Ada");
    print("%\n", name);
}
```

```diagnostic
error[defer_frees_returned]: `name` is returned, but the `defer` above frees it
  --> example.jai:7:12
   |
 7 |     return name;
   |            ^^^^
   |
help: the deferred call runs after the value is returned: drop the `defer`, or return a copy
   |
 6 -     defer free(name);
   |
   = note: `defer_frees_returned` is `deny` by default
```

Fixed:

```jai
#import "Basic";
#import "String";

full_name :: (first: string) -> string {
    name := join(first, " Smith");
    return name;
}

main :: () {
    name := full_name("Ada");
    print("%\n", name);
    free(name);
}
```

The `defer` must be a single `@FreesArgument` call on a local.

### double_free

Finds two `@FreesArgument` calls on the same variable with nothing in between.

Category `correctness`, `deny` by default.

Example:

```jai
#import "Basic";

Node :: struct {
    value: int;
}

main :: () {
    node := New(Node);
    node.value = 3;
    print("%\n", node.value);

    free(node);
    free(node);
}
```

```diagnostic
error[double_free]: `node` is freed a second time
  --> example.jai:13:5
   |
10 |     print("%\n", node.value);
   |                       ----- storage derived here
   ...
12 |     free(node);
   |     ---------- storage invalidated here
13 |     free(node);
   |     ^^^^^^^^^^
   |
help: remove the second call, or assign `node` a new value in between
   |
13 -     free(node);
   |
   = note: `double_free` is `deny` by default
```

Fixed:

```jai
#import "Basic";

Node :: struct {
    value: int;
}

main :: () {
    node := New(Node);
    node.value = 3;
    print("%\n", node.value);

    free(node);
}
```

A value freed or closed on every path, then freed or closed again (`free(p); free(p);`, `close(f); close(f);`), through copies and wrappers. See [lifetime rules](../tools/jailint.md#lifetime-rules).

### element_count_as_bytes

Finds `memcpy(d, xs.data, xs.count)` with elements wider than a byte.

Category `correctness`, `deny` by default.

Example:

```jai
#import "Basic";

copy_values :: (dest: [] s32, values: [] s32) {
    memcpy(dest.data, values.data, values.count);
}

main :: () {
    a: [4] s32;
    b := s32.[1, 2, 3, 4];
    copy_values(a, b);
    print("%\n", a);
}
```

```diagnostic
error[element_count_as_bytes]: `values.count` counts `s32` elements, but this argument is a number of bytes
  --> example.jai:4:36
   |
 4 |     memcpy(dest.data, values.data, values.count);
   |                                    ^^^^^^^^^^^^
   |
help: multiply by the element size: `values.count * size_of(s32)`
   |
 4 ~     memcpy(dest.data, values.data, values.count * size_of(s32));
   |
   = note: `element_count_as_bytes` is `deny` by default
```

Fixed:

```jai
#import "Basic";

copy_values :: (dest: [] s32, values: [] s32) {
    memcpy(dest.data, values.data, values.count * size_of(s32));
}

main :: () {
    a: [4] s32;
    b := s32.[1, 2, 3, 4];
    copy_values(a, b);
    print("%\n", a);
}
```

Only when `xs.data` is another argument of the same call, so the count clearly describes that array.

### impossible_comparison

Finds comparisons of one integer that never pass together (`x > 5 && x < 3`).

Category `correctness`, `deny` by default.

Example:

```jai
#import "Basic";

is_live :: (code: int) -> bool {
    return code != 404 || code != 410;
}

main :: () {
    print("%\n", is_live(200));
}
```

```diagnostic
error[impossible_comparison]: `code != 404 || code != 410` is always `true`
  --> example.jai:4:12
   |
 4 |     return code != 404 || code != 410;
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: every integer passes one of these comparisons: was `&&` meant?
   |
 4 ~     return code != 404 && code != 410;
   |
   = note: `impossible_comparison` is `deny` by default
```

Fixed:

```jai
#import "Basic";

is_live :: (code: int) -> bool {
    return code != 404 && code != 410;
}

main :: () {
    print("%\n", is_live(200));
}
```

Only integer expressions compared with integer literals, within one run of `&&` or `||`.

### min_max

Finds `min(0, max(100, x))`, a clamp that is always one bound.

Category `correctness`, `deny` by default.

Example:

```jai
#import "Basic";

main :: () {
    volume := 140;
    volume = min(0, max(100, volume));
    print("%\n", volume);
}
```

```diagnostic
error[min_max]: this is always `0`: the bounds are swapped
  --> example.jai:5:14
   |
 5 |     volume = min(0, max(100, volume));
   |              ^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: to keep a value between 0 and 100, write `clamp(value, 0, 100)`
   |
 5 ~     volume = clamp(volume, 0, 100);
   |
   = note: `min_max` is `deny` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    volume := 140;
    volume = min(100, max(0, volume));
    print("%\n", volume);
}
```

Literal bounds only, with procedures that select the smaller or larger argument (the library's `min`/`max`).

### misrefactored_assign_op

Finds `a += a + b`, an update that applies the operator to `a` twice.

Category `correctness`, `deny` by default.

Example:

```jai
#import "Basic";

main :: () {
    total := 10;
    price := 5;

    total += total + price;

    print("%\n", total);
}
```

```diagnostic
error[misrefactored_assign_op]: `total += total + ...` applies `+` to `total` twice
  --> example.jai:7:5
   |
 7 |     total += total + price;
   |     ^^^^^^^^^^^^^^^^^^^^^^
   |
help: `+=` already starts from `total`: write `total += price`
   |
 7 ~     total += price;
   |
   = note: `misrefactored_assign_op` is `deny` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    total := 10;
    price := 5;

    total += price;

    print("%\n", total);
}
```

The same operator on both sides, a side-effect-free target of a number type spelled the same in both places (`a op= a op b`), and for `+ * & | ^` on integers also `a op= b op a`.

### range_past_count

Finds `for i: 0..xs.count` indexing `xs[i]`, one past the end.

Category `correctness`, `deny` by default.

Example:

```jai
#import "Basic";

main :: () {
    names := string.["a", "b", "c"];
    for i: 0..names.count {
        print("%\n", names[i]);
    }
}
```

```diagnostic
error[range_past_count]: `names.count` is one past the last index of `names`
  --> example.jai:5:15
   |
 5 |     for i: 0..names.count {
   |               ^^^^^^^^^^^
   |
help: ranges include their end: write `names.count - 1`
   |
 5 ~     for i: 0..names.count - 1 {
   |
   = note: `range_past_count` is `deny` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    names := string.["a", "b", "c"];
    for i: 0..names.count - 1 {
        print("%\n", names[i]);
    }
}
```

Ranges include their end. Quiet when the body mentions `xs.count` (it may guard the index).

### remove_in_for

Finds removing from an array while a `for` walks it forward.

Category `correctness`, `deny` by default.

Example:

```jai
#import "Basic";

main :: () {
    xs: [..] int;

    array_add(*xs, 1, 2, 2, 3);

    for xs {
        if it == 2 array_unordered_remove_by_index(*xs, it_index);
    }

    print("%\n", xs);
}
```

```diagnostic
error[remove_in_for]: `array_unordered_remove_by_index` changes `xs` while this `for` walks it
  --> example.jai:9:20
   |
 8 |     for xs {
   |         -- this collection's cursor advances after removal
 9 |         if it == 2 array_unordered_remove_by_index(*xs, it_index);
   |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: the element moved into the freed slot is skipped; use `remove it;` (or loop backwards with `for < xs`)
   |
 9 ~         if it == 2 remove it;
   |
   = note: `remove_in_for` is `deny` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    xs: [..] int;

    array_add(*xs, 1, 2, 2, 3);

    for xs {
        if it == 2 remove it;
    }

    print("%\n", xs);
}
```

The loop steps past the element moved into the removed slot. Fires when a removal at the cursor reaches the loop's step on every path; `break`, `return` or writing the index yourself stops it. Removal inferred through a wrapper counts.

### returned_local_address

Finds `return *x;` with `x` a local, a pointer into a stack frame that is gone.

Category `correctness`, `deny` by default.

Example:

```jai
#import "Basic";

Point :: struct {
    x, y: int;
}

origin :: () -> *Point {
    p: Point;
    return *p;
}

main :: () {
    p := origin();
    print("%\n", p.x);
}
```

```diagnostic
error[returned_local_address]: this returns the address of the local `p`, which is gone once the procedure returns
  --> example.jai:9:12
   |
 9 |     return *p;
   |            ^^
   |            -- stack storage borrowed here
   |
   = help: return the value itself, take the memory from an allocator (`New`, `alloc`), or have the caller pass the place to fill
   = note: `returned_local_address` is `deny` by default
```

Fixed:

```jai
#import "Basic";

Point :: struct {
    x, y: int;
}

origin :: () -> *Point {
    p := New(Point);
    return p;
}

main :: () {
    p := origin();
    print("%\n", p.x);
    free(p);
}
```

Covers locals, by-value parameters (the copy is the callee's), `for` variables and views of local fixed arrays, through copies, fields and `Any`. Literals the compiler keeps alive are fine.

### reversed_range

Finds a range whose start is above its end, which never runs.

Category `correctness`, `deny` by default.

Example:

```jai
#import "Basic";

main :: () {
    for i: 10..0 {
        print("%\n", i);
    }
}
```

```diagnostic
error[reversed_range]: the range `10..0` is empty: ranges count up
  --> example.jai:4:12
   |
 4 |     for i: 10..0 {
   |            ^^^^^
   |
help: to count down, write `for < ...: 0..10`
   |
 4 ~     for < i: 0..10 {
   |
   = note: `reversed_range` is `deny` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    for < i: 0..10 {
        print("%\n", i);
    }
}
```

Counting down is `for < i: 0..10`. The fix changes behaviour, so it is editor-only.

### string_literal_write

Finds a write into a string literal, which is read-only (`s := "abc"; s[0] = x;`).

Category `correctness`, `deny` by default.

Example:

```jai
#import "Basic";

main :: () {
    word := "hello";
    word[0] = #char "j";
    print("%\n", word);
}
```

```diagnostic
error[string_literal_write]: `word[0]` writes into a string literal: `word` still holds one, and literals are read-only
  --> example.jai:5:5
   |
 5 |     word[0] = #char "j";
   |     ^^^^^^^
   |
help: copy it where `word` is declared: `copy_string("hello")` (free the copy when done)
   |
 4 ~     word := copy_string("hello");
   |
   = note: `string_literal_write` is `deny` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    word := copy_string("hello");
    word[0] = #char "j";
    print("%\n", word);
    free(word);
}
```

Literals are read-only and equal literals share storage, so the write is a runtime error (interpreter) or a crash (native).

### uninit_read

Finds a variable declared `= ---` read before anything is written to it.

Category `correctness`, `deny` by default.

Example:

```jai
#import "Basic";

sum :: (xs: [] int) -> int {
    total: int = ---;
    for xs total += it;
    return total;
}

main :: () {
    print("%\n", sum(int.[1, 2, 3]));
}
```

```diagnostic
error[uninit_read]: `total` is read here before anything is written to it
  --> example.jai:5:12
   |
 5 |     for xs total += it;
   |            ^^^^^
   |
help: `= ---` leaves `total` holding whatever was in memory: give it a value first, or drop the `= ---`
   |
 4 ~     total: int;
   |
   = note: `uninit_read` is `deny` by default
```

Fixed:

```jai
#import "Basic";

sum :: (xs: [] int) -> int {
    total := 0;
    for xs total += it;
    return total;
}

main :: () {
    print("%\n", sum(int.[1, 2, 3]));
}
```

The first mention after the declaration, in the same block, decides; a write anywhere (even inside an `if`) ends the check, so only straight-line reads are reported.

### use_after_free

Finds a variable read after a `@FreesArgument` call freed it (`free(p); use(p);`).

Category `correctness`, `deny` by default.

Example:

```jai
#import "Basic";

Node :: struct {
    value: int;
}

main :: () {
    node := New(Node);
    node.value = 7;

    free(node);
    print("%\n", node.value);
}
```

```diagnostic
error[use_after_free]: `node` is used after the call above freed it
  --> example.jai:12:18
   |
11 |     free(node);
   |     ---------- storage invalidated here
   |          ---- storage derived here
12 |     print("%\n", node.value);
   |                  ^^^^
   |
   = help: free it after its last use, or give it a new value before using it again
   = note: `use_after_free` is `deny` by default
```

Fixed:

```jai
#import "Basic";

Node :: struct {
    value: int;
}

main :: () {
    node := New(Node);
    node.value = 7;

    print("%\n", node.value);
    free(node);
}
```

A pointer read after every path to it freed its storage: `free(p)`, `array_free(*xs)` then `xs.data`, a wrapper that frees, a `defer` that already ran. See [lifetime rules](../tools/jailint.md#lifetime-rules).

### wrapping_constant

Finds a constant that wraps to the type the other operand gives it (`(0xffff_ffff - 40) / h`).

Category `correctness`, `deny` by default.

Example:

```jai
#import "Basic";

bucket :: (id: s32) -> s32 {
    return id % 0x8000_0000;
}

main :: () {
    print("%\n", bucket(5));
}
```

```diagnostic
error[wrapping_constant]: constant `0x8000_0000` is 2147483648, which does not fit in `s32`: `%` uses it as -2147483648
  --> example.jai:4:17
   |
 4 |     return id % 0x8000_0000;
   |                 ^^^^^^^^^^^ `s32` comes from `id`
   |
help: to use 2147483648, compute in `s64` (`cast(s64) id`); to mean -2147483648, write -2147483648
   |
 4 ~     return cast(s64) id % 0x8000_0000;
   |
   = note: `wrapping_constant` is `deny` by default
```

Fixed:

```jai
#import "Basic";

bucket :: (id: s32) -> s32 {
    return id % 0x7fff_ffff;
}

main :: () {
    print("%\n", bucket(5));
}
```

A constant takes the other operand's type, so the first divides `-41` by `h`.

## Suspicious

Code that is most likely wrong, or does nothing. Default: `warn`.

### bitwise_precedence

Finds a bitwise operator that binds differently than in C (`1 << n - 1`).

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";

low_bits :: (bits: int) -> int {
    return 1 << bits - 1;
}

main :: () {
    print("%\n", low_bits(4));
}
```

```diagnostic
warning[bitwise_precedence]: `<<` binds tighter than `-` in Jai, unlike in C
  --> example.jai:4:12
   |
 4 |     return 1 << bits - 1;
   |            ^^^^^^^^^^^^^
   |
help: `<<` binds first here: write `(1 << bits)`
   |
 4 ~     return (1 << bits) - 1;
   |
   = note: `bitwise_precedence` is `warn` by default
```

Fixed:

```jai
#import "Basic";

low_bits :: (bits: int) -> int {
    return (1 << bits) - 1;
}

main :: () {
    print("%\n", low_bits(4));
}
```

Jai gives all bitwise and shift operators one level above `*`, grouped left to right, so `1 << n - 1` is `(1 << n) - 1`.

### constant_condition

Finds an `if` or `while` whose condition the compiler shows is always true or false.

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";

check :: (count: s64) {
    if count > 10 {
        if count < 5 {
            print("never\n");
        }
    }
}

main :: () {
    check(20);
}
```

```diagnostic
warning[constant_condition]: this condition is always `false`
  --> example.jai:5:12
   |
 5 |         if count < 5 {
   |            ^^^^^^^^^
   |
   = help: the code it guards never runs: the values its operands can take here decide it; remove the test or fix the logic around it
   = note: `constant_condition` is `warn` by default
```

Fixed:

```jai
#import "Basic";

check :: (count: s64) {
    if count > 10 {
        if count < 50 {
            print("between\n");
        }
    }
}

main :: () {
    check(20);
}
```

Reads the compiler's reachability facts (`jaic::sema::reachability` and `jaic::ide::values`, the code the editor fades): reports the condition behind a branch that never runs, such as `if count < 5` inside `if count > 10`, or an `else` after a test that always holds.

### defer_in_loop

Finds a `defer` in a loop body that cleans up something from outside the loop.

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";

main :: () {
    buffer := alloc(64);
    names := string.["a", "b", "c"];

    for names {
        defer free(buffer);
        print("%\n", it);
    }
}
```

```diagnostic
warning[defer_in_loop]: this `defer` runs at the end of every iteration, not when the procedure returns
  --> example.jai:8:9
   |
 8 |         defer free(buffer);
   |         ^^^^^^^^^^^^^^^^^^
   |
   = help: move it before the loop if it should run once
   = note: `defer_in_loop` is `warn` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    buffer := alloc(64);

    defer free(buffer);

    names := string.["a", "b", "c"];

    for names {
        print("%\n", it);
    }
}
```

The loop body's scope ends every iteration, so `defer free(buffer)` frees an outer buffer after the first pass.

### dropped_result

Finds a call drops the `bool` that follows the values it keeps (`n := to_integer(s);`).

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";

parse :: (text: string) -> int, bool {
    if text == "7" return 7, true;
    return 0, false;
}

main :: () {
    value := parse("x");
    print("%\n", value);
}
```

```diagnostic
warning[dropped_result]: `parse` returns 2 values but this keeps 1; the next value is a `bool`, which usually says whether the call worked
  --> example.jai:9:14
   |
 9 |     value := parse("x");
   |              ^^^^^^^^^^
   |
help: receive the flag, or write `, _` to show it is ignored on purpose
   |
 9 ~     value, _ := parse("x");
   |
   = note: `dropped_result` is `warn` by default
```

Fixed:

```jai
#import "Basic";

parse :: (text: string) -> int, bool {
    if text == "7" return 7, true;
    return 0, false;
}

main :: () {
    value, ok := parse("x");
    if !ok print("not a number\n");
    else print("%\n", value);
}
```

A declaration or assignment whose right side is one call that returns more values than the left side names, where the first dropped value, the one right after the last kept, is a `bool`: `value := parse(text);` quietly turns a failed parse into `0`.

### duplicate_condition

Finds an `else if` that repeats an earlier condition.

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";

describe :: (key: int) {
    if key == 1 print("one\n");
    else if key == 2 print("two\n");
    else if key == 1 print("uno\n");
}

main :: () {
    describe(1);
}
```

```diagnostic
warning[duplicate_condition]: `key == 1` was already tested earlier in this chain
  --> example.jai:6:13
   |
 6 |     else if key == 1 print("uno\n");
   |             ^^^^^^^^
   |
help: this branch can never run: was another condition meant?
   |
 5 -     else if key == 2 print("two\n");
 6 -     else if key == 1 print("uno\n");
 5 +     else if key == 2 print("two\n");
   |
   = note: `duplicate_condition` is `warn` by default
```

Fixed:

```jai
#import "Basic";

describe :: (key: int) {
    if key == 1 print("one\n");
    else if key == 2 print("two\n");
    else if key == 3 print("three\n");
}

main :: () {
    describe(1);
}
```

Conditions must be side-effect-free; a call in between restarts the comparison.

### erasing_op

Finds an operation that is always `0` (`x * 0`, `x & 0`).

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";

offset :: (row: int, col: int) -> int {
    return row * 0 + col;
}

main :: () {
    print("%\n", offset(2, 3));
}
```

```diagnostic
warning[erasing_op]: `row * 0` is always `0`
  --> example.jai:4:12
   |
 4 |     return row * 0 + col;
   |            ^^^^^^^
   |
help: `*` with this literal erases `row`: was another value meant?
   |
 4 ~     return 0 + col;
   |
   = note: `erasing_op` is `warn` by default
```

Fixed:

```jai
#import "Basic";

offset :: (row: int, col: int) -> int {
    return row * 8 + col;
}

main :: () {
    print("%\n", offset(2, 3));
}
```

Left alone inside an index or array literal, where a `0 *` lines up with other rows.

### identical_branches

Finds an `if`/`else` or `ifx` whose branches are the same.

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";

main :: () {
    n := 3;
    label := ifx n > 0 then "positive" else "positive";
    print("%\n", label);
}
```

```diagnostic
warning[identical_branches]: both branches of this `ifx` are the same
  --> example.jai:5:14
   |
 5 |     label := ifx n > 0 then "positive" else "positive";
   |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: the condition decides nothing: was one branch meant to differ?
   |
 5 ~     label := "positive";
   |
   = note: `identical_branches` is `warn` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    n := 3;
    label := ifx n > 0 then "positive" else "not positive";
    print("%\n", label);
}
```

Compared by tokens. Empty branches, `#if`, and the last link of an `else if` chain are left alone.

### identical_operands

Finds an operator with the same expression on both sides (`a == a`).

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";

Vec :: struct {
    x: int;
    y: int;
}

same :: (a: Vec, b: Vec) -> bool {
    return a.x == b.x && a.y == a.y;
}

main :: () {
    print("%\n", same(.{ 1, 2 }, .{ 1, 3 }));
}
```

```diagnostic
warning[identical_operands]: both sides of `==` are `a.y`
  --> example.jai:9:26
   |
 9 |     return a.x == b.x && a.y == a.y;
   |                          ^^^^^^^^^^
   |
   = help: the result is always `true`; was one side meant to be something else?
   = note: `identical_operands` is `warn` by default
```

Fixed:

```jai
#import "Basic";

Vec :: struct {
    x: int;
    y: int;
}

same :: (a: Vec, b: Vec) -> bool {
    return a.x == b.x && a.y == b.y;
}

main :: () {
    print("%\n", same(.{ 1, 2 }, .{ 1, 3 }));
}
```

Float `==`, `!=` and `-` are left alone (NaN tests), as are operator procedures.

### infinite_loop

Finds a `while` whose condition nothing in the loop changes.

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";

sum :: (values: [] int) -> int {
    total := 0;
    i := 0;

    while i < values.count {
        total += values[i];
    }

    return total;
}

main :: () {
    print("%\n", sum(.[1, 2, 3]));
}
```

```diagnostic
warning[infinite_loop]: this loop's condition never changes inside it
  --> example.jai:7:11
   |
 7 |     while i < values.count {
   |           ^^^^^^^^^^^^^^^^
   |
   = help: nothing in the loop assigns `i` or `values`, and it has no `break` or `return`: once entered it never ends
   = note: `infinite_loop` is `warn` by default
```

Fixed:

```jai
#import "Basic";

sum :: (values: [] int) -> int {
    total := 0;
    i := 0;

    while i < values.count {
        total += values[i];
        i += 1;
    }

    return total;
}

main :: () {
    print("%\n", sum(.[1, 2, 3]));
}
```

Condition reads only locals, parameters and literals; skipped for bodies with `break`, `return`, `#insert`, `#asm`, `using`, macro calls or taken addresses.

### integer_division_in_float

Finds an integer division whose result is used as a float.

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";

main :: () {
    done := 3;
    total := 4;
    ratio := cast(float) (done / total);
    print("%\n", ratio);
}
```

```diagnostic
warning[integer_division_in_float]: integer division `done / total` is converted to a float: the remainder is lost first
  --> example.jai:6:27
   |
 6 |     ratio := cast(float) (done / total);
   |                           ^^^^^^^^^^^^
   |
help: divide as floats: `cast(float) done / total`
   |
 6 ~     ratio := cast(float) (cast(float) done / total);
   |
   = note: `integer_division_in_float` is `warn` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    done := 3;
    total := 4;
    ratio := cast(float) done / cast(float) total;
    print("%\n", ratio);
}
```

Operands are typed before the cast, so the integers divide first.

### leaked_temporary

Finds an inferred owned result that is only compared, measured, indexed or printed, so nothing can free it.

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";

is_seven :: (n: int) -> bool {
    return sprint("%", n) == "7";
}

main :: () {
    print("%\n", is_seven(7));
}
```

```diagnostic
warning[leaked_temporary]: the memory `sprint` allocates is only compared and never freed
  --> example.jai:4:12
   |
 4 |     return sprint("%", n) == "7";
   |            ^^^^^^^^^^^^^^
   |
   = help: keep it in a local and `defer free` it, or use a temporary-storage procedure such as `tprint`
   = note: `leaked_temporary` is `warn` by default
```

Fixed:

```jai
#import "Basic";

is_seven :: (n: int) -> bool {
    return tprint("%", n) == "7";
}

main :: () {
    print("%\n", is_seven(7));
}
```

A call of a procedure that allocates its result (`sprint`, `join`, `copy_string`, ...) whose result is used where it is read and dropped at once: an operand of a comparison, the receiver of `.count` or of an index, or a value argument of a `@PrintLike` call.

### modified_range_bound

Finds a variable bounding a `for` range assigned in the loop, which does not change it.

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";

count_nonzero :: (xs: [] int) -> int {
    n := xs.count;

    for 0..n - 1 {
        if xs[it] == 0 n -= 1;
    }

    return n;
}

main :: () {
    print("%\n", count_nonzero(.[1, 0, 2]));
}
```

```diagnostic
warning[modified_range_bound]: `n` bounds this loop's range, which was fixed before the first iteration: changing it here does not change the loop
  --> example.jai:7:24
   |
 7 |         if xs[it] == 0 n -= 1;
   |                        ^
   |
   = help: use a `while` loop if the bound should follow the change, or a separate variable if it should not
   = note: `modified_range_bound` is `warn` by default
```

Fixed:

```jai
#import "Basic";

count_nonzero :: (xs: [] int) -> int {
    n := xs.count;

    for xs {
        if it == 0 n -= 1;
    }

    return n;
}

main :: () {
    print("%\n", count_nonzero(.[1, 0, 2]));
}
```

Only locals named directly in a range end (`n` in `0..n-1`); `xs.count` bounds are left alone.

### no_effect

Finds a statement that computes a value and drops it (`x == 5;`).

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";

main :: () {
    total := 1;
    total + 1;
    print("%\n", total);
}
```

```diagnostic
warning[no_effect]: this statement has no effect
  --> example.jai:5:5
   |
 5 |     total + 1;
   |     ^^^^^^^^^
   |
help: to update `total`, write `total += 1;`
   |
 5 ~     total += 1;
   |
   = note: `no_effect` is `warn` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    total := 1;
    total += 1;
    print("%\n", total);
}
```

An expression statement containing no call. The editor offers the likely intent: `==` becomes `=`, `x + 1;` becomes `x += 1;`, a procedure name gets `()`, and any other statement is removed.

### pointer_into_resizable

Finds a pointer into a resizable array used after a `@GrowsArray` call may have moved the array (`p := *xs[i]; array_add(*xs, v); use(p);`).

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";

Item :: struct {
    value: int;
}

main :: () {
    items: [..] Item;

    defer array_free(items);

    array_add(*items, .{ 1 });
    first := *items[0];

    array_add(*items, .{ 2 });

    print("%\n", first.value);
}
```

```diagnostic
warning[pointer_into_resizable]: `first` points into `items`, which `array_add` may have moved by now
  --> example.jai:17:18
   |
13 |     first := *items[0];
   |              --------- pointer derived here
   ...
15 |     array_add(*items, .{ 2 });
   |     ------------------------- `items` storage invalidated here
   ...
17 |     print("%\n", first.value);
   |                  ^^^^^ pointer used here after its storage was invalidated
   |
   = help: take the pointer again after the array has grown, or keep the index instead
   = note: `pointer_into_resizable` is `warn` by default
```

Fixed:

```jai
#import "Basic";

Item :: struct {
    value: int;
}

main :: () {
    items: [..] Item;

    defer array_free(items);

    array_add(*items, .{ 1 });
    first := items[0];

    array_add(*items, .{ 2 });

    print("%\n", first.value);
}
```

A resizable array that outgrows its storage gets a new block and frees the old one, so a pointer taken into the old block dangles. Uses the [lifetime engine](../tools/jailint.md#lifetime-rules); `pointer_lifetime` also covers removals and moves.

### push_in_static_if

Finds a scoped push directly in an `#if` arm, which lasts past the arm.

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";

USE_ARENA :: true;

main :: () {
    arena := context.allocator;

    #if USE_ARENA {
        push_allocator(arena);
    }

    print("%\n", alloc(8) != null);
}
```

```diagnostic
warning[push_in_static_if]: this push lasts until the end of the enclosing procedure, not the end of the `#if` arm
  --> example.jai:9:9
   |
 9 |         push_allocator(arena);
   |         ^^^^^^^^^^^^^^^^^^^^^
   |
   = help: wrap it and the code it covers in a `{ }` block, or move it out of the `#if`
   = note: `push_in_static_if` is `warn` by default
```

`#if` adds no scope, so `push_allocator(a);` directly in an `#if` or `#else` arm lasts to the end of the enclosing block or procedure, not the arm.

### self_assignment

Finds a value assigned to itself (`x = x;`).

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";

Size :: struct {
    width: int;
    height: int;
}

set_width :: (size: *Size, width: int) {
    width = width;
}

main :: () {
    s: Size;
    set_width(*s, 4);
    print("%\n", s);
}
```

```diagnostic
warning[self_assignment]: `width` is assigned to itself
  --> example.jai:9:5
   |
 9 |     width = width;
   |     ^^^^^^^^^^^^^
   |
help: this does nothing; was another value or target meant? (after `using`, both sides name the same field)
   |
 9 ~ 
   |
   = note: `self_assignment` is `warn` by default
```

Fixed:

```jai
#import "Basic";

Size :: struct {
    width: int;
    height: int;
}

set_width :: (size: *Size, width: int) {
    size.width = width;
}

main :: () {
    s: Size;
    set_width(*s, 4);
    print("%\n", s);
}
```

Not auto-fixed: the slip may be on the other side.

### shadowed_it

Finds a nested `for` hides an `it` the enclosing loop uses.

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";

main :: () {
    rows := int.[1, 2];

    for rows {
        for 0..it print("%", it);
        print(" row %\n", it);
    }
}
```

```diagnostic
warning[shadowed_it]: this loop's `it` hides the `it` of the loop on line 6, which is also used
  --> example.jai:7:9
   |
 7 |         for 0..it print("%", it);
   |         ^^^^^^^^^
   |
   = help: name one of the iterators: `for name: ...`
   = note: `shadowed_it` is `warn` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    rows := int.[1, 2];

    for row: rows {
        for 0..row print("%", it);
        print(" row %\n", row);
    }
}
```

`row := it;` before the inner loop is the usual idiom and is not reported.

### shift_overflow

Finds a shift by at least the bit width of its operand (`x << 40` with `x: s32`).

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";

high_word :: (bits: u32) -> u32 {
    return bits << 32;
}

main :: () {
    print("%\n", high_word(1));
}
```

```diagnostic
warning[shift_overflow]: shifting a `u32` by 32 shifts out all 32 of its bits
  --> example.jai:4:12
   |
 4 |     return bits << 32;
   |            ^^^^^^^^^^
   |
   = help: the shift count must be below 32; widen the value first, `(cast(u64) x) << 32`, if more bits are needed
   = note: `shift_overflow` is `warn` by default
```

Fixed:

```jai
#import "Basic";

high_word :: (bits: u64) -> u64 {
    return bits << 32;
}

main :: () {
    print("%\n", high_word(1));
}
```

The result is `0` (or `-1` for `>>` on a negative signed value) whatever the operand holds, and the compiler accepts it.

### unclosed_resource

Finds a local opened by an `@Opens` call (`file_open`, `append(*builder, ..)`) that nothing closes.

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";
#import "File";

is_readable :: (path: string) -> bool {
    file, ok := file_open(path);
    if !ok return false;

    return true;
}

main :: () {
    print("%\n", is_readable("notes.txt"));
}
```

```diagnostic
warning[unclosed_resource]: `file` is opened by `file_open` but nothing closes it
  --> example.jai:5:17
   |
 5 |     file, ok := file_open(path);
   |                 ^^^^^^^^^^^^^^^
   |
   = help: close it when done, for example with a `defer` right after opening
   = note: `unclosed_resource` is `warn` by default
```

Fixed:

```jai
#import "Basic";
#import "File";

is_readable :: (path: string) -> bool {
    file, ok := file_open(path);
    if !ok return false;
    defer file_close(*file);

    return true;
}

main :: () {
    print("%\n", is_readable("notes.txt"));
}
```

A local that an `@Opens` call acquired something for, and that nothing closes.

### unknown_allow

Finds a `// jailint: allow(...)` comment naming a rule that does not exist.

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";

// jailint: allow(float_equlity)
is_zero :: (x: float) -> bool {
    return x == 0.0;
}

main :: () {
    print("%\n", is_zero(0.0));
}
```

```diagnostic
warning[unknown_allow]: `float_equlity` is not a jailint rule, so this comment allows nothing
  --> example.jai:3:19
   |
 3 | // jailint: allow(float_equlity)
   |                   ^^^^^^^^^^^^^
   |
help: did you mean `float_equality`? (`--list` shows every rule)
   |
 3 ~ // jailint: allow(float_equality)
   |
   = note: `unknown_allow` is `warn` by default
```

Fixed:

```jai
#import "Basic";

// jailint: allow(float_equality)
is_zero :: (x: float) -> bool {
    return x == 0.0;
}

main :: () {
    print("%\n", is_zero(0.0));
}
```

A name in a line comment's `allow(...)` or `allow-file(...)` list that is not a rule, a category or `all`.

### unpaired_lock

Finds a `@Locks` call with a path that returns or ends without a matching `@Unlocks`.

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";
#import "Thread";

State :: struct {
    mutex: Mutex;
    count: int;
    closed: bool;
}

bump :: (state: *State) -> bool {
    lock(*state.mutex);
    if state.closed return false;

    state.count += 1;
    unlock(*state.mutex);

    return true;
}

main :: () {
    state: State;
    init(*state.mutex);
    print("%\n", bump(*state));
}
```

```diagnostic
warning[unpaired_lock]: this `return` leaves `*state.mutex` locked
  --> example.jai:12:21
   |
12 |     if state.closed return false;
   |                     ^^^^^^^^^^^^^
   |
   = help: unlock it before returning, or `defer` the unlock right after locking
   = note: `unpaired_lock` is `warn` by default
```

Fixed:

```jai
#import "Basic";
#import "Thread";

State :: struct {
    mutex: Mutex;
    count: int;
    closed: bool;
}

bump :: (state: *State) -> bool {
    lock(*state.mutex);
    defer unlock(*state.mutex);

    if state.closed return false;

    state.count += 1;

    return true;
}

main :: () {
    state: State;
    init(*state.mutex);
    print("%\n", bump(*state));
}
```

A statement-level call of a `@Locks` procedure with an argument (`lock(*state.mutex);`) at the top level of a procedure body, followed by a path that reaches a.

### unsigned_range_end_minus_one

Finds `for 0..n-1` with an unsigned `n`, which wraps around when `n` is 0.

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";

sum :: (data: *u8, n: u32) -> int {
    total := 0;

    for i: 0..n - 1 {
        total += data[i];
    }

    return total;
}

main :: () {
    bytes := u8.[1, 2, 3];
    print("%\n", sum(bytes.data, 3));
}
```

```diagnostic
warning[unsigned_range_end_minus_one]: `n` is unsigned, so this range runs past the end when it is 0
  --> example.jai:6:15
   |
 6 |     for i: 0..n - 1 {
   |               ^^^^^
   |
   = help: `n` has type `u32` and `- 1` wraps around at 0: check for 0 first, or count with a signed integer
   = note: `unsigned_range_end_minus_one` is `warn` by default
```

Fixed:

```jai
#import "Basic";

sum :: (data: *u8, n: s64) -> int {
    total := 0;

    for i: 0..n - 1 {
        total += data[i];
    }

    return total;
}

main :: () {
    bytes := u8.[1, 2, 3];
    print("%\n", sum(bytes.data, 3));
}
```

Ranges include their end, so `0..n-1` visits the `n` indices below `n`, but with an unsigned `n` of `0` the end wraps around (or stops the program with overflow checks on) instead of giving an empty loop.

### unused_private_declaration

Finds a `#scope_file` declaration that nothing in its file uses.

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";

main :: () {
    print("hello\n");
}

#scope_file

LIMIT :: 64;
```

```diagnostic
warning[unused_private_declaration]: `LIMIT` is private to this file and nothing in the file uses it
  --> example.jai:9:1
   |
 9 | LIMIT :: 64;
   | ^^^^^
   |
   = help: delete it, or put it under `#scope_export` if another file should use it
   = note: `unused_private_declaration` is `warn` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    print("hello\n");
}
```

A procedure, struct, enum, constant or variable declared after `#scope_file` whose name appears nowhere else in its file (mentions in its own declaration do not count, so a procedure that only calls itself is unused; mentions in code an `#if` left out do).

### unused_result

Finds a call to a `@UseResult` procedure, as a statement.

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";
#import "String";

main :: () {
    line := "  hello  ";

    trim(line);

    print("[%]\n", line);
}
```

```diagnostic
warning[unused_result]: the result of `trim` is discarded; it does not change its arguments
  --> example.jai:7:5
   |
 7 |     trim(line);
   |     ^^^^^^^^^^
   |
help: to update the argument, write `line = trim(line);`
   |
 7 ~     line = trim(line);
   |
   = note: `unused_result` is `warn` by default
```

Fixed:

```jai
#import "Basic";
#import "String";

main :: () {
    line := "  hello  ";

    line = trim(line);

    print("[%]\n", line);
}
```

Procedures declared with `@UseResult` (many in `Basic`/`String`/`Math`), so a program's own `trim` is left alone unless it carries the note.

### unused_suppression

Finds a name in a `// jailint: allow(...)` comment or `@jailint_allow` note that silences nothing.

Category `suspicious`, `warn` by default.

Example:

```jai
#import "Basic";

scale :: (a: int) -> int {
    // jailint: allow(unused_variable)
    total := a * 2;

    return total;
}

main :: () {
    print("%\n", scale(4));
}
```

```diagnostic
warning[unused_suppression]: `unused_variable` allows nothing here
  --> example.jai:4:23
   |
 4 |     // jailint: allow(unused_variable)
   |                       ^^^^^^^^^^^^^^^
   |
   = help: nothing it covers is reported any more; remove it
   = note: `unused_suppression` is `warn` by default
```

Fixed:

```jai
#import "Basic";

scale :: (a: int) -> int {
    total := a * 2;

    return total;
}

main :: () {
    print("%\n", scale(4));
}
```

A name in a `// jailint: allow(...)`, `allow-file(...)` or `@jailint_allow(...)` that no finding was silenced by, reported on the name (or on the whole note).

## Complexity

Code that says something in a roundabout way. Default: `warn`.

### bool_comparison

Finds comparing a bool with `true` or `false`.

Category `complexity`, `warn` by default.

Example:

```jai
#import "Basic";

main :: () {
    ready := true;

    if ready == true print("ready\n");
}
```

```diagnostic
warning[bool_comparison]: comparing a bool with `true`: `ready == true`
  --> example.jai:6:8
   |
 6 |     if ready == true print("ready\n");
   |        ^^^^^^^^^^^^^
   |
help: write `ready`
   |
 6 ~     if ready print("ready\n");
   |
   = note: `bool_comparison` is `warn` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    ready := true;

    if ready print("ready\n");
}
```

Only when the other side is a non-constant `bool`.

### identity_op

Finds an operation that changes nothing (`x + 0`, `x * 1`).

Category `complexity`, `warn` by default.

Example:

```jai
#import "Basic";

size_in_bytes :: (count: int) -> int {
    return count * 1;
}

main :: () {
    print("%\n", size_in_bytes(8));
}
```

```diagnostic
warning[identity_op]: `count * 1` is just `count`: `*` with this operand changes nothing
  --> example.jai:4:12
   |
 4 |     return count * 1;
   |            ^^^^^^^^^
   |
help: write `count`
   |
 4 ~     return count;
   |
   = note: `identity_op` is `warn` by default
```

Fixed:

```jai
#import "Basic";

size_in_bytes :: (count: int) -> int {
    return count;
}

main :: () {
    print("%\n", size_in_bytes(8));
}
```

`0 + x`, operations inside indexes/array literals and shifts by `0` are left alone (they line up with neighbours).

### index_only_loop

Finds `for i: 0..xs.count-1` where `i` only indexes `xs`.

Category `complexity`, `warn` by default.

Example:

```jai
#import "Basic";

sum :: (xs: [] int) -> int {
    total := 0;

    for i: 0..xs.count - 1 {
        total += xs[i];
    }

    return total;
}

main :: () {
    print("%\n", sum(.[1, 2, 3]));
}
```

```diagnostic
warning[index_only_loop]: `i` is only used to index `xs`
  --> example.jai:6:5
   |
 6 |     for i: 0..xs.count - 1 {
   |     ^^^^^^^^^^^^^^^^^^^^^^
   |
help: loop over the elements: `for xs` and `it`
   |
 6 ~     for xs {
 7 ~         total += it;
   |
   = note: `index_only_loop` is `warn` by default
```

Fixed:

```jai
#import "Basic";

sum :: (xs: [] int) -> int {
    total := 0;

    for xs {
        total += it;
    }

    return total;
}

main :: () {
    print("%\n", sum(.[1, 2, 3]));
}
```

Becomes `for xs` (or `for *xs` when elements are written), other uses of `i` become `it_index`.

### int_plus_one

Finds `x >= y + 1`, which is `x > y`.

Category `complexity`, `warn` by default.

Example:

```jai
#import "Basic";

over_limit :: (count: int, limit: int) -> bool {
    return count >= limit + 1;
}

main :: () {
    print("%\n", over_limit(5, 4));
}
```

```diagnostic
warning[int_plus_one]: `count >= limit + 1` is a strict comparison written with `1`
  --> example.jai:4:12
   |
 4 |     return count >= limit + 1;
   |            ^^^^^^^^^^^^^^^^^^
   |
help: write `count > limit`
   |
 4 ~     return count > limit;
   |
   = note: `int_plus_one` is `warn` by default
```

Fixed:

```jai
#import "Basic";

over_limit :: (count: int, limit: int) -> bool {
    return count > limit;
}

main :: () {
    print("%\n", over_limit(5, 4));
}
```

Integers only, and only the literal `1`. The editor offers the strict comparison; `--fix` leaves it, since where `y + 1` wraps (the top of the type) the two forms differ.

### loop_with_break_condition

Finds `while true { if !c break; ... }`, a loop whose condition is its first statement.

Category `complexity`, `warn` by default.

Example:

```jai
#import "Basic";

main :: () {
    i := 0;

    while true {
        if i >= 10 break;
        i += 1;
    }

    print("%\n", i);
}
```

```diagnostic
warning[loop_with_break_condition]: this loop leaves on its first statement; the test belongs in the `while`
  --> example.jai:6:11
   |
 6 |     while true {
   |           ^^^^
   |
help: write `while i < 10` and drop the `if`
   |
 6 -     while true {
 7 -         if i >= 10 break;
 6 +     while i < 10 {
   |
   = note: `loop_with_break_condition` is `warn` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    i := 0;

    while i < 10 {
        i += 1;
    }

    print("%\n", i);
}
```

A `while true` (macros aside) whose body starts with an `if` without an `else` that only does an unlabelled `break`, with or without braces.

### manual_array_find

Finds a loop that searches an array and breaks with a found flag or index, which `array_find` does.

Category `complexity`, `warn` by default.

Example:

```jai
#import "Basic";

has :: (names: [] string, wanted: string) -> bool {
    found := false;

    for names {
        if it == wanted {
            found = true;
            break;
        }
    }

    return found;
}

main :: () {
    print("%\n", has(.["ada", "grace"], "grace"));
}
```

```diagnostic
warning[manual_array_find]: this loop looks for `wanted` in `names` by hand
  --> example.jai:6:5
   |
 6 |     for names {
   |     ^^^^^^^^^^
   |
   = help: `array_find(names, wanted)` returns whether it was found, and the index after that
   = note: `manual_array_find` is `warn` by default
```

Fixed:

```jai
#import "Basic";

has :: (names: [] string, wanted: string) -> bool {
    found := array_find(names, wanted);
    return found;
}

main :: () {
    print("%\n", has(.["ada", "grace"], "grace"));
}
```

Fires on a `for` over an array whose body is one `if` without an `else`: it compares the loop's value (`it`, or the name given to.

### manual_assign_op

Finds `a = a + b` instead of `a += b`.

Category `complexity`, `warn` by default.

Example:

```jai
#import "Basic";

main :: () {
    total := 10;
    price := 5;

    total = total + price;

    print("%\n", total);
}
```

```diagnostic
warning[manual_assign_op]: `total` is updated with `+` spelled out
  --> example.jai:7:5
   |
 7 |     total = total + price;
   |     ^^^^^^^^^^^^^^^^^^^^^
   |
help: write `total += price`
   |
 7 ~     total += price;
   |
   = note: `manual_assign_op` is `warn` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    total := 10;
    price := 5;

    total += price;

    print("%\n", total);
}
```

Target must be side-effect-free and the operation must keep its type.

### manual_clamp

Finds a value kept between two constant bounds by hand instead of with `clamp`.

Category `complexity`, `warn` by default.

Example:

```jai
#import "Basic";

main :: () {
    volume := 140;
    volume = max(0, min(100, volume));
    print("%\n", volume);
}
```

```diagnostic
warning[manual_clamp]: `volume` is kept between `0` and `100` by hand
  --> example.jai:5:14
   |
 5 |     volume = max(0, min(100, volume));
   |              ^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: write `clamp(volume, 0, 100)`
   |
 5 ~     volume = clamp(volume, 0, 100);
   |
   = note: `manual_clamp` is `warn` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    volume := 140;
    volume = clamp(volume, 0, 100);
    print("%\n", volume);
}
```

Numbers with constant bounds, the lower not above the upper (a swapped pair is `min_max`'s).

### manual_index_counter

Finds a counter incremented in a `for` loop that always equals `it_index`.

Category `complexity`, `warn` by default.

Example:

```jai
#import "Basic";

show :: (names: [] string) {
    n := 0;

    for names {
        print("% %\n", n, it);
        n += 1;
    }
}

main :: () {
    show(.["ada", "grace"]);
}
```

```diagnostic
warning[manual_index_counter]: `n` always equals `it_index` of the loop that follows
  --> example.jai:4:5
   |
 4 |     n := 0;
   |     ^
   |
help: use `it_index` instead of counting by hand
   |
 4 -     n := 0;
 5 - 
 6 -     for names {
 7 -         print("% %\n", n, it);
 8 -         n += 1;
 4 + 
 5 +     for names {
 6 +         print("% %\n", it_index, it);
   |
   = note: `manual_index_counter` is `warn` by default
```

Fixed:

```jai
#import "Basic";

show :: (names: [] string) {
    for names {
        print("% %\n", it_index, it);
    }
}

main :: () {
    show(.["ada", "grace"]);
}
```

`n := 0` right before a forward `for` (in a block, a `case` body or a `#if` branch), `n += 1` at the end of the body (or `defer n += 1`), `n` unused after the loop.

### needless_bool

Finds `ifx c then true else false` and `if c return true; else return false;`.

Category `complexity`, `warn` by default.

Example:

```jai
#import "Basic";

is_positive :: (n: int) -> bool {
    if n > 0 return true;
    else return false;
}

main :: () {
    print("%\n", is_positive(3));
}
```

```diagnostic
warning[needless_bool]: this `if` returns its own condition as a `bool`
  --> example.jai:4:5
   |
 4 |     if n > 0 return true;
   |     ^^^^^^^^^^^^^^^^^^^^^
   |
help: write `return n > 0;`
   |
 4 -     if n > 0 return true;
 5 -     else return false;
 4 +     return n > 0;
   |
   = note: `needless_bool` is `warn` by default
```

Fixed:

```jai
#import "Basic";

is_positive :: (n: int) -> bool {
    return n > 0;
}

main :: () {
    print("%\n", is_positive(3));
}
```

A trailing `if c return true; return false;` is left alone when an earlier guard in the block also returns a literal.

### redundant_alias

Finds a local that only exists to be copied into another local or returned.

Category `complexity`, `warn` by default.

Example:

```jai
#import "Basic";

square :: (n: int) -> int {
    result := n * n;
    return result;
}

main :: () {
    print("%\n", square(4));
}
```

```diagnostic
warning[redundant_alias]: `result` is returned right after it is declared
  --> example.jai:4:5
   |
 4 |     result := n * n;
   |     ^^^^^^
   |
help: return the value directly
   |
 4 -     result := n * n;
 5 -     return result;
 4 +     return n * n;
   |
   = note: `redundant_alias` is `warn` by default
```

Fixed:

```jai
#import "Basic";

square :: (n: int) -> int {
    return n * n;
}

main :: () {
    print("%\n", square(4));
}
```

Style rule, deliberately narrow so the fix never changes behaviour: the two statements are adjacent with nothing (not even a comment) between, the first name appears exactly twice in the body, the types are identical, and no `#insert`, `using`, notes or flags are involved.

### redundant_cast

Finds a cast to the type the value already has.

Category `complexity`, `warn` by default.

Example:

```jai
#import "Basic";

main :: () {
    x: s32 = 5;
    y := cast(s32) x;
    print("%\n", y);
}
```

```diagnostic
warning[redundant_cast]: `x` is already `s32`
  --> example.jai:5:10
   |
 5 |     y := cast(s32) x;
   |          ^^^^^^^^^^^
   |
help: remove the cast
   |
 5 ~     y := x;
   |
   = note: `redundant_cast` is `warn` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    x: s32 = 5;
    y := x;
    print("%\n", y);
}
```

Aliases like `c_long` differ per platform, so the value must be declared with the same type spelling.

### redundant_deref

Finds `p.*.field` where `p.field` means the same.

Category `complexity`, `warn` by default.

Example:

```jai
#import "Basic";

Point :: struct {
    x: int;
    y: int;
}

main :: () {
    p := Point.{ 1, 2 };
    q := *p;

    print("%\n", q.*.x + q.y);
}
```

```diagnostic
warning[redundant_deref]: `q.*` is not needed to reach `x`
  --> example.jai:12:19
   |
12 |     print("%\n", q.*.x + q.y);
   |                   ^^
   |
help: write `q.x`: member access already looks through the pointer
   |
12 ~     print("%\n", q.x + q.y);
   |
   = note: `redundant_deref` is `warn` by default
```

Fixed:

```jai
#import "Basic";

Point :: struct {
    x: int;
    y: int;
}

main :: () {
    p := Point.{ 1, 2 };
    q := *p;

    print("%\n", q.x + q.y);
}
```

Fires when a member is taken from the dereference of an expression whose type is a pointer to a struct or union.

### unnecessary_force_cast

Finds `cast,force` where the value already has the type or the conversion widens.

Category `complexity`, `warn` by default.

Example:

```jai
#import "Basic";

widen :: (a: s32) -> s64 {
    return cast,force(s64) a;
}

main :: () {
    print("%\n", widen(7));
}
```

```diagnostic
warning[unnecessary_force_cast]: `cast,force` from `s32` to `s64` reinterprets nothing and has no range to check
  --> example.jai:4:12
   |
 4 |     return cast,force(s64) a;
   |            ^^^^^^^^^^^^^^
   |
help: remove `,force`: the plain cast does the same
   |
 4 ~     return cast(s64) a;
   |
   = note: `unnecessary_force_cast` is `warn` by default
```

Fixed:

```jai
#import "Basic";

widen :: (a: s32) -> s64 {
    return cast(s64) a;
}

main :: () {
    print("%\n", widen(7));
}
```

`,force` reinterprets the bits of a number as a float (or back) or of an aggregate as another of the same size, and also skips the range check a plain cast makes when it narrows an integer.

### useless_format

Finds `tprint("%", s)` or `sprint("%", s)` with a string `s`, which only copies it.

Category `complexity`, `warn` by default.

Example:

```jai
#import "Basic";

greeting :: (name: string) -> string {
    return tprint("%", name);
}

main :: () {
    print("%\n", greeting("Ada"));
}
```

```diagnostic
warning[useless_format]: `tprint("%", name)` only copies a string
  --> example.jai:4:12
   |
 4 |     return tprint("%", name);
   |            ^^^^^^^^^^^^^^^^^
   |
help: use the string itself, or `copy_string` for a copy you own
   |
 4 ~     return name;
   |
   = note: `useless_format` is `warn` by default
```

Fixed:

```jai
#import "Basic";

greeting :: (name: string) -> string {
    return name;
}

main :: () {
    print("%\n", greeting("Ada"));
}
```

Fires on a call of a procedure noted `@FormatsToString` (the library's `tprint` and `sprint`) whose format is exactly `"%"` and whose single value is a `string`.

## Style

Code that works but is not written the usual way. Default: `warn`.

### approx_constant

Finds a hand-typed `3.14159` or `1.4142` where `Math` has the constant.

Category `style`, `warn` by default.

Example:

```jai
#import "Basic";

area :: (r: float) -> float {
    return 3.14159 * r * r;
}

main :: () {
    print("%\n", area(2));
}
```

```diagnostic
warning[approx_constant]: `3.14159` is an approximation of `PI`
  --> example.jai:4:12
   |
 4 |     return 3.14159 * r * r;
   |            ^^^^^^^
   |
help: write `PI` from `Math` (`PI64` for `float64`)
   |
 4 ~     return PI * r * r;
   |
   = note: `approx_constant` is `warn` by default
```

Fixed:

```jai
#import "Basic";
#import "Math";

area :: (r: float) -> float {
    return PI * r * r;
}

main :: () {
    print("%\n", area(2));
}
```

Needs at least four significant digits (`3.14` is a rough value on purpose) and a match where the literal stops, rounded or cut off.

### assert_constant

Finds `assert(true)` or `#assert true`, which can never fail.

Category `style`, `warn` by default.

Example:

```jai
#import "Basic";

main :: () {
    x := 3;

    assert(true, "x is positive");

    print("%\n", x);
}
```

```diagnostic
warning[assert_constant]: `assert(true)` can never fail
  --> example.jai:6:5
   |
 6 |     assert(true, "x is positive");
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: remove it, or assert the condition it stands in for
   |
 6 -     assert(true, "x is positive");
 7 - 
   |
   = note: `assert_constant` is `warn` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    x := 3;

    assert(x > 0, "x is positive");

    print("%\n", x);
}
```

The call must be to a procedure with the `@AssertsArgument` note (the library's `assert`).

### banner_comment

Finds a comment that is only a line of `-`, `=` or `*` fencing off a section.

Category `style`, `warn` by default.

Example:

```jai
#import "Basic";

// -----------------------------------------------------------------------------
// Helpers

double :: (n: int) -> int {
    return n * 2;
}

main :: () {
    print("%\n", double(4));
}
```

```diagnostic
warning[banner_comment]: this comment is only a line of rule characters
  --> example.jai:3:1
   |
 3 | // -----------------------------------------------------------------------------
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: drop the line, and name the section in one plain `//` line if it needs a name; a long section can move to its own file
   |
 3 - // -----------------------------------------------------------------------------
   |
   = note: `banner_comment` is `warn` by default
```

Fixed:

```jai
#import "Basic";

// Helpers

double :: (n: int) -> int {
    return n * 2;
}

main :: () {
    print("%\n", double(4));
}
```

A comment of three or more of `-=*#~_` and nothing else, line or block (`// ------`).

### broken_doc_link

Finds a link in a doc comment (`[name]`) that names no declaration.

Category `style`, `warn` by default.

Example:

```jai
#import "Basic";

Point :: struct {
    x: int;
    y: int;
}

// Scales a [Point] and prints it with [prnt].
scale :: (p: Point) {
    print("%\n", p.x * 2);
}

main :: () {
    scale(.{ 1, 2 });
}
```

```diagnostic
warning[broken_doc_link]: `[prnt]` doesn't resolve to a declaration
  --> example.jai:8:40
   |
 8 | // Scales a [Point] and prints it with [prnt].
   |                                        ^^^^^^ not found from the documented declaration's scope
   |
help: did you mean `print`?
   |
 8 ~ // Scales a [Point] and prints it with [print].
   |
   = note: `broken_doc_link` is `warn` by default
```

Fixed:

```jai
#import "Basic";

Point :: struct {
    x: int;
    y: int;
}

// Scales a [Point] and prints it with [print].
scale :: (p: Point) {
    print("%\n", p.x * 2);
}

main :: () {
    scale(.{ 1, 2 });
}
```

Only comments directly above a declaration (or trailing a field or parameter) are read, and links resolve like the language server's.

### enum_member_prefix

Finds every member of an enum starts with the enum's name.

Category `style`, `warn` by default.

Example:

```jai
#import "Basic";

Color :: enum {
    COLOR_RED;
    COLOR_GREEN;
    COLOR_BLUE;
}

main :: () {
    print("%\n", Color.COLOR_GREEN);
}
```

```diagnostic
warning[enum_member_prefix]: every member of `Color` starts with `Color`
  --> example.jai:3:1
   |
 3 | Color :: enum {
   | ^^^^^
   |
   = help: members are used as `Color.NAME`, so the prefix repeats the enum's name: drop it
   = note: `enum_member_prefix` is `warn` by default
```

Fixed:

```jai
#import "Basic";

Color :: enum {
    RED;
    GREEN;
    BLUE;
}

main :: () {
    print("%\n", Color.GREEN);
}
```

Two or more members, all beginning with the enum's name once case and `_` are ignored (`Token_Kind` and `TOKEN_KIND_IDENT`) and each with something left after it.

### lossy_float_literal

Finds a `float32` literal with more digits than a `float32` keeps (`31415.92653`).

Category `style`, `warn` by default.

Example:

```jai
#import "Basic";

main :: () {
    scale: float32 = 16777.2171234;
    print("%\n", scale);
}
```

```diagnostic
warning[lossy_float_literal]: `16777.2171234` has 12 significant digits, but a `float32` keeps 8
  --> example.jai:4:22
   |
 4 |     scale: float32 = 16777.2171234;
   |                      ^^^^^^^^^^^^^
   |
help: write `16777.217`, which is the same `float32`
   |
 4 ~     scale: float32 = 16777.217;
   |
   = note: `lossy_float_literal` is `warn` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    scale: float32 = 16777.217;
    print("%\n", scale);
}
```

A `float32` keeps about seven decimal digits. Digits past the shortest spelling that reads back as the same `float32` change nothing and suggest a precision the program does not have.

### mixed_case_hex_literals

Finds a hex literal with both upper and lower case digits (`0xDeadbeef`).

Category `style`, `warn` by default.

Example:

```jai
#import "Basic";

main :: () {
    mask := 0xFFfF;
    print("%\n", mask);
}
```

```diagnostic
warning[mixed_case_hex_literals]: `0xFFfF` mixes upper and lower case digits
  --> example.jai:4:13
   |
 4 |     mask := 0xFFfF;
   |             ^^^^^^
   |
help: write `0xFFFF`
   |
 4 ~     mask := 0xFFFF;
   |
   = note: `mixed_case_hex_literals` is `warn` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    mask := 0xFFFF;
    print("%\n", mask);
}
```

Read from the tokens, so it needs no types; literals in macros and generated files are skipped.

### needless_continue

Finds a `continue` as the last thing a loop body does.

Category `style`, `warn` by default.

Example:

```jai
#import "Basic";

main :: () {
    items := int.[1, 2, 3, 4];
    total := 0;

    for items {
        total += it;
        continue;
    }

    print("%\n", total);
}
```

```diagnostic
warning[needless_continue]: this `continue` is the last thing the loop does
  --> example.jai:9:9
   |
 9 |         continue;
   |         ^^^^^^^^^
   |
help: remove it
   |
 9 -         continue;
   |
   = note: `needless_continue` is `warn` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    items := int.[1, 2, 3, 4];
    total := 0;

    for items {
        total += it;
    }

    print("%\n", total);
}
```

An unlabelled `continue` that ends a `for` or `while` body, or ends a branch of an `if` that is the body's last statement.

### neg_multiply

Finds `x * -1`, which is `-x`.

Category `style`, `warn` by default.

Example:

```jai
#import "Basic";

flip :: (x: int) -> int {
    return x * -1;
}

main :: () {
    print("%\n", flip(5));
}
```

```diagnostic
warning[neg_multiply]: `x * -1` is a negation
  --> example.jai:4:12
   |
 4 |     return x * -1;
   |            ^^^^^^
   |
help: write `-x`
   |
 4 ~     return -x;
   |
   = note: `neg_multiply` is `warn` by default
```

Fixed:

```jai
#import "Basic";

flip :: (x: int) -> int {
    return -x;
}

main :: () {
    print("%\n", flip(5));
}
```

Signed integers and floats only, and only the literals `-1` and `-1.0`.

### single_case_switch

Finds a `switch` with a single case, which is an `if`.

Category `style`, `warn` by default.

Example:

```jai
#import "Basic";

Mode :: enum {
    QUIET;
    VERBOSE;
    DEBUG;
}

announce :: (mode: Mode) {
    if mode == {
        case .VERBOSE;
            print("loud\n");
    }
}

main :: () {
    announce(.VERBOSE);
}
```

```diagnostic
warning[single_case_switch]: this `switch` has a single case
  --> example.jai:10:5
   |
10 |     if mode == {
   |     ^^^^^^^^^^^^
   |
help: write an `if` instead
   |
10 -     if mode == {
11 -         case .VERBOSE;
12 -             print("loud\n");
13 -     }
10 +     if mode == .VERBOSE {
11 +         print("loud\n");
12 +     }
   |
   = note: `single_case_switch` is `warn` by default
```

Fixed:

```jai
#import "Basic";

Mode :: enum {
    QUIET;
    VERBOSE;
    DEBUG;
}

announce :: (mode: Mode) {
    if mode == .VERBOSE print("loud\n");
}

main :: () {
    announce(.VERBOSE);
}
```

One case of one value, plus at most a default case.

### unnecessary_semicolon

Finds a `;` that ends nothing (`x = 1;;`, `if c { ... };`).

Category `style`, `warn` by default.

Example:

```jai
#import "Basic";

main :: () {
    total := 0;;
    total += 1;
    print("%\n", total);
}
```

```diagnostic
warning[unnecessary_semicolon]: this `;` ends nothing
  --> example.jai:4:16
   |
 4 |     total := 0;;
   |                ^
   |
help: remove it
   |
 4 ~     total := 0;
   |
   = note: `unnecessary_semicolon` is `warn` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    total := 0;
    total += 1;
    print("%\n", total);
}
```

An empty statement in a block, a `case`, a procedure body or at the top level of a file.

### unused_import

Finds an `#import` nothing in its scope uses.

Category `style`, `warn` by default.

Example:

```jai
#import "Basic";
#import "Math";

main :: () {
    print("hello\n");
}
```

```diagnostic
warning[unused_import]: unused import "Math"
  --> example.jai:2:1
   |
 2 | #import "Math";
   | ^^^^^^^^^^^^^^
   |
help: remove it
   |
 2 - #import "Math";
   |
   = note: `unused_import` is `warn` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    print("hello\n");
}
```

Unnamed imports count as used when any lookup went through them, or (guard for unchecked code) when the module's files mention a name it exports.

### unused_parameter

Finds a parameter the procedure never uses.

Category `style`, `warn` by default.

Example:

```jai
#import "Basic";

area_of :: (value: int, factor: int, label: string) -> int {
    return value * factor;
}

main :: () {
    print("%\n", area_of(2, 3, "six"));
}
```

```diagnostic
warning[unused_parameter]: unused parameter `label`
  --> example.jai:3:38
   |
 3 | area_of :: (value: int, factor: int, label: string) -> int {
   |                                      ^^^^^
   |
help: start its name with `_` if the signature must stay, or remove it
   |
 3 ~ area_of :: (value: int, factor: int, _label: string) -> int {
   |
   = note: `unused_parameter` is `warn` by default
```

Fixed:

```jai
#import "Basic";

area_of :: (value: int, factor: int) -> int {
    return value * factor;
}

main :: () {
    print("%\n", area_of(2, 3));
}
```

Only for procedures whose every caller is in view: named, only called (never taken as a value), called at least once, not overloaded or exported, no `#c_call`/`#foreign`/`#expand`/notes.

### unused_variable

Finds a local variable that is never used.

Category `style`, `warn` by default.

Example:

```jai
#import "Basic";

main :: () {
    leftover := 5;
    print("hello\n");
}
```

```diagnostic
warning[unused_variable]: unused variable `leftover`
  --> example.jai:4:5
   |
 4 |     leftover := 5;
   |     ^^^^^^^^
   |
help: remove it
   |
 4 -     leftover := 5;
   |
   = note: `unused_variable` is `warn` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    print("hello\n");
}
```

A name that appears anywhere else in its block is never reported (covers `#if`-excluded code and `#asm`).

### zero_ptr

Finds `cast(*T) 0`, which is `null`.

Category `style`, `warn` by default.

Example:

```jai
#import "Basic";

main :: () {
    p := cast(*u8) 0;
    print("%\n", p);
}
```

```diagnostic
warning[zero_ptr]: `cast(*u8) 0` is a null pointer
  --> example.jai:4:10
   |
 4 |     p := cast(*u8) 0;
   |          ^^^^^^^^^^^
   |
help: write `null`
   |
 4 ~     p := null;
   |
   = note: `zero_ptr` is `warn` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    p: *u8 = null;
    print("%\n", p);
}
```

Only `cast(*T) 0` (also `0x0`) without cast modifiers.

## Perf

Code that could be faster. Default: `warn`.

### format_push_string

Finds `append(*b, tprint(...))`, which formats into a new string to add it to a builder.

Category `perf`, `warn` by default.

Example:

```jai
#import "Basic";

describe :: (name: string, value: int) -> string {
    builder: String_Builder;
    defer free_buffers(*builder);

    append(*builder, tprint("%: %\n", name, value));

    return builder_to_string(*builder);
}

main :: () {
    print("%\n", describe("x", 1));
}
```

```diagnostic
warning[format_push_string]: `tprint` formats into a new string just to append it to a builder
  --> example.jai:7:5
   |
 7 |     append(*builder, tprint("%: %\n", name, value));
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: `print_to_builder` writes into the builder directly
   |
 7 ~     print_to_builder(*builder, "%: %\n", name, value);
   |
   = note: `format_push_string` is `warn` by default
```

Fixed:

```jai
#import "Basic";

describe :: (name: string, value: int) -> string {
    builder: String_Builder;
    defer free_buffers(*builder);

    print_to_builder(*builder, "%: %\n", name, value);

    return builder_to_string(*builder);
}

main :: () {
    print("%\n", describe("x", 1));
}
```

Fires on a call of a procedure noted `@AppendsString` (the library's `append` of a string) whose string is a call of one noted `@FormatsToString` (`tprint`, `sprint`).

### manual_memcpy

Finds `for i: 0..n-1 { dst[i] = src[i]; }`, an element-by-element copy `memcpy` does at once.

Category `perf`, `warn` by default.

Example:

```jai
#import "Basic";

copy_all :: (dst: *int, src: [] int, count: int) {
    for i: 0..count - 1 {
        dst[i] = src[i];
    }
}

main :: () {
    values := int.[1, 2, 3, 4];
    out: [4] int;

    copy_all(out.data, values, 4);
    print("%\n", out);
}
```

```diagnostic
warning[manual_memcpy]: this loop copies `src` into `dst` one element at a time
  --> example.jai:4:5
   |
 4 |     for i: 0..count - 1 {
   |     ^^^^^^^^^^^^^^^^^^^^
   |
   = help: copy the elements at once: `memcpy(dst, src.data, count * size_of(s64))`
   = note: `manual_memcpy` is `warn` by default
```

Fixed:

```jai
#import "Basic";

copy_all :: (dst: *int, src: [] int, count: int) {
    memcpy(dst, src.data, count * size_of(int));
}

main :: () {
    values := int.[1, 2, 3, 4];
    out: [4] int;

    copy_all(out.data, values, 4);
    print("%\n", out);
}
```

Fires on a `for` over a range starting at `0` whose only statement is `dst[i] = src[i];` with `i` the loop's variable, `dst` and `src` different names or member paths, both arrays or pointers, and the same element type on both sides.

### useless_resizable

Finds a local `[..] T` that is only filled with a known number of `array_add` calls and never resized.

Category `perf`, `warn` by default.

Example:

```jai
#import "Basic";

main :: () {
    parts: [..] string;

    defer array_free(parts);

    array_add(*parts, "a");
    array_add(*parts, "b");

    for parts print("%\n", it);
}
```

```diagnostic
warning[useless_resizable]: `parts` is a resizable array that only ever holds 2 elements
  --> example.jai:4:5
   |
 4 |     parts: [..] string;
   |     ^^^^^
   |
   = help: a fixed array does not allocate: `parts: [2] string;`, and the call that frees it goes away
   = note: `useless_resizable` is `warn` by default
```

Fixed:

```jai
#import "Basic";

main :: () {
    parts := string.["a", "b"];

    for parts print("%\n", it);
}
```

A resizable array allocates and has to be freed; a handful of elements known up front fit in a fixed array on the stack (`parts: [2] string`).

## Pedantic

Stricter checks with more false positives. Default: `allow`.

### bool_to_int_with_if

Finds `ifx c then 1 else 0`, a `bool` turned into an integer with a branch.

Category `pedantic`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

main :: () {
    ready := true;
    count := ifx ready then 1 else 0;
    print("%\n", count);
}
```

```diagnostic
warning[bool_to_int_with_if]: a `bool` is turned into an integer with a branch
  --> example.jai:5:14
   |
 5 |     count := ifx ready then 1 else 0;
   |              ^^^^^^^^^^^^^^^^^^^^^^^
   |
help: write `cast(s64) ready`
   |
 5 ~     count := cast(s64) ready;
   |
```

Fixed:

```jai
#import "Basic";

main :: () {
    ready := true;
    count := cast(int) ready;
    print("%\n", count);
}
```

Off by default, since many read the `ifx` more easily than the cast.

### cognitive_complexity

Finds a procedure whose branching scores above `max_complexity` (default 25).

Category `pedantic`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

sum_above :: (items: [] int, limit: int) -> int {
    total := 0;

    for items {
        if it < 0 continue;

        if it > limit {
            total += it;
        }
    }

    return total;
}

main :: () {
    print("%\n", sum_above(.[3, -1, 8], 2));
}
```

Shown with these settings:

```toml
[lint]
max_complexity = 4
```

```diagnostic
warning[cognitive_complexity]: `sum_above` has a cognitive complexity of 5, past the limit of 4
  --> example.jai:3:1
   |
 3 | sum_above :: (items: [] int, limit: int) -> int {
   | ^^^^^^^^^
   |
   = help: split it into smaller procedures, return early, or name parts of long conditions
```

Fixed:

```jai
#import "Basic";

sum_above :: (items: [] int, limit: int) -> int {
    total := 0;

    for items {
        if it > limit && it >= 0 total += it;
    }

    return total;
}

main :: () {
    print("%\n", sum_above(.[3, -1, 8], 2));
}
```

Off by default. Scores how hard a procedure is to follow.

### detached_doc_comment

Finds a comment that reads as a declaration's docs but has a blank line between it and the declaration.

Category `pedantic`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

// Adds one to the number it is given.

bump :: (n: int) -> int {
    return n + 1;
}

main :: () {
    print("%\n", bump(1));
}
```

```diagnostic
warning[detached_doc_comment]: this comment is separated from `bump` by a blank line, so it is not its doc comment
  --> example.jai:3:1
   |
 3 | // Adds one to the number it is given.
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: remove the blank line to document it
   |
 4 - 
   |
```

Fixed:

```jai
#import "Basic";

// Adds one to the number it is given.
bump :: (n: int) -> int {
    return n + 1;
}

main :: () {
    print("%\n", bump(1));
}
```

A top-level procedure, struct or enum with a blank line between its declaration and a comment above that reads as a sentence (three or more words.

### excessive_nesting

Finds blocks nested deeper than `max_nesting` (default 6).

Category `pedantic`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

count_sums :: (a: [] int, b: [] int, c: [] int, target: int) -> int {
    hits := 0;

    for x: a {
        if x > 0 {
            for y: b {
                if y > 0 {
                    for z: c {
                        if z > 0 {
                            if x + y + z == target {
                                hits += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    return hits;
}

main :: () {
    print("%\n", count_sums(.[1, 2], .[3, 4], .[5, 6], 9));
}
```

```diagnostic
warning[excessive_nesting]: this block is nested 7 deep, past the limit of 6
  --> example.jai:12:52
   |
12 |                             if x + y + z == target {
   |                                                    ^
   |
   = help: return or `continue` early, or move the inner part into its own procedure
```

Fixed:

```jai
#import "Basic";

count_sums :: (a: [] int, b: [] int, c: [] int, target: int) -> int {
    hits := 0;

    for x: a {
        if x <= 0 continue;

        for y: b {
            if y <= 0 continue;

            for z: c {
                if z <= 0 continue;
                if x + y + z == target hits += 1;
            }
        }
    }

    return hits;
}

main :: () {
    print("%\n", count_sums(.[1, 2], .[3, 4], .[5, 6], 9));
}
```

Off by default. A level is a braced block or a `switch`; `else if` chains and `#if` add none.

### field_reassign_with_default

Finds `x: T; x.a = 1; x.b = 2;` where a struct literal `x := T.{ a = 1, b = 2 }` sets the fields at the declaration.

Category `pedantic`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

Config :: struct {
    width: int = 80;
    height: int = 24;
    title: string;
}

make :: (title: string) -> Config {
    config: Config;
    config.width = 100;
    config.title = title;

    return config;
}

main :: () {
    print("%\n", make("demo"));
}
```

```diagnostic
warning[field_reassign_with_default]: `config` is made with its defaults and then its fields set
  --> example.jai:10:5
   |
10 |     config: Config;
   |     ^^^^^^
   |
help: set the fields in a struct literal where the variable is declared
   |
10 -     config: Config;
11 -     config.width = 100;
12 -     config.title = title;
10 +     config := Config.{ width = 100, title = title };
   |
```

Fixed:

```jai
#import "Basic";

Config :: struct {
    width: int = 80;
    height: int = 24;
    title: string;
}

make :: (title: string) -> Config {
    return .{ width = 100, title = title };
}

main :: () {
    print("%\n", make("demo"));
}
```

The fields not named in the literal keep their defaults, as in the declaration, and the values are set where the variable is made instead of in statements a later edit can separate from it.

### float_equality

Finds `==` or `!=` between two computed floats.

Category `pedantic`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

same_total :: (a: float, b: float, expected: float) -> bool {
    return a + b == expected;
}

main :: () {
    print("%\n", same_total(0.1, 0.2, 0.3));
}
```

```diagnostic
warning[float_equality]: comparing floats for exact equality
  --> example.jai:4:12
   |
 4 |     return a + b == expected;
   |            ^^^^^^^^^^^^^^^^^
   |
   = help: compare the difference with a tolerance: `abs(a - b) < epsilon`
```

Fixed:

```jai
#import "Basic";
#import "Math";

same_total :: (a: float, b: float, expected: float) -> bool {
    return abs(a + b - expected) < 0.000001;
}

main :: () {
    print("%\n", same_total(0.1, 0.2, 0.3));
}
```

Off by default: exact comparison is often deliberate.

### float_loop_counter

Finds a `while` loop counted in floats (`while t < 1.0 { t += 0.1; }`).

Category `pedantic`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

main :: () {
    t := 0.0;

    while t < 1.0 {
        print("%\n", t);
        t += 0.1;
    }
}
```

```diagnostic
warning[float_loop_counter]: `t` counts this loop in floats, so rounding error decides how many times it runs
  --> example.jai:6:11
   |
 6 |     while t < 1.0 {
   |           ^
   |
   = help: count with an integer and compute the float from it: `for i: 0..n-1 { x := start + i * step; }`
```

Fixed:

```jai
#import "Basic";

main :: () {
    for i: 0..9 {
        print("%\n", i * 0.1);
    }
}
```

Adding `0.1` ten times does not give `1.0`, so whether the loop runs ten or eleven times depends on rounding error, which grows with the iteration count.

### identical_case_bodies

Finds two cases of a `switch` with the same body, which could list their values together.

Category `pedantic`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

Kind :: enum {
    PNG;
    JPEG;
    WAV;
}

describe :: (kind: Kind) -> string {
    if kind == {
        case .PNG;
            return "image";
        case .JPEG;
            return "image";
        case .WAV;
            return "audio";
    }

    return "";
}

main :: () {
    print("%\n", describe(.PNG));
}
```

```diagnostic
warning[identical_case_bodies]: `case .JPEG` does the same as `case .PNG`
  --> example.jai:13:9
   |
13 |         case .JPEG;
   |         ^^^^^^^^^^
   |
help: list the values together: `case .PNG, .JPEG;`
   |
11 -         case .PNG;
12 -             return "image";
13 -         case .JPEG;
14 -             return "image";
11 +         case .PNG, .JPEG;
12 +             return "image";
   |
```

Fixed:

```jai
#import "Basic";

Kind :: enum {
    PNG;
    JPEG;
    WAV;
}

describe :: (kind: Kind) -> string {
    if kind == {
        case .PNG; #through;
        case .JPEG;
            return "image";
        case .WAV;
            return "audio";
    }

    return "";
}

main :: () {
    print("%\n", describe(.PNG));
}
```

Off by default, since one case per row is sometimes the clearer table.

### if_not_else

Finds an `if` or `ifx` with a negated condition and an `else`.

Category `pedantic`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

main :: () {
    ready := false;

    if !ready {
        print("waiting\n");
    } else {
        print("going\n");
    }
}
```

```diagnostic
warning[if_not_else]: `if !ready` is negated, with an `else`
  --> example.jai:6:5
   |
 6 |     if !ready {
   |     ^^^^^^^^^
   |
help: test `ready` first and swap the branches
   |
 6 ~     if ready {
 7 ~         print("going\n");
 8 ~     } else {
 9 ~         print("waiting\n");
10 ~     }
   |
```

Fixed:

```jai
#import "Basic";

main :: () {
    ready := false;

    if ready {
        print("going\n");
    } else {
        print("waiting\n");
    }
}
```

Off by default, since putting the exceptional or shorter branch first is often right.

### incomplete_enum_switch

Finds a `switch` on an enum that neither covers every member nor has a default case.

Category `pedantic`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

Shape :: enum {
    CIRCLE;
    SQUARE;
    TRIANGLE;
}

sides :: (kind: Shape) -> int {
    if kind == {
        case .CIRCLE; return 0;
        case .SQUARE; return 4;
    }

    return -1;
}

main :: () {
    print("%\n", sides(.TRIANGLE));
}
```

```diagnostic
warning[incomplete_enum_switch]: this `switch` on `Shape` does not handle .TRIANGLE
  --> example.jai:10:5
   |
10 |     if kind == {
   |     ^^^^^^^^^^^^
   |
   = help: handle them, or add `case;` for the rest, or write `#complete` so the compiler checks the list
```

Fixed:

```jai
#import "Basic";

Shape :: enum {
    CIRCLE;
    SQUARE;
    TRIANGLE;
}

sides :: (kind: Shape) -> int {
    if kind == {
        case .CIRCLE; return 0;
        case .SQUARE; return 4;
        case .TRIANGLE; return 3;
    }

    return -1;
}

main :: () {
    print("%\n", sides(.TRIANGLE));
}
```

Lists the members no case mentions. Mark the switch `#complete` so the compiler keeps the list current, or add `case;` to name the fallback.

### large_stack_array

Finds a local fixed array bigger than `max_stack_array_bytes` (default 64 KiB).

Category `pedantic`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

main :: () {
    scratch: [1 << 20] u8;
    scratch[0] = 1;

    print("%\n", scratch[0]);
}
```

```diagnostic
warning[large_stack_array]: `scratch` is 1 MiB on the stack, past the limit of 64 KiB
  --> example.jai:4:5
   |
 4 |     scratch: [1 << 20] u8;
   |     ^^^^^^^
   |
   = help: allocate it (`NewArray`, `alloc`), use a resizable array, or keep it in a global
```

Fixed:

```jai
#import "Basic";

main :: () {
    scratch := NewArray(1 << 20, u8);
    defer free(scratch.data);
    scratch[0] = 1;

    print("%\n", scratch[0]);
}
```

Locals live on the stack, which is small (a thread's is often 1 MiB or less) and shared with deep recursion, so a local of a megabyte can overflow it with no message beyond a crash.

### leaked_allocation

Finds a local holding inferred owned storage that is never freed, returned or handed on.

Category `pedantic`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

Parser :: struct {
    depth: int;
}

main :: () {
    parser := New(Parser);
    parser.depth = 3;

    if parser.depth > 2 print("deep\n");
}
```

```diagnostic
warning[leaked_allocation]: `parser` holds memory from `New` that is never freed, returned or handed on
  --> example.jai:8:5
   |
 8 |     parser := New(Parser);
   |     ^^^^^^
   |
help: free it when done, for example `defer free(parser);` right after this line
   |
 8 -     parser := New(Parser);
 8 +     parser := New(Parser);
 9 +     defer free(parser);
   |
```

Fixed:

```jai
#import "Basic";

Parser :: struct {
    depth: int;
}

main :: () {
    parser := New(Parser);
    defer free(parser);
    parser.depth = 3;

    if parser.depth > 2 print("deep\n");
}
```

Off by default: ownership conventions differ, and a program that allocates once for its whole run may leak on purpose.

### lossy_xx

Finds `xx` that narrows a number to a smaller type.

Category `pedantic`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

main :: () {
    big := 70000;
    small: u16 = xx big;
    print("%\n", small);
}
```

```diagnostic
warning[lossy_xx]: `xx` converts `s64` to `u16`, which can lose the high bits
  --> example.jai:5:18
   |
 5 |     small: u16 = xx big;
   |                  ^^^^^^
   |
help: write `cast(u16)` to make the conversion visible
   |
 5 ~     small: u16 = cast(u16) big;
   |
```

Fixed:

```jai
#import "Basic";

main :: () {
    big := 70000;
    small: u16 = cast,trunc(u16) big;
    print("%\n", small);
}
```

Off by default: narrowing with `xx` is common Jai style.

### magic_number

Finds an unexplained number in arithmetic, a comparison or an assignment.

Category `pedantic`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

expires :: (start: int) -> int {
    return start + 86400;
}

main :: () {
    print("%\n", expires(0));
}
```

```diagnostic
warning[magic_number]: magic number `86400` in arithmetic
  --> example.jai:4:20
   |
 4 |     return start + 86400;
   |                    ^^^^^
   |
   = help: name it with a constant (`NAME :: value;`), or add it to `[lint] magic_number_allow`
```

Fixed:

```jai
#import "Basic";

SECONDS_PER_DAY :: 86400;

expires :: (start: int) -> int {
    return start + SECONDS_PER_DAY;
}

main :: () {
    print("%\n", expires(0));
}
```

Off by default. Reports a decimal literal written directly as an operand of `+ - * / %` or a comparison, or as the value of an assignment (`x = 37;`, `x += 37;`).

### neg_cmp_float

Finds `!(a < b)` on floats, which differs from `a >= b` when a value is NaN.

Category `pedantic`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

within :: (x: float, limit: float) -> bool {
    return !(x < limit);
}

main :: () {
    print("%\n", within(1, 2));
}
```

```diagnostic
warning[neg_cmp_float]: `!(x < limit)` is not `x >= limit` when a float is NaN
  --> example.jai:4:12
   |
 4 |     return !(x < limit);
   |            ^^^^^^^^^^^
   |
help: write the inverted comparison if NaN should not take the branch, or test for NaN explicitly if it should
   |
 4 ~     return x >= limit;
   |
```

Fixed:

```jai
#import "Basic";

within :: (x: float, limit: float) -> bool {
    return x >= limit;
}

main :: () {
    print("%\n", within(1, 2));
}
```

Every comparison with a NaN is false, so `!(a < b)` is true for NaN where `a >= b` is false.

### shadowed_local

Finds a local variable named like a parameter or another local in an enclosing scope of the same procedure.

Category `pedantic`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

clamp_count :: (count: int) -> int {
    if count > 10 {
        count := 10;
        return count;
    }

    return count;
}

main :: () {
    print("%\n", clamp_count(12));
}
```

```diagnostic
warning[shadowed_local]: `count` shadows the variable of that name declared on line 3
  --> example.jai:5:9
   |
 5 |         count := 10;
   |         ^^^^^
   |
   = help: rename one of them; an assignment here no longer reaches the outer one
```

Fixed:

```jai
#import "Basic";

clamp_count :: (count: int) -> int {
    if count > 10 {
        capped := 10;
        return capped;
    }

    return count;
}

main :: () {
    print("%\n", clamp_count(12));
}
```

A variable declaration (`name := ...`, `name: T`) in a nested block, loop body or `case` body whose name is a parameter or a variable declared earlier in an enclosing block of the same procedure.

### temp_storage_escape

Finds a `@TempAllocates` result (`tprint`, `talloc`) stored in a global or through a pointer, or returned.

Category `pedantic`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

current_label: string;

set_label :: (id: int) {
    current_label = tprint("item %", id);
}

main :: () {
    set_label(1);
    print("%\n", current_label);
}
```

```diagnostic
warning[temp_storage_escape]: `tprint` holds temporary storage, which is reset, but it is kept in a global
  --> example.jai:6:21
   |
 6 |     current_label = tprint("item %", id);
   |                     ^^^^^^^^^^^^^^^^^^^^
   |                     -------------------- temporary storage allocated here
   |
   = help: copy it first (`copy_string`) so it outlives the reset
```

Fixed:

```jai
#import "Basic";

current_label: string;

set_label :: (id: int) {
    free(current_label);
    current_label = sprint("item %", id);
}

main :: () {
    set_label(1);
    print("%\n", current_label);
}
```

Off by default: handing back a temporary string from a helper is a common, documented convention. Fires when temporary memory is kept past a reset or a restored mark (`auto_release_temp`) that frees it. Values baked at compile time are copied out, so a `tprint` in a constant is fine.

### unreadable_literal

Finds a long number without `_` separators (`1000000`, `0xdeadbeef`).

Category `pedantic`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

TIMEOUT :: 1000000;

main :: () {
    print("%\n", TIMEOUT);
}
```

```diagnostic
warning[unreadable_literal]: `1000000` is hard to read without separators
  --> example.jai:3:12
   |
 3 | TIMEOUT :: 1000000;
   |            ^^^^^^^
   |
help: write `1_000_000`
   |
 3 ~ TIMEOUT :: 1_000_000;
   |
```

Fixed:

```jai
#import "Basic";

TIMEOUT :: 1_000_000;

main :: () {
    print("%\n", TIMEOUT);
}
```

Off by default: the standard library alone has thousands of such constants.

### using_pollution

Finds a `using` of a struct whose members share names with parameters, locals or globals already in scope.

Category `pedantic`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

Style :: struct {
    count: int;
    color: int;
}

draw :: (count: int, using style: Style) -> int {
    return count + color;
}

main :: () {
    print("%\n", draw(1, .{ 2, 3 }));
}
```

```diagnostic
warning[using_pollution]: `using style` brings in `count`, which is already in scope
  --> example.jai:8:22
   |
 8 | draw :: (count: int, using style: Style) -> int {
   |                      ^^^^^^^^^^^^^^^^^^
   |
   = help: list the members that are meant (`using,only(...)`) or write `name.member`
```

Fixed:

```jai
#import "Basic";

Style :: struct {
    count: int;
    color: int;
}

draw :: (count: int, style: Style) -> int {
    return count + style.color;
}

main :: () {
    print("%\n", draw(1, .{ 2, 3 }));
}
```

A `using x;` statement, `using x: T;` declaration or `using` parameter whose struct (its own members and those of structs it `using`s, through a pointer too).

## Restriction

Checks that forbid things a project may choose to avoid. Default: `allow`.

### autocast_in_arg

Finds `xx`, the automatic cast, written as an argument of a call.

Category `restriction`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

draw :: (x: s32, y: s32) -> s32 {
    return x + y;
}

main :: () {
    width: s64 = 640;

    print("%\n", draw(xx width, 10));
}
```

```diagnostic
warning[autocast_in_arg]: `xx` leaves the target type to the parameter (`s64` to `s32`)
  --> example.jai:10:23
   |
10 |     print("%\n", draw(xx width, 10));
   |                       ^^
   |
help: write `cast(s32)` to say so
   |
10 ~     print("%\n", draw(cast(s32) width, 10));
   |
```

Fixed:

```jai
#import "Basic";

draw :: (x: s32, y: s32) -> s32 {
    return x + y;
}

main :: () {
    width: s64 = 640;

    print("%\n", draw(cast(s32) width, 10));
}
```

The call site then depends on the parameter types: changing `draw` to take a `float` instead of an `s32` changes what `xx` does at every call, and nothing there says so.

### autocast_used

Finds `xx`, the automatic cast, outside a call's arguments.

Category `restriction`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

main :: () {
    big: s64 = 200;
    small: u8 = xx big;

    print("%\n", small);
}
```

```diagnostic
warning[autocast_used]: `xx` leaves the target type to the context (`s64` to `u8`)
  --> example.jai:5:17
   |
 5 |     small: u8 = xx big;
   |                 ^^
   |
help: write `cast(u8)` to say so
   |
 5 ~     small: u8 = cast(u8) big;
   |
```

Fixed:

```jai
#import "Basic";

main :: () {
    big: s64 = 200;
    small := cast(u8) big;

    print("%\n", small);
}
```

`xx` takes its target type from the context, so the line does not say what the value becomes, and changing the type on the left silently changes the conversion on the right.

### excessive_bools

Finds a procedure with more `bool` parameters than `max_bools` (default 3).

Category `restriction`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

open :: (path: string, read: bool, write: bool, create: bool, truncate: bool) -> int {
    return path.count;
}

main :: () {
    print("%\n", open("a.txt", true, false, true, false));
}
```

```diagnostic
warning[excessive_bools]: `open` takes 4 `bool` parameters, past the limit of 3
  --> example.jai:3:1
   |
 3 | open :: (path: string, read: bool, write: bool, create: bool, truncate: bool) -> int {
   | ^^^^
   |
   = help: use an enum or a `#flags` enum so each argument says what it sets, or split the procedure
```

Fixed:

```jai
#import "Basic";

Open_Flags :: enum_flags u8 {
    READ;
    WRITE;
    CREATE;
    TRUNCATE;
}

open :: (path: string, flags: Open_Flags) -> int {
    return path.count;
}

main :: () {
    print("%\n", open("a.txt", .READ | .CREATE));
}
```

A parameter counts when its type is written `bool`, or it has no type and defaults to `true` or `false`.

### exit_in_module

Finds a module ends the process.

Category `restriction`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

open_config :: (path: string) -> int {
    if path.count == 0 {
        exit(1);
    }

    return path.count;
}
```

```diagnostic
warning[exit_in_module]: a module should not end the process
  --> example.jai:5:9
   |
 5 |         exit(1);
   |         ^^^^^^^
   |
   = help: return an error and let the program decide whether to exit
```

Fixed:

```jai
#import "Basic";

open_config :: (path: string) -> int, bool {
    if path.count == 0 return 0, false;

    return path.count, true;
}
```

A file belongs to a module when it or a directory above it holds a `module.jai`, with no `tests` or `test` folder in between (those hold programs that use the module).

### missing_doc

Finds an exported procedure, struct or enum with no doc comment.

Category `restriction`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

parse :: (text: string) -> int {
    return text.count;
}

main :: () {
    print("%\n", parse("abc"));
}
```

```diagnostic
warning[missing_doc]: the exported procedure `parse` has no doc comment
  --> example.jai:3:1
   |
 3 | parse :: (text: string) -> int {
   | ^^^^^
   |
   = help: describe it in a `//` comment on the lines above
```

Fixed:

```jai
#import "Basic";

// Counts the bytes in `text`.
parse :: (text: string) -> int {
    return text.count;
}

main :: () {
    print("%\n", parse("abc"));
}
```

A procedure, struct, enum or union declared at the top level of a file while the scope is exported (the default, until a `#scope_file` or `#scope_module`) needs a doc: the comment directly above it.

### platform_leak

Finds portable code imports something that asserts an operating system it is not built for.

Category `restriction`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
P :: #import "POSIX";

main :: () {
    P.getpid();
}
```

```diagnostic
warning[platform_leak]: POSIX is not available on windows
 --> example.jai:1:1
  |
1 | P :: #import "POSIX";
  | ^^^^^^^^^^^^^^^^^^^^ only builds on linux and macos
  |
  = help: guard the import with an `#if OS ==` check, or list this file under `[lint.portability.platforms]`
  = note: the `#assert` at stdlib/POSIX/module.jai:1:9 fails on windows: POSIX requires its documented native platform
```

Code meant to build everywhere that imports a module asserting an operating system it is not built for (`#assert OS == .LINUX || OS == .MACOS` in `POSIX`), found only when someone builds for that system.

### pointer_lifetime

Finds a pointer used after its storage moves or its element becomes invalid.

Category `restriction`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

main :: () {
    xs: [..] int;

    defer array_free(xs);

    array_add(*xs, 1);
    p := *xs[0];

    array_add(*xs, 2);

    print("%\n", p.*);
}
```

```diagnostic
warning[pointer_lifetime]: `p` points into `xs`, which may be resized: array_add(*xs, 2)
  --> example.jai:13:18
   |
 9 |     p := *xs[0];
   |          ------ pointer derived here
   ...
11 |     array_add(*xs, 2);
   |     ----------------- `xs` storage invalidated here
   ...
13 |     print("%\n", p.*);
   |                  ^ pointer used here after its storage was invalidated
   |
   = help: take the pointer again after growth, or store an index instead
```

Fixed:

```jai
#import "Basic";

main :: () {
    xs: [..] int;

    defer array_free(xs);

    array_add(*xs, 1);
    first := xs[0];

    array_add(*xs, 2);

    print("%\n", first);
}
```

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

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

parse :: (text: string) -> int {
    print("parsing %\n", text);
    return text.count;
}
```

```diagnostic
warning[print_in_module]: a module should not write to the program's output
  --> example.jai:4:5
   |
 4 |     print("parsing %\n", text);
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: return the text or an error, or log it, and let the program decide what to print
```

Fixed:

```jai
#import "Basic";

parse :: (text: string) -> int {
    return text.count;
}
```

A file belongs to a module when it or a directory above it holds a `module.jai`, with no `tests` or `test` folder in between (those hold programs that use the module).

### todo_marker

Finds a comment with a `TODO`, `FIXME`, `XXX` or `HACK` marker.

Category `restriction`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

// TODO: handle the empty list
first :: (items: [] int) -> int {
    return items[0];
}

main :: () {
    print("%\n", first(.[1, 2]));
}
```

```diagnostic
warning[todo_marker]: this comment has a `TODO` marker
  --> example.jai:3:1
   |
 3 | // TODO: handle the empty list
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: finish it, or track it outside the source and remove the marker
```

Fixed:

```jai
#import "Basic";

first :: (items: [] int) -> int {
    if items.count == 0 return 0;
    return items[0];
}

main :: () {
    print("%\n", first(.[1, 2]));
}
```

A marker is `TODO`, `FIXME`, `XXX` or `HACK` in capitals, standing alone (`TODO:` and `(FIXME)` count; `todo`, `TODOS` and `AUTODOC` do not).

### too_many_arguments

Finds a procedure with more parameters than `max_arguments` (default 7).

Category `restriction`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

draw :: (x: int, y: int, w: int, h: int, r: u8, g: u8, b: u8, a: u8) -> int {
    return x + y + w + h + r + g + b + a;
}

main :: () {
    print("%\n", draw(0, 0, 10, 10, 255, 0, 0, 255));
}
```

```diagnostic
warning[too_many_arguments]: `draw` takes 8 parameters, past the limit of 7
  --> example.jai:3:1
   |
 3 | draw :: (x: int, y: int, w: int, h: int, r: u8, g: u8, b: u8, a: u8) -> int {
   | ^^^^
   |
   = help: group parameters that belong together in a struct and pass that
```

Fixed:

```jai
#import "Basic";

Rect :: struct {
    x, y, w, h: int;
}

Color :: struct {
    r, g, b, a: u8;
}

draw :: (rect: Rect, color: Color) -> int {
    return rect.x + rect.y + rect.w + rect.h + color.r + color.g + color.b + color.a;
}

main :: () {
    print("%\n", draw(.{ 0, 0, 10, 10 }, .{ 255, 0, 0, 255 }));
}
```

Every parameter counts, defaulted, baked (`$T`) and variadic ones included.

### unchecked_directive

Finds `#no_abc`, `#no_aoc` or `#asm` with no comment saying why.

Category `restriction`, `allow` by default.

Example (off by default; shown with the rule enabled):

```jai
#import "Basic";

sum :: (items: [] int) -> int #no_abc {
    total := 0;
    for items total += it;
    return total;
}

main :: () {
    print("%\n", sum(.[1, 2, 3]));
}
```

```diagnostic
warning[unchecked_directive]: `#no_abc` has no comment saying why it is needed
  --> example.jai:3:31
   |
 3 | sum :: (items: [] int) -> int #no_abc {
   |                               ^^^^^^^
   |
   = help: add a `//` comment on this line or the line above: why the checks are not needed here
```

Fixed:

```jai
#import "Basic";

sum :: (items: [] int) -> int {
    total := 0;
    for items total += it;
    return total;
}

main :: () {
    print("%\n", sum(.[1, 2, 3]));
}
```

Each `#no_abc`, `#no_aoc` and `#asm` needs a comment on its own line or the line just above.
