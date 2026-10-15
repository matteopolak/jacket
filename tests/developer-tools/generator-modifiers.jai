#import "Basic";
using B :: #import "Bindings_Generator";

Capture :: struct {
    saved_context: #Context;
    fields: int;
    public_fields: int;
    protected_fields: int;
    private_fields: int;
}

visit :: (cur: CXCursor, _parent: CXCursor, data: CXClientData) -> CXChildVisitResult #c_call {
    if cur.kind != .FieldDecl return .Recurse;

    state := cast(*Capture) data;

    push_context {
        context = state.saved_context;
        decl: Declaration;
        decl.decl_flags = .DEPRECATED;
        add_access_flags(*decl, cur);
        access := clang_getCXXAccessSpecifier(cur);

        assert((decl.decl_flags & .DEPRECATED) != 0);

        if access == .CXXPublic {
            assert(!(decl.decl_flags & (.PRIVATE | .PROTECTED)));
            state.public_fields += 1;
        } else if access == .CXXProtected {
            assert((decl.decl_flags & .PROTECTED) != 0);
            assert(!(decl.decl_flags & .PRIVATE));
            state.protected_fields += 1;
        } else if access == .CXXPrivate {
            assert((decl.decl_flags & .PRIVATE) != 0);
            assert(!(decl.decl_flags & .PROTECTED));
            state.private_fields += 1;
        } else assert(false);

        old := decl.decl_flags;

        add_access_flags(*decl, cur);
        assert(decl.decl_flags == old);
        state.fields += 1;
    }

    return .Continue;
}

main :: () {
    test_copy_on_write();
    test_directives();
    test_policies();
    signature: Function_Type;
    type: CType;
    type.type_of_function = *signature;
    function: Function;
    function.type = *type;
    assert(!function_has_a_valist_arg(*function));
    integer: CType;
    integer.number_flags = .SIGNED | ._32BIT;
    integer.size = 4;
    ordinary: Declaration;
    ordinary.type = *integer;
    array_add(*signature.arguments, *ordinary);
    defer array_free(signature.arguments);
    assert(!function_has_a_valist_arg(*function));
    va_alias: Typedef;
    va_alias.name = "va_list";
    va_type: CType;
    va_type.type_of_typedef = *va_alias;
    argument: Declaration;
    argument.type = *va_type;
    array_add(*signature.arguments, *argument);
    assert(function_has_a_valist_arg(*function));
    assert(signature.arguments.count == 2 && argument.type == *va_type);
    va_alias.name = "__builtin_va_list";
    assert(function_has_a_valist_arg(*function));
    va_alias.name = "__gnuc_va_list";
    assert(function_has_a_valist_arg(*function));
    va_alias.name = "other_list";
    assert(!function_has_a_valist_arg(*function));
    signature.flags = .VARIADIC;
    assert(!function_has_a_valist_arg(*function));
    va_alias.name = "va_list";
    signature.return_type = *va_type;
    signature.arguments.count = 1;
    assert(!function_has_a_valist_arg(*function));
    // Matching parameters at every position, including more than one match.
    signature.arguments[0] = *argument;
    assert(function_has_a_valist_arg(*function));
    array_add(*signature.arguments, *ordinary);
    array_add(*signature.arguments, *argument);
    assert(function_has_a_valist_arg(*function));
    signature.arguments[0] = *ordinary;
    signature.arguments[1] = *argument;
    signature.arguments[2] = *ordinary;
    assert(function_has_a_valist_arg(*function));
    assert(signature.arguments.count == 3);
    assert(signature.arguments[0] == *ordinary && signature.arguments[1] == *argument && signature.arguments[2] == *ordinary);
    assert(argument.type == *va_type && ordinary.type == *integer);

    index := clang_createIndex(0, 0);

    assert(index != null);
    defer clang_disposeIndex(index);
    filename := to_c_string("modifiers-original.cpp");

    defer free(filename);
    source := "class Sample { int implicit_private; public: int exposed; protected: int guarded; private: int hidden; }; struct Other { int implicit_public; }; int global;";
    unsaved: CXUnsavedFile;
    unsaved.Filename = filename;
    unsaved.Contents = source.data;
    unsaved.Length = cast(type_of(unsaved.Length)) source.count;
    unit: CXTranslationUnit;
    status := clang_parseTranslationUnit2(index, filename, null, 0, *unsaved, 1, 0, *unit);

    assert(status == .Success && unit != null);
    defer clang_disposeTranslationUnit(unit);
    assert(clang_getNumDiagnostics(unit) == 0);
    state: Capture;
    state.saved_context = context;
    clang_visitChildren(clang_getTranslationUnitCursor(unit), visit, *state);
    assert(state.fields == 5 && state.public_fields == 2 && state.protected_fields == 1 && state.private_fields == 2);
    decl: Declaration;
    decl.decl_flags = .IS_CONST;
    add_access_flags(*decl, clang_getNullCursor());
    assert(decl.decl_flags == .IS_CONST);
    print("ok\n");
}

test_copy_on_write :: () {
    child: CType;
    expression: Expression;
    original: CType;
    original.pointer_to = *child;
    original.comment = "kept";
    original.size = 8;
    original.alignment = 8;
    array_add(*original.template_instantiation_params, *expression);
    defer array_free(original.template_instantiation_params);
    same := add_type_flags(*original, .CONST);

    assert(same == *original && original.flags == .CONST);
    assert(shared_type(*original) == *original);
    assert(original.flags == (.CONST | .SHARED));
    copied := add_type_flags(*original, .VOLATILE);

    defer free(copied);
    assert(copied != *original && copied.flags == (.CONST | .VOLATILE));
    assert(original.flags == (.CONST | .SHARED));
    assert(copied.pointer_to == *child && copied.comment == "kept");
    assert(copied.size == 8 && copied.alignment == 8);
    assert(copied.template_instantiation_params.data == original.template_instantiation_params.data);
    copied_again := add_type_flags(*original, .SHARED | .REFERENCE);

    defer free(copied_again);
    assert(copied_again != *original && copied_again.flags == (.CONST | .SHARED | .REFERENCE));
    unchanged_flags := add_type_flags(*original, 0);

    defer free(unchanged_flags);
    assert(unchanged_flags != *original && unchanged_flags.flags == .CONST);
    assert(add_type_flags(copied, .CONST) == copied);
}

expect_text :: (builder: *String_Builder, expected: string) {
    text := builder_to_string(builder);
    defer free(text);
    assert(text == expected, "expected %, got %", expected, text);
}

test_directives :: () {
    for target: Operating_System_Tag.[.WINDOWS, .MACOS, .LINUX, .IOS] {
        options: Generate_Bindings_Options;
        options.os = target;

        push_context {
            context._bindings_generator_options = *options;
            test_directives_for_target(target);
        }
    }
}

test_directives_for_target :: (target: Operating_System_Tag) {
    builder: String_Builder;

    defer free_buffers(*builder);
    scalar: CType;
    scalar.flags = .VOID;
    signature: Function_Type;
    signature.return_type = *scalar;
    append_calling_convention_directives(*builder, *signature);
    expect_text(*builder, " #c_call");
    append_calling_convention_directives(*builder, *signature, assume_foreign = true);
    expect_text(*builder, "");
    signature.flags = .CPP_METHOD | .VARIADIC;
    append_calling_convention_directives(*builder, *signature, assume_foreign = true);
    expect_text(*builder, " #cpp_method");
    record: Struct;

    scalar.flags = 0;
    scalar.type_of_struct = *record;
    record.flags = .HAS_CONSTRUCTOR;
    append_calling_convention_directives(*builder, *signature);
    if target == .WINDOWS expect_text(*builder, " #cpp_method #cpp_return_type_is_non_pod");
    else expect_text(*builder, " #cpp_method");

    record.flags = .HAS_COPY_CONSTRUCTOR;
    alias: Typedef;
    alias.type = *scalar;
    alias_type: CType;
    alias_type.type_of_typedef = *alias;
    signature.return_type = *alias_type;
    append_calling_convention_directives(*builder, *signature);
    if target == .WINDOWS expect_text(*builder, " #cpp_method");
    else expect_text(*builder, " #cpp_method #cpp_return_type_is_non_pod");

    record.flags = .HAS_MOVE_CONSTRUCTOR;
    append_calling_convention_directives(*builder, *signature, true);
    if target == .WINDOWS expect_text(*builder, " #cpp_method");
    else expect_text(*builder, " #cpp_method #cpp_return_type_is_non_pod");

    record.flags = .HAS_DESTRUCTOR;
    signature.flags = .OBJC_METHOD | .CPP_METHOD;
    append_calling_convention_directives(*builder, *signature);
    expect_text(*builder, "");
    signature.flags = 0;
    append_calling_convention_directives(*builder, *signature, true);
    if target == .WINDOWS expect_text(*builder, "");
    else expect_text(*builder, " #cpp_return_type_is_non_pod");

    decl: Declaration;

    append_type_info_directives(*builder, *decl);
    expect_text(*builder, "");
    decl.decl_flags = .TYPE_INFO_NONE | .TYPE_INFO_PROCEDURES_ARE_VOID_POINTERS;
    append_type_info_directives(*builder, *decl);
    expect_text(*builder, " #type_info_none");
    decl.decl_flags = .TYPE_INFO_PROCEDURES_ARE_VOID_POINTERS | .DEPRECATED;
    append_type_info_directives(*builder, *decl);
    expect_text(*builder, " #type_info_procedures_are_void_pointers");
}

policy_logs: int;
last_policy: string;

capture_policy :: (text: string, _data: *void, _info: Log_Info) {
    free(last_policy);
    last_policy = copy_string(text);
    policy_logs += 1;
}

test_policies :: () {
    generator: Generator;
    options: Generate_Bindings_Options;

    defer free(last_policy);

    push_context {
        context.generator = *generator;
        context._bindings_generator_options = *options;
        options.strip_flags = 0;
        options.log_stripped_declarations = false;
        context.logger = capture_policy;
        type: CType;
        signature: Function_Type;

        type.type_of_function = *signature;
        function: Function;
        function.kind = .FUNCTION;
        function.type = *type;
        function.name = "sample";
        check_if_foreign_function_should_be_omitted(*function);
        assert(function.decl_flags == 0 && generator.counts.stripped == 0);
        options.strip_flags = .SYMBOLS_WITH_UNKNOWN_FOREIGN_LIBS | .INLINED_FUNCTIONS;
        function.flags = .VIRTUAL;
        check_if_foreign_function_should_be_omitted(*function);
        assert(function.decl_flags == 0 && generator.counts.stripped == 0);
        signature.flags = .INLINE;
        check_if_foreign_function_should_be_omitted(*function);
        assert(function.decl_flags == .OMIT_FROM_OUTPUT && generator.counts.stripped == 1);
        assert(policy_logs == 0);
        function.flags = 0;
        function.decl_flags = .DEPRECATED;
        signature.flags = .OBJC_METHOD;
        check_if_foreign_function_should_be_omitted(*function);
        assert(function.decl_flags == .DEPRECATED && generator.counts.stripped == 1);
        signature.flags = 0;
        options.log_stripped_declarations = true;
        check_if_foreign_function_should_be_omitted(*function);
        assert(function.decl_flags == (.DEPRECATED | .OMIT_FROM_OUTPUT));
        assert(generator.counts.stripped == 2 && policy_logs == 1);
        assert(S.contains(last_policy, "missing foreign library"));
        library: Library_Info;

        function.library = *library;
        function.decl_flags = 0;
        check_if_foreign_function_should_be_omitted(*function);
        assert(function.decl_flags == 0 && generator.counts.stripped == 2);
        alias: Typedef;
        alias.name = "va_list";
        argument_type: CType;
        argument_type.type_of_typedef = *alias;
        argument: Declaration;
        argument.type = *argument_type;
        array_add(*signature.arguments, *argument);
        defer array_free(signature.arguments);
        options.strip_flags = .FUNCTIONS_WITH_VALIST | .SYMBOLS_WITH_UNKNOWN_FOREIGN_LIBS | .INLINED_FUNCTIONS;
        signature.flags = .INLINE;
        function.library = null;
        check_if_foreign_function_should_be_omitted(*function);
        assert(generator.counts.stripped == 3 && policy_logs == 2);
        assert(function.decl_flags == .OMIT_FROM_OUTPUT);
        assert(S.contains(last_policy, "va_list"));
        // Counting is per matching invocation, with only the first policy used.
        check_if_foreign_function_should_be_omitted(*function);
        assert(generator.counts.stripped == 4 && policy_logs == 3);
        assert(S.contains(last_policy, "va_list"));
        signature.arguments.count = 0;
        check_if_foreign_function_should_be_omitted(*function);
        assert(generator.counts.stripped == 5 && policy_logs == 4);
        assert(S.contains(last_policy, "missing foreign library"));
        function.flags = .PURE_VIRTUAL;
        check_if_foreign_function_should_be_omitted(*function);
        assert(generator.counts.stripped == 6 && policy_logs == 5);
        assert(S.contains(last_policy, "inline function"));
        function.flags = 0;
        signature.flags = .OBJC_METHOD | .INLINE;
        check_if_foreign_function_should_be_omitted(*function);
        assert(generator.counts.stripped == 7 && policy_logs == 6);
        assert(S.contains(last_policy, "inline function"));
        nested_generator: Generator;
        nested_options: Generate_Bindings_Options;
        nested_options.strip_flags = .INLINED_FUNCTIONS;
        nested_options.log_stripped_declarations = false;

        push_context {
            context.generator = *nested_generator;
            context._bindings_generator_options = *nested_options;
            check_if_foreign_function_should_be_omitted(*function);
            assert(nested_generator.counts.stripped == 1);
        }

        assert(context.generator == *generator && context._bindings_generator_options == *options);
        assert(generator.counts.stripped == 7 && policy_logs == 6);
        assert(!should_be_moved_to_global_scope(*function));
        function.name = "operator+";
        assert(should_be_moved_to_global_scope(*function));
        function.name = "sample";
        options.move_struct_functions_to_global_scope = true;
        assert(should_be_moved_to_global_scope(*function));
        decl: Declaration;
        decl.kind = .DECLARATION;
        assert(!should_be_moved_to_global_scope(*decl));
        decl.storage_class = .Extern;
        assert(should_be_moved_to_global_scope(*decl));
        decl.storage_class = .Static;
        assert(!should_be_moved_to_global_scope(*decl));
    }
}

#scope_file
S :: #import "String";
