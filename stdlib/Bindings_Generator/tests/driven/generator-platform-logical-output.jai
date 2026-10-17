#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

main :: () {
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%generator-platform-logical.h", #filepath));
    defer array_free(options.source_files);
    array_add(*options.libraries, .{ filename = "liboriginal_resolved", identifier = "Native" });
    defer array_free(options.libraries);
    options.generated_library_path_prefix = ".";
    options.add_generator_command = false;
    options.generate_compile_time_struct_checks = false;
    builder, ok := B.generate_bindings(options);

    assert(ok && !builder.failed);
    defer free_buffers(*builder);
    text := builder_to_string(*builder, do_reset = false);

    defer free(text);
    assert(contains(text, "Native :: #library \"./liboriginal_resolved\";"));
    assert(!contains(text, ",shared") && !contains(text, ",no_dll"));
    assert(write_entire_file(tprint("%generator-platform-logical-generated.jai", #filepath), text));

    options.libraries[0].filename = "./provider/images/original_archive.a";
    static_builder, static_ok := B.generate_bindings(options);

    assert(static_ok && !static_builder.failed);
    defer free_buffers(*static_builder);
    static_text := builder_to_string(*static_builder, do_reset = false);

    defer free(static_text);
    assert(contains(static_text, "Native :: #library,no_dll \"././provider/images/original_archive\";"));

    options.libraries.count = 0;
    array_add(*options.system_libraries, .{ filename = "./liboriginal_resolved.dylib", identifier = "System" });
    defer array_free(options.system_libraries);
    options.generated_library_path_prefix = "./unused-prefix/";
    system_builder, system_ok := B.generate_bindings(options);

    assert(system_ok && !system_builder.failed);
    defer free_buffers(*system_builder);
    system_text := builder_to_string(*system_builder, do_reset = false);

    defer free(system_text);
    assert(contains(system_text, "System :: #library,system \"liboriginal_resolved\";"));
    assert(!contains(system_text, "unused-prefix"));
    print("ok\n");
}
