#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

prints: int;

will_print :: () {
    assert(context.generator.libraries.count == 3);
    primary := context.generator.libraries[0];
    archive := context.generator.libraries[1];
    system := context.generator.libraries[2];

    assert(primary.is_shared_library && !primary.is_system_library);
    assert(!archive.is_shared_library && !archive.is_system_library);
    assert(system.is_shared_library && system.is_system_library);
    archive.flags |= .LINK_ALWAYS;
    prints += 1;
}

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-canonical-libraries.h", #filepath));
    array_add(*options.system_include_paths, #filepath);
    array_add(*options.system_library_search_paths, ".");
    array_add(*options.library_search_paths, ".");
    array_add(*options.libraries, B.Library.{ "libdriver_primary", "primary" }, B.Library.{ "libdriver_static", "archive" });
    array_add(*options.system_libraries, B.Library.{ "libdriver_system", "system" });
    defer array_free(options.source_files);
    defer array_free(options.system_include_paths);
    defer array_free(options.system_library_search_paths);
    defer array_free(options.library_search_paths);
    defer array_free(options.libraries);
    defer array_free(options.system_libraries);
    options.add_generator_command = false;
    options.log_stripped_declarations = false;
    options.generated_library_path_prefix = "./";
    options.will_print_bindings = will_print;
    builder, success := B.generate_bindings(options);

    defer free_buffers(*builder);
    assert(success && prints == 1 && context.generator == null);
    text := builder_to_string(*builder, do_reset = false);

    defer free(text);
    assert(contains(text, "primary :: #library \"./"));
    assert(contains(text, "archive :: #library,no_dll,link_always \"./"));
    assert(contains(text, "system :: #library,system \"libdriver_system\";"));
    assert(!contains(text, ",shared") && !contains(text, ".dylib\"") && !contains(text, ".a\""));
    assert(write_entire_file(tprint("%/developer-tools-libraries-generated.jai", args[1]), text));
    print("ok\n");
}
