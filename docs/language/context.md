# The implicit context

`#Context` names the compiler-synthesized context type for the whole program; `context` is the active value. Every ordinary procedure receives a hidden pointer to this record: allocator, logger, temporary storage, thread index, plus whatever `#add_context` declares {#ctx.1}. `push_context` swaps in a modified copy for a lexical scope {#ctx.2}.

```jai
#add_context depth: int = 7;
show :: () { print("depth=%\n", context.depth); }
inner :: () {
    new_context := context;
    new_context.depth = 99;
    push_context new_context { show(); }   // depth=99
    show();                                // depth=7
}
```

The example prints `depth=99` inside the block and `depth=7` after it {#ctx.4}.

## How it works

- The compiler evaluates the selected prelude's `FIRST_ADD_CONTEXT` block, then adds program `#add_context` fields with their defaults {#ctx.3}. The runtime's `Context_Base` supplies the base members; it is not the completed program type and is not interchangeable with `#Context`.
- `context.depth += 1` writes through the active pointer, so callees see it {#ctx.5}.
- `push_context` restores the previous context on any exit, including `return`, `break` and `continue` {#ctx.6}.
- `push_context,defer_pop ctx;` keeps the context until the end of the enclosing block {#ctx.7}.
- With no context given, `push_context { }` and `push_context,defer_pop;` push a copy of the current one, so the block's changes stay inside it. In a `#c_call` or `#no_context` procedure they push a default `#Context` {#ctx.14}.
- `#add_context` is only legal at file scope {#ctx.11}. All additions share one program `#Context`, so field names must be unique, even across differently parameterised imports. Declare a shared extension once in a common module.
- `#add_context name :: value;` is a constant reachable as `context.name`, resolved in the declaring file {#ctx.12}. It shows up in `type_info(type_of(context)).members` with the `CONSTANT` flag.
- `#c_call`, `#no_context` and `#intrinsic` procedures have no hidden parameter {#ctx.8}; direct and indirect calls pass the active pointer {#ctx.13}. Using `context` in a procedure without one is an error {#ctx.9}.
- A `#c_call` body can make one: `new_context: #Context; push_context new_context { ... }` {#ctx.10}.

## How to change it

Add context fields with `#add_context` in Jai, not in the compiler. Plain `#if` is expanded before the initial type is built, so conditional additions participate in that layout. A file imported while compile-time code is running may append members; the compiler initializes their new storage with their defaults. The live compile-time context reserves at least 4096 bytes (or the initial type's size, if larger), and later additions must fit that reserved capacity.

Code: `has_context` in `crates/jaic/src/sema/procs/signature.rs`, `context_type` in `crates/jaic/src/sema/structs/fields.rs`.

## Configuration and dependencies

The selected prelude (`JAIC_PRELUDE` or the compiler's default), its `FIRST_ADD_CONTEXT`, and the runtime's `Context_Base` determine the base members. Program `#add_context` declarations extend them. Procedure flags `#c_call`, `#no_context` and `#intrinsic` control whether a hidden context pointer is passed. Do not assume one fixed field layout across programs or preludes.

This guide describes the implicit type. A user declaration named `Context` is an ordinary declaration with its own API page; it does not define `#Context`.
