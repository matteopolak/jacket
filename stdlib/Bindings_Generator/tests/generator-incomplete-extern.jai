// test: only(.LINUX,.MACOS)
// An extern variable of an incomplete record has no layout to bind. It is left out, and the rest of the header is still emitted.
#import "Basic";
#import "String";
B :: #import "Bindings_Generator";

main :: () {
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%driven/generator-incomplete-extern.h", #filepath));
    defer array_free(options.source_files);
    options.add_generator_command = false;
    options.log_stripped_declarations = false;
    builder, success := B.generate_bindings(options);

    defer free_buffers(*builder);
    assert(success && !builder.failed, "generation fails on an incomplete extern");
    text := builder_to_string(*builder);

    defer free(text);
    assert(contains(text, "Hidden :: struct {}"), "the opaque record is emitted");
    assert(contains(text, "visible_count"), "a complete variable is emitted");
    assert(contains(text, "hidden_pointer: *Hidden") && contains(text, "hidden_table_pointer"), "pointers to the incomplete record are still emitted");
    assert(!contains(text, "hidden_class:") && !contains(text, "hidden_tag_class:") && !contains(text, "hidden_table:"), "incomplete variables are omitted");
}
