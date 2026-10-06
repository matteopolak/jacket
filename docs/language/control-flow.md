# Control flow: loops, cases and defer

## What it is

How `jaic` checks and lowers `if`, `while`, `for`, `if x == { case ... }` and `defer`, including named loops, reverse and pointer iteration, `remove`, and `for_expansion` iteration.

## How it works

Statements are checked in `crates/jaic/src/sema/stmt.rs` (`check_if`, `check_while`, `check_for`, `check_for_expansion`) and parsed in `crates/jaic/src/parser/stmt.rs`. Jumps resolve against the innermost or the named enclosing loop at check time, so there is no runtime label lookup.

```jai
for outer: 1..4 {
    for < inner: 1..3 {              // reverse: 3 2 1
        if inner == 2 continue outer;
        total += outer;
    }
}                                    // total == 10
for 5..3 print("never");             // a descending range is empty, even without `<`
for v, i: arr print("%:% ", i, v);   // value then index
for *p: arr p.* += 1;                // pointer iteration
while x := i < 3 { i += 1; }         // named while condition
```

Range endpoints are inclusive and evaluated once, left to right. The default iterator is `it` (index `it_index`); naming the iterator or the index replaces them. `break` and `continue` target the innermost loop unless given a loop name.

`remove it;` (or bare `remove;`) inside `for` over a dynamic array is an unordered remove: the last element is moved into the hole and revisited. Removing `2`, `4`, `6` from `[1..6]` leaves `[1, 5, 3]`.

`for #v2 a..b` is accepted and means the same inclusive range (`parser/tests.rs` has a parse case).

Cases:

```jai
if c == {
    case .RED;   print("red\n");
    case .GREEN; print("green\n"); #through;   // falls into the next body
    case .BLUE;  print("blue\n");
}
if n == { case 1; ...; case; print("default\n"); }   // bare `case;` is the default
```

A match runs only its own body unless `#through` is used; `case;` without a value is the default label.

`if #complete c == {` on a (non-flags) enum must name every member in some `case`, default label or not; otherwise the switch is rejected with `#complete switch on Color has no case for .BLUE` (`check_switch_complete` in `sema/stmt.rs`; negative case `tests/corpus/negative/complete-switch-missing-case.jai`). Members are compared by value, so aliases with the same value count as covered. `#complete` on an `enum_flags` value or a non-enum is accepted without a check, and a switch whose value is a compile-time constant takes the constant path and is not checked.

`defer` bodies run at scope exit in reverse order (`d2` before `d1`) and also on `return`, `break` and `continue` out of the scope. `push_context,defer_pop ctx;` holds a context for the rest of the block (`tests/stdlib/push-context-defer-pop.jai`).

Custom iteration: `for x: value` on a type with a `for_expansion` macro runs `check_for_expansion`. The macro may rewrite the body's jumps with `#insert (break=..., continue=..., remove=...) body;`; see `tests/stdlib/insert-replacements.jai` and `tests/stdlib/for-expansion-renamed-index.jai`, and [macros and custom iteration](macros-and-custom-iteration.md). Gotcha: a `continue` coming from the user body jumps to your macro's loop head, so put the index increment in a `defer` or at the top of the loop, otherwise the iteration never advances.

- `while name := cond` also binds `name` as a local holding the condition's value (re-evaluated each iteration) besides naming the loop for `break name` / `continue name`; `while :name cond` only labels (`bind_label` on `StmtKind::While`, `check_while` in `sema/stmt.rs`).

## How to change it

Add syntax in `parser/stmt.rs` (and a parser test in `parser/tests.rs`), then handle the new `StmtKind` in `check_stmt`-style dispatch in `sema/stmt.rs`. Loop bodies that macros may rewrite are plumbed through `ForBody` in `sema/lower.rs`. Add a regression program under `tests/stdlib/` that exits 0.

## Configuration

None.

## Dependencies

`parser/stmt.rs`, `sema/stmt.rs`, `sema/lower.rs`; custom iteration also needs [macros and custom iteration](macros-and-custom-iteration.md) support (`#expand`, `Code`, `#insert`).
