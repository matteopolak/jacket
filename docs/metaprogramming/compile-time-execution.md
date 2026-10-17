# Compile-time execution, code values and insertion

## What it is

`#run` executes Jai code inside the compiler, in the interpreter that backs `jaic -interp` ([interpreter](../compiler/interpreter.md)); its result becomes a constant, type or global initializer. `#code` makes a `Code` value (unevaluated syntax plus its scope) and `#insert` splices code or a string back into the program.

## How it works

### `#run`

`Compiler::check_run` (`sema/consteval.rs`) handles every form. The expression is compiled to a throwaway thunk (`thunk_ctx`, `run_thunk`), run with `call_thunk` and read back with `read_value`.

```jai
F10   :: #run fact(10);
Names :: #run -> [3] string { return .["a","b","c"]; };
T     :: #run -> Type { return int; };
#run { counter = 99; print("compile time\n"); }
```

- Top-level `#run`s run in declaration order (`run_top_level`); top-level `#assert`s are checked after all of them. A `#run` that fails only because a `#placeholder` is undefined is retried later. `#compile_time` is true during `#run`. `#run,stallable` parses but runs synchronously. `jaic -check` also runs them.
- Failures report `error during compile-time execution: <trap message>` at the failing statement with a note at the `#run`. Metaprogram-reported errors have no prefix. Deep recursion ends with `stack overflow (recursion too deep)`.
- Compile-time code has full host access (files, `Process.run_command`, foreign procedures). `compile_time_context` builds one default context from Runtime_Support's `Context` (null without it).
- Limits: `STACK_SIZE` (32 MiB) and `MAX_DEPTH` (20,000) in `interp/mod.rs`. `jaic` sets no instruction budget; the playground, jailsp and jailint set `Interp::block_budget`.

### Results and globals

`read_value` copies the result out of interpreter memory; scalars, strings and types become plain `Value`s and aggregates an `Aggregate` (bytes plus relocations). `freeze` / `freeze_pointer` replace pointers with relocations: pointers into globals relocate, heap memory (a `[..]` from `array_add`) is copied into a read-only blob, small integers (< `0x10000`) and top-bit-set values (sign-extended handles) stay numbers, a pointer to memory of unknown size (`*void` from `alloc`) is an error, and union members are copied as raw bytes.

```jai
TABLE :: #run make();      // runtime: TABLE.tags prints [1, 4, 9, 16]

counter := 5;
#no_reset keep := 0;
#run { counter = 99; keep = 7; }     // main sees counter = 5, keep = 7
```

Compile-time changes to globals are undone before `main`, matching Jai: `resolve_global_var` (`sema/decls.rs`) records user globals in `Program.reset_globals`, and `Interp::reset_globals` zeroes and re-initialises them. `#no_reset` globals are read back with `read_aggregate` and baked into `init` (`bake_no_reset_globals` in `driver.rs`), so an executable sees what compile-time code stored (chess-jai loads network weights this way). `get_runtime_info()` reads `__runtime_info`, created by `sema/runtime_info.rs` and filled by `fill_runtime_info` with `type_table` and `global_data_info`; the layout mirrors `Runtime_Info` in `stdlib/Compiler/contract-records.jai`.

### `#code` and `#insert`

`Compiler::add_code` (`sema/mod.rs`) stores the AST in `Compiler::codes` and its scope in `code_scopes`; the value is `Value::Code(CodeId)`, compile-time only. `#insert X` evaluates `X` with `eval_insert_operand`, and `insert_stmts_from` turns `Code` into its statements, a string into a parsed source named `<#insert at path>`, `void` into nothing; anything else is `#insert needs a string or Code`.

```jai
make :: (name: string) -> string { return tprint("% :: 42;\n", name); }
#insert #run make("ANSWER");                    // top-level declaration from a string
mac :: (c: Code) #expand { for 0..1 { `defer print("d%\n", it); #insert,scope() c; } }
main :: () {
    mac(#code print("body\n"));                 // body d0 body d1
    x := 5;
    #insert #run tprint("x += %;", 3);          // x is now 8
}
```

Scope: in statement position inserted code resolves in the `Code`'s defining scope, so a macro needs `#insert,scope()` to see `it`. `#insert,scope(code)` / `,scope()` use the given scope or the insertion site (`check_insert`, `sema/stmt.rs`). In expression position (`eval_insert_expr`) `Code` checks in its scope and a string at the insertion site. `compiler_get_code` results without a scope resolve at the site, then where their nodes were written. `#insert (break=..., continue=..., remove=...) body` replaces loop control (`InsertReplacements`). Top-level insertions that cannot resolve yet (a later `#placeholder`) are retried when the workspace settles ([sema: polymorphism and declarations](../compiler/sema.md)).

## How to change it

- New run syntax: `parser/directive.rs`, then `E::Run` in `sema/expr.rs` and `check_run`. Work needed before `main` that is not a `#run` belongs in `Compiler::run_program` (`sema/driver.rs`).
- A new value kind crossing the `#run` boundary: extend `read_value` and `freeze` together. New user global storage: push it to `reset_globals` where the `Global` is created. Runtime info layout: change `runtime_info.rs` and `contract-records.jai` together.
- A new insertion context (enum bodies use `eval_insert_enum_items`): evaluate with `eval_insert_operand`, convert with `insert_stmts_from`, check in the right scope. Changing a `Code` argument's scope changes macro hygiene everywhere.

Tests: `tests/compiler/regressions/compile-time-globals-reset.jai`, `no-reset-globals-baked.jai`, `lang-insert-block.jai`, `insert-scope-named-code.jai`, `insert-replacements.jai`. Depends on the interpreter, `parser::parse_file`, `sema/scope.rs`.

## Rules

Claims that tests cite by ID (`tools/check_rule_coverage.py`).

- `#code` makes a `Code` value: unevaluated syntax plus the scope it was written in {#code.1}. `#insert` splices a `Code` value or a string back into the program at top level, as statements, or as an expression {#code.2}.
- `#insert X` evaluates `X` with `eval_insert_operand` (`sema/consteval.rs`). A parameterless `-> string { ... }` or `-> Code { ... }` operand runs at compile time like `#run` {#code.3}. `insert_stmts_from` then turns the result into statements:
- `Code` yields its statements {#code.4};
- a string is parsed as a new source file named `<#insert at path>`, so diagnostics point into the generated text {#code.5};
- `void` inserts nothing {#code.6}; anything else fails with `#insert needs a string or Code` {#code.7}.
- Statement position: the `Code`'s defining scope. So in a macro, a plain `#insert c` can't see names the macro defines, such as `it` (``unknown identifier `it` ``); use `#insert,scope()` {#code.14}.
- `#insert,scope(code)` / `#insert,scope()`: the given scope, or the insertion site (`check_insert` in `sema/stmt.rs`). A string becomes `Code` first {#code.11}.
- Expression position (`eval_insert_expr`): a `Code` value checks in its defining scope {#code.9}; a string is parsed as a parenthesized expression and checked at the insertion site {#code.10}.
- Code from `compiler_get_code` without a scope to copy resolves at the insertion site, falling back to where its nodes were written (see [compiler records](compiler-module.md)) {#code.8}.
- `#insert (break=..., continue=..., remove=...) body` replaces loop control inside an inserted `for` body (`InsertReplacements`) {#code.12}.
- The comments show the program's output {#code.13}.
- Top-level insertions that can't resolve yet, such as a `#placeholder` a metaprogram defines later, wait and are retried when the workspace settles {#code.15}; see [sema: polymorphism and declarations](../compiler/sema.md).
- After a thunk returns, `read_value` (`sema/consteval.rs`) copies the result out of interpreter memory. Scalars, strings and types become plain `Value`s {#ctdata.1}; aggregates become an `Aggregate` (bytes plus relocations). `freeze` and `freeze_pointer` walk the type and replace each pointer with a relocation:
- a pointer into a global becomes a relocation to that global {#ctdata.2};
- heap memory (say, a `[..]` built with `array_add`) is copied into a new read-only blob, recursively {#ctdata.3};
- integers below `0x10000` cast to pointers stay plain numbers {#ctdata.4}, and so do values with the top bit set, which are never user-space addresses on a 64-bit host (sign-extended handles such as `HKEY_CLASSES_ROOT :: cast(HKEY) cast(s64) cast,trunc(s32) 0x80000000`) {#ctdata.14};
- a pointer to memory of unknown size (`*void` from `alloc`) fails with `a compile-time value holds a pointer to memory of unknown size` {#ctdata.5};
- union members are copied as raw bytes, without following pointers {#ctdata.6}.
- Verified {#ctdata.7}:
- Changes `#run` code makes to globals do not reach the running program, matching Jai {#ctdata.8}.
- The example above prints `counter = 5, keep = 7` {#ctdata.11}.
- `resolve_global_var` (`sema/decls.rs`) adds every user global not marked `#no_reset` to `Program.reset_globals`. `run_program` calls `Interp::reset_globals`, which zeroes each one, copies its `init` bytes and re-applies relocations. `x := #run f();` keeps its value because the result is the initializer {#ctdata.9}. `jaic` emits only `init` data, so the reset is implicit there.
- `#no_reset` globals go to `sema.no_reset_globals`. Before codegen, `bake_no_reset_globals` (`driver.rs`, called from `prepare_compiled_output`) reads each one back from the interpreter with `read_aggregate`, the same path `#run` constants take, and makes that its `init`. An executable therefore sees what compile-time code stored, including strings, arrays and pointers into other globals. chess-jai loads its network weights this way {#ctdata.10}.
- `get_runtime_info()` from the Compiler module reads the `#elsewhere` symbol `__runtime_info`. `sema/runtime_info.rs` creates that global on first reference; `fill_runtime_info` fills it once the program is final with `type_table` (every type descriptor) {#ctdata.12} and `global_data_info` (the data and read-only data segments, which the memory debugger scans) {#ctdata.13}. The layout mirrors `Runtime_Info` in `stdlib/Compiler/workspace.jai`.
- `#run` executes Jai code inside the compiler, in the same interpreter that backs `jaic -interp` (see [interpreter](../compiler/interpreter.md)). The result becomes a constant, a type, or a global initializer of the program being compiled {#ctexec.1}.
- `Compiler::check_run` in `sema/consteval.rs` handles every form {#ctexec.2}:
- Top-level `#run`s run in declaration order from `run_top_level`, and top-level `#assert`s are checked after all of them {#ctexec.3}. One that fails only because a `#placeholder` isn't defined yet is retried after the metaprogram has had a chance to define it {#ctexec.4}.
- `#compile_time` is true during `#run` and false once `main` runs (`Interp::compile_time`, the `IsCompileTime` intrinsic) {#ctexec.5}.
- `print` output goes straight to stdout, before the program's own output {#ctexec.6}.
- Failures report `error during compile-time execution: <trap message>` at the user's statement that failed, with a note pointing at the `#run` (or other compile-time code) that started it {#ctexec.7}. An error a metaprogram reports itself (`compiler_report`, a workspace marked as failed) is shown without that prefix, at the place it names. Deep recursion ends with `stack overflow (recursion too deep)` {#ctexec.8}. A failed `assert` stops with ``assertion failed: msg``, or ``assertion failed: `x == 4` is false`` when it has no message, at the `assert` call {#ctexec.9}.
- `#assert cond, "msg"` is evaluated with `eval_static_condition`; failure is `#assert failed: msg` {#ctexec.10}.
- Compile-time code has full host access: files, `Process.run_command`, and foreign procedures through the real dynamic loader {#ctexec.11}. A metaprogram can do anything the user can {#ctexec.12}. `fork` in the child is guarded; see the interpreter page.
- `#run,stallable` parses (`ast::Expr::Run::flags`), but sema ignores the flag; running is always synchronous {#ctexec.13}.
