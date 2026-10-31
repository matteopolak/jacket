# The implicit context

Every ordinary procedure receives a hidden pointer to a `Context` record: allocator, logger, temporary storage, thread index, plus whatever `#add_context` declares {#ctx.1}. `push_context` swaps in a modified copy for a lexical scope {#ctx.2}.

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

- The record starts from `Context_Base` in the prelude, then each `#add_context` field with its default {#ctx.3}.
- `context.depth += 1` writes through the active pointer, so callees see it {#ctx.5}.
- `push_context` restores the previous context on any exit, including `return`, `break` and `continue` {#ctx.6}.
- `push_context,defer_pop ctx;` keeps the context until the end of the enclosing block {#ctx.7}.
- With no context given, `push_context { }` and `push_context,defer_pop;` push a copy of the current one, so the block's changes stay inside it. In a `#c_call` or `#no_context` procedure they push a default `#Context` {#ctx.14}.
- `#add_context` is only legal at file scope {#ctx.11}. All additions share one program `#Context`, so field names must be unique, even across differently parameterised imports. Declare a shared extension once in a common module.
- `#add_context name :: value;` is a constant reachable as `context.name`, resolved in the declaring file {#ctx.12}. It shows up in `type_info(type_of(context)).members` with the `CONSTANT` flag.
- `#c_call`, `#no_context` and `#intrinsic` procedures have no hidden parameter {#ctx.8}; direct and indirect calls pass the active pointer {#ctx.13}. Using `context` in a procedure without one is an error {#ctx.9}.
- A `#c_call` body can make one: `new_context: #Context; push_context new_context { ... }` {#ctx.10}.

## How to change it

Add context fields with `#add_context` in Jai, not in the compiler. The layout must be final before any `#run` uses it. Plain `#if` is expanded first so a conditional `#add_context` works; one that depends on a `#run` result cannot. A file loaded while a `#run` is already using the context may still append members, up to 4096 bytes in all.

Code: `has_context` in `sema/procs.rs`, `context_type` in `sema/structs.rs`.
