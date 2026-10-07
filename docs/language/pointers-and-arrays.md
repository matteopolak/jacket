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

In a built executable the failure path calls `runtime_support_check_failed` in `stdlib/Runtime_Support.jai` with the index, the count and the line, which prints `bounds.jai:5: error: array bounds check failed: index 7 is outside an array of 4 elements` and then traps. The passing path is a single compare and branch {#ptr.18}.

`#no_abc` turns the check off for a procedure, a block, or a `for`/`while`/`if` {#ptr.14}. `Build_Options.array_bounds_check = .OFF` in a metaprogram turns it off for a whole workspace (`build.rs`) {#ptr.15}.

### Null pointers

The interpreter stops on any load, store or copy through the first page (`null pointer dereference: read through a null pointer`). Native code has no check on plain loads: the access faults on the host, and a wasm build reads its address 0 without stopping.

A place reached through a pointer and boxed into an `Any` is not read where it is boxed: the `Any`'s value pointer is the place's address, so `p.*` (or a member, or an element) of a null `p` gives an `Any` with a type and a null value pointer, and the program goes on. This holds for `..Any` arguments, `a: Any = p.*` and `cast(Any)` {#ptr.19}. Code relies on it: jaison's `assert(!table_find_pointer(table, name), "...", ..., (.*) table_find_pointer(table, name))` boxes the null result whenever the assertion holds, and its tests pass. Whoever reads the value finds the null: `print` (`__append_item` in `Basic/Print.jai`), given a typed `Any` of nonzero size with a null value pointer, writes `error: null pointer dereference: read through a null pointer` to standard error and stops through `debug_break`, on every backend, rather than show `null` {#ptr.21}. An `Any` with no type (a zero-initialized one) still prints `null`.

Taking the address back reads nothing and is not checked: `*(p.*)` is `p`, and `*p.x` is `p` plus the member's offset, even for a null `p` (the `offsetof` idiom) {#ptr.20}. Converting `p.*` of a fixed array to a view does not check either; indexing the view does.

## How to change it

Index lowering is in `sema/expr.rs` around `BoundsCheck`. The opt-out is `FnCtx::no_abc` (`sema/lower.rs`), set in `procs.rs` and `stmt.rs`; any new construct that indexes memory must consult it. `remove` and `for` lowering are in `stmt.rs`.

The check lives in the IR, so the interpreter and LLVM agree. Don't add a check to only one backend.

`box_any` (`sema/convert.rs`) must not check or read the place: a check there broke jaison. The stop for a null value lives in the reader (`Basic/Print.jai`).

Tests: `tests/stdlib/reverse-for-remove.jai`, `array-bounds-check-opt-out.jai` (every `#no_abc` placement), `cpp-method-and-array-decay.jai`; for null `Any`s, the corpus cases `null-deref-print-any` (a runtime error on every backend) and `null-deref-any-member` (runs to the end).

## Configuration

`Build_Options.array_bounds_check` and `#no_abc`. The arithmetic counterpart, `#no_aoc`, goes in the same places; see [arithmetic overflow checks](arithmetic-overflow-checks.md).

## Dependencies

`sema/expr.rs`, `sema/stmt.rs`, `ir.rs`, and the interpreter for the trap message.
