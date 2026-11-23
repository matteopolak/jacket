#import "Basic";

// Turns a title into a URL slug: lowercase words joined by `-`.
//
// ```jai
// assert(slugify("Hello, World!") == "hello-world");
// ```
//
// - title: the text to convert.
slugify :: (title: string) -> string {
    builder: String_Builder;
    pending_dash := false;

    for title {
        c := it;
        if c >= #char "A" && c <= #char "Z" c += 32;

        if (c >= #char "a" && c <= #char "z") || (c >= #char "0" && c <= #char "9") {
            if pending_dash && builder_string_length(*builder) > 0 append(*builder, "-");
            pending_dash = false;
            append(*builder, string.{ 1, *c });
        } else {
            pending_dash = true;
        }
    }

    return builder_to_string(*builder);
}
