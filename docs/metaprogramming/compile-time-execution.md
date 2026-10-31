# Compile-time execution, code values and insertion

`#run` executes Jai inside the compiler, in the interpreter behind `jaic -interp` ([interpreter](../compiler/interpreter.md)); its result becomes a constant, type or global initializer. `#code` makes a `Code` value (syntax plus its scope), and `#insert` splices code or a string back into the program.

```jai
F10   :: #run fact(10);
Names :: #run -> [3] string { return .["a","b","c"]; };
T     :: #run -> Type { return int; };
#run { counter = 99; print("compile time\n"); }

make :: (name: string) -> string { return tprint("% :: 42;\n", name); }
#insert #run make("ANSWER");                    // top-level declaration from a string
mac :: (c: Code) #expand { for 0..1 { `defer print("d%\n", it); #insert,scope() c; } }
main :: () {
    mac(#code print("body\n"));                 // body d0 body d1
    x := 5;
    #insert #run tprint("x += %;", 3);          // x is now 8
}
```

## How it works

**`#run`.** `Compiler::check_run` (`sema/consteval.rs`) compiles the expression to a throwaway thunk, runs it with `call_thunk` and reads the result with `read_value`. Top-level `#run`s run in declaration order (`run_top_level`); top-level `#assert`s are checked after them. State lives in `Compiler::ct` (`sema/compile_time.rs`). Limits are `STACK_SIZE` (32 MiB) and `MAX_DEPTH` (20,000) in `interp/mod.rs`; the playground, jailsp and jailint also set `Interp::block_budget`. `Drop for Compiler` calls `Interp::shutdown` before fields drop, because Jai threads started at compile time can borrow the program.

**Results.** `read_value` copies out of interpreter memory: scalars, strings and types become plain `Value`s, aggregates an `Aggregate` (bytes plus relocations). `freeze` replaces pointers with relocations.

```jai
TABLE :: #run make();      // runtime: TABLE.tags prints [1, 4, 9, 16]

counter := 5;
#no_reset keep := 0;
#run { counter = 99; keep = 7; }     // main prints counter = 5, keep = 7
```

Compile-time writes to globals are undone before `main`: `resolve_global_var` (`sema/decls.rs`) records user globals in `Program.reset_globals` and `Interp::reset_globals` re-initialises them. `#no_reset` globals are read back and baked into `init` (`bake_no_reset_globals` in `driver.rs`), so an executable sees what compile-time code stored.

**`#code` and `#insert`.** `Compiler::add_code` stores the AST in `Compiler::pr.codes` and its scope in `code_scopes`; the value is `Value::Code`, compile-time only. `#insert X` evaluates `X` with `eval_insert_operand`, and `insert_stmts_from` turns the result into statements. In statement position, inserted code resolves in the `Code`'s defining scope, so a macro needs `#insert,scope()` to see `it`.

## How to change it

- New run syntax: `parser/directive.rs`, then `E::Run` in `sema/expr.rs` and `check_run`. Work needed before `main` that is not a `#run` goes in `Compiler::run_program` (`sema/driver.rs`).
- A new value kind crossing the `#run` boundary: extend `read_value` and `freeze` together. New user global storage: push it to `reset_globals`. Runtime info layout: change `runtime_info.rs` and `contract-records.jai` together.
- A new insertion context: evaluate with `eval_insert_operand`, convert with `insert_stmts_from`, check in the right scope. Changing a `Code` argument's scope changes macro hygiene everywhere.

Tests: `tests/compiler/regressions/compile-time-globals-reset.jai`, `no-reset-globals-baked.jai`, `lang-insert-block.jai`, `insert-scope-named-code.jai`, `insert-replacements.jai`.

## Rules

Claims that tests cite by ID (`tools/dev/check_rule_coverage.py`).

### `#run`

- `#run` executes Jai inside the compiler, in the interpreter behind `jaic -interp`; the result becomes a constant, type or global initializer {#ctexec.1}.
- `Compiler::check_run` handles every form {#ctexec.2}.
- Top-level `#run`s run in declaration order, and top-level `#assert`s are checked after all of them {#ctexec.3}. One that fails only because a `#placeholder` is not defined yet is retried after the metaprogram can define it {#ctexec.4}.
- `#compile_time` is true during `#run` and false once `main` runs {#ctexec.5}.
- `print` output goes to stdout, before the program's own output {#ctexec.6}.
- A failure reports `error during compile-time execution: <trap message>` at the failing statement, with a note at the `#run` that started it {#ctexec.7}. Errors a metaprogram reports itself (`compiler_report`, a failed workspace) have no prefix.
- Deep recursion ends with `stack overflow (recursion too deep)` {#ctexec.8}. A failed `assert` stops with ``assertion failed: msg``, or ``assertion failed: `x == 4` is false`` without a message {#ctexec.9}.
- `#assert cond, "msg"` is evaluated by `eval_static_condition`; failure is `#assert failed: msg` {#ctexec.10}.
- Compile-time code has full host access: files, `Process.run_command`, foreign procedures through the real dynamic loader {#ctexec.11}. A metaprogram can do anything the user can {#ctexec.12}.
- `#run,stallable` parses but is always synchronous {#ctexec.13}.

### Results and globals

- After a thunk returns, `read_value` copies the result out. Scalars, strings and types become plain `Value`s {#ctdata.1}.
- A pointer into a global becomes a relocation to that global {#ctdata.2}.
- Heap memory (a `[..]` built with `array_add`) is copied recursively into a read-only blob {#ctdata.3}.
- Integers below `0x10000` cast to pointers stay plain numbers {#ctdata.4}, and so do values with the top bit set, which are never user-space addresses on a 64-bit host {#ctdata.14}.
- A pointer to memory of unknown size (`*void` from `alloc`) keeps its address and captures no data {#ctdata.5}.
- Union members are copied as raw bytes, without following pointers {#ctdata.6}.
- These behaviours are verified by tests {#ctdata.7}.
- Changes `#run` code makes to globals do not reach the running program {#ctdata.8}.
- `resolve_global_var` adds every user global not marked `#no_reset` to `Program.reset_globals`; `run_program` calls `Interp::reset_globals`. `x := #run f();` keeps its value because the result is the initializer {#ctdata.9}.
- `#no_reset` globals go to `sema.no_reset_globals`, and `bake_no_reset_globals` reads each back, including strings, arrays and pointers into other globals, as its `init` {#ctdata.10}.
- The example above prints `counter = 5, keep = 7` {#ctdata.11}.
- `get_runtime_info()` reads `__runtime_info`, created by `sema/runtime_info.rs` and filled once the program is final with `type_table` {#ctdata.12} and `global_data_info` (the data and read-only segments) {#ctdata.13}.

### `#code` and `#insert`

- `#code` makes a `Code` value: unevaluated syntax plus its scope {#code.1}. `#insert` splices a `Code` value or a string at top level, as statements, or as an expression {#code.2}.
- `#insert X` evaluates `X` with `eval_insert_operand`. A parameterless `-> string { ... }` or `-> Code { ... }` operand runs at compile time like `#run` {#code.3}.
- `Code` yields its statements {#code.4}. A string is parsed as a new source named `<#insert at path>`, so diagnostics point into it {#code.5}. `void` inserts nothing {#code.6}. Anything else fails with `#insert needs a string or Code` {#code.7}.
- `compiler_get_code` results with no scope to copy resolve at the insertion site, then where their nodes were written {#code.8}.
- Expression position: a `Code` value checks in its defining scope {#code.9}; a string is parsed as a parenthesized expression and checked at the insertion site {#code.10}.
- `#insert,scope(code)` and `#insert,scope()` use the given scope or the insertion site; a string becomes `Code` first {#code.11}.
- `#insert (break=..., continue=..., remove=...) body` replaces loop control inside an inserted `for` body {#code.12}.
- Comments in the examples show the program's output {#code.13}.
- Statement position resolves in the `Code`'s defining scope, so a plain `#insert c` in a macro cannot see the macro's `it` (``unknown identifier `it` ``); use `#insert,scope()` {#code.14}.
- Top-level insertions that cannot resolve yet, such as a `#placeholder` defined later, wait and are retried when the workspace settles {#code.15}.
