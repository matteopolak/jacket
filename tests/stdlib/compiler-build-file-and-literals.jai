// Compiler metaprogram APIs from a #run whose results main checks: add_build_file next to
// add_build_string in a child workspace, Program_Print's print_declaration on the typechecked
// declarations, make_integer_literal / make_string_literal printed back as source,
// Code_Visit's maybe_array_add, developer_debug and compiler_destroy_workspace.
#import "Basic";
#import "Compiler";
Program_Print :: #import "Program_Print";
#import "Code_Visit";
#import "String";

EXTRA :: #string END
main :: () {}
typed_value : int = 3;
END

// Each wanted declaration printed on its own line, in name order.
build_and_print :: () -> string {
    w := compiler_create_workspace("build file");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    // Rules_Single.jai declares `single_value :: 11;`.
    add_build_file(tprint("%modules/Rules_Single.jai", path_strip_filename(#file)), w);
    add_build_string(EXTRA, w);
    single, typed: string;
    while true {
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE {
            complete := cast(*Message_Complete) message;
            assert(complete.error_code == .NONE, "the child workspace compiles: %", complete.error_code);
            break;
        }
        if message.kind != .TYPECHECKED continue;
        typechecked := cast(*Message_Typechecked) message;
        for typechecked.declarations {
            declaration := it.expression;
            if declaration.name != "single_value" && declaration.name != "typed_value" continue;
            builder: String_Builder;
            assert(Program_Print.print_declaration(*builder, declaration), "% prints", declaration.name);
            if declaration.name == "single_value" single = builder_to_string(*builder);
            else typed = builder_to_string(*builder);
        }
    }
    compiler_destroy_workspace(w);
    return tprint("%\n%\n", single, typed);
}

literals :: () -> string {
    source := "a\"b";
    number := make_integer_literal(-42);
    text := make_string_literal(source);
    assert(number.kind == .LITERAL && number.value_type == .NUMBER && number._s64 == -42);
    assert(text.kind == .LITERAL && text.value_type == .STRING && text._string == source);
    assert(text._string.data != source.data, "make_string_literal copies its text");
    developer_debug(number);

    nodes: [..] *Code_Node;
    maybe_array_add(*nodes, number);
    maybe_array_add(*nodes, null);
    maybe_array_add(*nodes, text);
    assert(nodes.count == 2 && nodes[1] == text, "maybe_array_add skips null: %", nodes.count);

    builder: String_Builder;
    for nodes {
        if it_index append(*builder, " ");
        assert(Program_Print.print_expression(*builder, it), "literal % prints", it_index);
    }
    return builder_to_string(*builder);
}

PRINTED :: #run build_and_print();
LITERALS :: #run literals();

main :: () {
    assert(PRINTED == "single_value :: 11\ntyped_value : int = 3\n", "declarations:\n%", PRINTED);
    assert(LITERALS == "-42 \"a\\\"b\"", "literals: %", LITERALS);

    // maybe_array_add also works on ordinary arrays at run time.
    values: [..] *int;
    a := 5;
    maybe_array_add(*values, null);
    maybe_array_add(*values, *a);
    assert(values.count == 1 && values[0].* == 5);
    array_free(values);
    print("ok\n");
}
