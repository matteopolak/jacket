// A `#run` that asks for typed nodes is run again from the start; `compiler_get_code` on the
// same tree must give the same code in each run, so rendering compiler_get_code(root)
// inside one expression finishes instead of asking for new nodes forever.
// rules: records.29
#import "Basic";
#import "Compiler";
#import "Program_Print";
#import "String";

render :: (code: Code) -> string {
    node := compiler_get_nodes(code);

    assert(node.kind == .DECLARATION);
    declaration := cast(*Code_Declaration) node;

    assert(declaration.name == "value");
    assert(declaration.expression.kind == .LITERAL);
    assert((cast(*Code_Literal) declaration.expression)._s64 == 42);
    builder: String_Builder;

    assert(print_expression(*builder, node));

    return builder_to_string(*builder);
}

#run {
    code :: #code value :: 42;
    root, expressions := compiler_get_nodes(code);

    print("nodes % %\n", expressions.count, render(compiler_get_code(root)));
    rendered := render(compiler_get_code(root));

    assert(contains(rendered, "value") && contains(rendered, "42"), "rendered: %", rendered);
}

main :: () {
    print("ok\n");
}
