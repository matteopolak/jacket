#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);
    options: B.Generate_Bindings_Options;
    options.generate_library_declarations = false;
    options.log_stripped_declarations = false;
    array_add(*options.source_files, tprint("%bindings-native-defaults.hpp", #filepath));
    array_add(*options.extra_clang_arguments, "-x", "c++", "-std=c++17");
    assert(B.generate_bindings(options, args[1]));
    text, ok := read_entire_file(args[1]);

    assert(ok);
    defer free(text);
    assert(contains(text, "absent: *void = cast(*void) (null)"));
    assert(contains(text, "bytes: u64 = cast(u64) (8)"));
    assert(contains(text, "AbstractNative_VTable"));
    assert(contains(text, "Forward_VTable"));
    assert(contains(text, "Inner :: struct"));
    assert(contains(text, "slot_0:") && contains(text, "// C++ virtual: apply"));
    assert(!contains(text, "__UnknownLib") && !contains(text, "virtual_apply"));
}
