# Code values and #insert

## What it is

`#code` makes a `Code` value: unevaluated syntax plus the scope it was written in {#code.1}. `#insert` splices a `Code` value or a string back into the program at top level, as statements, or as an expression {#code.2}.

## How it works

`Compiler::add_code` (`sema/mod.rs`) stores the AST (`ast::CodeBody::Block` or `Expr`) in `Compiler::codes` and its defining scope in `code_scopes`. The value is `Value::Code(CodeId)` and exists only at compile time.

`#insert X` evaluates `X` with `eval_insert_operand` (`sema/consteval.rs`). A parameterless `-> string { ... }` or `-> Code { ... }` operand runs at compile time like `#run` {#code.3}. `insert_stmts_from` then turns the result into statements:

- `Code` yields its statements {#code.4};
- a string is parsed as a new source file named `<#insert at path>`, so diagnostics point into the generated text {#code.5};
- `void` inserts nothing {#code.6}; anything else fails with `#insert needs a string or Code` {#code.7}.

Which scope the inserted code resolves in:

- Statement position: the `Code`'s defining scope. So in a macro, a plain `#insert c` can't see names the macro defines, such as `it` (``unknown identifier `it` ``); use `#insert,scope()` {#code.14}.
- `#insert,scope(code)` / `#insert,scope()`: the given scope, or the insertion site (`check_insert` in `sema/stmt.rs`). A string becomes `Code` first {#code.11}.
- Expression position (`eval_insert_expr`): a `Code` value checks in its defining scope {#code.9}; a string is parsed as a parenthesized expression and checked at the insertion site {#code.10}.
- Code from `compiler_get_code` without a scope to copy resolves at the insertion site, falling back to where its nodes were written (see [compiler records](compiler-records.md)) {#code.8}.

`#insert (break=..., continue=..., remove=...) body` replaces loop control inside an inserted `for` body (`InsertReplacements`) {#code.12}.

```jai
make :: (name: string) -> string { return tprint("% :: 42;\n", name); }
#insert #run make("ANSWER");                    // top-level declaration from a string
mac :: (c: Code) #expand { for 0..1 { `defer print("d%\n", it); #insert,scope() c; } }
code_val :: #code print("hi\n");
main :: () {
    print("%\n", ANSWER);                       // 42
    mac(#code print("body\n"));                 // body d0 body d1
    #insert code_val;                           // hi
    #insert -> string { return "print(\"from string\\n\");"; };
    x := 5;
    #insert #run tprint("x += %;", 3);          // x is now 8
}
```

The comments show the program's output {#code.13}.

Top-level insertions that can't resolve yet, such as a `#placeholder` a metaprogram defines later, wait and are retried when the workspace settles {#code.15}; see [sema: polymorphism and declarations](../compiler/sema-polymorphism-and-declarations.md).

Metaprograms read code with `compiler_get_nodes` and write it back with `compiler_modify_procedure` or `add_build_string`, both via [compiler records](compiler-records.md).

## How to change it

- A new insertion context (enum bodies do this in `eval_insert_enum_items`): evaluate with `eval_insert_operand`, convert with `insert_stmts_from`, check the result in the right scope.
- Changing which scope a `Code` argument uses changes macro hygiene everywhere.
- Tests: `tests/stdlib/lang-insert-block.jai`, `insert-scope-named-code.jai`, `insert-scope-string-top-level.jai`, `insert-expression-code-scope.jai`, `insert-replacements.jai`, `macro-code-variable-arg.jai`, `enum-insert-members.jai`.

## Dependencies

The interpreter (to run the producing code), `parser::parse_file` for strings, `sema/scope.rs`.
