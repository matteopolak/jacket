// A function that returns a procedure is printed with the result in parentheses, so the directives that follow the
// declaration belong to it and not to the result, and the generated file compiles.
#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

main :: () {
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-procedure-results.h", #filepath));
    array_add(*options.extra_clang_arguments, "-x");
    array_add(*options.extra_clang_arguments, "c");
    array_add(*options.library_search_paths, tprint("%../../../../target", #filepath));
    array_add(*options.libraries, B.Library.{ "libdeveloper_tools_procedure_results", "native" });

    defer array_free(options.source_files);
    defer array_free(options.extra_clang_arguments);
    defer array_free(options.library_search_paths);
    defer array_free(options.libraries);
    options.add_generator_command = false;
    options.log_stripped_declarations = false;
    builder, success := B.generate_bindings(options);

    defer free_buffers(*builder);
    assert(success && !builder.failed);
    text := builder_to_string(*builder);

    defer free(text);

    assert(contains(text, "set_synchronize :: (display: *Display, enable: s32) -> ((unknown0: *Display) -> s32 #c_call) #c_call #foreign native;"));
    assert(contains(text, "set_after_function :: (display: *Display, callback: (unknown0: *Display) -> s32 #c_call) -> ((unknown0: *Display) -> s32 #c_call) #c_call #foreign native;"));
    assert(contains(text, "set_signal :: (number: s32, handler: (unknown0: s32) -> void #c_call) -> ((unknown0: s32) -> void #c_call) #c_call #foreign native;"));
    assert(contains(text, "set_error_handler :: (handler: ErrorHandler) -> ErrorHandler #c_call #foreign native;"));
    assert(!contains(text, "#c_call #c_call"));
    assert(write_entire_file(tprint("%../../../../target/developer-tools-procedure-results-generated.jai", #filepath), text));
    print("ok\n");
}
