// String literal printing escapes quotes, backslashes, and line feeds.
#import "Basic";
Compiler :: #import "Compiler";
Program_Print :: #import "Program_Print";

main :: () {
    builder: String_Builder;

    init_string_builder(*builder);
    Program_Print.print_string_literal(*builder, "say\"hi\\\n");
    text := builder_to_string(*builder);

    assert(text == "\"say\\\"hi\\\\\\n\"");

    expression_builder: String_Builder;

    init_string_builder(*expression_builder);
    literal := Compiler.make_integer_literal(47);
    printed := Program_Print.print_expression(*expression_builder, cast(*Compiler.Code_Node) literal);

    assert(printed && builder_to_string(*expression_builder) == "47");
    print("ok\n");
}
