quoted :: (builder: *Basic.String_Builder, s: string) {
    Basic.append(builder, "\"");
    digits :: "0123456789abcdef";

    for c: s {
        if c == #char "\"" Basic.append(builder, "\\\"");
        else if c == #char "\\" Basic.append(builder, "\\\\");
        else if c == #char "\n" Basic.append(builder, "\\n");
        else if c == #char "\r" Basic.append(builder, "\\r");
        else if c == #char "\t" Basic.append(builder, "\\t");
        else if c < 32 || c >= 127 {
            Basic.append(builder, "\\x");
            Basic.append(builder, digits[c >> 4]);
            Basic.append(builder, digits[c & 15]);
        } else Basic.append(builder, c);
    }

    Basic.append(builder, "\"");
}

#load "numbers.jai";
#load "attributes.jai";
#load "assembly.jai";

using_prefix :: (builder: *Basic.String_Builder, node: *C.Code_Using, bodies: bool, depth: s64) -> bool {
    if node.no_parameters return false;

    Basic.append(builder, "using");

    if node.filter_type != .NONE {
        if node.filter_type == .ONLY Basic.append(builder, ",only(");
        else if node.filter_type == .EXCEPT Basic.append(builder, ",except(");
        else if node.filter_type == .MAP Basic.append(builder, ",map(");
        else return false;

        if node.filter_expression {
            if !expression(builder, node.filter_expression, true, bodies, depth + 1) return false;
        } else {
            // Owned static name syntax is separate from the editable public kind.
            original_filter := syntax_int(node, "__syntax_static_filter");
            if !original_filter return false;
            // A name list and a mapping have different argument grammars.
            if (original_filter == 3) != (node.filter_type == .MAP) return false;

            Basic.append(builder, syntax_string(node, "__syntax_filter_arguments"));
        }

        Basic.append(builder, ")");
    }

    Basic.append(builder, " ");

    return true;
}

// A named initializer is a member/index path, not a parenthesized value expression.
literal_target :: (builder: *Basic.String_Builder, node: *C.Code_Node, bodies: bool, depth: s64) -> bool {
    if !node || depth > 512 return false;
    if node.kind == .IDENT return expression(builder, node, true, bodies, depth + 1);
    if node.kind != .BINARY_OPERATOR return false;

    n := cast(*C.Code_Binary_Operator) node;
    if cast(s32) n.operator_type == #char "." {
        if !n.right || n.right.kind != .IDENT return false;
        if !literal_target(builder, n.left, bodies, depth + 1) return false;
        Basic.append(builder, ".");
        return literal_target(builder, n.right, bodies, depth + 1);
    }

    if cast(s32) n.operator_type == 500 {
        if !literal_target(builder, n.left, bodies, depth + 1) return false;

        Basic.append(builder, "[");
        if !expression(builder, n.right, true, bodies, depth + 1) return false;

        Basic.append(builder, "]");

        return true;
    }

    return false;
}

expression :: (builder: *Basic.String_Builder, node: *C.Code_Node, skip: bool, bodies: bool, depth: s64) -> bool {
    if !expression_inner(builder, node, skip, bodies, depth) return false;

    notes := syntax_child(node, "__syntax_statement_notes");
    if notes {
        if notes.kind != .COMMA_SEPARATED_ARGUMENTS return false;

        for arg: (cast(*C.Code_Comma_Separated_Arguments) notes).arguments {
            if !arg.node || arg.node.kind != .NOTE return false;
            Basic.append(builder, " @");
            Basic.append(builder, (cast(*C.Code_Note) arg.node).text);
        }
    }

    return true;
}

expression_inner :: (builder: *Basic.String_Builder, node: *C.Code_Node, skip: bool, bodies: bool, depth: s64) -> bool {
    if !node || depth > 512 return false;
    if syntax_int(node, "__syntax_incomplete") return false;

    original_kind := syntax_int(node, "__syntax_original_kind");
    if original_kind && original_kind != cast(int) node.kind return false;

    next := depth + 1;

    if node.kind == {
        case .IDENT;
            n := cast(*C.Code_Ident) node;
            if n.flags & .HAS_SCOPE_MODIFIER Basic.append(builder, "`");

            if n.flags & .DEFINES_POLYMORPH_VARIABLE {
                Basic.append(builder, "$");
                if syntax_int(node, "__syntax_poly_baked") Basic.append(builder, "$");
            }

            Basic.append(builder, n.name);
            restriction := syntax_child(node, "__syntax_restriction");
            if restriction {
                Basic.append(builder, "/");
                if syntax_int(node, "__syntax_interface") Basic.append(builder, "interface ");
                if !expression(builder, restriction, false, bodies, next) return false;
            }

            return true;
        case .LITERAL;
            n := cast(*C.Code_Literal) node;
            if n.value_type == .NUMBER {
                if n.value_flags & .FLOAT return floating(builder, n);

                if n.value_flags & .HEX || n.value_flags & .BINARY {
                    magnitude := n._u64;

                    if n.value_flags & .MINUS_SIGN {
                        Basic.append(builder, "-");
                        if magnitude != 0 magnitude = (~magnitude) + 1;
                    }

                    base := 16;

                    if n.value_flags & .BINARY {
                        Basic.append(builder, "0b");
                        base = 2;
                    } else Basic.append(builder, "0x");

                    return Basic.print_to_builder(builder, "%", Basic.formatInt(magnitude, base = base));
                }

                if n.value_flags & .MINUS_SIGN return Basic.print_to_builder(builder, "%", n._s64);

                return Basic.print_to_builder(builder, "%", n._u64);
            }

            if n.value_type == .STRING {
                quoted(builder, n._string);
                return true;
            }

            if n.value_type == .BOOLEAN {
                if n._s64 Basic.append(builder, "true");
                else Basic.append(builder, "false");
                return true;
            }

            if n.value_type == .POINTER && n.pointer_literal_info && n.pointer_literal_info.pointer_literal_type == .CONSTANT_VALUE && !n.pointer_literal_info.data_pointer {
                Basic.append(builder, "null");
                return true;
            }

            if n.value_type == .ARRAY && n.array_literal_info {
                if n.array_literal_info.element_type && !expression(builder, n.array_literal_info.element_type, true, bodies, next) return false;

                Basic.append(builder, ".[");

                for member, i: n.array_literal_info.array_members {
                    if i > 0 Basic.append(builder, ", ");
                    if !expression(builder, member, true, bodies, next) return false;
                }

                Basic.append(builder, "]");

                return true;
            }

            if n.value_type == .STRUCT && n.struct_literal_info {
                if n.struct_literal_info.type_expression && !expression(builder, n.struct_literal_info.type_expression, true, bodies, next) return false;

                Basic.append(builder, ".{");

                for member, i: n.struct_literal_info.arguments {
                    if i > 0 Basic.append(builder, ", ");

                    if member && member.kind == .BINARY_OPERATOR && cast(s32) (cast(*C.Code_Binary_Operator) member).operator_type == #char "=" {
                        assignment := cast(*C.Code_Binary_Operator) member;
                        if !literal_target(builder, assignment.left, bodies, next) return false;
                        Basic.append(builder, " = ");
                        if !expression(builder, assignment.right, true, bodies, next) return false;
                    } else if !expression(builder, member, true, bodies, next) return false;
                }

                Basic.append(builder, "}");

                return true;
            }

            if n.value_type == .UNINITIALIZED {
                Basic.append(builder, "---");
                return true;
            }

            return false;
        case .BINARY_OPERATOR;
            n := cast(*C.Code_Binary_Operator) node;
            if cast(s32) n.operator_type == 500 {
                if !expression(builder, n.left, false, bodies, next) return false;

                Basic.append(builder, "[");
                if !expression(builder, n.right, true, bodies, next) return false;

                Basic.append(builder, "]");

                return true;
            }

            op := C.operator_to_string(n.operator_type);

            if op.count == 0 return false;
            if !skip Basic.append(builder, "(");

            // A null left operand is contextual dot syntax, e.g. take(.FIRST).
            // Other binary operators always require both operands.
            if n.left {
                if !expression(builder, n.left, false, bodies, next) return false;
            } else if cast(s32) n.operator_type != #char "." return false;

            if cast(s32) n.operator_type == #char "." Basic.append(builder, ".");
            else {
                Basic.append(builder, " ");
                Basic.append(builder, op);
                Basic.append(builder, " ");
            }

            if !expression(builder, n.right, false, bodies, next) return false;
            if !skip Basic.append(builder, ")");

            return true;
        case .UNARY_OPERATOR;
            n := cast(*C.Code_Unary_Operator) node;
            if cast(s32) n.operator_type == 169 {
                if !expression(builder, n.subexpression, false, bodies, next) return false;
                Basic.append(builder, ".*");
                return true;
            }

            op := C.operator_to_string(n.operator_type);

            if op.count == 0 return false;
            if !skip Basic.append(builder, "(");

            Basic.append(builder, op);
            if !expression(builder, n.subexpression, false, bodies, next) return false;
            if !skip Basic.append(builder, ")");

            return true;
        case .PROCEDURE_CALL;
            n := cast(*C.Code_Procedure_Call) node;
            hint := syntax_int(node, "__syntax_call_hint");

            if hint == 1 Basic.append(builder, "inline ");
            if hint == 2 Basic.append(builder, "no_inline ");
            if !expression(builder, n.procedure_expression, false, bodies, next) return false;

            Basic.append(builder, "(");

            for arg, i: n.arguments_unsorted {
                if i > 0 Basic.append(builder, ", ");

                if arg.name {
                    Basic.append(builder, arg.name.name);
                    Basic.append(builder, " = ");
                }

                if arg.expression && arg.expression.node_flags & .EXPRESSION_IS_SPREAD Basic.append(builder, "..");
                if !expression(builder, arg.expression, true, bodies, next) return false;
            }

            Basic.append(builder, ")");

            return true;
        case .DECLARATION;
            return declaration(builder, cast(*C.Code_Declaration) node, true, null, true, bodies, next);
        case .COMPOUND_DECLARATION;
            n := cast(*C.Code_Compound_Declaration) node;

            return declaration(builder, n.declaration_properties, true, n, true, bodies, next);
        case .COMMA_SEPARATED_ARGUMENTS;
            n := cast(*C.Code_Comma_Separated_Arguments) node;
            for arg, i: n.arguments {
                if i Basic.append(builder, ", ");
                if !expression(builder, arg.node, true, bodies, next) return false;
            }

            return true;
        case .DIRECTIVE_EXISTS;
            n := cast(*C.Code_Directive_Exists) node;
            if n.sync_expression return false;

            Basic.append(builder, "#exists(");
            if !expression(builder, n.query_expression, true, bodies, next) return false;

            Basic.append(builder, ")");

            return true;
        case .BLOCK;
            n := cast(*C.Code_Block) node;
            if n.block_flags & .NO_ARRAY_BOUNDS_CHECK Basic.append(builder, "#no_abc ");
            if n.block_flags & .NO_ARITHMETIC_OVERFLOW_CHECK Basic.append(builder, "#no_aoc ");

            Basic.append(builder, "{\n");

            for statement: n.statements {
                Basic.append(builder, "    ");
                if !expression(builder, statement, true, bodies, next) return false;
                Basic.append(builder, ";\n");
            }

            Basic.append(builder, "}");

            return true;
        case .PROCEDURE_HEADER;
            n := cast(*C.Code_Procedure_Header) node;
            block: *C.Code_Block;

            if n.body_or_null block = n.body_or_null.block;

            return procedure(builder, n, block, bodies, next);
        case .DIRECTIVE_RUN;
            n := cast(*C.Code_Directive_Run) node;
            body := syntax_child(node, "__syntax_run_body");

            if !body && !n.procedure return false;

            if n.flags & .ASSERTION Basic.append(builder, "#assert(");
            else {
                Basic.append(builder, "#run");
                Basic.append(builder, syntax_string(node, "__syntax_flags"));
                Basic.append(builder, " ");
            }

            if n.procedure {
                Basic.append(builder, "(");
                if !expression(builder, n.procedure, true, bodies, next) return false;
                Basic.append(builder, ")()");
            } else if !expression(builder, body, true, bodies, next) return false;

            if n.flags & .ASSERTION {
                message := syntax_child(node, "__syntax_assert_message");
                args := syntax_child(node, "__syntax_assert_args");

                if message {
                    Basic.append(builder, ", ");
                    if !expression(builder, message, true, bodies, next) return false;
                }

                if args {
                    Basic.append(builder, ", ");
                    if !expression(builder, args, true, bodies, next) return false;
                }

                Basic.append(builder, ")");
            }

            return true;
        case .DIRECTIVE_MODIFY;
            n := cast(*C.Code_Directive_Modify) node;

            Basic.append(builder, "#modify ");
            return expression(builder, n.block, true, bodies, next);
        case .ENUM;
            n := cast(*C.Code_Enum) node;
            if n.is_flags Basic.append(builder, "enum_flags ");
            else Basic.append(builder, "enum ");
            if n.internal_type_inst && !expression(builder, n.internal_type_inst, true, bodies, next) return false;
            if n.marked_as_complete Basic.append(builder, " #complete ");
            if n.marked_as_specified Basic.append(builder, " #specified ");
            if !n.block return false;

            if !enum_block(builder, n.block, bodies, next) return false;

            for note: n.notes {
                Basic.append(builder, " @");
                Basic.append(builder, note.text);
            }

            return true;
        case .STRUCT;
            n := cast(*C.Code_Struct) node;
            if n.textual_flags & .UNION Basic.append(builder, "union ");
            else Basic.append(builder, "struct ");

            if n.textual_flags & .UNION_IS_TAGGED {
                tag := syntax_child(node, "__syntax_union_tag");
                if !tag || !expression(builder, tag, true, bodies, next) return false;
                Basic.append(builder, " ");
            }

            if n.arguments_block {
                Basic.append(builder, "(");

                for argument, i: n.arguments_block.statements {
                    if i Basic.append(builder, ", ");
                    if !expression(builder, argument, true, bodies, next) return false;
                }

                Basic.append(builder, ") ");
            }

            if n.textual_flags & .NO_PADDING Basic.append(builder, "#no_padding ");
            if n.textual_flags & .TYPE_INFO_NONE Basic.append(builder, "#type_info_none ");
            if n.textual_flags & .TYPE_INFO_NO_SIZE_COMPLAINT Basic.append(builder, "#type_info_no_size_complaint ");
            if n.textual_flags & .TYPE_INFO_PROCEDURES_ARE_VOID_POINTERS Basic.append(builder, "#type_info_procedures_are_void_pointers ");

            alignment := syntax_child(node, "__syntax_alignment");
            if n.alignment > 0 Basic.print_to_builder(builder, "#align % ", n.alignment);
            else if alignment {
                Basic.append(builder, "#align ");
                if !expression(builder, alignment, false, bodies, next) return false;
                Basic.append(builder, " ");
            }

            if n.modify_directives.count > 1 return false;

            for modify: n.modify_directives {
                if !expression(builder, modify, true, bodies, next) return false;
                Basic.append(builder, " ");
            }

            if !expression(builder, n.block, true, bodies, next) return false;

            for note: n.notes {
                Basic.append(builder, " @");
                Basic.append(builder, note.text);
            }

            return true;
        case .PROCEDURE_BODY;
            n := cast(*C.Code_Procedure_Body) node;

            return procedure(builder, n.header, n.block, bodies, next);
        case .RETURN;
            n := cast(*C.Code_Return) node;
            if n.return_flags & .IS_BACKTICKED Basic.append(builder, "`");

            Basic.append(builder, "return");

            for arg, i: n.arguments_unsorted {
                if i == 0 Basic.append(builder, " ");
                else Basic.append(builder, ", ");
                if arg.expression && arg.expression.node_flags & .EXPRESSION_IS_SPREAD Basic.append(builder, "..");
                if !expression(builder, arg.expression, true, bodies, next) return false;
            }

            return true;
        case .CONTEXT;
            Basic.append(builder, "context");
            return true;
        case .TYPE_INSTANTIATION;
            n := cast(*C.Code_Type_Instantiation) node;
            if n.inst_flags & .VARARGS Basic.append(builder, "..");
            if n.inst_flags & .TYPE_DIRECTIVE Basic.append(builder, "#type ");
            if n.inst_flags & .TYPE_DIRECTIVE_DISTINCT Basic.append(builder, "#type,distinct ");
            if n.inst_flags & .TYPE_DIRECTIVE_ISA Basic.append(builder, "#type,isa ");

            if n.pointer_to {
                Basic.append(builder, "*");
                return expression(builder, n.pointer_to, true, bodies, next);
            }

            if n.array_element_type {
                Basic.append(builder, "[");
                if n.inst_flags & .RESIZABLE Basic.append(builder, "..");
                else if n.array_dimension && !expression(builder, n.array_dimension, true, bodies, next) return false;

                Basic.append(builder, "] ");

                return expression(builder, n.array_element_type, true, bodies, next);
            }

            if n.type_directive_target return expression(builder, n.type_directive_target, true, bodies, next);
            if n.type_valued_expression return expression(builder, n.type_valued_expression, true, bodies, next);

            return false;
        case .CAST;
            n := cast(*C.Code_Cast) node;

            if !skip Basic.append(builder, "(");

            if n.cast_flags & .IS_AUTO Basic.append(builder, "xx ");
            else {
                Basic.append(builder, "cast");
                if n.cast_flags & .NO_BOUNDS_CHECK Basic.append(builder, ",no_check");
                if n.cast_flags & .TRUNCATE Basic.append(builder, ",truncate");
                if n.cast_flags & .FORCE Basic.append(builder, ",force");

                Basic.append(builder, "(");
                if !expression(builder, n.target_type, true, bodies, next) return false;

                Basic.append(builder, ") ");
            }

            if !expression(builder, n.expression, false, bodies, next) return false;
            if !skip Basic.append(builder, ")");

            return true;
        case .DEFER;
            n := cast(*C.Code_Defer) node;
            if n.is_backticked Basic.append(builder, "`");

            Basic.append(builder, "defer ");

            return expression(builder, n.block, true, bodies, next);
        case .IF;
            n := cast(*C.Code_If) node;
            if n.if_flags & .IS_IFX {
                if !skip Basic.append(builder, "(");
                if n.if_flags & .IS_STATIC Basic.append(builder, "#ifx ");
                else Basic.append(builder, "ifx ");
                if !expression(builder, n.condition, false, bodies, next) return false;

                if n.then_block {
                    Basic.append(builder, " then ");
                    if !value_block(builder, n.then_block, bodies, next) return false;
                }

                if n.else_block {
                    Basic.append(builder, " else ");
                    if !value_block(builder, n.else_block, bodies, next) return false;
                }

                if !skip Basic.append(builder, ")");

                return true;
            }

            if n.if_flags & .IS_STATIC Basic.append(builder, "#if ");
            else Basic.append(builder, "if ");
            if !expression(builder, n.condition, true, bodies, next) return false;
            if n.if_flags & .IS_SWITCH_STATEMENT Basic.append(builder, " ==");
            if n.if_flags & .MARKED_AS_COMPLETE Basic.append(builder, " #complete");

            Basic.append(builder, " ");
            if !expression(builder, n.then_block, true, bodies, next) return false;

            if n.else_block {
                Basic.append(builder, " else ");
                return expression(builder, n.else_block, true, bodies, next);
            }

            return true;
        case .CASE;
            n := cast(*C.Code_Case) node;

            Basic.append(builder, "case");

            if n.condition {
                Basic.append(builder, " ");
                if !expression(builder, n.condition, true, bodies, next) return false;
            }

            Basic.append(builder, ";\n");
            if n.then_block && !statements(builder, n.then_block, bodies, next) return false;
            if n.marked_as_fallthrough Basic.append(builder, "#through;\n");

            return true;
        case .FOR;
            n := cast(*C.Code_For) node;
            if n.macro_expansion_procedure_call || n.want_replacement_for_expansion return false;

            Basic.append(builder, "for ");
            Basic.append(builder, syntax_string(node, "__syntax_flags"));
            iterator := syntax_string(node, "__syntax_iterator");
            if iterator.count {
                Basic.append(builder, ":");
                Basic.append(builder, iterator);
                Basic.append(builder, " ");
            }

            if n.want_pointer_expression {
                Basic.append(builder, "*=");
                if !expression(builder, n.want_pointer_expression, false, bodies, next) return false;
                Basic.append(builder, " ");
            } else if n.for_flags & .POINTER Basic.append(builder, "* ");

            if n.want_reverse_expression {
                Basic.append(builder, "<=");
                if !expression(builder, n.want_reverse_expression, false, bodies, next) return false;
                Basic.append(builder, " ");
            } else if n.for_flags & .REVERSE Basic.append(builder, "< ");

            if n.ident_it {
                if !expression(builder, n.ident_it, true, bodies, next) return false;

                if n.ident_it_index {
                    Basic.append(builder, ", ");
                    if !expression(builder, n.ident_it_index, true, bodies, next) return false;
                }

                Basic.append(builder, ": ");
            } else if n.ident_it_index return false;

            if !expression(builder, n.iteration_expression, false, bodies, next) return false;

            if n.iteration_expression_right {
                Basic.append(builder, " .. ");
                if !expression(builder, n.iteration_expression_right, false, bodies, next) return false;
            }

            Basic.append(builder, " ");

            return expression(builder, n.block, true, bodies, next);
        case .WHILE;
            n := cast(*C.Code_While) node;

            Basic.append(builder, "while ");
            label := syntax_string(node, "__syntax_label");
            if label.count {
                if !syntax_int(node, "__syntax_bind_label") Basic.append(builder, ":");
                if syntax_int(node, "__syntax_backtick_label") Basic.append(builder, "`");

                Basic.append(builder, label);
                if syntax_int(node, "__syntax_bind_label") Basic.append(builder, " :=");

                Basic.append(builder, " ");
            }

            if !expression(builder, n.condition, true, bodies, next) return false;

            Basic.append(builder, " ");

            return expression(builder, n.block, true, bodies, next);
        case .DIRECTIVE_CODE;
            n := cast(*C.Code_Directive_Code) node;
            if n.code_flags & .NULL {
                Basic.append(builder, "#code,null");
                return true;
            }

            Basic.append(builder, "#code ");

            return expression(builder, n.expression, true, bodies, next);
        case .DIRECTIVE_BYTES;
            Basic.append(builder, "#bytes ");
            return expression(builder, (cast(*C.Code_Directive_Bytes) node).expression, false, bodies, next);
        case .DIRECTIVE_THIS;
            Basic.append(builder, "#this");
            return true;
        case .DIRECTIVE_COMPILE_TIME;
            Basic.append(builder, "#compile_time");
            return true;
        case .DIRECTIVE_LOCATION;
            n := cast(*C.Code_Directive_Location) node;
            if syntax_int(node, "__syntax_caller_code") Basic.append(builder, "#caller_code");
            else if n.is_caller_location Basic.append(builder, "#caller_location");
            else {
                Basic.append(builder, "#location");

                if n.expression {
                    Basic.append(builder, "(");
                    if !expression(builder, n.expression, true, bodies, next) return false;
                    Basic.append(builder, ")");
                }
            }

            return true;
        case .DIRECTIVE_PROCEDURE_NAME;
            n := cast(*C.Code_Directive_Procedure_Name) node;

            Basic.append(builder, "#procedure_name");

            if n.argument {
                Basic.append(builder, "(");
                if !expression(builder, n.argument, true, bodies, next) return false;
                Basic.append(builder, ")");
            }

            return true;
        case .DIRECTIVE_ADD_CONTEXT;
            Basic.append(builder, "#add_context ");
            return expression(builder, (cast(*C.Code_Directive_Add_Context) node).expression, true, bodies, next);
        case .DIRECTIVE_PLACE;
            Basic.append(builder, "#place ");
            return expression(builder, syntax_child(node, "__syntax_operand"), true, bodies, next);
        case .DIRECTIVE_OVERLAY;
            n := cast(*C.Code_Directive_Overlay) node;

            Basic.append(builder, "#overlay(");
            if !expression(builder, n.target_expression, true, bodies, next) return false;

            Basic.append(builder, ")");

            if n.declaration_or_using {
                Basic.append(builder, " ");
                return expression(builder, n.declaration_or_using, true, bodies, next);
            }

            return true;
        case .DIRECTIVE_BAKE;
            n := cast(*C.Code_Directive_Bake) node;
            if n.bake_type == .CONSTANTS_BAKE Basic.append(builder, "#bake_constants ");
            else if n.bake_type == .PARAMETER_VALUE_BAKE Basic.append(builder, "#bake_arguments ");
            else return false;

            return expression(builder, n.procedure_call, true, bodies, next);
        case .DIRECTIVE_MODULE_PARAMETERS;
            n := cast(*C.Code_Directive_Module_Parameters) node;

            Basic.append(builder, "#module_parameters ");
            if !expression(builder, n.module_parameters, true, bodies, next) return false;

            if n.program_parameters {
                Basic.append(builder, " ");
                if !expression(builder, n.program_parameters, true, bodies, next) return false;
            }

            if n.common_code {
                Basic.append(builder, " ");
                return expression(builder, n.common_code, true, bodies, next);
            }

            return true;
        case .PLACEHOLDER;
            name := syntax_string(node, "__syntax_directive_name");
            if name.count == 0 return false;

            Basic.append(builder, "#");
            Basic.append(builder, name);
            Basic.append(builder, syntax_string(node, "__syntax_flags"));
            args := syntax_child(node, "__syntax_directive_args");
            if args {
                Basic.append(builder, " ");
                return expression(builder, args, true, bodies, next);
            }

            return true;
        case .ASM;
            return assembly(builder, node, next);
        case .DIRECTIVE_INSERT;
            n := cast(*C.Code_Directive_Insert) node;

            Basic.append(builder, "#insert");
            Basic.append(builder, syntax_string(node, "__syntax_flags"));

            if n.scope_redirection {
                Basic.append(builder, ",scope(");
                if !expression(builder, n.scope_redirection, true, bodies, next) return false;
                Basic.append(builder, ") ");
            }

            Basic.append(builder, " ");

            if n.break_replacement || n.continue_replacement || n.remove_replacement {
                Basic.append(builder, "(");
                separator := false;

                if n.break_replacement {
                    Basic.append(builder, "break = ");
                    if !expression(builder, n.break_replacement, true, bodies, next) return false;
                    separator = true;
                }

                if n.continue_replacement {
                    if separator Basic.append(builder, ", ");
                    Basic.append(builder, "continue = ");
                    if !expression(builder, n.continue_replacement, true, bodies, next) return false;
                    separator = true;
                }

                if n.remove_replacement {
                    if separator Basic.append(builder, ", ");
                    Basic.append(builder, "remove = ");
                    if !expression(builder, n.remove_replacement, true, bodies, next) return false;
                }

                Basic.append(builder, ") ");
            }

            return expression(builder, n.expression, true, bodies, next);
        case .PUSH_CONTEXT;
            n := cast(*C.Code_Push_Context) node;
            if n.push_context_flags & .IS_BACKTICKED Basic.append(builder, "`");

            Basic.append(builder, "push_context");
            if n.push_context_flags & .DEFER_POP Basic.append(builder, ",defer_pop");

            Basic.append(builder, " ");
            bare := syntax_int(node, "__syntax_context_bare") && n.to_push == syntax_child(node, "__syntax_original_context");
            if n.to_push && !bare && !expression(builder, n.to_push, false, bodies, next) return false;

            if n.block {
                Basic.append(builder, " ");
                if !expression(builder, n.block, true, bodies, next) return false;
            }

            return true;
        case .DIRECTIVE_LOAD;
            Basic.append(builder, "#load ");
            quoted(builder, (cast(*C.Code_Directive_Load) node).short_name);
            return true;
        case .DIRECTIVE_IMPORT;
            n := cast(*C.Code_Directive_Import) node;

            if syntax_int(node, "__syntax_using") {
                if !private_using(builder, node, bodies, next) return false;
            }

            alias := syntax_string(node, "__syntax_alias");
            if alias.count {
                Basic.append(builder, alias);
                Basic.append(builder, " :: ");
            }

            Basic.append(builder, "#import");
            kind := syntax_int(node, "__syntax_import_kind");
            if kind == 1 Basic.append(builder, ",file");
            if kind == 2 Basic.append(builder, ",dir");
            if kind == 3 Basic.append(builder, ",string");

            Basic.append(builder, syntax_string(node, "__syntax_flags"));
            Basic.append(builder, " ");
            quoted(builder, n.name);

            calls: [2] *C.Code_Procedure_Call = .[n.module_parameters_call, n.program_parameters_call];

            if n.program_parameters_call && !n.module_parameters_call Basic.append(builder, "()");

            for call: calls {
                if !call continue;

                Basic.append(builder, "(");

                for arg, i: call.arguments_unsorted {
                    if i Basic.append(builder, ", ");

                    if arg.name {
                        Basic.append(builder, arg.name.name);
                        Basic.append(builder, " = ");
                    }

                    if !expression(builder, arg.expression, true, bodies, next) return false;
                }

                Basic.append(builder, ")");
            }

            return true;
        case .DIRECTIVE_THROUGH;
            Basic.append(builder, "#through");
            return true;
        case .DIRECTIVE_SCOPE;
            n := cast(*C.Code_Directive_Scope) node;
            if n.scope_type == .EXPORT Basic.append(builder, "#scope_export");
            else if n.scope_type == .FILE Basic.append(builder, "#scope_file");
            else if n.scope_type == .INTERNAL Basic.append(builder, "#scope_module");
            else return false;

            return true;
        case .USING;
            n := cast(*C.Code_Using) node;
            if !using_prefix(builder, n, bodies, next) return false;

            return expression(builder, n.expression, true, bodies, next);
        case .LOOP_CONTROL;
            n := cast(*C.Code_Loop_Control) node;
            if n.control_type == .BREAK Basic.append(builder, "break");
            else if n.control_type == .CONTINUE Basic.append(builder, "continue");
            else Basic.append(builder, "remove");

            if n.target_ident {
                Basic.append(builder, " ");
                Basic.append(builder, n.target_ident.name);
            }

            return true;
    }

    return false;
}

declaration :: (builder: *Basic.String_Builder, decl: *C.Code_Declaration, print_using: bool, compound: *C.Code_Compound_Declaration, print_type: bool, bodies: bool, depth: s64, parameter := false) -> bool {
    if !decl || depth > 512 return false;
    if syntax_int(decl, "__syntax_incomplete") return false;

    tag := syntax_child(decl, "__syntax_union_tag");
    if tag {
        if !expression(builder, tag, true, bodies, depth + 1) return false;
        Basic.append(builder, " ,, ");
    }

    if decl.flags & .IS_MARKED_AS_AS Basic.append(builder, "#as ");

    if print_using && syntax_int(decl, "__syntax_using") {
        if !private_using(builder, decl, bodies, depth + 1) return false;
    }

    if !compound && decl.name.count == 0 {
        if decl.type_inst && !expression(builder, decl.type_inst, true, bodies, depth + 1) return false;

        if decl.expression {
            Basic.append(builder, " = ");
            return expression(builder, decl.expression, true, bodies, depth + 1);
        }

        return decl.type_inst != null;
    }

    if print_using && decl.flags & .IS_MARKED_AS_AS Basic.append(builder, "#as ");

    if compound {
        if !compound.comma_separated_assignment return false;

        for arg, i: compound.comma_separated_assignment.arguments {
            if i Basic.append(builder, ", ");
            if !arg.node return false;

            if arg.modifier == .DECLARE {
                if arg.node.kind != .DECLARATION return false;
                member := cast(*C.Code_Declaration) arg.node;
                if member.flags & .HAS_SCOPE_MODIFIER Basic.append(builder, "`");
                Basic.append(builder, member.name);
            } else if arg.modifier == .ASSIGN {
                if !expression(builder, arg.node, true, bodies, depth + 1) return false;
                Basic.append(builder, "=");
            } else return false;
        }
    } else {
        if decl.flags & .HAS_SCOPE_MODIFIER Basic.append(builder, "`");
        Basic.append(builder, decl.name);
    }

    Basic.append(builder, ":");

    if print_type && decl.type_inst {
        Basic.append(builder, " ");
        if !expression(builder, decl.type_inst, true, bodies, depth + 1) return false;
    }

    value := decl.expression;
    alias := syntax_child(decl, "__syntax_alias_expression");

    if alias && value == syntax_child(decl, "__syntax_initial_expression") value = alias;

    if value {
        if !parameter && decl.flags & .IS_CONSTANT Basic.append(builder, ": ");
        else Basic.append(builder, " = ");
        if !expression(builder, value, true, bodies, depth + 1) return false;
    }

    if !decl.expression && decl.flags & .IS_UNINITIALIZED Basic.append(builder, " = ---");

    if decl.alignment_expression {
        Basic.append(builder, " #align ");
        if !expression(builder, decl.alignment_expression, false, bodies, depth + 1) return false;
    }

    Basic.append(builder, syntax_string(decl, "__syntax_flags"));

    if syntax_int(decl, "__syntax_foreign") {
        Basic.append(builder, " #foreign");
        library := syntax_child(decl, "__syntax_foreign_library");
        if library {
            Basic.append(builder, " ");
            if !expression(builder, library, true, bodies, depth + 1) return false;
        }

        if syntax_int(decl, "__syntax_foreign_named") {
            Basic.append(builder, " ");
            quoted(builder, syntax_string(decl, "__syntax_foreign_symbol"));
        }
    }

    for note: decl.notes {
        if decl.expression && decl.expression.kind == .PROCEDURE_HEADER continue;
        Basic.append(builder, " @");
        Basic.append(builder, note.text);
    }

    return true;
}

collect :: (message: *C.Message) {
    if !message || message.kind != .TYPECHECKED return;

    typed := cast(*C.Message_Typechecked) message;
    for item: typed.declarations {
        decl := item.expression;
        filename := C.get_filename(decl);

        if filename.count == 0 continue;

        index: s64 = -1;

        for file, i: files if file.name == filename {
            index = i;
            break;
        }

        if index < 0 {
            Basic.array_add(*files, File_Group.{ Basic.copy_string(filename) });
            index = files.count - 1;
        }

        Basic.array_add(*files[index].declarations, decl);
    }
}

finish :: (bodies: bool) {
    File :: #import "File";

    for *file: files {
        builder: Basic.String_Builder;

        Basic.init_string_builder(*builder);

        for decl: file.declarations {
            if !declaration(*builder, decl, true, null, true, bodies, 0) {
                Basic.append(*builder, " /* expression not printed */");
            }

            Basic.append(*builder, ";\n");
        }

        basename := C.get_basename_and_path(file.name);
        output := Basic.sprint("pp_%.txt", basename);
        text := Basic.builder_to_string(*builder);
        success := File.write_entire_file(output, text);

        Basic.assert(success, "Program_Print could not write %", output);
        Basic.free(text);
        Basic.free(output);
        Basic.free_buffers(*builder);
        Basic.array_free(file.declarations);
        Basic.free(file.name);
    }

    Basic.array_reset(*files);
}

#scope_file
C :: #import "Compiler";
Basic :: #import "Basic";

File_Group :: struct {
    name: string;
    declarations: [..] *C.Code_Declaration;
}

files: [..] File_Group;

private_using :: (builder: *Basic.String_Builder, node: *C.Code_Node, bodies: bool, depth: int) -> bool {
    Basic.append(builder, "using");
    filter := syntax_child(node, "__syntax_filter_expression");
    if filter {
        kind := syntax_int(node, "__syntax_filter_kind");
        if kind == 1 Basic.append(builder, ",only(");
        else if kind == 2 Basic.append(builder, ",except(");
        else if kind == 3 Basic.append(builder, ",map(");
        else return false;
        if !expression(builder, filter, true, bodies, depth + 1) return false;

        Basic.append(builder, ")");
    } else Basic.append(builder, syntax_string(node, "__syntax_filter"));

    Basic.append(builder, " ");

    return true;
}

statements :: (builder: *Basic.String_Builder, block: *C.Code_Block, bodies: bool, depth: int) -> bool {
    for statement: block.statements {
        if !expression(builder, statement, true, bodies, depth + 1) return false;
        Basic.append(builder, ";\n");
    }

    return true;
}

value_block :: (builder: *Basic.String_Builder, block: *C.Code_Block, bodies: bool, depth: int) -> bool {
    if block.statements.count == 1 return expression(builder, block.statements[0], false, bodies, depth + 1);
    return expression(builder, block, true, bodies, depth + 1);
}

enum_block :: (builder: *Basic.String_Builder, block: *C.Code_Block, bodies: bool, depth: int) -> bool {
    if depth > 512 return false;

    Basic.append(builder, "{\n");

    for member: block.statements {
        if member.kind == .DECLARATION {
            declaration := cast(*C.Code_Declaration) member;

            Basic.append(builder, declaration.name);

            if declaration.expression {
                Basic.append(builder, " :: ");
                if !expression(builder, declaration.expression, true, bodies, depth + 1) return false;
            }

            for note: declaration.notes {
                Basic.append(builder, " @");
                Basic.append(builder, note.text);
            }
        } else if member.kind == .IF {
            condition := cast(*C.Code_If) member;

            Basic.append(builder, "#if ");
            if !expression(builder, condition.condition, true, bodies, depth + 1) return false;

            Basic.append(builder, " ");
            if !enum_block(builder, condition.then_block, bodies, depth + 1) return false;

            if condition.else_block {
                Basic.append(builder, " else ");
                if !enum_block(builder, condition.else_block, bodies, depth + 1) return false;
            }
        } else if !expression(builder, member, true, bodies, depth + 1) return false;

        Basic.append(builder, ";\n");
    }

    Basic.append(builder, "}");

    return true;
}

procedure :: (builder: *Basic.String_Builder, n: *C.Code_Procedure_Header, block: *C.Code_Block, bodies: bool, depth: int) -> bool {
    if !n || depth > 512 return false;

    node: *C.Code_Node = n;
    next := depth + 1;
    lambda := syntax_child(node, "__syntax_lambda_body");

    Basic.append(builder, "(");

    for arg, i: n.arguments {
        if i > 0 Basic.append(builder, ", ");

        for using_node: n.parameter_usings {
            if !using_node.expression || using_node.expression.kind != .IDENT return false;
            ident := cast(*C.Code_Ident) using_node.expression;
            if ident.name == arg.name && !using_prefix(builder, using_node, bodies, next) return false;
        }

        if arg.flags & .IS_MARKED_AS_DISCARD Basic.append(builder, "#discard ");
        if arg.flags & .AUTO_VALUE_BAKE Basic.append(builder, "$$");
        else if arg.flags & .IS_CONSTANT Basic.append(builder, "$");
        if lambda && !arg.type_inst Basic.append(builder, arg.name);
        // Parameter usings above carry the public filter graph. Repeating the
        // declaration's private using marker would overwrite that filter on parse.
        else if !declaration(builder, arg, false, null, true, bodies, next, true) return false;
    }

    Basic.append(builder, ")");

    if lambda {
        Basic.append(builder, " => ");
        return expression(builder, lambda, true, bodies, next);
    }

    if n.returns.count > 0 {
        Basic.append(builder, " -> (");

        for arg, i: n.returns {
            if i > 0 Basic.append(builder, ", ");
            if !declaration(builder, arg, false, null, true, bodies, next) return false;
            if syntax_int(arg, "__syntax_return_must") Basic.append(builder, " #must");
        }

        Basic.append(builder, ")");
    }

    if n.procedure_flags & .C_CALL Basic.append(builder, " #c_call");
    if n.procedure_flags & .SYNTACTICALLY_MARKED_AS_NO_CONTEXT Basic.append(builder, " #no_context");
    if n.procedure_flags & .MACRO Basic.append(builder, " #expand");
    if n.procedure_flags & .NO_DEBUG Basic.append(builder, " #no_debug");
    if n.procedure_flags & .SYMMETRIC Basic.append(builder, " #symmetric");
    if n.procedure_flags & .CPP_METHOD Basic.append(builder, " #cpp_method");
    if n.procedure_flags & .NO_CALL Basic.append(builder, " #no_call");
    if n.procedure_flags & .SYNTACTICALLY_MARKED_AS_INLINE_YES Basic.append(builder, " inline");
    if n.procedure_flags & .SYNTACTICALLY_MARKED_AS_INLINE_NO Basic.append(builder, " no_inline");

    if n.procedure_flags & .SYNTACTICALLY_MARKED_AS_COMPILER || n.procedure_flags & .INTRINSIC {
        if n.procedure_flags & .INTRINSIC Basic.append(builder, " #intrinsic");
        else Basic.append(builder, " #compiler");

        builtin := syntax_string(node, "__syntax_builtin_name");
        if builtin.count {
            Basic.append(builder, " ");
            quoted(builder, builtin);
        }
    }

    if n.procedure_flags & .DEPRECATED {
        Basic.append(builder, " #deprecated");

        if n.deprecation_string.count || syntax_int(node, "__syntax_deprecation_named") {
            Basic.append(builder, " ");
            quoted(builder, n.deprecation_string);
        }
    }

    if syntax_int(node, "__syntax_foreign") || n.procedure_flags & .ELSEWHERE {
        if n.procedure_flags & .ELSEWHERE Basic.append(builder, " #elsewhere");
        else Basic.append(builder, " #foreign");

        if n.library_identifier {
            Basic.append(builder, " ");
            if !expression(builder, n.library_identifier, true, bodies, next) return false;
        }

        if n.foreign_function_name.count || syntax_int(node, "__syntax_foreign_named") {
            Basic.append(builder, " ");
            quoted(builder, n.foreign_function_name);
        }
    }

    Basic.append(builder, syntax_string(node, "__syntax_flags"));

    if syntax_int(node, "__syntax_program_export") {
        Basic.append(builder, " #program_export");

        if syntax_int(node, "__syntax_export_named") {
            Basic.append(builder, " ");
            quoted(builder, syntax_string(node, "__syntax_export_name"));
        }
    }

    if n.modify_directives.count > 1 return false;

    for modify: n.modify_directives {
        Basic.append(builder, " ");
        if !expression(builder, modify, true, bodies, next) return false;
    }

    if bodies && block {
        Basic.append(builder, " ");
        if !expression(builder, block, true, bodies, next) return false;
    }

    for note: n.notes {
        Basic.append(builder, " @");
        Basic.append(builder, note.text);
    }

    return true;
}
