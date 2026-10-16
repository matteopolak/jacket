#import "Basic";
B :: #import "Bindings_Generator";

errors: int;

capture_error :: (_text: string, _data: *void, info: Log_Info) {
    if (info.common_flags & .ERROR) != 0 errors += 1;
}

exercise :: (mode: int) {
    left_scalar := B.CType.{ size = 4, alignment = 4, number_flags = .SIGNED | ._32BIT };
    right_scalar := left_scalar;

    if mode == 0 {
        left_scalar.flags = .VOID;
        right_scalar.flags = .VOID;
    }

    if mode == 1 {
        left_scalar.flags = .BOOL;
        right_scalar.flags = .BOOL;
    }

    left_nodes, right_nodes: [192] B.CType;
    left_aliases, right_aliases: [192] B.Typedef;
    left := *left_scalar;
    right := *right_scalar;

    for i: 0..left_nodes.count - 1 {
        if mode <= 2 {
            left_aliases[i].type = left;
            right_aliases[i].type = right;
            left_nodes[i].type_of_typedef = *left_aliases[i];
            right_nodes[i].type_of_typedef = *right_aliases[i];
        } else {
            left_nodes[i].pointer_to = left;
            right_nodes[i].pointer_to = right;
            left_nodes[i].size = 8;
            right_nodes[i].size = 8;
        }

        left = *left_nodes[i];
        right = *right_nodes[i];
    }

    if mode == 0 {
        assert(B.is_void_type(left) && B.is_void_type(right));
        return;
    }

    if mode == 1 {
        assert(B.is_bool_type(left) && B.is_bool_type(right));
        return;
    }

    assert(B.is_same_type(left, right));
    left_argument := B.Declaration.{ kind = .DECLARATION, type = left };
    right_argument := B.Declaration.{ kind = .DECLARATION, type = right };
    left_function := B.Function_Type.{ return_type = *left_scalar };
    right_function := B.Function_Type.{ return_type = *right_scalar };

    array_add(*left_function.arguments, *left_argument);
    array_add(*right_function.arguments, *right_argument);
    defer array_free(left_function.arguments);
    defer array_free(right_function.arguments);
    left_type := B.CType.{ type_of_function = *left_function };
    right_type := B.CType.{ type_of_function = *right_function };
    first := B.Function.{ kind = .FUNCTION, name = "overload", output_name = "overload", type = *left_type };
    second := B.Function.{ kind = .FUNCTION, name = "overload", output_name = "overload", type = *right_type };

    assert(B.functions_are_equivalent(*first, *second));
    right_scalar.size = 8;
    right_scalar.number_flags = .SIGNED | ._64BIT;
    assert(!B.is_same_type(left, right) && !B.functions_are_equivalent(*first, *second));
    existing := (*B.Declaration).[cast(*B.Declaration) *first];
    found, _ := B.find_name_clash(existing, cast(*B.Declaration) *second, true);

    assert(!found);
}

branch_cases :: () {
    scalar := B.CType.{ size = 4, number_flags = .SIGNED | ._32BIT };
    other := scalar;
    a := B.Function_Type.{ return_type = *scalar };
    b := B.Function_Type.{ return_type = *other };

    defer array_free(a.arguments);
    defer array_free(b.arguments);
    x := B.CType.{ type_of_function = *a };
    y := B.CType.{ type_of_function = *b };

    assert(B.is_same_type(*x, *y)); // Empty argument lists.
    left := B.Declaration.{ type = *scalar };
    right := B.Declaration.{ type = *other };

    array_add(*a.arguments, *left);
    array_add(*a.arguments, *left);
    array_add(*b.arguments, *right);
    array_add(*b.arguments, *right);
    assert(B.is_same_type(*x, *y)); // Repeated DAG branches are not cycles.
    wider := B.CType.{ size = 8, number_flags = .SIGNED | ._64BIT };
    last := B.Declaration.{ type = *wider };

    b.arguments[1] = *last;
    assert(!B.is_same_type(*x, *y)); // Every argument participates.
    b.arguments[1] = *right;
    signed := B.CType.{ size = 16, number_flags = .SIGNED, hardcoded_jai_string = "S128" };
    unsigned := B.CType.{ size = 16, hardcoded_jai_string = "U128" };
    signed_copy := signed;
    unsigned_copy := unsigned;

    assert(B.is_same_type(*signed, *signed_copy));
    assert(B.is_same_type(*unsigned, *unsigned_copy));
    assert(!B.is_same_type(*signed, *unsigned));
    unnamed := B.CType.{ size = 16 };
    unnamed_copy := unnamed;

    assert(B.is_same_type(*unnamed, *unnamed_copy));
    assert(B.is_same_type(*unnamed, *unsigned));
    left.type = *signed;
    right.type = *unsigned;
    first := B.Function.{ kind = .FUNCTION, name = "wide", type = *x };
    second := B.Function.{ kind = .FUNCTION, name = "wide", type = *y };
    existing := (*B.Declaration).[cast(*B.Declaration) *first];
    found, _ := B.find_name_clash(existing, cast(*B.Declaration) *second, true);

    assert(!found);
    // Impossible byte extents must be rejected before any argument dereference.
    saved := b.arguments.count;

    b.arguments.count = 0x7fffffffffffffff;
    before := errors;

    assert(!B.is_same_type(*x, *y) && errors > before);
    before = errors;
    found, _ = B.find_name_clash(existing, cast(*B.Declaration) *second, true);
    assert(found && errors > before);
    b.arguments.count = saved;
}

main :: () {
    push_context, defer_pop;
    context.logger = capture_error;
    args := get_command_line_arguments();

    defer array_free(args);

    if args.count > 1 {
        mode, valid, rest := string_to_int(args[1]);
        assert(valid && !rest);
        exercise(mode);
    } else for mode: 0..3 exercise(mode);

    assert(errors == 0);
    branch_cases();
    // Preserve diagnosed cycles rather than assuming coinductive equality.
    left, right: B.CType;
    left.pointer_to = *left;
    right.pointer_to = *right;
    assert(!B.is_same_type(*left, *right) && errors > 0);
    alias: B.Typedef;
    cycle := B.CType.{ type_of_typedef = *alias };

    alias.type = *cycle;
    before := errors;

    assert(!B.is_void_type(*cycle) && errors > before);
    print("ok\n");
}
