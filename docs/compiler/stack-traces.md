# Stack traces (`context.stack_trace`)

## What it is

While a program runs, under `jaic run` or as a `jaic build` executable, `context.stack_trace` points at a linked list of `Stack_Trace_Node`s, innermost procedure first, as in Jai. `print_stack_trace`, assertion failures and the memory debugger's leak reports read it.

## How it works

- At lowering (`sema/procs.rs`, `lower_body_code`) every procedure that takes a context, and is
  not `inline` or `#no_debug`, gets `Func.trace = Some(TraceInfo { name, file, line, col })`.
- The interpreter tracks `trace_loc`: inside untraced procedures (`inline`, `#no_debug`) it holds the
  location of the call that entered them, so a traced call made there records the caller's own line
  (test: `tests/stdlib/stack-trace-line-through-inline.jai`).
- `Interp::exec` (`interp/mod.rs`) pushes a node when such a function is entered. The node lives in
  the callee's interpreter stack frame (right after the frame). Its fields: `next` (the
  previous top), `info` (a leaked `Stack_Trace_Procedure_Info` cached per `FuncId`), `hash`,
  `call_depth` (previous + 1, first node 1) and `line_number`.
- The caller's node `line_number` is the line of the call being made: on entry the callee writes
  the interpreter's current source location (`Inst::Loc`) into `previous.line_number`. A fresh
  node starts with the procedure's declaration line.
- On return `context.stack_trace` is restored to `next`. `exec` also restores `self.loc` so a
  statement with several calls reports the right line for each.
- `Program.stack_trace` (an `ir::TraceLayout`: where `stack_trace` sits in `#Context`, and
  the field offsets and sizes of `Stack_Trace_Node` and `Stack_Trace_Procedure_Info`, read
  from their declarations in the Preload) and `Program.file_paths` are filled by
  `Compiler::enable_stack_traces`, called from `run_program` and
  from `call_thunk` (so `#run` code also gets traces; the thunk itself has no node, so the first
  node's `next` is null). Nothing is pushed when the layout is `None`, which is also what a Preload whose fields have other sizes gets.
- **Compiled output**: `Compiler::prepare_compiled_output` (called by both `jaic build` and the
  metaprogram's output path before the backend writes) runs `stack_trace::instrument`
  (`crates/jaic/src/stack_trace.rs`), an IR pass. Each traced function gets a node slot; a new
  entry block links it (depth and hash from the previous top, or 1 and a seed for the first node) and
  makes it the top; every `Ret` restores the previous top; after each `Loc` whose statement makes a
  call (looking through the blocks the statement continues into), the line is stored into the node,
  so callees see their call line. A
  `Stack_Trace_Procedure_Info` global per function holds name, declaration site and address. The pass
  clears `Func.trace`, so the interpreter does not push a second node. Test: `stack_traces` in
  `crates/jaic-cli/tests/native.rs`.
- `runtime_support_assertion_failed` (`stdlib/Runtime_Support.jai`) prints
  `path:line:col: error: assertion failed: message` (`assertion failed` alone when there is
  none), then `call stack (innermost first):` and one `name at path:line` per node, the same
  layout `jaic run` uses for runtime errors (see [diagnostics](diagnostics.md)).
  The stack starts at the node running the line the assertion names (`loc`), which leaves out
  the assertion machinery above it (`assert_helper`, the context's handler, the report itself)
  without naming it; the interpreter's report skips its frames the same way (`trap_report.rs`).

## How to change it

- Field offsets come from `Stack_Trace_Node` and `Stack_Trace_Procedure_Info` in
  `stdlib/prelude/module.jai` (`Compiler::trace_layout`), so fields can be moved or added there.
  Their sizes are fixed: `trace_layout` checks them and turns traces off on a mismatch, since
  `trace_enter` and the pass write a `u64` hash and `u32` depth and line.
- Hash values differ from Jai's; they only mix the caller hash, procedure id and call line.
- There is no sentinel node from `push_context`, so call depths can differ in that case.
- Compiled code gives no node to a procedure that contains no call and never uses its context
  pointer (`makes_no_calls`): nothing can read the trace from inside it, and Jai
  omits leaf procedures as well. The interpreter still pushes a node for every traced procedure.
- The pass's own loads and stores through the context and the previous node are listed in
  `Func::trusted`, so the backend does not null-check them; keep new bookkeeping accesses
  in that list.
- Cost: with nodes `fib(40)` runs about twice as long as without (1.0 s and 0.5 s at `-O0`).

## Configuration

`Build_Options.stack_trace` (default true) is read for workspaces created by a metaprogram
(`BuildSettings.stack_trace`, `Options.stack_trace`).

## Dependencies

`ir::Func::trace`, `ir::Program::{file_paths, stack_trace}`, `interp::Interp::trace_enter`.
Tests: `tests/stdlib/stack-trace-nodes.jai`, `tests/stdlib/compile-time-stack-trace.jai`.
