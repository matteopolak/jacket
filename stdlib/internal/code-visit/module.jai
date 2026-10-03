get_subexpressions :: (node: *C.Code_Node, pointers: *[..] *C.Code_Node) {
    if !node return;

    if node.kind == {
        case .BLOCK;
            n := cast(*C.Code_Block) node;
            for n.statements add(pointers, it);
        case .LITERAL;
            n := cast(*C.Code_Literal) node;
            if n.value_type == .ARRAY && n.array_literal_info {
                add(pointers, n.array_literal_info.element_type);
                add(pointers, n.array_literal_info.alignment);
                for n.array_literal_info.array_members add(pointers, it);
            } else if n.value_type == .STRUCT && n.struct_literal_info {
                add(pointers, n.struct_literal_info.type_expression);
                for n.struct_literal_info.arguments add(pointers, it);
            }
        case .UNARY_OPERATOR;
            add(pointers, (cast(*C.Code_Unary_Operator) node).subexpression);
        case .BINARY_OPERATOR;
            n := cast(*C.Code_Binary_Operator) node;

            add(pointers, n.left);
            add(pointers, n.right);
        case .PROCEDURE_BODY;
            n := cast(*C.Code_Procedure_Body) node;

            add(pointers, n.header);
            add(pointers, n.block);
        case .PROCEDURE_HEADER;
            n := cast(*C.Code_Procedure_Header) node;

            add(pointers, n.constants_block);
            for n.arguments add(pointers, it);
            for n.returns add(pointers, it);
            for n.parameter_usings add(pointers, it);
        // body_or_null is deliberately not a header child.
        case .PROCEDURE_CALL;
            n := cast(*C.Code_Procedure_Call) node;

            add(pointers, n.procedure_expression);
            for n.arguments_unsorted add(pointers, it.expression);
        case .RETURN;
            n := cast(*C.Code_Return) node;
            for n.arguments_unsorted add(pointers, it.expression);
        case .WHILE;
            n := cast(*C.Code_While) node;

            add(pointers, n.condition);
            add(pointers, n.block);
        case .FOR;
            n := cast(*C.Code_For) node;

            add(pointers, n.iteration_expression);
            add(pointers, n.iteration_expression_right);
            add(pointers, n.want_pointer_expression);
            add(pointers, n.want_reverse_expression);
            add(pointers, n.block);
        case .IF;
            n := cast(*C.Code_If) node;

            add(pointers, n.condition);
            add(pointers, n.then_block);
            add(pointers, n.else_block);
        case .CASE;
            n := cast(*C.Code_Case) node;

            add(pointers, n.condition);
            add(pointers, n.then_block);
        case .DECLARATION;
            n := cast(*C.Code_Declaration) node;

            add(pointers, n.type_inst);
            add(pointers, n.expression);
            add(pointers, n.alignment_expression);
        case .COMPOUND_DECLARATION;
            n := cast(*C.Code_Compound_Declaration) node;

            add(pointers, n.comma_separated_assignment);
            add(pointers, n.declaration_properties);
            add(pointers, n.alignment_expression);
        case .TYPE_INSTANTIATION;
            n := cast(*C.Code_Type_Instantiation) node;

            add(pointers, n.type_valued_expression);
            add(pointers, n.must_implement);
            add(pointers, n.pointer_to);
            add(pointers, n.type_directive_target);
            add(pointers, n.array_dimension);
            add(pointers, n.array_element_type);
        case .STRUCT;
            n := cast(*C.Code_Struct) node;

            add(pointers, n.arguments_block);
            add(pointers, n.constants_block);
            add(pointers, n.block);
        case .ENUM;
            n := cast(*C.Code_Enum) node;

            add(pointers, n.internal_type_inst);
            add(pointers, n.block);
        case .CAST;
            n := cast(*C.Code_Cast) node;

            add(pointers, n.target_type);
            add(pointers, n.expression);
        case .USING;
            n := cast(*C.Code_Using) node;

            add(pointers, n.filter_expression);
            add(pointers, n.expression);
        case .DEFER;
            add(pointers, (cast(*C.Code_Defer) node).block);
        case .PUSH_CONTEXT;
            n := cast(*C.Code_Push_Context) node;

            add(pointers, n.to_push);
            add(pointers, n.block);
        case .TYPE_QUERY;
            add(pointers, (cast(*C.Code_Type_Query) node).type_to_query);
        case .EXPRESSION_QUERY;
            add(pointers, (cast(*C.Code_Expression_Query) node).expression_to_query);
        case .DIRECTIVE_CODE;
            add(pointers, (cast(*C.Code_Directive_Code) node).expression);
        case .DIRECTIVE_BYTES;
            add(pointers, (cast(*C.Code_Directive_Bytes) node).expression);
        case .DIRECTIVE_ADD_CONTEXT;
            add(pointers, (cast(*C.Code_Directive_Add_Context) node).expression);
        case .DIRECTIVE_LOCATION;
            add(pointers, (cast(*C.Code_Directive_Location) node).expression);
        case .DIRECTIVE_PROCEDURE_NAME;
            add(pointers, (cast(*C.Code_Directive_Procedure_Name) node).argument);
        case .DIRECTIVE_EXISTS;
            n := cast(*C.Code_Directive_Exists) node;

            add(pointers, n.query_expression);
            add(pointers, n.sync_expression);
        case .DIRECTIVE_INSERT;
            n := cast(*C.Code_Directive_Insert) node;

            add(pointers, n.expression);
            add(pointers, n.scope_redirection);
        case .DIRECTIVE_OVERLAY;
            n := cast(*C.Code_Directive_Overlay) node;

            add(pointers, n.target_expression);
            add(pointers, n.declaration_or_using);
        case .DIRECTIVE_RUN;
            add(pointers, (cast(*C.Code_Directive_Run) node).procedure);
        case .DIRECTIVE_BAKE;
            add(pointers, (cast(*C.Code_Directive_Bake) node).procedure_call);
        case .DIRECTIVE_MODIFY;
            add(pointers, (cast(*C.Code_Directive_Modify) node).block);
        case .DIRECTIVE_MODULE_PARAMETERS;
            n := cast(*C.Code_Directive_Module_Parameters) node;

            add(pointers, n.module_parameters);
            add(pointers, n.program_parameters);
            add(pointers, n.common_code);
        case .COMMA_SEPARATED_ARGUMENTS;
            for (cast(*C.Code_Comma_Separated_Arguments) node).arguments add(pointers, it.node);
        case .EXTRACT;
            add(pointers, (cast(*C.Code_Extract) node).from);
        case .MAKE_VARARGS;
            for (cast(*C.Code_Make_Varargs) node).expressions add(pointers, it);
        case .DIRECTIVE_IMPORT;
            n := cast(*C.Code_Directive_Import) node;

            add(pointers, n.module_parameters_call);
            add(pointers, n.program_parameters_call);
    }
}

get_recursive_nodes_breadth_first :: (node: *C.Code_Node, pointers: *[..] *C.Code_Node) {
    queue: [..] *C.Code_Node;

    add(*queue, node);
    cursor: s64 = 0;
    while cursor < queue.count {
        current := queue[cursor];

        cursor += 1;
        if contains(pointers.*, current) continue;

        add(pointers, current);
        get_subexpressions(current, *queue);
    }

    Basic.array_free(queue);
}

get_recursive_nodes_depth_first :: (node: *C.Code_Node, pointers: *[..] *C.Code_Node) {
    stack: [..] *C.Code_Node;
    children: [..] *C.Code_Node;

    add(*stack, node);

    while stack.count > 0 {
        current := Basic.pop(*stack);
        if contains(pointers.*, current) continue;

        add(pointers, current);
        children.count = 0;
        get_subexpressions(current, *children);
        for i: 0..children.count - 1 add(*stack, children[children.count - i - 1]);
    }

    Basic.array_free(children);
    Basic.array_free(stack);
}

get_values :: (node: *C.Code_Node, values: *[..] V.Dumb_Value) {
    stack: [..] V.Dumb_Value;
    seen: [..] *C.Code_Node;
    children: [..] *C.Code_Node;

    if node Basic.array_add(*stack, .{ node, true });

    while stack.count > 0 {
        event := Basic.pop(*stack);
        if !event.preorder {
            Basic.array_add(values, event);
            continue;
        }

        if contains(seen, event.node) continue;

        add(*seen, event.node);
        Basic.array_add(values, event);
        Basic.array_add(*stack, .{ event.node, false });
        children.count = 0;
        get_subexpressions(event.node, *children);
        for i: 0..children.count - 1 Basic.array_add(*stack, .{ children[children.count - i - 1], true });
    }

    Basic.array_free(children);
    Basic.array_free(stack);
    Basic.array_free(seen);
}

#scope_file
C :: #import "Compiler";
V :: #import "Code_Visit";
Basic :: #import "Basic";

add :: (array: *[..] *C.Code_Node, node: *C.Code_Node) {
    if node Basic.array_add(array, node);
}

contains :: (array: [] *C.Code_Node, node: *C.Code_Node) -> bool {
    for array if it == node return true;
    return false;
}
