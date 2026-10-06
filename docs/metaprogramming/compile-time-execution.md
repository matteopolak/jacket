# Compile-time execution (`#run`)

## What it is

`#run` executes Jai code inside the compiler, in the same interpreter that backs `jaic run` (see [interpreter](../compiler/interpreter.md)). The result becomes a constant, a type, or a global initializer of the program being compiled.

## How it works

`Compiler::check_run` in `sema/consteval.rs` handles every form:

```jai
F10   :: #run fact(10);                                  // expression
Names :: #run -> [3] string { return .["a","b","c"]; };  // anonymous body with a result type
T     :: #run -> Type { return int; };                   // may yield a Type
#run { counter = 99; print("compile time\n"); }          // top level, for effect
main :: () { v := #run make(); }                         // inside procedure bodies too
```

The expression or block is compiled into a throwaway thunk (`thunk_ctx`, `run_thunk`) that stores into a `run.result` global, runs with `call_thunk`, and is read back with `read_value`. See [compile-time values](compile-time-data-and-state.md) for how results are frozen.

- Top-level `#run`s and `#assert`s run in declaration order from `run_top_level`. One that fails only because a `#placeholder` isn't defined yet is retried after the metaprogram has had a chance to define it.
- `#compile_time` is true during `#run` and false once `main` runs (`Interp::compile_time`, the `IsCompileTime` intrinsic).
- `print` output goes straight to stdout, before the program's own output.
- Failures report `error during compile-time execution: <trap message>` with a note naming the file and line being executed. Deep recursion ends with `stack overflow (recursion too deep)`. A failed `assert` prints `Assertion failed: msg` and stops.
- Compile-time code has full host access: files, `Process.run_command`, and foreign procedures through the real dynamic loader. A metaprogram can do anything the user can. `fork` in the child is guarded; see the interpreter page.
- `#run,stallable` parses (`ast::Expr::Run::flags`), but sema ignores the flag; running is always synchronous.
- `jaic check` also runs `#run`s and `#assert`s.

`compile_time_context` builds one shared default context for compile-time code from Runtime_Support's `Context` type; without Runtime_Support it passes null.

## How to change it

- New run syntax: `parser/directive.rs`, then `E::Run` in `sema/expr.rs` and `check_run`.
- There is no instruction budget, so an infinite loop in `#run` hangs the compiler. Add one in the interpreter's step loop if needed.
- Work that must happen before `main` but isn't a `#run` (global reset, runtime info) belongs in `Compiler::run_program` (`sema/driver.rs`).

## Configuration

`STACK_SIZE` (32 MiB) and `MAX_DEPTH` (20,000 frames) in `crates/jaic/src/interp/mod.rs` are the only limits.

## Dependencies

`sema/consteval.rs`, `crates/jaic/src/interp`, and Runtime_Support's `Context`.
