// A Code variable passed to a macro's Code parameter passes its value, not its name.
#import "Basic";
#import "Compiler";
#import "Program_Print";

code_text :: (code: Code) -> string #expand {
    root := compiler_get_nodes(code);
    builder: String_Builder;
    print_expression(*builder, root);
    return builder_to_string(*builder);
}

TEXT :: #run -> string {
    code1 := #code a_constant :: 10;
    return code_text(code1);
};

main :: () -> s64 {
    if TEXT != "a_constant :: 10" {
        print("%\n", TEXT);
        return 1;
    }
    return 0;
}
