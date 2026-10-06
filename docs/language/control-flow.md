# Control flow: loops, cases and defer

## What it is

How `jaic` checks and lowers `if`, `while`, `for`, `if x == { case ... }` and `defer`, including named loops, reverse and pointer iteration, `remove`, and `for_expansion`.

## How it works

Parsing is in `parser/stmt.rs`; checking is `check_if`, `check_while`, `check_for` and `check_for_expansion` in `sema/stmt.rs`. Jumps resolve to the innermost or named loop at check time, so there is no runtime label lookup.

### Loops

```jai
for outer: 1..4 {
    for < inner: 1..3 {              // reverse: 3 2 1
        if inner == 2 continue outer;
        total += outer;
    }
}
for 5..3 print("never");             // descending range is empty, even without `<`
for v, i: arr print("%:% ", i, v);   // value, then index
for *p: arr p.* += 1;                // pointer iteration
while x := i < 3 { i += 1; }         // binds x and names the loop
```

- Ranges are inclusive; both ends are evaluated once, left to right. `for #v2 a..b` means the same.
- The default names are `it` and `it_index`; naming either replaces it.
- `while name := cond` binds `name` to the condition's value (re-evaluated each iteration) and names the loop for `break name`. `while :name cond` only names it.
- `remove it;` (or `remove;`) over a dynamic array is an unordered remove: the last element moves into the hole and is visited next. Removing `2`, `4`, `6` from `[1..6]` leaves `[1, 5, 3]`.

### Cases

```jai
if c == {
    case .RED;   print("red\n");
    case .GREEN; print("green\n"); #through;   // falls into the next body
    case .BLUE;  print("blue\n");
}
if n == { case 1; ...; case; print("default\n"); }   // bare `case;` is the default
```

A case runs only its own body unless it ends with `#through`.

`if #complete c == {` on a non-flags enum must name every member (a default label does not count), or it fails with `#complete switch on Color has no case for .BLUE` (`check_switch_complete`). Members compare by value, so aliases count. `#complete` on an `enum_flags` value, a non-enum, or a compile-time constant switch value is not checked.

### defer

`defer` bodies run at scope exit in reverse order, including on `return`, `break` and `continue` out of the scope.

### Custom iteration

`for x: value` on a type with a `for_expansion` macro goes through `check_for_expansion`. The macro can rewrite the body's jumps with `#insert (break=..., continue=..., remove=...) body;`. See [macros and custom iteration](macros-and-custom-iteration.md).

Gotcha: a `continue` from the user's body jumps to your macro's loop head. Put the index increment in a `defer` or at the top of the loop, or the loop never advances.

## How to change it

Add syntax in `parser/stmt.rs` with a test in `parser/tests.rs`, then handle the new `StmtKind` in `sema/stmt.rs`. Loop bodies that macros may rewrite go through `ForBody` in `sema/lower.rs`. Add a regression program under `tests/stdlib/` that exits 0 (existing ones: `insert-replacements.jai`, `for-expansion-renamed-index.jai`, `push-context-defer-pop.jai`); a program that must be rejected goes in `tests/corpus/negative/` (e.g. `complete-switch-missing-case.jai`).

## Dependencies

`parser/stmt.rs`, `sema/stmt.rs`, `sema/lower.rs`. Custom iteration also relies on `#expand`, `Code` and `#insert`.
