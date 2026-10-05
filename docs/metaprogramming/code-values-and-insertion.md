# Code values and #insert

## What it is

`#code` creates a `Code` value (an unevaluated piece of syntax plus the scope it was written in). `#insert` splices a `Code` value or a string back into the program, either at top level, as a statement or as an expression.

## How it works

`Compiler::add_code` (`sema/mod.rs`) stores the AST (`ast::CodeBody::Block` or `Expr`) in `Compiler::codes` and its defining scope in `code_scopes`; the value is `Value::Code(CodeId)`. It is compile-time only.

`#insert X` evaluates `X` with `eval_insert_operand` (`sema/consteval.rs`). If `X` is `-> string { ... }` or `-> Code { ... }` with no parameters, it runs at compile time as a `#run`. The operand then becomes statements (`insert_stmts_from`):

- a `Code` value yields its statements;
- a string is parsed as a new source file named `<#insert at path>`, so diagnostics point into the text;
- `void` inserts nothing; anything else is an error ("#insert needs a string or Code").

Code made by `compiler_get_code` without a scope to copy is the exception: it resolves at the insertion site, falling back on where its nodes were written (see [compiler-records](compiler-records.md)). In expression position (`eval_insert_expr`) a `Code` value is checked in its own defining scope and a string is parsed as a parenthesized expression and checked in the insertion scope. `#insert,scope(code)` / `#insert,scope()` checks in an explicit scope instead (`check_insert` in `sema/stmt.rs`); a string becomes `Code` there first. `#insert (break=..., continue=..., remove=...) body` replaces loop control inside an inserted `for` body (`InsertReplacements`).

Verified output of this program:

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

A plain `#insert c` of a `Code` argument checks the code where it was written, so a name the macro defines (such as `it`) is unknown there (`unknown identifier 'it'`); use `#insert,scope()`. Top-level insertions that cannot resolve yet (a `#placeholder` that a metaprogram defines later) wait and are retried when the workspace settles; see [sema-polymorphism-and-declarations.md](../compiler/sema-polymorphism-and-declarations.md).

Metaprograms read code with `compiler_get_nodes` and write it back with `compiler_modify_procedure` or `add_build_string`; those go through records ([compiler-records.md](compiler-records.md)).

## How to change it

- New insertion context (enum bodies already do this via `eval_insert_enum_items` in `sema/structs.rs`): evaluate the operand with `eval_insert_operand`, convert with `insert_stmts_from`, then check the result in the right scope.
- Scope rules live in `code_scopes`; changing which scope a `Code` argument uses affects macro hygiene everywhere.
- Regression programs: `tests/stdlib/lang-insert-block.jai`, `insert-scope-named-code.jai`, `insert-scope-string-top-level.jai`, `insert-expression-code-scope.jai`, `insert-replacements.jai`, `macro-code-variable-arg.jai`, `enum-insert-members.jai`.

## Configuration

None.

## Dependencies

The interpreter (running the producing procedure), `parser::parse_file` for strings, `sema/scope.rs`.
