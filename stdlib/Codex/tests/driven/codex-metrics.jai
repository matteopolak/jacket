#import "Basic";
#import "Compiler";
C :: #import "Codex";

#run {
    workspace := compiler_create_workspace("Original Codex typed metrics");
    options := get_build_options(workspace);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, workspace);
    compiler_begin_intercept(workspace);
    add_build_file(tprint("%codex-metrics-input.jai", #filepath), workspace);
    inspected := false;
    loops_inspected := false;

    while true {
        message := compiler_wait_for_message();
        if !message break;

        if message.kind == .TYPECHECKED {
            for tc: (cast(*Message_Typechecked) message).procedure_bodies {
                if tc.expression.header && tc.expression.header.name == "original_loops" {
                    metrics := C.collect_metrics(tc);
                    assert(metrics.maximum_loop_depth == 2 && metrics.maximum_if_depth == 0 && metrics.if_points == 0);
                    assert(metrics.reassignments == 1 && metrics.total_calls == 0);
                    loops_inspected = true;
                }

                if !tc.expression.header || tc.expression.header.name != "original_linear" continue;

                metrics := C.collect_metrics(tc);

                assert(metrics.total_calls == 2 && metrics.same_file_calls == 2 && metrics.same_module_calls == 2);
                assert(metrics.nonconstant_calls == 0 && metrics.allocations == 0 && metrics.frees == 0);
                // Two variable references and two global procedure identifiers.
                assert(metrics.reassignments == 2 && metrics.global_reads == 4 && metrics.global_writes == 1);
                assert(metrics.if_points == 0 && metrics.maximum_if_depth == 0 && metrics.maximum_loop_depth == 0);
                inspected = true;
            }
        }

        if message.kind == .COMPLETE {
            assert((cast(*Message_Complete) message).error_code == .NONE);
            break;
        }
    }

    compiler_end_intercept(workspace);
    assert(inspected && loops_inspected);
}

main :: () {
    assert(size_of(C.Metrics) == 48 && align_of(C.Metrics) == 1);
    // Original public typed-node model exercises collector code in native runtime too.
    declaration: Code_Declaration;
    declaration.flags = .IS_GLOBAL;
    identifier: Code_Ident;
    identifier.kind = .IDENT;
    identifier.resolved_declaration = *declaration;
    assignment: Code_Binary_Operator;
    assignment.kind = .BINARY_OPERATOR;
    assignment.operator_type = cast(Operator_Type) #char "=";
    assignment.left = cast(*Code_Node) *identifier;
    unresolved: Code_Procedure_Call;
    unresolved.kind = .PROCEDURE_CALL;
    statements := (*Code_Node).[cast(*Code_Node) *assignment, cast(*Code_Node) *unresolved];
    block: Code_Block;
    block.kind = .BLOCK;
    block.statements = .{ statements.count, statements.data };
    body: Code_Procedure_Body;
    body.kind = .PROCEDURE_BODY;
    body.block = *block;
    tc: Typechecked(Code_Procedure_Body);
    tc.expression = *body;
    metrics := C.collect_metrics(tc);

    assert(metrics.total_calls == 1 && metrics.nonconstant_calls == 1);
    assert(metrics.reassignments == 1 && metrics.global_reads == 1 && metrics.global_writes == 1);
    assert(metrics.same_file_calls == 0 && metrics.same_module_calls == 0);
    // All reviewed assignment operators, excluding comparisons and arithmetic.
    operators := Operator_Type.[.PLUS_ASSIGN, .MINUS_ASSIGN, .TIMES_ASSIGN, .DIV_ASSIGN, .MOD_ASSIGN,
        .SHIFT_LEFT_ASSIGN, .SHIFT_RIGHT_ASSIGN, .ROTATE_LEFT_ASSIGN, .ROTATE_RIGHT_ASSIGN,
        .BITWISE_AND_ASSIGN, .BITWISE_OR_ASSIGN, .BITWISE_XOR_ASSIGN, .LOGICAL_AND_ASSIGN, .LOGICAL_OR_ASSIGN];
    for operators {
        assignment.operator_type = it;
        measured := C.collect_metrics(tc);
        assert(measured.reassignments == 1 && measured.global_writes == 1);
    }

    assignment.operator_type = .IS_EQUAL;
    metrics = C.collect_metrics(tc);
    assert(metrics.reassignments == 0 && metrics.global_writes == 0 && metrics.global_reads == 1);

    source_module, other_module: Message_Import;
    source_file, other_file: Message_File;
    source_file.enclosing_import = *source_module;
    other_file.enclosing_import = *source_module;
    body.enclosing_load = *source_file;
    header: Code_Procedure_Header;
    header.kind = .PROCEDURE_HEADER;
    header.enclosing_load = *source_file;
    unresolved.resolved_procedure_expression = cast(*Code_Node) *header;
    metrics = C.collect_metrics(tc);
    assert(metrics.total_calls == 1 && metrics.nonconstant_calls == 0);
    assert(metrics.same_file_calls == 1 && metrics.same_module_calls == 1);
    header.enclosing_load = *other_file;
    metrics = C.collect_metrics(tc);
    assert(metrics.same_file_calls == 0 && metrics.same_module_calls == 1);
    other_file.enclosing_import = *other_module;
    metrics = C.collect_metrics(tc);
    assert(metrics.same_file_calls == 0 && metrics.same_module_calls == 0);
    call_declaration: Code_Declaration;
    call_declaration.expression = cast(*Code_Node) *header;
    callee_ident: Code_Ident;
    callee_ident.kind = .IDENT;
    callee_ident.resolved_declaration = *call_declaration;
    overload: Code_Resolved_Overload;
    overload.kind = .RESOLVED_OVERLOAD;
    overload.result = cast(*Code_Node) *callee_ident;
    cast_node: Code_Cast;
    cast_node.kind = .CAST;
    cast_node.expression = cast(*Code_Node) *overload;
    unresolved.resolved_procedure_expression = cast(*Code_Node) *cast_node;
    metrics = C.collect_metrics(tc);
    assert(metrics.total_calls == 1 && metrics.nonconstant_calls == 0);
    structure: Code_Struct;
    structure.kind = .STRUCT;
    unresolved.resolved_procedure_expression = cast(*Code_Node) *structure;
    metrics = C.collect_metrics(tc);
    assert(metrics.total_calls == 0 && metrics.nonconstant_calls == 0);

    inner: Code_While;
    inner.kind = .WHILE;
    inner_nodes := (*Code_Node).[cast(*Code_Node) *inner];
    inner_block: Code_Block;
    inner_block.kind = .BLOCK;
    inner_block.statements = .{ inner_nodes.count, inner_nodes.data };
    outer: Code_For;
    outer.kind = .FOR;
    outer.block = *inner_block;
    statements[1] = cast(*Code_Node) *outer;
    metrics = C.collect_metrics(tc);
    assert(metrics.maximum_loop_depth == 2 && metrics.if_points == 0 && metrics.maximum_if_depth == 0);
    print("ok\n");
}
