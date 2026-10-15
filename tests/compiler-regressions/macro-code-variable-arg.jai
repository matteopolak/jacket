// A Code variable passed to a macro's Code parameter passes its value, not its name.
// rules: macro.7
#import "Basic";
#import "Compiler";
#import "Program_Print";
#import "String";

code_text :: (code: Code) -> string #expand {
    root := compiler_get_nodes(code);

    assert(root.kind == .DECLARATION);
    declaration := cast(*Code_Declaration) root;

    assert(declaration.name == "a_constant" && declaration.expression.kind == .LITERAL);
    assert((cast(*Code_Literal) declaration.expression)._s64 == 10);
    builder: String_Builder;

    assert(print_expression(*builder, root));

    return builder_to_string(*builder);
}

TEXT :: #run -> string {
    code1 := #code a_constant :: 10;
    return code_text(code1);
};

main :: () -> s64 {
    if !contains(TEXT, "a_constant") || !contains(TEXT, "10") {
        print("%\n", TEXT);
        return 1;
    }

    return 0;
}
