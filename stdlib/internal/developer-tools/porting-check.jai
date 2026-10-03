should_skip :: (node: *Code_Node, settings: C_Porting_Check_Settings) -> bool {
    if !node return true;
    if !settings.skip_modules || !node.enclosing_load || !node.enclosing_load.enclosing_import return false;
    return node.enclosing_load.enclosing_import.module_type != .MAIN_PROGRAM;
}

is_of_fixed_array_type :: (node: *Code_Node) -> bool {
    if !node return false;
    type := porting_base_type(node.type);
    return type != null && type.type == .ARRAY && (cast(*Type_Info_Array) type).array_type == .FIXED;
}

check_c_porting :: (message: *Message, settings: C_Porting_Check_Settings) {
    if !message || message.kind != .TYPECHECKED return;
    for tc: (cast(*Message_Typechecked) message).procedure_bodies check_c_porting(tc, settings);
}

check_c_porting :: (tc: Typechecked(Code_Procedure_Body), settings: C_Porting_Check_Settings) {
    for node: tc.subexpressions {
        if should_skip(node, settings) continue;

        if node.kind == .BINARY_OPERATOR {
            binary := cast(*Code_Binary_Operator) node;
            if settings.check_treatment_of_arrays_as_pointers_in_assignments && binary.operator_type == cast(Operator_Type) #char "=" {
                if porting_pointer(binary.left) && porting_implicit_array(binary.right) porting_error(node, "fixed array used as a pointer in assignment");
            }

            if settings.check_intermediate_cast_to_int_when_adding_small_integers && binary.operator_type == cast(Operator_Type) #char "+" {
                // Inspect operand types: an explicit intermediate cast widens an
                // operand before addition. The result node need not be typed to
                // recognize an addition that still has two small operands.
                if porting_small_integer(binary.left) && porting_small_integer(binary.right)
                    porting_error(node, "small integer addition requires an explicit intermediate integer cast for C promotion semantics");
            }
        }

        if settings.check_treatment_of_arrays_as_pointers_in_procedure_calls && node.kind == .PROCEDURE_CALL {
            call := cast(*Code_Procedure_Call) node;
            if !call.resolved_procedure_expression continue;

            type := porting_base_type(call.resolved_procedure_expression.type);
            if !type || type.type != .PROCEDURE continue;

            procedure := cast(*Type_Info_Procedure) type;

            for argument, index: call.arguments_sorted {
                if index >= procedure.argument_types.count break;
                expected := porting_base_type(procedure.argument_types[index]);
                if expected && expected.type == .POINTER && porting_implicit_array(argument)
                    porting_error(argument, "fixed array used as a pointer in a procedure call");
            }
        }
    }
}

do_error_checking :: (p: *Metaprogram_Plugin, message: *Message) {
    if p && message && message.workspace != p.workspace return;

    check_c_porting(message, C_Porting_Check_Settings.{});

    #if CHECK_BINDINGS {
        if !message || message.kind != .TYPECHECKED return;
        options := get_build_options(message.workspace);
        for header: (cast(*Message_Typechecked) message).procedure_headers
            verify_elsewhere_binding(header.expression, options.os_target, options.cpu_target);
    }
}

#scope_file
#import "Basic";
#import "Compiler";

porting_base_type :: (type: *Type_Info) -> *Type_Info {
    while type && type.type == .VARIANT type = (cast(*Type_Info_Variant) type).variant_of;
    return type;
}

porting_pointer :: (node: *Code_Node) -> bool {
    if !node return false;
    type := porting_base_type(node.type);
    return type != null && type.type == .POINTER;
}

porting_small_integer :: (node: *Code_Node) -> bool {
    if !node return false;
    type := porting_base_type(node.type);
    return type != null && type.type == .INTEGER && type.runtime_size > 0 && type.runtime_size < 4;
}

porting_implicit_array :: (node: *Code_Node) -> bool {
    if !node return false;
    if is_of_fixed_array_type(node) return true;
    if node.kind != .CAST return false;

    cast_node := cast(*Code_Cast) node;

    return (cast_node.cast_flags & .IS_IMPLICIT) != 0 && is_of_fixed_array_type(cast_node.expression);
}

porting_error :: (node: *Code_Node, text: string) {
    loc: Source_Code_Location;

    if node && node.enclosing_load {
        loc.fully_pathed_filename = node.enclosing_load.fully_pathed_filename;
        loc.line_number = node.l0;
        loc.character_number = node.c0;
    }

    compiler_report(tprint("Check: %", text), loc = loc, mode = .ERROR_CONTINUABLE);
}
