// Writes the bindings for bindings-output-format.hpp, a header of bitfield records and C++ interfaces, so a checker
// can compare the text with bindings-output-format-expected.txt.
#import "Basic";
#import "File";
B :: #import "Bindings_Generator";

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-output-format.hpp", #filepath));
    array_add(*options.extra_clang_arguments, "-x", "c++");
    defer array_free(options.source_files);
    defer array_free(options.extra_clang_arguments);
    options.add_generator_command = false;
    options.log_stripped_declarations = false;
    builder, success := B.generate_bindings(options);

    defer free_buffers(*builder);
    assert(success && !builder.failed);
    assert(write_entire_file(args[1], builder_to_string(*builder)));
    print("ok\n");
}
