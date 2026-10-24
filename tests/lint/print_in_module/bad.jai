#import "Basic";

parse :: (text: string) -> int {
    print("parsing %\n", text);
    return text.count;
}

banner :: () {
    write_string("banner\n");
    write_strings("a", "b", "\n");
}

dump :: (builder: *String_Builder) {
    write_builder(builder);
}

warn :: (text: string) {
    print("warning: %\n", text, to_standard_error = true);
}
