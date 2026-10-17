#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-generic-record.hpp", #filepath));
    array_add(*options.extra_clang_arguments, "-x", "c++", "-std=c++17");
    defer array_free(options.source_files);
    defer array_free(options.extra_clang_arguments);
    options.add_generator_command = false;
    options.log_stripped_declarations = false;
    options.generate_compile_time_struct_checks = true;
    builder, success := B.generate_bindings(options);

    defer free_buffers(*builder);
    assert(success && !builder.failed);
    text := builder_to_string(*builder);

    defer free(text);
    assert(contains(text, "IndependentGeneric :: struct(T: Type = s32, count: s32 = cast(s32) (3))"));
    assert(!contains(text, "#assert") && !contains(text, "#align"));
    consumer := tprint("#import \"Basic\";\n%\n#scope_export\nnative :: #library \"generic_fixture\";\nDefault :: IndependentGeneric();\nBytes :: IndependentGeneric(u8, 7);\noriginal_generic_default :: (value: *Default) -> s32 #foreign native;\noriginal_generic_bytes :: (value: *Bytes) -> s32 #foreign native;\nmain :: () { a: IndependentGeneric(); b: IndependentGeneric(u8, 7); a.data[2] = 42; b.data[6] = 17; assert(a.data.count == 3 && a.data[2] == 42 && b.data.count == 7 && b.data[6] == 17); a.data[0] = 10; assert(original_generic_default(*a) == size_of(Default) && a.data[0] == 17 && a.data[2] == 42); assert(original_generic_bytes(*b) == size_of(Bytes) && b.data[6] == 42); print(\"ok\\n\"); }\n", text);

    assert(write_entire_file(args[1], consumer));
    print("ok\n");
}
