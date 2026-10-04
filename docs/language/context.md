# The implicit context

## What it is

Every ordinary procedure receives a hidden pointer to a `Context` record (allocator, logger, temporary storage, thread index, and anything added with `#add_context`). `push_context` swaps in a modified copy for a lexical scope.

## How it works

The record starts from `Context_Base` in `stdlib/Runtime_Support.jai`; `stdlib/prelude/module.jai` splices it in first, then each `#add_context` declaration (collected in `Compiler::add_contexts`, `sema/modules.rs`, merged into the struct in `sema/structs.rs`) extends it with its own field and default.

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

Observed behavior (checked with `jaic run`):

- Assigning `context.depth += 1` writes through the active pointer, so callees see it.
- `push_context` restores the previous context when its block ends, including by `return`/`break`/`continue` (`check_stmt` in `sema/stmt.rs` keeps the active address in `FnCtx::context`).
- `push_context,defer_pop ctx;` keeps the pushed context until the end of the enclosing block (`tests/stdlib/push-context-defer-pop.jai`).
- `#no_context` and `#c_call` procedures take no hidden parameter. Using `context` there is an error: `'context' is not available here (procedure is #c_call or #no_context; use push_context)`. A `#c_call` body can establish one with `new_context: #Context; push_context new_context { ... }`.
- `#add_context` is only legal at file scope.

Whether a signature has the hidden parameter is computed once in `sema/procs.rs` (`has_context`: not `#c_call`, not `#no_context`, not `#intrinsic`). Direct and indirect calls pass the active pointer.

## How to change it

New context fields: declare them with `#add_context` in Jai, not in the compiler. Changing the base fields means editing `Context_Base` in `stdlib/Runtime_Support.jai`. `expand_plain_ifs` in `sema/scope.rs` expands plain `#if` items first so that an `#add_context` inside a conditional `#load` is present before any `#run` lays the Context out; keep `#add_context` out of code that depends on a `#run` result.

## Configuration

None. There are no environment variables for the context.

## Dependencies

`stdlib/Runtime_Support.jai`, `stdlib/prelude/module.jai`, `sema/procs.rs`, `sema/stmt.rs`, `sema/structs.rs`, `sema/modules.rs`.
