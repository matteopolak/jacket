# Pointers, arrays and bounds checks

Pointers (`*T`, `*void`, `null`), fixed arrays (`[N]T`), views (`[]T`), dynamic arrays (`[..]T`), `remove` during iteration, and the bounds and null checks with their opt-outs.

```jai
buf: [4]int = .[2, 4, 5, 7];
values: []int = buf;                       // view over the fixed array
for v: values { if (v & 1) == 0 remove v; }
// values.count == 2; buf is now 7, 5, 5, 7
```

## Pointers

- `p += 1` and `p - q` scale by element size {#ptr.4}; `p - *buf[0]` is an element count {#ptr.5}.
- `null` takes its type from context {#ptr.6}; `!p` and `p == null` test for null {#ptr.7}.
- `*T` converts to `*void` implicitly; `cast(*int) void_ptr` goes back {#ptr.8}.
- `arr.data` is a `*T` to the first element; pass it to hand an array to C {#ptr.9}.
- A `(.*)` after an expression is a prefix dereference starting the next statement, not an argument list: `if (found) (.*)out = value;` assigns through `out` {#ptr.22}.
- `null.*` is a `void` value, not a read. It fills the value slot of a `Table(K, void)` used as a set, and doesn't convert to a pointer parameter {#ptr.23}.

## Arrays

- Fixed arrays convert to views implicitly {#ptr.10}. Views and dynamic arrays start with an `s64` count, then the data pointer {#ptr.11}. An empty array's `.data` is `null` {#ptr.17}.
- A fixed array's size is any constant integer expression, including an element of a constant array (`SIZES :: int.[1, 4, 9]; a: [SIZES[1]] int;`).
- Arrays don't compare with `==` or `!=` (an error naming the type); strings do. Compare lengths and elements.
- `for v, i: arr`, `for *e: arr` and `for < arr` iterate by value, by pointer and in reverse {#ptr.12}.
- A constant literal like `.["a", "b"]` used as a `[] T` points at its own writable global, so a procedure can return it {#ptr.16}. A literal with runtime elements is a stack temporary.
- `remove v` copies the last element into the slot, shrinks the count and revisits the slot, so order isn't kept {#ptr.1}. It works only in a `for` over a view or dynamic array {#ptr.2}; otherwise the error says why (fixed array, range, `while`, or no enclosing loop) {#ptr.3}.

## Bounds checks

Indexing checks against the fixed length or count {#ptr.13}. A failure prints and traps; the passing path is one compare and branch {#ptr.18}:

```
bounds.jai:5: error: array bounds check failed: index 7 is outside an array of 4 elements
```

- `#no_abc` turns the check off for a procedure, a block, or a `for`/`while`/`if` {#ptr.14}. `Build_Options.array_bounds_check = .OFF` turns it off for a whole workspace {#ptr.15}. The arithmetic counterpart `#no_aoc` goes in the same places ([numbers](numbers.md)).

## Null pointers

- The interpreter stops on a load, store or copy below address 4096 (`null pointer dereference: read through a null pointer`). Native code does the same. `Build_Options.null_pointer_check = .OFF` (the shipping presets) or `#no_abc` drops it. `memcpy`/`memset` by name aren't checked natively.
- Boxing a place reached through a pointer into an `Any` reads nothing: `p.*` of a null `p` gives an `Any` with a type and a null value pointer, and the program goes on. This holds for `..Any` arguments, `a: Any = p.*` and `cast(Any)` {#ptr.19}. Existing code relies on it (asserts that box a null result whenever the condition holds).
- The reader stops: `print`, given a typed `Any` of nonzero size with a null value pointer, writes `error: null pointer dereference: read through a null pointer` and stops, rather than show `null` {#ptr.21}. An `Any` with no type still prints `null`.
- Taking the address back reads nothing and isn't checked: `*(p.*)` is `p`, and `*p.x` is `p` plus the member's offset, even for null `p` (the `offsetof` idiom) {#ptr.20}.

## How to change it

Index lowering is around `BoundsCheck` in `sema/expr.rs`. The opt-out is `FnCtx::no_abc`, which any new indexing construct must consult. Checks live in the IR so the interpreter and LLVM agree; don't add one to a single backend. `box_any` (`sema/convert.rs`) must not check or read the place. `check_remove` is in `sema/stmt/loops.rs`.
