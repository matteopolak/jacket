# Control flow: loops, cases and defer

How `if`, `while`, `for`, `if x == { case ... }` and `defer` behave, including named loops, reverse and pointer iteration, and `remove`. Jumps resolve to the innermost or named loop at check time, so there is no runtime label lookup {#flow.10}.

```jai
for outer: 1..4 {
    for < inner: 1..3 {              // reverse: 3 2 1
        if inner == 2 continue outer;
    }
}
for 5..3 print("never");             // descending range is empty
for v, i: arr print("%:% ", i, v);   // value, then index
for *p: arr p.* += 1;                // pointer iteration
while x := i < 3 { i += 1; }         // binds x and names the loop

if c == {
    case .RED;   print("red\n");
    case .GREEN; print("green\n"); #through;   // falls into the next body
    case;        print("default\n");           // bare case is the default
}
```

`for <` visits `3` down to `1` {#flow.1}; `continue outer` leaves the inner loop and continues the outer one {#flow.2}; `for 5..3` runs zero times {#flow.3}; `for v, i` binds the value then the index {#flow.4}; `for *p` writes through `p` into the array {#flow.5}; `while x := ...` names the loop {#flow.6}.

## Loops

- Ranges are inclusive {#flow.7}; both ends are evaluated once, left to right {#flow.8}. `for #v2 a..b` means the same {#flow.12}.
- Default names are `it` and `it_index`; naming either replaces it {#flow.9}.
- Over an array, a by-value `it` over aggregate elements (structs, strings, arrays, views) is the element itself, so `*it` stays valid after the loop: `for table if it.key == key return *it.value;` {#flow.28}. Assigning through such an `it` is an error (`Can't assign to a large-value iterator. Perhaps you want to iterate by pointer (for *).`).
- A scalar element is copied into `it`; changing it leaves the array alone, and after `remove it` the copy still holds the removed value {#flow.29}.
- `remove it;` over a dynamic array is an unordered remove: the last element moves into the hole and is visited next. Removing `2`, `4`, `6` from `[1..6]` leaves `[1, 5, 3]` {#flow.11}.
- `while name := cond` re-evaluates `cond`, binds it, and names the loop for `break name` {#flow.24}. `while :name cond` only names it {#flow.25}.
- `for x: value` on a type with a `for_expansion` macro goes through that macro {#flow.22}, which can rewrite the body's jumps with `#insert (break=..., continue=..., remove=...) body;` {#flow.23}. A `continue` from the body jumps to the macro's loop head, so advance the index in a `defer`. See [macros and custom iteration](macros-and-custom-iteration.md).

## Cases

- A case runs only its own body unless it ends with `#through` {#flow.13}; a bare `case;` is the default {#flow.14}.
- Case values must be constants and may not repeat (`Duplicate case.`). Switching on a struct or array is an error.
- `if #complete c == {` on a non-flags enum must name every member; a default label does not count: `#complete switch on Color has no case for .BLUE` {#flow.15}. Members compare by value, so aliases count {#flow.16}. `#complete` on an `enum_flags` value is not checked {#flow.17}; on a non-enum it is an error.
- A switch on a compile-time constant runs only the matching case, but every case is still type-checked. Only `#if` skips code {#flow.18}.
- At run time a `#complete` switch with no default stops the program when the value matches nothing: `error: runtime error: no case of the `#complete` switch matches its value, 7` {#flow.27}.

## ifx and defer

- `ifx c then x = 1 else x = 2;` whose branches assign is an `if` statement {#flow.26}, and a branch may be parenthesized {#flow.30}. `ifx value == { case ...; }` at the start of a statement is a switch {#flow.31}.
- An `ifx` whose `then` is an untyped constant that can't take the expected type (`v * (ifx c then 0.9 else 1.0)`) takes its type from the `else` value or the constant's default {#flow.32}.
- `defer` bodies run at scope exit in reverse order {#flow.19}, including on `return`, `break` and `continue` out of the scope {#flow.20}. `push_context,defer_pop ctx;` holds a context for the rest of the block {#flow.21}.

## How to change it

Add syntax in `parser/stmt.rs` with a parser test, then handle the new `StmtKind` in `sema/stmt.rs` (`check_if`, `check_while`, `check_for`). Loop bodies that macros rewrite go through `ForBody` in `sema/lower.rs`. Rejected programs go in `tests/corpus/negative/`.
