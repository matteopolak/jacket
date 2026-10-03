is_void_type :: (t: *CType) -> bool {
    return model_has_scalar_flag(t, .VOID);
}

is_bool_type :: (t: *CType) -> bool {
    return model_has_scalar_flag(t, .BOOL);
}

is_same_location :: (a: Location, b: Location) -> bool {
    return a.file == b.file && a.line == b.line && a.column == b.column;
}

is_same_type :: (a: *CType, b: *CType) -> bool {
    path: [..] Model_Type_Pair;
    defer array_free(path);
    return model_equal_type(a, b, true, *path);
}

is_same_expression :: (a: *Expression, b: *Expression) -> bool {
    if a == b return true;
    if !a || !b || a.kind != .LITERAL || b.kind != .LITERAL return false;
    return is_same_literal(cast(*Literal) a, cast(*Literal) b);
}

is_same_literal :: (a: *Literal, b: *Literal) -> bool {
    if a == b return true;
    if !a || !b || a.literal_kind != b.literal_kind return false;
    if a.literal_kind == .NULL return true;
    if a.literal_kind == .INTEGER return a.int_value == b.int_value;
    if a.literal_kind == .UNSIGNED_INTEGER return a.uint_value == b.uint_value;
    if a.literal_kind == .FLOAT return a.float_value == b.float_value;
    if a.literal_kind == .STRING return a.string_value == b.string_value;

    if a.literal_kind == .ENUM {
        return a.enum_type != null && a.enum_type == b.enum_type && a.enum_value != null && b.enum_value != null && a.enum_value.name == b.enum_value.name;
    }

    if a.literal_kind == .IDENTIFIER return a.identifier != null && a.identifier == b.identifier;
    if a.literal_kind == .MACRO return false;

    log_error("Bindings_Generator: literal comparison lacks a reviewed raw-token payload mapping\n");

    return false;
}

#scope_file
#import "Basic";

Model_Type_Pair :: struct {
    a: *CType;
    b: *CType;
}

Model_Comparison_Work :: struct {
    a: *CType;
    b: *CType;
    top: bool;
    leave: bool;
}

model_unalias :: (type: *CType) -> *CType, bool {
    seen: [..] *CType;

    defer array_free(seen);

    while type && type.type_of_typedef {
        if array_find(seen, type) {
            log_error("Bindings_Generator: cyclic comparison typedef chain\n");
            return null, false;
        }

        array_add(*seen, type);
        type = type.type_of_typedef.type;
    }

    return type, true;
}

model_equal_type :: (a_in: *CType, b_in: *CType, top: bool, path: *[..] Model_Type_Pair) -> bool {
    work: [..] Model_Comparison_Work;

    defer array_free(work);
    initial_path := path.count;

    defer path.count = initial_path;
    array_add(*work, Model_Comparison_Work.{ a_in, b_in, top, false });

    while work.count > 0 {
        current := work[work.count - 1];

        work.count -= 1;

        if current.leave {
            path.count -= 1;
            continue;
        }

        if current.a == current.b continue;
        if !current.a || !current.b return false;

        a, a_ok := model_unalias(current.a);
        b, b_ok := model_unalias(current.b);

        if !a_ok || !b_ok || !a || !b return false;

        cv :: CType.Flags.CONST | .VOLATILE;
        if !current.top && ((current.a.flags & cv) != (current.b.flags & cv) || (a.flags & cv) != (b.flags & cv)) {
            log_error("Bindings_Generator: nested qualifier comparison requires reviewed precision\n");
            return false;
        }

        if a == b continue;

        for pair: path.* if pair.a == a && pair.b == b {
            log_error("Bindings_Generator: cyclic comparison type graph\n");
            return false;
        }

        array_add(path, Model_Type_Pair.{ a, b });
        array_add(*work, Model_Comparison_Work.{ a, b, current.top, true });

        if a.array_element_type || b.array_element_type || (a.hardcoded_jai_string && !gm_named_wide_scalar(a)) ||
           (b.hardcoded_jai_string && !gm_named_wide_scalar(b)) || a.type_of_declaration || b.type_of_declaration ||
           a.template_instantiation_params.count || b.template_instantiation_params.count || a.number_flags & .CHAR || b.number_flags & .CHAR {
            log_error("Bindings_Generator: type comparison contains an unreviewed model category\n");
            return false;
        }

        if a.pointer_to || b.pointer_to {
            if !a.pointer_to || !b.pointer_to return false;
            array_add(*work, Model_Comparison_Work.{ a.pointer_to, b.pointer_to, false, false });
            continue;
        }

        if a.flags & .REFERENCE || b.flags & .REFERENCE {
            log_error("Bindings_Generator: reference model lacks a represented pointee\n");
            return false;
        }

        if a.type_of_enum || b.type_of_enum {
            if !a.type_of_enum || a.type_of_enum != b.type_of_enum return false;
            continue;
        }

        if a.type_of_struct || b.type_of_struct {
            if !a.type_of_struct || a.type_of_struct != b.type_of_struct return false;
            continue;
        }

        if a.type_of_function || b.type_of_function {
            if !a.type_of_function || !b.type_of_function return false;

            x := a.type_of_function;
            y := b.type_of_function;

            if !gm_dependency_view_valid(x.arguments) || !gm_dependency_view_valid(y.arguments) {
                log_error("Bindings_Generator: malformed comparison function argument storage\n");
                return false;
            }

            if x.arguments.count != y.arguments.count return false;

            if (x.flags & .VARIADIC) != (y.flags & .VARIADIC) {
                log_error("Bindings_Generator: variadic function comparison requires reviewed precision\n");
                return false;
            }

            i := x.arguments.count;
            while i > 0 {
                i -= 1;
                if !x.arguments[i] || !y.arguments[i] return false;
                array_add(*work, Model_Comparison_Work.{ x.arguments[i].type, y.arguments[i].type, true, false });
            }

            array_add(*work, Model_Comparison_Work.{ x.return_type, y.return_type, true, false });
            continue;
        }

        categories :: CType.Flags.VOID | .BOOL;
        if (a.flags & categories) != (b.flags & categories) return false;
        if a.flags & .VOID continue;

        if a.size < 0 || b.size < 0 {
            log_error("Bindings_Generator: scalar comparison requires represented sizes\n");
            return false;
        }

        if a.flags & .BOOL {
            if a.size != b.size return false;
            continue;
        }

        // An unnamed size16 unsigned scalar is also the established public
        // numeric encoding; hardcoded opaque shapes were excluded above.
        if (!a.number_flags && a.size != 16) || (!b.number_flags && b.size != 16) return false;
        if (a.number_flags & .FLOAT) != (b.number_flags & .FLOAT) return false;
        if !(a.number_flags & .FLOAT) && (a.number_flags & .SIGNED) != (b.number_flags & .SIGNED) return false;
        if a.size != b.size return false;
    }

    return true;
}

model_has_scalar_flag :: (type: *CType, flag: CType.Flags) -> bool {
    seen: [..] *CType;

    defer array_free(seen);

    while type {
        if array_find(seen, type) {
            log_error("Bindings_Generator: cyclic scalar typedef chain\n");
            return false;
        }

        array_add(*seen, type);
        if type.pointer_to || type.array_element_type || type.type_of_function || type.type_of_struct || type.type_of_enum || type.flags & .REFERENCE return false;
        if type.flags & flag return true;
        if !type.type_of_typedef return false;

        type = type.type_of_typedef.type;
    }

    return false;
}
