# Compile-time execution (`#run`)

## What it is

`#run` executes Jai code inside the compiler, in the same interpreter that backs `jaic run`
(`crates/jaic/src/interp`, see [interpreter](../compiler/interpreter.md)). Its result becomes a constant,
a type or a global initializer of the program being compiled.

## How it works

Forms (all checked in `Compiler::check_run`, `crates/jaic/src/sema/consteval.rs`):

```jai
F10   :: #run fact(10);                          // expression form
Names :: #run -> [3] string { return .["a","b","c"]; };   // anonymous body with a return type
T     :: #run -> Type { return int; };           // may yield a Type
#run { counter = 99; print("compile time\n"); }  // top-level statement: runs for effect
main :: () { v := #run make(); }                 // also valid inside procedure bodies
```

- The expression or block is compiled into a throwaway "thunk" function (`thunk_ctx`, `run_thunk`), stored
  in a `run.result` global, executed with `call_thunk`, and the bytes read back with `read_value`.
- Top-level `#run` directives and `#assert`s run in declaration order from `run_top_level`. A run that
  fails only because a `#placeholder` is not defined yet is retried after the metaprogram has had a
  chance to define it.
- `#compile_time` is true inside such code (`Interp::compile_time`, the `IsCompileTime` intrinsic) and false
  once `main` runs, so `Phase :: #run -> bool { return #compile_time; };` captures `true`, while the same test
  in `main` yields `false`.
- Output of `print` at compile time goes straight to stdout; it appears before the program's own output.
- Failures are reported as `error during compile-time execution: <trap message>` with a note naming the
  file and line being executed. Deep recursion ends with `stack overflow (recursion too deep)`
  (`MAX_DEPTH` = 20,000 frames in `interp/mod.rs`). A failed `assert` prints `Assertion failed: msg` and stops.
- `#assert cond, "msg"` is evaluated with `eval_static_condition`; failure is `#assert failed: msg`.
- Compile-time code has full host access: `read_entire_file` and `Process.run_command` work natively
  (verified: a `#run` calling `run_command("echo", ...)` with `capture_and_return_output = true` returns the
  child's output). Foreign procedures are called through the real dynamic loader, so a metaprogram can do
  anything the user can. `fork` in the child is guarded; see the interpreter page.

`#run,stallable` parses (flags are kept in `ast::Expr::Run::flags`) but sema ignores the flag; running is
always synchronous.

## How to change it

- New run syntax: parser in `crates/jaic/src/parser/directive.rs`; then `E::Run` handling in
  `sema/expr.rs` and `check_run`.
- Resource limits are only the interpreter's stack size and frame depth. There is no instruction budget, so
  an infinite loop in `#run` never returns. Add one in the interpreter's step loop if you need it.
- Anything that must run before the user's `main` but is not a `#run` (state reset, runtime info) belongs in
  `Compiler::run_program` (`sema/driver.rs`).

## Configuration

`STACK_SIZE` and `MAX_DEPTH` in `crates/jaic/src/interp/mod.rs`. No environment variables. `jaic check` also
executes `#run` directives and `#assert`s (a failing `#assert` fails the check with `#assert failed: msg`).

## Dependencies

`sema/consteval.rs`, `interp`, the `Context` type from Runtime Support (`compile_time_context` builds one
shared default context for compile-time code; without Runtime Support it passes null).
