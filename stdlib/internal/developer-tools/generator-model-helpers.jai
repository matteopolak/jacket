#scope_module
#import "Basic";
#import "String";

gm_omit_visitor :: (plain: Declaration_Visitor, with_data: Declaration_Visitor_With_Data, data: *void) -> Declaration_Visitor_With_Data, *void {
    closure := New(Cpp_Functions_Visitor_Data);
    closure.visitor = plain;
    closure.visitor_with_data = with_data;
    closure.visitor_data = data;

    return gm_visit_without_cpp, closure;
}

gm_visit_without_cpp :: (decl: *Declaration, parent: *Declaration, data: *void) -> Declaration_Visit_Result {
    assert(data != null, "Bindings_Generator: missing omission visitor closure");
    closure := cast(*Cpp_Functions_Visitor_Data) data;

    if decl && decl.kind == .FUNCTION {
        if decl.foreign_name != decl.name {
            decl.decl_flags |= .OMIT_FROM_OUTPUT;
            return .STOP;
        }
    }

    if closure.visitor && closure.visitor(decl, parent) == .STOP return .STOP;
    if closure.visitor_with_data return closure.visitor_with_data(decl, parent, closure.visitor_data);

    return .RECURSE;
}

gm_underlying :: (type: *CType) -> *CType {
    seen: [..] *CType;

    defer array_free(seen);

    while type && type.type_of_typedef {
        if array_find(seen, type) {
            log_error("Bindings_Generator: cyclic typedef chain\n");
            return null;
        }

        array_add(*seen, type);
        type = type.type_of_typedef.type;
    }

    return type;
}

gm_integer :: (type: *CType) -> *CType {
    seen: [..] *CType;

    defer array_free(seen);

    while type {
        if array_find(seen, type) {
            log_error("Bindings_Generator: cyclic numeric type graph\n");
            return null;
        }

        array_add(*seen, type);
        if type.pointer_to || type.array_element_type || type.type_of_function || type.type_of_struct || type.type_of_declaration || (type.flags & (.VOID | .BOOL | .REFERENCE)) != 0 return null;

        if type.type_of_typedef {
            type = type.type_of_typedef.type;
            continue;
        }

        if type.type_of_enum {
            type = type.type_of_enum.type;
            continue;
        }

        if (!type.number_flags && type.size != 16) || (type.number_flags & .FLOAT) != 0 return null;

        return type;
    }

    return null;
}

gm_operator :: (name: string) -> bool {
    if name.count <= 8 || slice(name, 0, 8) != "operator" return false;
    next := name[8];
    return !is_valid_identifier_start(next) && !(next >= #char "0" && next <= #char "9");
}

gm_assignment :: (name: string) -> bool {
    for op: string.["operator=", "operator+=", "operator-=", "operator*=", "operator/=", "operator%=", "operator&=", "operator|=", "operator^=", "operator<<=", "operator>>="] if name == op return true;
    return false;
}

gm_prefix_comment :: (comment: Comment) -> bool {
    if comment.location.line != 0 return true;
    for character: comment.text if character == #char "\n" return true;
    return false;
}

gm_va_list :: (type: *CType) -> bool {
    seen: [..] *CType;

    defer array_free(seen);

    while type {
        if array_find(seen, type) {
            log_error("Bindings_Generator: cyclic va_list typedef chain\n");
            return false;
        }

        array_add(*seen, type);
        if !type.type_of_typedef return false;

        name := type.type_of_typedef.name;
        if name == "va_list" || name == "__builtin_va_list" || name == "__gnuc_va_list" return true;

        type = type.type_of_typedef.type;
    }

    return false;
}

gm_struct_field :: (decl: *Declaration) -> bool, *Struct {
    if !decl || decl.kind != .DECLARATION || !decl.parent || decl.parent.kind != .STRUCT || decl.storage_class == .Static || (decl.decl_flags & .IS_METADATA) != 0 return false, null;
    parent := cast(*Struct) decl.parent;
    if (parent.flags & .IS_ANON) != 0 return true, parent;
    return true, null;
}

gm_alignment :: (type: *CType, decl: *Declaration) -> s64, bool {
    t := gm_underlying(type);
    natural: s64 = -1;

    if t natural = t.alignment;
    if decl && decl.explicit_field_alignment > 0 return decl.explicit_field_alignment, decl.explicit_field_alignment == natural;
    if decl && decl.field_alignment > 0 return decl.field_alignment, decl.field_alignment == natural;

    return natural, natural > 0;
}

gm_named_wide_scalar :: (type: *CType) -> bool {
    if !type || type.size != 16 || type.pointer_to || type.array_element_type || type.type_of_struct || type.type_of_enum ||
       type.type_of_typedef || type.type_of_function || type.type_of_declaration || type.template_instantiation_params.count return false;
    if (type.flags & (.VOID | .BOOL | .REFERENCE)) != 0 || (type.number_flags & (.FLOAT | .CHAR)) != 0 return false;

    return (type.hardcoded_jai_string == "S128" && (type.number_flags & .SIGNED) != 0) ||
           (type.hardcoded_jai_string == "U128" && (type.number_flags & .SIGNED) == 0);
}

gm_primitive_size :: (type: *CType, _decl: *Declaration) -> int {
    count: s64 = 1;
    seen: [..] *CType;

    defer array_free(seen);

    while type {
        if array_find(seen, type) {
            log_error("Bindings_Generator: cyclic primitive type graph\n");
            return -1;
        }

        array_add(*seen, type);

        if type.type_of_typedef {
            type = type.type_of_typedef.type;
            continue;
        }

        if type.array_element_type {
            n := type.array_element_count;
            if n < 0 return -1;

            if n > 0 && count > 0x7fffffffffffffff / n {
                log_error("Bindings_Generator: primitive array size overflow\n");
                return -1;
            }

            count *= n;
            type = type.array_element_type;
            continue;
        }

        if type.pointer_to || type.type_of_function || type.type_of_struct || type.type_of_declaration return -1;

        // These spellings encode the reviewed canonical numeric types. An
        // arbitrary hardcoded Jai name still does not establish a primitive.
        if type.hardcoded_jai_string && !gm_named_wide_scalar(type) return -1;

        if type.type_of_enum {
            type = type.type_of_enum.type;
            continue;
        }

        if (type.flags & (.VOID | .REFERENCE)) != 0 return -1;
        if type.size < 0 || (!type.number_flags && !(type.flags & .BOOL) && type.size != 16) return -1;

        if type.size > 0 && count > 0x7fffffffffffffff / type.size {
            log_error("Bindings_Generator: primitive size overflow\n");
            return -1;
        }

        return count * type.size;
    }

    return -1;
}

gm_functions_equal :: (a: *Function, b: *Function, skip: bool) -> bool {
    if !a || !b || a.name != b.name return false;

    x := gm_underlying(a.type);
    y := gm_underlying(b.type);

    if !x || !y || !x.type_of_function || !y.type_of_function return false;

    left := x.type_of_function;
    right := y.type_of_function;

    if left.arguments.count < 0 || right.arguments.count < 0 || (left.arguments.count > 0 && !left.arguments.data) || (right.arguments.count > 0 && !right.arguments.data) {
        log_error("Bindings_Generator: malformed function argument storage\n");
        return false;
    }

    if left.arguments.count != right.arguments.count return false;

    for i: (ifx skip then 1 else 0)..left.arguments.count - 1 {
        if !left.arguments[i] || !right.arguments[i] || !left.arguments[i].type || !right.arguments[i].type {
            log_error("Bindings_Generator: malformed function argument declaration\n");
            return false;
        }

        if !is_same_type(left.arguments[i].type, right.arguments[i].type) return false;
    }

    return true;
}

gm_name :: (decl: *Declaration) -> string {
    if decl.outputted_jai_name return decl.outputted_jai_name;
    if decl.output_name return decl.output_name;
    return decl.name;
}

// Conservatively gate overload acceptance on the existing comparator's supported domain.
GM_Comparison_Support_Work :: struct {
    type: *CType;
    nested: bool;
    leave: bool;
}

gm_comparison_supported :: (type: *CType) -> bool {
    work: [..] GM_Comparison_Support_Work;
    active, completed: [..] *CType;

    defer array_free(work);
    defer array_free(active);
    defer array_free(completed);
    array_add(*work, GM_Comparison_Support_Work.{ type, false, false });

    while work.count > 0 {
        current := work[work.count - 1];

        work.count -= 1;

        if current.leave {
            active.count -= 1;
            array_add(*completed, current.type);
            continue;
        }

        type = current.type;
        if !type || type.array_element_type || (type.hardcoded_jai_string && !gm_named_wide_scalar(type)) || (type.number_flags & .CHAR) != 0 return false;
        if current.nested && (type.flags & (.CONST | .VOLATILE)) != 0 return false;
        if array_find(active, type) return false;
        if array_find(completed, type) continue;

        array_add(*active, type);
        array_add(*work, GM_Comparison_Support_Work.{ type, current.nested, true });

        if type.type_of_typedef {
            array_add(*work, GM_Comparison_Support_Work.{ type.type_of_typedef.type, true, false });
            continue;
        }

        if type.pointer_to {
            array_add(*work, GM_Comparison_Support_Work.{ type.pointer_to, true, false });
            continue;
        }

        if type.template_instantiation_params.count || type.type_of_declaration return false;

        if type.type_of_function {
            function := type.type_of_function;
            if !gm_dependency_view_valid(function.arguments) return false;

            for argument: function.arguments {
                if !argument return false;
                array_add(*work, GM_Comparison_Support_Work.{ argument.type, true, false });
            }

            array_add(*work, GM_Comparison_Support_Work.{ function.return_type, true, false });
            continue;
        }

        if type.type_of_enum || type.type_of_struct || (type.flags & .VOID) != 0 continue;
        if (type.flags & .REFERENCE) != 0 || type.size < 0 return false;
        if !type.number_flags && (type.flags & .BOOL) == 0 && type.size != 16 return false;
    }

    return true;
}

gm_function_arguments_supported :: (function: *Function_Type) -> bool {
    if !function || !gm_dependency_view_valid(function.arguments) return false;
    for argument: function.arguments if !argument || !gm_comparison_supported(argument.type) return false;
    return true;
}

gm_name_clash :: (existing: [] *Declaration, decl: *Declaration, overloads: bool) -> bool, *Declaration {
    if !decl return false, null;

    name := gm_name(decl);

    for other: existing {
        if !other || other == decl || gm_name(other) != name continue;

        if overloads && decl.kind == .FUNCTION && other.kind == .FUNCTION {
            left_type := gm_underlying(decl.type);
            right_type := gm_underlying(other.type);

            if !left_type || !right_type || !gm_function_arguments_supported(left_type.type_of_function) || !gm_function_arguments_supported(right_type.type_of_function) {
                log_error("Bindings_Generator: cannot safely disambiguate unsupported function signatures\n");
                return true, other;
            }

            if !gm_functions_equal(cast(*Function) decl, cast(*Function) other, false) continue;
        }

        return true, other;
    }

    return false, null;
}

gm_declaration :: (name: string, type: *CType, location: Location, comment: Comment) -> *Declaration {
    result := New(Declaration);
    result.kind = .DECLARATION;
    result.name = copy_string(name);
    result.output_name = copy_string(name);
    result.type = type;
    result.location = location;
    result.comment = comment;
    result.comment.text = copy_string(comment.text);

    return result;
}

gm_enum :: (name: string) -> *Enum {
    result := New(Enum);
    result.kind = .ENUM;
    result.name = copy_string(name);
    result.output_name = copy_string(name);

    return result;
}

gm_empty_struct :: (name: string) -> *CType {
    st := New(Struct);
    st.kind = .STRUCT;
    st.name = copy_string(name);
    st.output_name = copy_string(name);
    type := New(CType);
    type.type_of_struct = st;
    st.type = type;

    return type;
}
