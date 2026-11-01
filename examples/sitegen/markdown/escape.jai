// HTML escaping, and the id a heading gets.

// Appends `text` with the characters HTML treats as markup written as entities.
append_escaped :: (out: *String_Builder, text: string) {
    for c: text {
        if c == {
            case #char "<"; append(out, "&lt;");
            case #char ">"; append(out, "&gt;");
            case #char "&"; append(out, "&amp;");
            case #char "\""; append(out, "&quot;");
            case; append(out, c);
        }
    }
}

// `Hello, World!` becomes `hello-world`: lower case letters and digits, one dash for each run of
// anything else. The result is in temporary storage.
slugify :: (text: string) -> string {
    out: String_Builder;
    pending_dash := false;

    for text {
        c := it;

        if c >= #char "A" && c <= #char "Z" c += 32;

        if is_word_character(c) {
            if pending_dash && builder_string_length(*out) > 0 append(*out, #char "-");

            pending_dash = false;
            append(*out, c);
        } else {
            pending_dash = true;
        }
    }

    return builder_to_string(*out,, temp);
}

is_ascii_punctuation :: (c: u8) -> bool {
    return (c >= 33 && c <= 47) || (c >= 58 && c <= 64) || (c >= 91 && c <= 96) || (c >= 123 && c <= 126);
}

is_word_character :: (c: u8) -> bool {
    return (c >= #char "a" && c <= #char "z") || (c >= #char "A" && c <= #char "Z") || (c >= #char "0" && c <= #char "9");
}
