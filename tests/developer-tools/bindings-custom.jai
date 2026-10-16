#import "Basic";
#import "String";
#import "File";
B :: #import "Bindings_Generator";

visits: s64;
arguments: s64;
printed_arguments: s64;
errors: s64;
replacement_types: [..] *B.CType;

capture_error :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

invalidate :: (decl: *B.Declaration, _parent: *B.Declaration) -> B.Declaration_Visit_Result {
    if decl.kind == .FUNCTION decl.type = null;
    return .STOP;
}

invalidate_variadic :: (decl: *B.Declaration, _parent: *B.Declaration) -> B.Declaration_Visit_Result {
    if decl.kind == .FUNCTION decl.type.type_of_function.flags |= .VARIADIC;
    return .STOP;
}

get_arguments :: (func: *B.Function, type: *B.Function_Type, mode: B.Arg_Print_Mode) -> [] *B.Declaration {
    assert(func != null && mode == .FOREIGN_FUNC_DECLARATION);
    printed_arguments += type.arguments.count;
    return type.arguments;
}

visit_with_data :: (_decl: *B.Declaration, _parent: *B.Declaration, data: *void) -> B.Declaration_Visit_Result {
    count := cast(*s64) data;
    count.* += 1;
    return .RECURSE;
}

customize :: (decl: *B.Declaration, parent: *B.Declaration) -> B.Declaration_Visit_Result {
    visits += 1;
    if decl.kind == .FUNCTION && decl.name == "callback_apply" decl.output_name = "call_callback";

    if parent && parent.kind == .FUNCTION {
        arguments += 1;

        if decl.name == "value" {
            decl.output_name = "number";
            assert(B.change_type_to_enum(decl, "Mode"));
            array_add(*replacement_types, decl.type);
        }
    }

    if decl.kind == .STRUCT && decl.name == "Sample" {
        assert(decl.type.size == 32);
        decl.output_name = "Sample_Record";
    }

    return .RECURSE;
}

main :: () {
    defer {
        for type: replacement_types free(type);
        array_free(replacement_types);
    }

    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-input.h", #filepath));
    defer array_free(options.source_files);
    array_add(*options.libraries, B.Library.{ filename = "developer_tools_bindings", identifier = "native" });
    defer array_free(options.libraries);
    array_add(*options.library_search_paths, tprint("%../../target", #filepath));
    defer array_free(options.library_search_paths);
    options.visitor = customize;
    count: s64;

    options.visitor_with_data = visit_with_data;
    options.visitor_data = *count;
    options.get_func_args_for_printing = get_arguments;
    options.add_generator_command = false;
    builder, ok := B.generate_bindings(options);

    defer free_buffers(*builder);
    assert(ok && visits >= 9 && arguments == 4 && count == visits && printed_arguments == 4);
    text := builder_to_string(*builder);

    defer free(text);
    assert(contains(text, "call_callback ::") && contains(text, "Sample_Record :: struct"));
    assert(contains(text, "#foreign native \"callback_apply\""));
    assert(B.generate_bindings(options, tprint("%../../target/developer-tools-custom-generated.jai", #filepath), log_results = false));
    options.source_files[0] = tprint("%bindings-model-input.h", #filepath);
    options.visitor = null;
    options.get_func_args_for_printing = B._default_get_func_args_for_printing;
    assert(B.generate_bindings(options, tprint("%../../target/developer-tools-model-generated.jai", #filepath), log_results = false));
    options.source_files[0] = tprint("%bindings-input.h", #filepath);
    path := tprint("%../../target/developer-tools-custom-rejected.jai", #filepath);

    assert(write_entire_file(path, "existing output"));
    options.visitor = invalidate;
    options.visitor_with_data = null;

    push_context {
        context.logger = capture_error;
        assert(!B.generate_bindings(options, path, log_results = false));
        options.visitor = invalidate_variadic;
        assert(!B.generate_bindings(options, path, log_results = false));
    }

    preserved, preserved_ok := read_entire_file(path);

    defer free(preserved);
    assert(errors == 2 && preserved_ok && preserved == "existing output");
    print("ok\n");
}
