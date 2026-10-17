#import "Basic";
C :: #import "Compiler";
P :: #import "Program_Print";

Pair :: struct {
    x: s64;
    y: s64;
}

rename_member :: (names: [] string) #no_context {
    for *names if it.* == "x" it.* = "axis";
}

copy_procedure :: (source: Code, change_filter: bool = false) -> Code {
    root := C.compiler_get_nodes(source);
    header := cast(*C.Code_Procedure_Header) root;

    assert(header.parameter_usings.count == 1);
    using_node := header.parameter_usings[0];

    if change_filter {
        // Public graph edits take precedence over retained private syntax.
        assert(using_node.filter_type == .ONLY);
        using_node.filter_type = .EXCEPT;
    }

    builder: String_Builder;

    init_string_builder(*builder);
    assert(P.print_expression(*builder, root));
    printed := builder_to_string(*builder);
    count: s64;

    for i: 0..printed.count - 5 {
        matches := true;
        for c, j: "using" if printed[i + j] != c matches = false;
        if matches count += 1;
    }

    assert(count == 1);
    free_buffers(*builder);
    copied := C.compiler_get_code(root, source);
    copied_root := cast(*C.Code_Procedure_Header) C.compiler_get_nodes(copied);

    assert(copied_root.parameter_usings.count == 1);
    assert(copied_root.parameter_usings[0].filter_type == using_node.filter_type);

    return copied;
}

only_static :: #insert #run copy_procedure(#code (using,only(x) point: *Pair) -> s64 {
    return x;
});

except_computed :: #insert #run copy_procedure(#code (using,except(.["y"]) point: *Pair) -> s64 {
    return x;
});

bare :: #insert #run copy_procedure(#code (using point: *Pair) -> s64 {
    return x + y;
});

edited :: #insert #run copy_procedure(change_filter = true, source = #code (using,only(.["y"]) point: *Pair) -> s64 {
    return x;
});

edited_static :: #insert #run copy_procedure(change_filter = true, source = #code (using,only(y) point: *Pair) -> s64 {
    return x;
});

mapped :: #insert #run copy_procedure(#code (using,map(rename_member) point: *Pair) -> s64 {
    axis += 1;
    return axis + y;
});

main :: () {
    point: Pair = .{ 13, 29 };

    assert(only_static(*point) == 13);
    assert(except_computed(*point) == 13);
    assert(bare(*point) == 42);
    assert(edited(*point) == 13);
    assert(edited_static(*point) == 13);
    assert(mapped(*point) == 43 && point.x == 14);
    print("using parameters ok\n");
}
