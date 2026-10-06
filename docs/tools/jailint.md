# jailint (Jai linter)

## What it is

`jailint` reports common mistakes and unidiomatic code in Jai programs, like clippy does for Rust, and fixes the ones it can. Its rules run on the program after `jaic` has type-checked it, so they know each expression's type and what each name refers to. The same findings appear in editors through `jailsp`, as diagnostics with quick fixes.

```sh
cargo build -p jailint --release
jailint src/                  # lint every .jai file under src/
jailint main.jai --fix        # apply the fixes that are safe to apply
jailint -D warnings stdlib    # fail on any finding (CI)
jailint --list                # the rules and their default levels
```

Output follows rustc's layout:

```text
warning[index_only_loop]: `i` counts through `names` to index it
  --> list.jai:21:5
   |
21 |     for i: 0..names.count - 1 {
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: loop over the elements: `for names`, with `it` and `it_index`
   |
21 ~     for names {
22 ~         if it_index > 0 print(", ");
23 ~         print("%", it);
   |
   = note: `index_only_loop` is `warn` by default
```

Options: `--fix`, `--config <file>`, `-A`/`-W`/`-D <rule>` (allow, warn, deny; `all` names every rule, `-D warnings` turns every warning into an error), `-I <dir>` (import directory), `-j <n>` (programs compiled at once), `--list`, `--color auto|always|never`, `-v` (each program as it is compiled, and programs that did not compile completely).

Exit status: 0 when nothing at level `deny` was found, 1 when something was, 2 on a usage or configuration error. A summary goes to stderr.

Findings and command-line errors go through jaic's shared renderer (`jaic::render`), so they look like compiler diagnostics and follow `--color`, `NO_COLOR` and `JAIC_DIAGNOSTICS` the same way (see [diagnostics](../compiler/diagnostics.md)). Mistakes say what to change: an unknown option or rule suggests the closest one, a missing path says so, and a bad `jailint.toml` is reported as ``in `path`, line N: ...`` with a `help:` line.

## Rules

| Rule | Default | Fix | Finds |
| --- | --- | --- | --- |
| `absurd_comparison` | warn | no | `u >= 0`, `u < 0` with an unsigned `u`: always true or always false |
| `almost_swapped` | warn | no | `a = b; b = a;`, a swap that sets both to `b` |
| `bitwise_precedence` | warn | yes | `1 << n - 1`, `flags \| 1 << 3`: bitwise operators that group unlike C |
| `bool_comparison` | warn | yes | `x == true`, `x != false`, `false == x` |
| `defer_in_loop` | warn | no | a `defer` in a loop body that cleans up something from outside the loop |
| `duplicate_condition` | warn | no | an `else if` or `case` that repeats an earlier one and can never run |
| `erasing_op` | warn | no | `x * 0`, `x & 0`, `x % 1`: always `0` |
| `float_equality` | allow | no | `==` or `!=` between two computed floats |
| `format_arg_count` | deny | no | a format string that uses more or fewer arguments than the call passes |
| `identical_branches` | warn | no | `if c { A } else { A }`, `ifx c then a else a` |
| `identical_operands` | warn | no | `a == a`, `x - x`, `ok && ok` |
| `identity_op` | warn | yes | `x + 0`, `x * 1`, `x / 1` |
| `index_only_loop` | warn | yes | `for i: 0..xs.count-1` where `i` only indexes `xs` |
| `infinite_loop` | warn | no | a `while` whose condition nothing in the loop changes |
| `integer_division_in_float` | warn | no | `cast(float)(a / b)`, `1 / 2 * w`: an integer quotient used as a float |
| `lossy_xx` | allow | yes | `xx` that narrows a number to a smaller type |
| `manual_assign_op` | warn | yes | `a = a + b` → `a += b` |
| `manual_index_counter` | warn | yes | a counter kept next to a `for` loop that always equals `it_index` |
| `min_max` | warn | no | `min(0, max(100, x))`: a clamp with swapped bounds, always one value |
| `needless_bool` | warn | yes | `ifx c then true else false`, `if c return true; else return false;` |
| `no_effect` | warn | no | `x == 5;`, `count + 1;`, `flush;`: a statement that does nothing |
| `range_past_count` | warn | no | `for i: 0..xs.count` indexing `xs[i]`: one past the end |
| `redundant_cast` | warn | yes | a cast to the type the value already has |
| `remove_in_for` | warn | no | `array_*_remove_*` on the array a `for` is walking |
| `reversed_range` | warn | no | `for i: 10..0`: a range that never runs |
| `self_assignment` | warn | no | `x = x;` |
| `shadowed_it` | warn | no | a nested `for` hides an `it` the enclosing loop still uses |
| `unused_import` | warn | yes | an `#import` nothing in its scope uses |
| `unused_parameter` | warn | no | a parameter the procedure never uses |
| `unused_result` | warn | no | `trim(line);`: a library call that only computes a value, as a statement |
| `unused_variable` | warn | yes | a local variable that is never used |
| `wrapping_constant` | warn | no | `(0xffff_ffff - 40) / h`, `h < 0x8000_0000` with `h: s32`: a constant that wraps to the other operand's type |

Each rule's module (`crates/jailint/src/rules/<rule>.rs`) starts with a doc comment that explains why the rule exists and exactly when it fires. `tests/lint/<rule>/` has code it fires on (`bad.jai`, with the expected output in `bad.expected` and the fixed code in `bad.fixed.jai`) and code it must not fire on (`good.jai`). Every rule has a section below; editors link a finding to it (`#<rule>`).

"Side-effect-free" below means names, member paths, literals, indexing, casts and operators over them: no calls. Rules that compare two expressions compare them as written, ignoring whitespace.

### absurd_comparison

An unsigned value is never below zero, so `if i < 0 return;` guards nothing and `while i >= 0 { ...; i -= 1; }` counting down an unsigned `i` never stops: `i` wraps to its largest value. Fires when an integer literal is compared with a non-constant integer whose type holds nothing on the other side of it (`u >= 0`, `0 > u`, `u > -1`, `s8 < -128`). Only the bottom of a type is checked: the width of aliases such as `c_ulong` differs between platforms, so a comparison with the top may matter on another one.

### almost_swapped

`a = b; b = a;` sets both to `b`. Fires on two consecutive statements of that shape with side-effect-free sides of the same type. Between two types (`u = handle; handle = u;`) it is a round trip, not a swap. A swap is `a, b = b, a;`.

### bitwise_precedence

Jai puts every bitwise and shift operator on one level that binds tighter than `*` and groups left to right. C puts shifts below `+` and gives `&`, `^`, `|` levels of their own, so C-style code means something else:

| Written | Jai reads | C reads |
| --- | --- | --- |
| `1 << n - 1` | `(1 << n) - 1` | `1 << (n - 1)` |
| `flags \| 1 << 3` | `(flags \| 1) << 3` | `flags \| (1 << 3)` |
| `x & 0xFF + 1` | `(x & 0xFF) + 1` | `x & (0xFF + 1)` |

Fires on an unparenthesized bitwise operation that is an operand of `+ - * / %`, or the left operand of a bitwise operator C would apply first (`a | b << c`, `a ^ b & c`). The fix adds the parentheses Jai already implies, so behaviour does not change. If the C reading was meant, move them.

### bool_comparison

`if ready == true` → `if ready`; `x == false` → `!x`. Only when the other side is a non-constant `bool`, so integers and types with `operator ==` are left alone.

### defer_in_loop

A loop body's scope ends every iteration, so this frees `buffer` after the first file:

```jai
buffer := alloc(SIZE);
for files {
    defer free(buffer);
    read_into(buffer, it);
}
```

It fires only when everything the deferred statement uses was declared before the loop and is not mentioned in the body before the `defer`. That leaves out the usual per-iteration pairs (`lock(*m); defer unlock(*m);`, `f := open(it); defer close(f);`), and a deferred loop step such as `while i < n { defer i += 1; ... }`, whose variable the loop's header reads.

### duplicate_condition

```jai
if key == .LEFT       move(-1);
else if key == .RIGHT move(1);
else if key == .LEFT  jump();     // never runs
```

Fires when a side-effect-free condition in an `if`/`else if` chain repeats an earlier one (a call in between, which could change what they read, starts the comparison over), and when a value in `if x == { case ...; }` or `#if x == {}` repeats an earlier `case`.

### erasing_op

`x * 0`, `0 * x`, `x & 0`, `x % 1`, `0 / x`, `0 % x`, `0 << x` are `0` whatever `x` is, so the literal is probably wrong (`flags & 0` for `flags & MASK`). Fires when the other operand is a non-constant integer. Inside an index or an array literal (`m[0 * 4 + 1]`, lined up with the other rows) it is left alone.

### float_equality

Off by default. `if total == expected` with two computed floats. Comparisons with a constant (`x == 0`), the NaN test `x != x` and comparisons inside `operator ==` are not reported. Exact comparison is often deliberate ("did this value change"), and on the stdlib it finds only such cases.

### format_arg_count

Deny by default. `print("% is %\n", name)` prints an error marker in place of the second value at run time, and `print("done\n", n)` drops `n` silently. A procedure counts as print-like when a `string` parameter is followed by a variadic `..Any` and its body passes both to one call, as `print`, `sprint`, `tprint`, `log`, `print_to_builder` and user wrappers do. A `greet :: (name: string, extras: ..Any)` that uses them apart is not a format. Only literal format strings are read. Calls that spread an array (`..args`) or name arguments are skipped. Directives are read as `Basic` reads them: `%`, `%N`, `%00`, and `\%` for a literal percent. It is `deny` because the mistake is always visible at run time.

### identical_branches

`if c { A } else { A }` and `ifx c then a else a`: the condition decides nothing, so one branch was probably meant to differ. Compared by tokens (comments and layout do not count). Empty branches, `#if` (often the same code for two platforms) and the last link of an `else if` chain (`else if f == .NEVER { return .Never; } else { return .Never; }` names its case before the default on purpose) are left alone.

### identical_operands

`a.y == a.y` (for `a.y == b.y`), `n - n`, `ok || ok`: the result is fixed or the operand itself. Fires for comparisons, `-`, `/`, `%`, `&`, `|`, `^`, `&&` and `||` between the same side-effect-free, non-constant expression of a number, bool, pointer or enum type. Float `==`, `!=` and `-` are left alone (`x != x` is the NaN test, `x - x == 0` tests for a finite value), and so are operator procedures.

### identity_op

`x + 0`, `x * 1`, `x | 0`, `x / 1` leave `x` unchanged. The fix drops the operation. `0 + x` is left alone (a base plus an offset, usually next to `K + x`), and so are operations inside an index or array literal (`data[i * 4 + 0]` lined up with `+ 1`, `+ 2`), and shifts by `0` (`(rgb >> 0) & 0xFF` beside `>> 8`).

### index_only_loop

`for i: 0..xs.count - 1` whose `i` only indexes `xs` becomes `for xs` with `it`. When elements are written (`xs[i].x += 1`) it becomes `for *xs` with `it`. Other uses of the index become `it_index`. It stays quiet when:

- the loop could change `xs` itself (assignment, `array_add`, taking its address, `remove`);
- it walks a grid (`for c: 0..xs[i].count - 1` inside);
- the index is used with other arrays or computed with inside an index (`ys[i]`, `xs[i + 1]`);
- `xs` is a constant (`xs :: T.[...]`) and the suggestion would be `for *xs`, since a constant has no elements to point at.;
- elements are filled from the index (`xs[i] = i * i`).

### infinite_loop

```jai
i := 0;
while i < count {
    total += values[i];      // forgot `i += 1;`
}
```

Fires when a `while` condition reads only local variables and parameters (numbers, bools, enums, members of local structs and arrays such as `xs.count`), literals and operators, and the body never assigns them or takes their address and has no `break`, `return`, `remove`, `#insert`, `#asm`, `using` or macro call (macros can assign the caller's variables). Variables whose address is taken anywhere in the procedure, and procedures with `using`, are skipped.

### integer_division_in_float

Operands are typed before the operator sees its context, so in `cast(float)(done / total)` the integers divide first and the remainder is gone before the conversion; `1 / 2 * width` is `0 * width`. Fires on an integer `/` whose value is cast to a float or is an operand of float arithmetic. A variable divided by a literal power of two (`cast(float)(w / 2)`, centring on whole pixels) is taken as deliberate.

### lossy_xx

Off by default. `small: u16 = xx big` with `big: s64`, `n: int = xx 2.75`, `f32: float32 = xx f64`. The fix spells the conversion out as `cast(T)`. Values masked, shifted or reduced with `%` to fit, constants, widening and enum targets are skipped. `xx` for narrowing is common, deliberate Jai style (graphics APIs take `s32`/`u32`).

### manual_assign_op

`total = total + x;` → `total += x;`. The compound form names a long target once (`state.players[i].score`), so it cannot be misspelled on one side. Fires for the operators with a compound form when the target is side-effect-free, of a number or enum type, and the operation keeps its type. `a = b OP a` is reported for `+`, `*`, `&`, `|` and `^` on integers.

### manual_index_counter

```jai
n := 0;
for names {
    print("% %\n", n, it);
    n += 1;
}
```

It fires when `n := 0` comes right before a forward `for` over an array, the body ends with `n += 1` (and has no `continue`) or starts with `defer n += 1;`, and `n` is not used after the loop. The fix removes the counter and uses `it_index`.

### min_max

`min(0, max(100, x))` is always 0: `max(100, x)` is at least 100. Fires on `min(a, max(b, x))` with literal bounds `a < b` and `max(a, min(b, x))` with `a > b`, in either argument order, when `min` and `max` are `Basic`'s or `Math`'s. Use `clamp(x, 0, 100)`.

### needless_bool

The condition already is the `bool`. Fixes:

- `ifx c then true else false` → `c` (and `!c` for the reverse);
- `if c return true; else return false;` → `return c;`;
- `if c return true; return false;` → `return c;`, unless an earlier `if` in the same block returns a literal too (`if a return false; x := f(); if b return false; return true;`): it is then the last of a series of guards and reads best like the others;
- `if c x = true; else x = false;` → `x = c;`.

A negated integer comparison is flipped (`!(n < 10)` → `n >= 10`). Comments inside a rewritten `if` keep the fix from being offered.

### no_effect

`x == 5;` (for `x = 5;`), `count + 1;` (for `count += 1;`), `flush;` (for `flush();`). Fires on an expression statement with no call in it. Statements inside an expression (a block's value, `#code`) are left alone.

### range_past_count

Ranges include their end, so `for i: 0..xs.count { xs[i] }` reads one past the last element. Fires when a range ends at exactly `xs.count` for an array or string and the body indexes `xs[i]` with the loop variable, unless the body mentions `xs.count` (it may guard the last index). The suggested `xs.count - 1` changes what the loop does, so `--fix` does not apply it.

### redundant_cast

`cast(s32) x` where `x: s32`, or `v: u8 = xx w` where `w: u8`. Aliases such as `c_long` can be `s64` on one platform and `s32` on another, so equal types are not enough. The value must be declared with the same type spelling as the cast (`int`≡`s64`, `float`≡`float32`), or be another cast to it. Polymorphic bodies, macros and cast flags are skipped.

### remove_in_for

Removing from the array a forward `for` walks moves another element into the slot just visited (or shifts the rest down), and the loop steps past it. Fires on `Basic`'s `array_unordered_remove_by_index`, `array_ordered_remove_by_index` and the `_by_value` forms with `*xs` inside `for xs` over the same array. Not reported when the loop runs backwards, the call is directly followed by `break` or `return`, or the body assigns the loop's index (stepping back by hand). Use `remove it;`.

### reversed_range

A range counts up, so `for i: 10..0` runs zero times; counting down is `for < i: 0..10`. Fires on a forward range whose ends are integer literals with the start greater, or that ends at `0` and starts at a `.count` expression (`xs.count - 1..0`). The suggested fix changes behaviour and is not applied by `--fix`.

### self_assignment

`x = x;` does nothing. After `using info;`, `width = width;` assigns the field to itself rather than the local of the same name. Fires on `=` between the same side-effect-free expression.

### shadowed_it

An unnamed `for` inside another hides the outer `it`, and the outer loop uses `it` again after the inner loop. Copying the outer `it` to a name first (`row := it;`), the usual idiom, is not reported. Uses are told apart by what the compiler resolved each `it` to.

### unused_import

Unnamed imports count as used when any lookup went through them during checking, which covers operators and `for_expansion`. As a guard for unchecked code, an identifier anywhere in the importing module's files that the module exports also counts. That includes files `#load`ed under an `#if` for another platform. Named imports (`M :: #import "X"`) count as used when `M` is used or written anywhere in the module. These imports are not reported:

- imports with module parameters;
- imports of modules with `#program_export`;
- unnamed imports in a program that did not compile completely.

### unused_parameter

Only procedures whose every caller is in view. That means named procedures that are only ever called (never taken as a value, which could make them a callback), and that are:

- called at least once;
- not overloaded;
- not exported from a module;
- not `#c_call`, `#foreign`, `#expand`, an operator or `for_expansion`;
- without notes.

Empty bodies (stubs), bodies with `#insert`, and parameters whose type has a `$` (the argument settles a polymorphic type) are skipped. Prefix a name with `_` to keep a parameter on purpose.

### unused_result

`trim(line);` does nothing: `trim` returns the trimmed string and leaves `line` alone. Fires on an expression statement calling a value-returning procedure from a fixed list in `Basic`, `String` and `Math` (`trim`, `to_upper_copy`, `replace`, `copy_string`, `sprint`, `min`, `max`, `clamp`, `abs`, `sqrt`, `normalize`, ...), checked by what the name resolved to, so a program's own `trim` is not affected. Calls with a pointer argument (`normalize(*v)` works in place) are skipped.

### unused_variable

Locals in bodies that checked cleanly, decided by what the compiler resolved names to across every polymorph instance. A variable whose name appears anywhere else in its block is never reported, which covers code an `#if` left out and `#asm`. Backtick declarations, `using` and `_`-prefixed names are skipped, and so are blocks with `#insert` (inserted code may use any name) and macro bodies. The fixes:

- remove a pure single declaration;
- drop trailing results of a call (`value, found := f();` → `value := f();`);
- drop a name from `a, b: T;`;
- otherwise rename to `_`.

### wrapping_constant

A constant operand takes the other operand's type. A value outside that type's range but within its bits is taken as that bit pattern without a word: with `h: s32`, `(0xffff_ffff - 40) / h` divides `-41` by `h`, and `h < 0x8000_0000` compares with `-2147483648` (always false). (A value too wide for the bits widens the expression instead; `u8_value + 300` is an `s64`.)

Fires on constants written with literals and arithmetic that are operands of `/`, `%`, `<`, `<=`, `>`, `>=` (or the right side of `/=`, `%=`) next to a non-constant integer. The help offers both readings: compute in `s64` (or `u64`) by casting the other operand, which is the suggested fix (not applied by `--fix`, since the result's type changes), or write the wrapped value if it is meant. Not reported: `+`, `-` and `*`, which give the same bits whether the constant or the result wraps (`h + 0xffff_ffff` is `h - 1` either way); bitwise operators and `==`/`!=`, where the bit pattern is the point (`h & 0xffff_ffff`, `handle == 0xFFFF_FFFF`); expressions with `~` (`flags & ~0x7`); named constants; casts and `xx`; enums.

### Clippy lints considered

The rules above that have a Clippy counterpart: `absurd_extreme_comparisons`, `almost_swapped`, `precedence`, `ifs_same_cond` and `match_same_arms`-style duplicate arms, `erasing_op`, `if_same_then_else`, `eq_op`, `identity_op`, `needless_range_loop`, `while_immutable_condition`, `assign_op_pattern`, `explicit_counter_loop`, `min_max`, `needless_bool`/`needless_bool_assign`, `no_effect`, `unnecessary_cast`, `reversed_empty_ranges`, `self_assignment`, `bool_comparison`, `float_cmp`. `range_past_count`, `remove_in_for`, `integer_division_in_float`, `unused_result` (Clippy's `#[must_use]` checks) and `defer_in_loop` are Jai-specific.

Considered and left out:

- `never_loop`: `for table { first = it; break; }` is the idiom for a table's first entry.
- `collapsible_if`, `collapsible_else_if`, `needless_return`, `let_and_return`, `redundant_else`: style with no bug behind it; Jai code often keeps them on purpose.
- `cast_possible_truncation`, `cast_sign_loss`: covered by `lossy_xx`, which is off by default for the same reason.
- `approx_constant` (`3.14159` for `PI`): common and harmless in examples; little value.
- `manual_swap`: `t := a; a = b; b = t;` is clear and correct.
- `len_zero` (`xs.count == 0`): that is idiomatic Jai.
- `cmp_null`: `p == null` is idiomatic Jai.
- `out_of_bounds_indexing`: constant indices into fixed arrays are already checked by the compiler.
- `zero_ptr` (`cast(*T) 0`): rare, and only style.
- `double_comparisons`, `int_plus_one`, `nonminimal_bool`, `manual_range_contains`: style; rewrites read no better in Jai.
- `size_of_ref`-style mistakes (`size_of(type_of(ptr))` in `memcpy`): needs to know which argument is a size; deferred.
- `suspicious_assignment_formatting` (`a =- b`): `jaifmt` rewrites the spacing, so formatted code cannot show it.
- `mut_range_bound`, `explicit_iter_loop`, iterator, `Option`/`Result`, borrow, trait, `unsafe`, `async`, lifetime, macro, attribute, `Cargo.toml` and doc-comment lints: no Jai counterpart.

### Measured on this repository

Every finding below was checked by hand. Findings that turned out wrong were fixed in the rules (the exceptions listed above), not suppressed. The repository's own Jai code now has none at the default levels:

- **stdlib, examples and `jaifmt/`**: 84 findings were fixed in the stdlib: 74 unused variables (mostly extra results nobody read), 5 unused imports, 4 index loops and 1 shadowed `it`. `float_equality` and `lossy_xx` would add 6 and 8 hits, all deliberate (exact comparisons in tests and sorting, `xx` from floats to pixel coordinates).
- **`tests/corpus/positive`**: no findings.
- **`wrapping_constant`** (stdlib, examples, `jaifmt/`, `tests/corpus` and every upstream project but `focus` and `open-jai`): one finding, genuine: `Clipboard`'s overflow guard `pitch > (0xffff_ffff - 40) / height` divided `-41` by an `s32` height, so it rejected every bitmap (fixed by dividing by `cast(s64) height`). Also reporting `+`, `-`, `*`, `==`, `!=` and the bitwise operators found nothing more there; they stay out because the bits come out the same, or the bit pattern is what is meant.
- **Upstream corpus** (`tools/upstream-cases.json` entry points): 127 findings, all genuine. `lossy_xx` adds 23 deliberate narrowings.
- **Upstream projects, whole trees** (the bug and style rules from `absurd_comparison` to `unused_result`, each project linted as a directory; `focus` and `open-jai` were skipped after 240 s): 46 findings, all genuine: `manual_assign_op` 22, `needless_bool` 13, `no_effect` 7 (`exit;` without the call, `if !ok false;` without `return`, a lone `barrier.srcAccessMask;`), and one each of `bitwise_precedence` (`a | b & c` written for C), `identical_operands` (`assert(window == window)`), `identical_branches` (two blocks a comment says should differ) and `self_assignment` (`presentMode = presentMode` where a `using` made both sides the same field). The other twelve rules found nothing. The first run's wrong hits shaped the exceptions above: `0 + ply` and `(rgb >> 0) & 0xFF` (`identity_op`), a round trip through a distinct type (`almost_swapped`), a removal loop that steps its own index back (`remove_in_for`), `height / 2` for a pixel centre (`integer_division_in_float`), guard chains (`needless_bool`) and an explicit last case before the default (`identical_branches`).

## How it works

1. **Plan** (`driver.rs`). Paths become `.jai` files, minus `exclude`. A file that another listed file `#load`s is compiled as part of it. `module.jai`, or a file directly in an import directory, is a module: it is compiled by importing it from an empty program, so every procedure is checked whether or not anything calls it. A file the `module.jai` of its directory (or one above) `#load`s is compiled as that module even when only the file is listed (`enclosing_module`): compiled as a program, the module's exported procedures would have every caller in view and `unused_parameter` would flag their interface (`stdlib/Compiler/workspace.jai`'s `get_runtime_info(w)` did). Anything else is a program.
2. **Compile** with lint facts on (`facts::enable` sets `IdeFacts::lint`). On top of the editor facts (`jaic::sema::ide`), the checker then records:
   - each expression's type and constness (`exprs`; marked conflicting when polymorph instances disagree);
   - each cast's source and target types (`casts`);
   - every entity a name resolved to (`used`);
   - which import answered each lookup (`used_imports`).

   Compile-time code runs in the sandboxed host with a block budget. Workspaces a metaprogram creates are compiled too: a `build.jai` that adds `src/main.jai` gets `src/` linted. This uses `jaic::build::WorkspaceObserver`, which hands each workspace's compiler to jailint. Relative paths a metaprogram names resolve against the root file's directory.
3. **Lint** (`lib.rs`, `lint_files`). Per file:
   - re-parse and lex;
   - build the in-source suppressions;
   - collect procedure bodies with whether every instance checked to the end (`syntax::Cx::procs`);
   - run each enabled rule.

   Rules walk the syntax tree (`syntax::walk`) and ask the facts about types and names. Where the compiler did not look (an `#if` branch left out, a body that failed), they fall back to the text and stay quiet.
4. **Report**. Findings get their level, suppressed ones are dropped, and the rest are rendered (`render.rs`). `--fix` applies each machine-applicable fix whose edits do not overlap one already taken (`fix.rs`); run again for the rest.

A file compiled by several roots is reported once, from the first root. Within a root, a workspace's compile wins over the metaprogram's.

### In editors

`jailsp` lints each open document that parses. It uses the same cached, type-checked compile as hover and inlay hints, so an edit costs one compile however many features ask. Findings are diagnostics with the rule as `code`, `jailint` as `source` and a link to the rule's section above as `codeDescription`; `deny` rules are errors. Each fix is a `quickfix` on the finding (preferred when machine-applicable) with the rule in `data.rule`, and `source.fixAll.jailint` applies every safe fix at once, as `--fix` does. Settings come from the nearest `jailint.toml`; a client without a disk (the browser build) sends it as an open document (`didOpen` of a URI ending in `/jailint.toml`). `format_arg_count` is left to jailsp's own format-string diagnostics, which work without type checking. See [the language server](../compiler/language-server.md#lints-and-quick-fixes).

## Suppression

```jai
// jailint: allow(unused_variable)        covers the next line with code
x := compute(); // jailint: allow(all)     after code: covers its own line
// jailint: allow-file(float_equality)     anywhere: covers the whole file

callback :: (event: int, data: *void) {
    print("%\n", event);
} @jailint_allow(unused_parameter)        // after a declaration: covers all of it
```

Several rules can be listed, separated by commas, and `all` stands for every rule. A suppression only silences findings. A name starting with `_` is how code says "unused on purpose" for variables and parameters.

## Configuration

`jailint.toml`, found by walking up from the first path (or `--config`):

```toml
# Globs relative to this file: `*`, `**`, `?`; a directory pattern covers what is under it.
exclude = ["tests/corpus/negative/**", "generated/**"]

[rules]
float_equality = "warn"
unused_parameter = "allow"
```

Only this subset of TOML is read. An unknown key, rule or level is an error, so a typo cannot silently leave a rule on. Command-line `-A`/`-W`/`-D` override the file.

Settings live in their own file rather than `jaifmt.toml` because the formatter is written in Jai and parses its own file. Keeping the two separate means neither tool has to accept the other's keys or reject them as typos.

The repository's `jailint.toml` excludes the negative compiler cases, each rule's bad cases in `tests/lint`, the formatter's golden inputs and the fuzzer seeds; everything else the repository owns is linted with the default levels. Deliberate findings in tests (a test of operator grouping, of `cast(bool) x == true`, of an empty range) carry an `allow` with the reason. CI runs `jailint -D warnings -j 2 prelude stdlib examples tools jaifmt tests benchmarks` (see [continuous integration](continuous-integration.md)).

## How to change it

**Add a rule:**

1. Create `crates/jailint/src/rules/<name>.rs`. Start it with a doc comment saying what it finds, why it matters and exactly when it fires. Then write `pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>)`.
2. Register it in `rules/mod.rs` (`mod` line and a `RULES` entry with the default level and a one-line summary).
3. Add `tests/lint/<name>/bad.jai` and `good.jai` (each a whole program with `main`), then generate the expected files with `JAILINT_BLESS=1 cargo test -p jailint`. Read them before committing.
4. Run it over the repository and the upstream corpus with `-W <name>` and check every hit. A rule that is wrong once in a while costs more trust than it earns.
5. Add it to the table above, give it a `### <name>` section (editors link to that anchor; the `every_rule_is_documented` test checks both), and add it to the changelog.

Inside a rule:

- `cx.procs` lists the procedure bodies. Use `p.clean` (every instance checked to the end) or `p.typed()` (clean, not polymorphic, not a macro) before trusting facts.
- `cx.ty(expr)` and `cx.constant(expr)` give recorded types and constness.
- `cx.entities(span)` gives what a name resolved to, and `cx.compiler.is_used(entity)` whether anything used it.
- `cx.mentions(name, start, end, except)` is the textual guard for unchecked code.
- `syntax::walk` visits statements and expressions with their ancestors. It does not enter nested procedures or struct bodies.

A fix is a list of byte edits. Mark it `machine_applicable` only when applying it cannot change behaviour, and make the result compile (the `fixed_cases_are_clean` test lints `bad.fixed.jai` again).

**New facts.** If a rule needs something the checker does not record, add it to `IdeFacts` (`crates/jaic/src/sema/ide.rs`) behind `lint`, so normal compiles do not pay for it.

Gotchas:

- Facts are recorded only for files under the root's directory (the `IdeFacts` prefix), so module files reached from a program are not linted from that program.
- `ide_check_all` checks bodies nothing called. A body that failed in any polymorph instance is not clean.
- Compile-time code runs in the sandbox. A metaprogram that needs a foreign call the sandbox lacks (running a process, `localtime`) stops there. Its root is reported with `-v`, and only what was compiled is linted.

## Configuration reference

| Setting | Where | Effect |
| --- | --- | --- |
| `exclude` | `jailint.toml` | glob patterns of files and directories not to lint |
| `[rules] <rule> = "allow" \| "warn" \| "deny"` | `jailint.toml` | a rule's level |
| `-A`/`-W`/`-D <rule>`, `-D warnings` | command line | override levels; make warnings errors |
| `-I <dir>` | command line | extra import directory (searched after the root's `modules/`, before the stdlib) |
| `JAIC_STDLIB` | environment | stdlib directory (default: `stdlib/` next to the executable, else the checkout's) |
| `JAILINT_BLESS=1` | environment, tests | rewrite `tests/lint/*/bad.expected` and `bad.fixed.jai` |
| `BLOCK_BUDGET` | `driver.rs` | basic blocks compile-time code may run per root |

## Dependencies

- `crates/jaic`: parser, lexer, the type checker with IDE and lint facts (`sema::ide`), the workspace observer (`build::WorkspaceObserver`) and the sandboxed compile-time host.
- `crates/jai-language-server` depends on `jailint` to publish lints. Its format-string features share `jailint::format_string`.
- No other crates.
