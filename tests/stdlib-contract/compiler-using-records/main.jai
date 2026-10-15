#import "Basic";
C :: #import "Compiler";
P :: #import "Program_Print";

Point :: struct {
    x: s64;
    y: s64;
}

#run {
    code := #code {
        point: Point;
        using,only(.["x"]) point;
    };

    root := C.compiler_get_nodes(code);
    block := cast(*C.Code_Block) root;
    using_node := cast(*C.Code_Using) block.statements[1];

    assert(using_node.kind == .USING && using_node.filter_type == .ONLY);
    assert(using_node.filter_expression && using_node.filter_expression.kind == .LITERAL);
    builder: String_Builder;

    init_string_builder(*builder);
    assert(P.print_expression(*builder, using_node));
    assert(builder_to_string(*builder) == "using,only(.[\"x\"]) point");
    free_buffers(*builder);
    copied := C.compiler_get_code(root, code);
    copied_root := C.compiler_get_nodes(copied);
    copied_block := cast(*C.Code_Block) copied_root;

    assert((cast(*C.Code_Using) copied_block.statements[1]).filter_type == .ONLY);

    static_code := #code {
        point: Point;
        using,only(x) point;
    };

    static_root := C.compiler_get_nodes(static_code);
    static_using := cast(*C.Code_Using) (cast(*C.Code_Block) static_root).statements[1];

    assert(static_using.filter_type == .ONLY);
    init_string_builder(*builder);
    assert(P.print_expression(*builder, static_using));
    printed_static := builder_to_string(*builder);

    assert(printed_static == "using,only(x) point");
    free(printed_static);
    free_buffers(*builder);
    copied_static := C.compiler_get_nodes(C.compiler_get_code(static_using, static_code));

    assert(copied_static.kind == .USING);
    assert((cast(*C.Code_Using) copied_static).filter_type == .ONLY);

    procedure := #code (using,except(.["y"]) item: *Point) #no_context {
        x = 13;
    };

    procedure_root := C.compiler_get_nodes(procedure);

    assert(procedure_root.kind == .PROCEDURE_HEADER);
    header := cast(*C.Code_Procedure_Header) procedure_root;

    assert(header.parameter_usings.count == 1);
    assert(header.parameter_usings[0].filter_type == .EXCEPT);
    copied_procedure := C.compiler_get_code(procedure_root, procedure);
    copied_procedure_root := C.compiler_get_nodes(copied_procedure);

    assert(copied_procedure_root.kind == .PROCEDURE_HEADER);
    copied_header := cast(*C.Code_Procedure_Header) copied_procedure_root;

    assert(copied_header.parameter_usings.count == 1);
    assert(copied_header.parameter_usings[0].filter_type == .EXCEPT);
    assert(copied_header.body_or_null && copied_header.body_or_null.block.statements.count == 1);
}

main :: () {
    print("ok\n");
}
