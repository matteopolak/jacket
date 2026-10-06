# The implicit context

## What it is

Every ordinary procedure receives a hidden pointer to a `Context` record: allocator, logger, temporary storage, thread index, plus whatever `#add_context` declares {#ctx.1}. `push_context` swaps in a modified copy for a lexical scope {#ctx.2}.

## How it works

The record starts from `Context_Base` in `stdlib/Runtime_Support.jai`. `stdlib/prelude/module.jai` splices that in first, then each `#add_context` field (collected in `Compiler::add_contexts` in `sema/modules.rs`, merged in `sema/structs.rs`) with its default {#ctx.3}.

```jai
#add_context depth: int = 7;
show :: () { print("depth=%\n", context.depth); }
inner :: () {
    new_context := context;
    new_context.depth = 99;
    push_context new_context { show(); }   // depth=99
    show();                                // depth=7 again
}
```

The example prints `depth=99` inside the `push_context` block and `depth=7` after it {#ctx.4}.

- `context.depth += 1` writes through the active pointer, so callees see it {#ctx.5}.
- `push_context` restores the previous context when the block exits by any path, including `return`, `break` and `continue` {#ctx.6}. `check_stmt` in `sema/stmt.rs` tracks the active address in `FnCtx::context`.
- `push_context,defer_pop ctx;` keeps the context until the end of the enclosing block {#ctx.7}. The context is required: `push_context,defer_pop;` is a parse error (`parse_push_context`), since there is nothing to push and re-pushing the current context would let the block's changes leak out of it.
- `#add_context` is only legal at file scope {#ctx.11}.
- `#add_context name :: value;` declares a constant reachable as `#Context.name` or `context.name`, not a field. The Iprof and Tracy plugins use this to insert a module alias for their runtime. It resolves in the declaring file (`context_type` in `sema/structs.rs`) {#ctx.12}.

`has_context` in `sema/procs.rs` decides once per signature whether the hidden parameter exists: not for `#c_call`, `#no_context` or `#intrinsic` {#ctx.8}. Direct and indirect calls pass the active pointer {#ctx.13}. Using `context` in such a procedure is an error {#ctx.9}:

```
`context` is not available here (procedure is #c_call or #no_context; use push_context)
```

A `#c_call` body can create one with `new_context: #Context; push_context new_context { ... }` {#ctx.10}.

## How to change it

Add context fields with `#add_context` in Jai, not in the compiler. Base fields live in `Context_Base`.

Gotcha: the Context layout must be final before any `#run` lays it out. `expand_plain_ifs` in `sema/scope.rs` expands plain `#if` items first so an `#add_context` inside a conditionally `#load`ed file is seen in time, but an `#add_context` that depends on a `#run` result cannot work.

Tests: `tests/stdlib/push-context-defer-pop.jai`, `tests/stdlib/add-context-constant.jai`.

## Dependencies

`stdlib/Runtime_Support.jai`, `stdlib/prelude/module.jai`, `sema/procs.rs`, `sema/stmt.rs`, `sema/structs.rs`, `sema/modules.rs`.
