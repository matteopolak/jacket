#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

will_print :: () {
    generator := context.generator;

    assert(generator.type_def_s128.hardcoded_jai_string == "S128" && generator.type_def_u128.hardcoded_jai_string == "U128");
    assert(generator.type_def_s128.size == 16 && generator.type_def_u128.size == 16);
    assert((generator.type_def_s128.number_flags & .SIGNED) != 0 && (generator.type_def_u128.number_flags & .SIGNED) == 0);
    signed := B.type_to_string(generator.type_def_s128, null);
    unsigned := B.type_to_string(generator.type_def_u128, null);

    defer free(signed);
    defer free(unsigned);
    assert(signed == "S128" && unsigned == "U128");
}

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-generic-record.hpp", #filepath), tprint("%generator-model-wide.hpp", #filepath));
    array_add(*options.extra_clang_arguments, "-x", "c++", "-std=c++17");
    defer array_free(options.source_files);
    defer array_free(options.extra_clang_arguments);
    options.will_print_bindings = will_print;
    options.add_generator_command = false;
    options.log_stripped_declarations = false;
    builder, success := B.generate_bindings(options);

    defer free_buffers(*builder);
    assert(success && !builder.failed);
    text := builder_to_string(*builder);

    defer free(text);
    assert(contains(text, "T: Type = s32") && contains(text, "count: s32 = cast(s32) (3)"));
    assert(contains(text, "S128") && contains(text, "U128"));
    assert(write_entire_file(tprint("%/generator-model-defaults-generated.jai", args[1]), text));
    print("ok\n");
}
