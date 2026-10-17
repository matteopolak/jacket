// Reviewed callable omission visitor, using the canonical public closure record.
#import "Basic";
B :: #import "Bindings_Generator";

plain_calls: int;
data_calls: int;
stop_plain: bool;

plain :: (decl: *B.Declaration, parent: *B.Declaration) -> B.Declaration_Visit_Result {
    assert(decl.name != "hidden_argument");
    plain_calls += 1;
    return ifx stop_plain then .STOP else .RECURSE;
}

with_data :: (decl: *B.Declaration, parent: *B.Declaration, data: *void) -> B.Declaration_Visit_Result {
    assert(data == *data_calls);
    (cast(*int) data).* += 1;
    return .RECURSE;
}

main :: () {
    integer := B.CType.{ size = 4, number_flags = ._32BIT | .SIGNED };
    argument := B.Declaration.{ kind = .DECLARATION, name = "hidden_argument", type = *integer };
    method_type := B.Function_Type.{ return_type = *integer, flags = .CPP_METHOD };

    array_add(*method_type.arguments, *argument);
    defer array_free(method_type.arguments);
    callable := B.CType.{ type_of_function = *method_type };
    method := B.Function.{ kind = .FUNCTION, name = "hidden", foreign_name = "foreign_hidden", type = *callable };
    ordinary_type := B.Function_Type.{ return_type = *integer };
    ordinary_callable := B.CType.{ type_of_function = *ordinary_type };
    ordinary := B.Function.{ kind = .FUNCTION, name = "ordinary", foreign_name = "ordinary", type = *ordinary_callable };
    root := B.Namespace.{ kind = .NAMESPACE, name = "root" };

    method.parent = cast(*B.Declaration) *root;
    ordinary.parent = cast(*B.Declaration) *root;
    array_add(*root.declarations, cast(*B.Declaration) *method, cast(*B.Declaration) *ordinary);
    defer array_free(root.declarations);
    closure := B.Cpp_Functions_Visitor_Data.{ visitor = plain, visitor_with_data = with_data, visitor_data = *data_calls };

    B.visit_declarations(B.omit_cpp_functions_visitor, *closure, cast(*B.Declaration) *root);
    assert((method.decl_flags & .OMIT_FROM_OUTPUT) != 0);
    assert((ordinary.decl_flags & .OMIT_FROM_OUTPUT) == 0);
    assert(plain_calls == 2 && data_calls == 2);
    ordinary_type.flags = .CPP_METHOD;
    assert(B.omit_cpp_functions_visitor(cast(*B.Declaration) *ordinary, null, *closure) == .RECURSE);
    assert(plain_calls == 3 && data_calls == 3);
    ordinary_type.flags = 0;
    global_cpp := B.Function.{ kind = .FUNCTION, name = "global_cpp", foreign_name = "different_global_name", type = *ordinary_callable };

    assert(B.omit_cpp_functions_visitor(cast(*B.Declaration) *global_cpp, null, null) == .STOP);
    assert((global_cpp.decl_flags & .OMIT_FROM_OUTPUT) != 0 && plain_calls == 3 && data_calls == 3);
    stop_plain = true;
    assert(B.omit_cpp_functions_visitor(decl = cast(*B.Declaration) *ordinary, parent_decl = null, user_data = *closure) == .STOP);
    assert(plain_calls == 4 && data_calls == 3);
    empty: B.Cpp_Functions_Visitor_Data;

    assert(B.omit_cpp_functions_visitor(cast(*B.Declaration) *ordinary, null, *empty) == .RECURSE);
    factory, data := B.create_omit_cpp_functions_visitor(plain, with_data, *data_calls);

    defer free(data);
    assert(factory(cast(*B.Declaration) *ordinary, null, data) == .STOP);
    assert(plain_calls == 5 && data_calls == 3);
    print("ok\n");
}
