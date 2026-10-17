# Stack traces (`context.stack_trace`)

## What it is

While a program runs, under `jaic -interp` or as a `jaic` executable, `context.stack_trace` points at a linked list of `Stack_Trace_Node`s, innermost procedure first, as in Jai. `print_stack_trace`, assertion failures and the memory debugger's leak reports read it.

## How it works

- At lowering (`sema/procs.rs`, `lower_body_code`) every procedure that takes a context and is not `inline` or `#no_debug` gets `Func.trace = Some(TraceInfo { name, file, line, col })`.
- **Interpreter**: `Interp::exec` pushes a node in the callee's stack frame with `next`, `info` (a leaked `Stack_Trace_Procedure_Info` cached per `FuncId`), `hash`, `call_depth` and `line_number`. On entry the callee writes the current source location into the caller node's `line_number`; on return `context.stack_trace` is restored. Inside untraced procedures the interpreter keeps the location of the call that entered them (`trace_loc`), so a traced call made there records the caller's line.
- `Program.stack_trace` (`ir::TraceLayout`: where `stack_trace` sits in `#Context`, and the field offsets and sizes of the node types, read from the Preload) and `Program.file_paths` are filled by `Compiler::enable_stack_traces` from `run_program` and `call_thunk` (so `#run` code gets traces too). Nothing is pushed when the layout is `None`.
- **Compiled output**: `Compiler::prepare_compiled_output` runs `stack_trace::instrument` (`crates/jaic/src/stack_trace.rs`), an IR pass. Each traced function gets a node slot; a new entry block links it and makes it the top; every `Ret` restores the previous top; after each `Loc` whose statement can call, the line is stored into the node. A `Stack_Trace_Procedure_Info` global per function holds name, site and address. The pass clears `Func.trace` so the interpreter does not push a second node.
- `runtime_support_assertion_failed` (`stdlib/Runtime_Support/module.jai`) prints `path:line:col: error: assertion failed: message`, then `call stack (innermost first):` with one `name at path:line` per node, the layout `jaic -interp` uses for runtime errors ([diagnostics](diagnostics.md)). The stack starts at the node running the asserting line, leaving out assertion machinery.

## How to change it

- Field offsets come from `Stack_Trace_Node` and `Stack_Trace_Procedure_Info` in `stdlib/prelude/module.jai` (`Compiler::trace_layout`). Their sizes are fixed (`u64` hash, `u32` depth and line); `trace_layout` turns traces off on a mismatch.
- Compiled code gives no node to a quiet procedure (`quiet_funcs`): one that uses its context only to pass it on and calls only quiet procedures directly (found by fixed point; recursion never qualifies). An `assert`, allocator or indirect call disqualifies it. This keeps small helper chains free of push and pop. The interpreter still pushes for every traced procedure.
- The pass's own loads and stores through the context and previous node are listed in `Func::trusted` so the backend does not null-check them; keep new bookkeeping accesses in that list.
- Cost: with nodes `fib(40)` runs about twice as long (1.0 s against 0.5 s at `-O0`).
- Hash values differ from Jai's. There is no sentinel node from `push_context`, so call depths can differ there.

## Configuration

`Build_Options.stack_trace` (default true), read for workspaces a metaprogram creates.

## Dependencies

`ir::Func::trace`, `ir::Program::{file_paths, stack_trace}`, `interp::Interp::trace_enter`. Tests: `tests/stdlib/stack-trace-nodes.jai`, `stack-trace-line-through-inline.jai`, `compile-time-stack-trace.jai`, and `stack_traces` in `crates/jaic-cli/tests/native.rs`.
