// test: only(.LINUX,.MACOS)
// Every listed source is bound, also one that marks itself a system header, and also with other sources beside it.
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

count_occurrences :: (text: string, needle: string) -> int {
    count := 0;
    rest := text;

    while true {
        index := find_index_from_left(rest, needle);
        if index < 0 break;
        count += 1;
        rest = slice(rest, index + needle.count, rest.count - index - needle.count);
    }

    return count;
}

main :: () {
    one := generate(.["generator-sources-first.h"]);

    defer free(one);
    assert(contains(one, "FirstRecord :: struct"), "a single source marked as a system header is bound");

    both := generate(.["generator-sources-first.h", "generator-sources-second.h"]);

    defer free(both);
    assert(contains(both, "FirstRecord :: struct") && contains(both, "SecondRecord :: struct"), "both sources are bound");

    // `second.h` includes `first.h`: its record is bound once, ahead of the record that uses it.
    assert(count_occurrences(both, "FirstRecord :: struct") == 1, "a record included by a later source is bound once");
    assert(find_index_from_left(both, "FirstRecord :: struct") < find_index_from_left(both, "SecondRecord :: struct"), "declarations keep source order");

    reversed := generate(.["generator-sources-second.h", "generator-sources-first.h"]);

    defer free(reversed);
    assert(count_occurrences(reversed, "FirstRecord :: struct") == 1 && find_index_from_left(reversed, "FirstRecord :: struct") < find_index_from_left(reversed, "SecondRecord :: struct"), "the order follows the includes, not the list");

    // The first source reaches the second through a `..` spelling, so clang names the file after that spelling.
    spelled := generate(.["generator-sources-spelled.h", "generator-sources-late.h"]);

    defer free(spelled);
    assert(contains(spelled, "SpelledRecord :: struct") && contains(spelled, "LateRecord :: struct"), "a source reached by another spelling is bound");
}
