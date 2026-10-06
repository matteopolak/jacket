# Pointers, arrays and bounds checks

## What it is

Pointers (`*T`, `*void`, `null`), fixed arrays (`[N]T`), array views (`[]T`), dynamic arrays (`[..]T`), `remove` during iteration, and the bounds check with its `#no_abc` opt-out.

## How it works

```jai
buf: [4]int = .[2, 4, 5, 7];
values: []int = buf;                       // view over the fixed array
for v: values { if (v & 1) == 0 remove v; }
// values.count == 2; buf is now 7, 5, 5, 7
```

`remove v` copies the last element into the current slot, shrinks the count, and revisits the slot, so order is not preserved {#ptr.1}. It only works in a `for` over a view or dynamic array {#ptr.2}; otherwise `check_remove` (`sema/stmt.rs`) reports `remove is only valid inside a for loop over an array` {#ptr.3}.

Pointers:

- `p += 1` and `p - q` scale by element size {#ptr.4}; `p - *buf[0]` is an element count {#ptr.5}.
- `null` takes its type from context {#ptr.6}; `!p` and `p == null` test for null {#ptr.7}.
- `*T` converts to `*void` implicitly; `cast(*int) void_ptr` goes back {#ptr.8}.
- `arr.data` is a `*T` to the first element; pass it to hand an array to C {#ptr.9}.

Arrays:

- Fixed arrays convert to views implicitly {#ptr.10}. Views and dynamic arrays start with an `s64` count, then the data pointer {#ptr.11}.
- `for v, i: arr`, `for *e: arr` and `for < arr` iterate by value, by pointer and in reverse {#ptr.12}.
- A constant literal like `.["a", "b"]` used as a `[] T` points at its own writable global (`convert.rs`), so a procedure can return it and callers can write through it {#ptr.16}. A literal with runtime elements is a stack temporary.
- An empty array's `.data` is `null` {#ptr.17}.

### Bounds checks

Indexing emits `Intrinsic::BoundsCheck` against the fixed length or the count (`sema/expr.rs`) {#ptr.13}. A failure stops the program:

```
error: runtime error: array bounds check failed: index 7 is outside an array of 4 elements
```

`#no_abc` turns the check off for a procedure, a block, or a `for`/`while`/`if` {#ptr.14}. `Build_Options.array_bounds_check = .OFF` in a metaprogram turns it off for a whole workspace (`build.rs`) {#ptr.15}.

## How to change it

Index lowering is in `sema/expr.rs` around `BoundsCheck`. The opt-out is `FnCtx::no_abc` (`sema/lower.rs`), set in `procs.rs` and `stmt.rs`; any new construct that indexes memory must consult it. `remove` and `for` lowering are in `stmt.rs`.

The check lives in the IR, so the interpreter and LLVM agree. Don't add a check to only one backend.

Tests: `tests/stdlib/reverse-for-remove.jai`, `array-bounds-check-opt-out.jai` (every `#no_abc` placement), `cpp-method-and-array-decay.jai`.

## Configuration

`Build_Options.array_bounds_check` and `#no_abc`. The arithmetic counterpart, `#no_aoc`, goes in the same places; see [arithmetic overflow checks](arithmetic-overflow-checks.md).

## Dependencies

`sema/expr.rs`, `sema/stmt.rs`, `ir.rs`, and the interpreter for the trap message.
