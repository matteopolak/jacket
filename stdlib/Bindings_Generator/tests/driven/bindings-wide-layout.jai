#import "Basic";
#import "File";
B :: #import "Bindings_Generator";

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-wide-layout.h", #filepath));
    defer array_free(options.source_files);
    options.add_generator_command = false;
    options.log_stripped_declarations = false;
    builder, success := B.generate_bindings(options);

    defer free_buffers(*builder);
    assert(success && !builder.failed);
    text := builder_to_string(*builder);

    defer free(text);
    assert(write_entire_file(args[1], tprint("#import \"Basic\";\n%\n#scope_export\nnative :: #library \"wide_fixture\";\noriginal_wide_exchange :: (value: *IndependentWide) -> s32 #foreign native;\nmain :: () { value: IndependentWide; value.signed_value.low = 42; value.signed_value.high = 1; value.unsigned_value.high = 17; assert(original_wide_exchange(*value) == 1); assert(value.tag == 11 && value.tail == 9 && value.signed_value.low == 25 && value.signed_value.high == 2 && value.unsigned_value.low == 7 && value.unsigned_value.high == 3); print(\"ok\\n\"); }\n", text)));
    print("ok\n");
}
