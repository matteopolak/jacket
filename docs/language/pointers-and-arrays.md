# Pointers, arrays and bounds checks

## What it is

Pointers (`*T`, `*void`, `null`), fixed arrays (`[N]T`), array views (`[]T`), dynamic arrays (`[..]T`), array iteration with `remove`, and the array bounds check with its `#no_abc` opt-out.

## How it works

```jai
buf: [4]int = .[2, 4, 5, 7];
values: []int = buf;                       // view over the fixed array
for v: values { if (v & 1) == 0 remove v; }
// values.count == 2; buf is now 7, 5, 5, 7
```

`remove v` overwrites the current slot with the last element and shrinks the view's (or dynamic array's) count, then the loop revisits the slot. Order is therefore not preserved. Fixed arrays and strings cannot be removed from; `check_remove` in `crates/jaic/src/sema/stmt.rs` reports "remove is only valid inside a for loop over an array" otherwise. See `tests/stdlib/reverse-for-remove.jai`.

Pointer rules, all checked with `jaic run`:

- `p += 1` and `p - q` scale by element size; `p - *buf[0]` is an element count.
- `null` takes its type from context; `!p` and `p == null` test for null.
- `*T` converts to `*void` implicitly; `cast(*int) void_ptr` goes back.
- `arr.data` is a `*T` to the first element; passing it where a pointer is expected is the way to hand arrays to C (`tests/stdlib/cpp-method-and-array-decay.jai`).
- Array-to-view conversion is implicit (`values: []int = buf`). Views and dynamic arrays start with an `s64` count followed by the data pointer; `for v, i: arr`, `for *e: arr` and `for < arr` iterate by value, by pointer and in reverse.

Indexing is bounds checked by default. In `sema/expr.rs` the index emits `Intrinsic::BoundsCheck` against the fixed length or the descriptor count, and a failure stops the program:

```
error: runtime error: array bounds check failed: index 7 is outside an array of 4 elements
```

(the text comes from `crates/jaic/src/interp/mod.rs`). `#no_abc` turns the check off for a procedure, a block, or a `for`/`while`/`if`, so a view may read past its count. `tests/stdlib/array-bounds-check-opt-out.jai` demonstrates each placement. The same check is off globally when a metaprogram sets `Build_Options.array_bounds_check = .OFF` (`crates/jaic/src/build.rs`).

- **Constant array literals as views**: `.["a", "b"]` viewed as `[] T` points at its own writable global
  (`convert.rs`, fixed → view), so a procedure may return it and callers may write through it. A literal with
  runtime elements is still a stack temporary. An empty array's `.data` is `null`.

## How to change it

Index lowering is in `crates/jaic/src/sema/expr.rs` around the `BoundsCheck` intrinsic; the `#no_abc` state is carried as `FnCtx::no_abc` (`sema/lower.rs`) and set in `procs.rs` and `stmt.rs`. A new construct that indexes memory must consult `f.no_abc` or it will ignore the opt-out. Loop and `remove` handling is in `stmt.rs` (`check_remove` and the `for` lowering).

Gotcha: the checks live in the IR, so the interpreter and the LLVM backend agree; do not add a check in only one of them.

## Configuration

`Build_Options.array_bounds_check` (read in `build.rs`) and the `#no_abc` directive. Nothing else. The arithmetic counterpart, `#no_aoc`, uses the same placements; see [arithmetic-overflow-checks.md](arithmetic-overflow-checks.md).

## Dependencies

`sema/expr.rs`, `sema/stmt.rs`, `ir.rs`, the interpreter for the trap message.
