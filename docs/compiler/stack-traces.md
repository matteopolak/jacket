# Stack traces

While a program runs (under `jaic -interp` or compiled), `context.stack_trace` points at a linked list of `Stack_Trace_Node`s, innermost procedure first. `print_stack_trace`, assertion failures and the memory debugger's leak reports read it.

## Capturing a trace

`pack_stack_trace()` copies the current chain, with its own helper frame first,
then its caller and the caller's ancestors. Inline and `#no_debug` procedures have
no frame. When tracing is disabled, the result is empty. The explicit-node overload
copies exactly the supplied chain, without adding a capture frame.

```jai
trace := pack_stack_trace();
defer free(trace.data);
// trace[0].info.name is "pack_stack_trace" when tracing is enabled.
```

The copied `next` pointers refer to nodes in the returned allocation, so the trace
remains valid after the captured procedures return. Procedure-info records belong
to the program. The helper lives in `stdlib/internal/basic/stack-traces.jai` and
uses Basic's allocator; there are no additional flags or dependencies. Change its
tracing policy with the real-chain regression in `stack-capture-frame.jai` and the
synthetic-chain test in `stdlib/Basic/tests/stack-traces.jai`.

## How it works

- At lowering, every procedure that takes a context and is not `inline` or `#no_debug` gets `Func.trace` (name, file, line, column).
- Interpreter: `Interp::exec` pushes a node in the callee's frame and restores `context.stack_trace` on return. Untraced procedures keep the location of the call that entered them.
- Compiled output: `stack_trace::instrument` (`crates/jaic/src/stack_trace.rs`) is an IR pass. It gives each traced function a node slot, links it before the first block every use goes through (so early returns that call nothing pay nothing), unlinks on every `Ret`, and stores the line after each statement that can call. It then clears `Func.trace` so the interpreter doesn't push twice.
- `Program.stack_trace` holds the layout of `stack_trace` in `#Context` and of the node types, read from the prelude. No layout, no traces.
- `runtime_support_assertion_failed` (`stdlib/Runtime_Support/module.jai`) prints `path:line:col: error: assertion failed: message` then `call stack (innermost first):` with one `name at path:line` per node.

```text
call stack (innermost first):
  check at main.jai:12
  main at main.jai:20
```

## How to change it

- Node and info layouts come from `Stack_Trace_Node` and `Stack_Trace_Procedure_Info` in `stdlib/prelude/module.jai`. Their field sizes are fixed; a mismatch turns traces off.
- Compiled code gives no node to a quiet procedure (`quiet_funcs`): one that uses its context only to pass it to other quiet procedures. If lowering stores the context somewhere new, update `context_slot_addr`, or every leaf gets a node again (`quiet_procedures_push_no_stack_trace_node` in `native.rs` catches it).
- The pass's own context accesses go in `Func::trusted` so the backend skips null checks.
- Cost: nodes roughly double the time of `fib(40)` at `-O0`.

`Build_Options.stack_trace` (default true) controls workspaces a metaprogram creates. `_STACK_TRACE` is always `true`; modules write `#if _STACK_TRACE` to give per-thread state a node.
