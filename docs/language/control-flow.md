# Control flow: loops, cases and defer

## What it is

How `jaic` checks and lowers `if`, `while`, `for`, `if x == { case ... }` and `defer`, including named loops, reverse and pointer iteration, `remove`, and `for_expansion`.

## How it works

Parsing is in `parser/stmt.rs`; checking is `check_if`, `check_while`, `check_for` and `check_for_expansion` in `sema/stmt.rs`. Jumps resolve to the innermost or named loop at check time {#flow.10}, so there is no runtime label lookup. A statement `ifx c then x = 1 else x = 2;` whose branches assign is an `if` statement {#flow.26}.

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

Here `for <` visits `3` down to `1` {#flow.1}, `continue outer` leaves the inner loop and continues the outer one {#flow.2}, `for 5..3` runs zero times {#flow.3}, `for v, i` binds the value then the index {#flow.4}, `for *p` writes through `p` into the array {#flow.5}, and `while x := ...` names the loop {#flow.6}.

- Ranges are inclusive {#flow.7}; both ends are evaluated once, left to right {#flow.8}. `for #v2 a..b` means the same {#flow.12}.
- The default names are `it` and `it_index`; naming either replaces it {#flow.9}.
- Over an array (fixed, view or dynamic), a by-value `it` is the element itself, not a copy: `*it` is the element's address and stays valid after the loop, so `for table if it.key == key return *it.value;` returns a pointer into the array {#flow.28}. Third-party code depends on this (toml-jai's `find_or_insert_key`, Photon's `for buffers reset(*it);`).
- `while name := cond` binds `name` to the condition's value (re-evaluated each iteration) and names the loop for `break name` {#flow.24}. `while :name cond` only names it {#flow.25}.
- `remove it;` (or `remove;`) over a dynamic array is an unordered remove: the last element moves into the hole and is visited next. Removing `2`, `4`, `6` from `[1..6]` leaves `[1, 5, 3]` {#flow.11}.

### Cases

```jai
if c == {
    case .RED;   print("red\n");
    case .GREEN; print("green\n"); #through;   // falls into the next body
    case .BLUE;  print("blue\n");
}
if n == { case 1; ...; case; print("default\n"); }   // bare `case;` is the default
```

A case runs only its own body unless it ends with `#through` {#flow.13}; a bare `case;` is the default label {#flow.14}.

`if #complete c == {` on a non-flags enum must name every member (a default label does not count), or it fails with `#complete switch on Color has no case for .BLUE` (`check_switch_complete`) {#flow.15}. Members compare by value, so aliases count {#flow.16}. `#complete` on an `enum_flags` value, a non-enum {#flow.17}, or a compile-time constant switch value is not checked {#flow.18}.

At run time, a `#complete` switch on a non-flags enum without a default label stops the program when the value is none of the members (a cast from an integer, or uninitialized memory), instead of running no case: `error: runtime error: no case of the `#complete` switch matches its value, 7`. `unmatched_switch_check` in `sema/stmt.rs` sends the no-match path to `Intrinsic::CheckFailed` (`ir::TRAP_SWITCH_UNMATCHED`); native code reports it through `runtime_support_check_failed` {#flow.27}.

### defer

`defer` bodies run at scope exit in reverse order {#flow.19}, including on `return`, `break` and `continue` out of the scope {#flow.20}. `push_context,defer_pop ctx;` holds a context for the rest of the block {#flow.21}.

### Custom iteration

`for x: value` on a type with a `for_expansion` macro goes through `check_for_expansion` {#flow.22}. The macro can rewrite the body's jumps with `#insert (break=..., continue=..., remove=...) body;` {#flow.23}. See [macros and custom iteration](macros-and-custom-iteration.md).

Gotcha: a `continue` from the user's body jumps to your macro's loop head. Put the index increment in a `defer` or at the top of the loop, or the loop never advances.

## How to change it

Add syntax in `parser/stmt.rs` with a test in `parser/tests.rs`, then handle the new `StmtKind` in `sema/stmt.rs`. Loop bodies that macros may rewrite go through `ForBody` in `sema/lower.rs`. Add a regression program under `tests/stdlib/` that exits 0 (existing ones: `insert-replacements.jai`, `for-expansion-renamed-index.jai`, `push-context-defer-pop.jai`); a program that must be rejected goes in `tests/corpus/negative/` (e.g. `complete-switch-missing-case.jai`).

## Dependencies

`parser/stmt.rs`, `sema/stmt.rs`, `sema/lower.rs`. Custom iteration also relies on `#expand`, `Code` and `#insert`.
