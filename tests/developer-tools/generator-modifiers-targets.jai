#import "Basic";
B :: #import "Bindings_Generator";
W :: #import "Bindings_Generator"(TARGET_OS = .WINDOWS);
I :: #import "Bindings_Generator"(TARGET_OS = .IOS);

main :: () {
    original_options := context._bindings_generator_options;
    original_generator := context.generator;
    windows_options: W.Generate_Bindings_Options;
    windows_generator: W.Generator;
    windows_result: W.CType;
    windows_record: W.Struct;
    windows_record.flags = .HAS_CONSTRUCTOR;
    windows_result.type_of_struct = *windows_record;
    windows_signature: W.Function_Type;
    windows_signature.return_type = *windows_result;
    builder: String_Builder;

    defer free_buffers(*builder);

    push_context {
        context.generator = *windows_generator;
        context._bindings_generator_options = *windows_options;
        W.append_calling_convention_directives(*builder, *windows_signature);
        result := builder_to_string(*builder);

        defer free(result);
        assert(result == " #c_call #cpp_return_type_is_non_pod");
        windows_type: W.CType;
        windows_type.type_of_function = *windows_signature;
        windows_function: W.Function;
        windows_function.type = *windows_type;
        windows_options.log_stripped_declarations = false;
        W.check_if_foreign_function_should_be_omitted(*windows_function);
        assert(windows_generator.counts.stripped == 1);
        assert(windows_function.decl_flags == .OMIT_FROM_OUTPUT);
    }

    ios_options: I.Generate_Bindings_Options;
    ios_result: I.CType;
    ios_record: I.Struct;
    ios_record.flags = .HAS_CONSTRUCTOR;
    ios_result.type_of_struct = *ios_record;
    ios_signature: I.Function_Type;
    ios_signature.return_type = *ios_result;

    push_context {
        context._bindings_generator_options = *ios_options;
        I.append_calling_convention_directives(*builder, *ios_signature);
        result := builder_to_string(*builder);

        defer free(result);
        assert(result == " #c_call");
        ios_record.flags = .HAS_DESTRUCTOR;
        I.append_calling_convention_directives(*builder, *ios_signature);
        next := builder_to_string(*builder);

        defer free(next);
        assert(next == " #c_call #cpp_return_type_is_non_pod");
    }

    assert(context._bindings_generator_options == original_options && context.generator == original_generator);
    type: B.CType;

    assert(B.shared_type(*type) == *type && type.flags == .SHARED);
    print("ok\n");
}
