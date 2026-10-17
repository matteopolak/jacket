// Unnamed record members become `using` members, a plain `char` is an unsigned byte
// while `signed char` stays signed, and options without any source file generate nothing.
#import "Basic";
#import "String";
B :: #import "Bindings_Generator";

main :: () {
    options: B.Generate_Bindings_Options;

    array_add(*options.system_include_paths, ..B.get_default_system_include_paths());
    array_add(*options.source_files, tprint("%bindings-anonymous-members.h", #filepath));
    defer array_free(options.source_files);
    options.generate_library_declarations = false;
    options.add_generator_command = false;

    output, ok := B.generate_bindings(options);

    assert(ok);

    text := builder_to_string(*output);

    assert(contains(text, "using anonymous_0: union"));
    assert(contains(text, "using anonymous_1: struct"));
    assert(contains(text, "tag: u8;"));
    assert(contains(text, "delta: s8;"));

    empty: B.Generate_Bindings_Options;
    nothing, nothing_ok := B.generate_bindings(empty);

    assert(nothing_ok);
    assert(!contains(builder_to_string(*nothing), "::"));

    print("ok\n");
}
