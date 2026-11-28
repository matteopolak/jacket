// test: only(.LINUX,.MACOS)
// A source bound beside others keeps the comments on its members, as it does alone.
#import "Basic";
#import "String";
B :: #import "Bindings_Generator";

generate :: (names: [] string) -> string {
    options: B.Generate_Bindings_Options;

    for name: names array_add(*options.source_files, tprint("%driven/%", #filepath, name));

    defer array_free(options.source_files);
    options.add_generator_command = false;
    options.log_stripped_declarations = false;
    builder, success := B.generate_bindings(options);

    defer free_buffers(*builder);
    assert(success && !builder.failed, "generation fails");

    return builder_to_string(*builder);
}

check :: (text: string, label: string) {
    for needle: string.["A point on the screen.", "Distance from the left edge.", "Distance from the top edge.", "Where the top left corner is."] {
        if label == "first" && it_index == 3 continue;
        assert(contains(text, needle), tprint("% source: comment '%' is kept", label, needle));
    }
}

main :: () {
    alone := generate(.["generator-comments-first.h"]);

    defer free(alone);
    check(alone, "first");

    together := generate(.["generator-comments-first.h", "generator-comments-second.h"]);

    defer free(together);
    check(together, "both");
}
