// Reviewed VARIADIC model flag; raw C stdarg ABI, with printf wrappers disabled.
#import "Basic";
#import "File";
B :: #import "Bindings_Generator";

inspect :: (decl: *B.Declaration, parent: *B.Declaration) -> B.Declaration_Visit_Result {
    if decl.kind == .FUNCTION && decl.name == "original_variadic" {
        assert((decl.type.type_of_function.flags & .VARIADIC) != 0);
        assert(decl.type.type_of_function.arguments.count == 1);
        assert(decl.type.type_of_function.arguments[0].output_name == "varargs_0");
    }

    if decl.kind == .DECLARATION && parent && parent.kind == .FUNCTION && parent.name == "original_variadic_numbers" && decl.name == "count"
        decl.output_name = "number_count";

    return .RECURSE;
}

main :: () {
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-variadic.h", #filepath));
    defer array_free(options.source_files);
    array_add(*options.libraries, B.Library.{ identifier = "native", filename = "developer_tools_variadic" });
    defer array_free(options.libraries);
    array_add(*options.library_search_paths, tprint("%../../../../target", #filepath));
    defer array_free(options.library_search_paths);
    options.generate_printf_wrappers = false;
    options.add_generator_command = false;
    assert(B.generate_bindings(options, tprint("%../../../../target/developer-tools-variadic-default.jai", #filepath), log_results = false));
    options.visitor = inspect;
    assert(B.generate_bindings(options, tprint("%../../../../target/developer-tools-variadic-generated.jai", #filepath), log_results = false));
    integer: B.CType;
    integer.number_flags = .SIGNED | ._32BIT;
    integer.size = 4;
    integer.alignment = 4;
    argument: B.Declaration;
    argument.type = *integer;
    argument.output_name = "varargs_0";
    function: B.Function_Type;
    function.return_type = *integer;
    function.flags = .VARIADIC;
    array_add(*function.arguments, *argument);
    defer array_free(function.arguments);
    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, "Signature :: #type ");
    B.print_function_type_to_builder(*builder, *function, null);
    append(*builder, ";\n");
    assert(!builder.failed);
    text := builder_to_string(*builder);

    defer free(text);
    assert(write_entire_file(tprint("%../../../../target/developer-tools-variadic-signature.jai", #filepath), text));
    print("ok\n");
}
