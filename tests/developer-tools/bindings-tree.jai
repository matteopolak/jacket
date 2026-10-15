#import "Basic";
B :: #import "Bindings_Generator";

visits: s64;

visit :: (decl: *B.Declaration, parent: *B.Declaration, data: *void) -> B.Declaration_Visit_Result {
    count := cast(*s64) data;
    count.* += 1;
    if decl.kind == .FUNCTION assert(parent != null);

    if decl.kind == .ENUM && parent && parent.kind == .NAMESPACE {
        scope := cast(*B.Namespace) parent;
        replacement: [..] *B.Declaration;

        for child: scope.declarations array_add(*replacement, child);

        old := scope.declarations;

        scope.declarations = replacement;
        array_free(old);
    }

    return .RECURSE;
}

stop :: (_decl: *B.Declaration, _parent: *B.Declaration) -> B.Declaration_Visit_Result {
    visits += 1;
    return .STOP;
}

main :: () {
    scope: B.Namespace;
    scope.kind = .NAMESPACE;
    color: B.Enum;
    color.kind = .ENUM;
    color.name = "Color";
    color.parent = *scope;
    function: B.Function;
    function.kind = .FUNCTION;
    function.name = "paint";
    function.parent = *scope;
    argument: B.Declaration;
    argument.kind = .DECLARATION;
    argument.name = "value";
    argument.parent = *function;
    signature: B.Function_Type;

    array_add(*signature.arguments, *argument);
    function_type: B.CType;
    function_type.type_of_function = *signature;
    function.type = *function_type;
    array_add(*scope.declarations, *color);
    array_add(*scope.declarations, *function);
    array_add(*scope.declarations, *function); // Graph aliases are visited once.
    defer array_free(scope.declarations);
    defer array_free(signature.arguments);
    count: s64;

    B.visit_declarations(visit, *count, *scope);
    assert(count == 4);
    B.visit_declarations(stop, *scope);
    assert(visits == 1);
    assert(B.find_parent_enum(*argument, "Color") == *color);
    assert(B.change_argument_type_to_enum(*function, "value", "Color"));
    defer free(argument.type);
    assert(argument.type.type_of_enum == *color);
    assert(!B.change_argument_type_to_enum(*function, "missing", *color));
    B.change_return_type_to_enum(*function, *color);
    first_return := signature.return_type;

    defer free(first_return);
    assert(signature.return_type.type_of_enum == *color);
    assert(B.change_return_type_to_enum("paint", *signature, "Color"));
    second_return := signature.return_type;

    defer free(second_return);
    assert(first_return.type_of_enum == *color);
    integer := B.get_int_type(4, true);

    defer free(integer);
    assert(integer.size == 4 && integer.number_flags & .SIGNED);
    pointer := B.create_pointer_to(integer);

    defer free(pointer);
    assert(pointer.pointer_to == integer && pointer.size == size_of(*void));
    print("ok\n");
}
