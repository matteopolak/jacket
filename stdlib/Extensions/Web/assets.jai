// Eight lowercase hex digits: the high half of FNV-1a 64, matching API-site asset URLs.
// The result is in temporary storage.
// ```jai
// assert(content_stamp("") == "cbf29ce4");
// ```
content_stamp :: (data: string) -> string {
    hash: u64 = 0xcbf29ce484222325;
    for data hash = (hash ^ cast(u64) it) * 0x100000001b3;
    return tprint("%", formatInt(hash >> 32, base = 16, minimum_digits = 8));
} @TempAllocates

// Stamps a byte slice, including embedded zero bytes.
// ```jai
// data := u8.[104, 101, 108, 108, 111];
// assert(content_stamp(data) == "a430d846");
// ```
content_stamp :: (data: [] u8) -> string {
    return content_stamp(cast(string) data);
} @TempAllocates

// Removes CSS block comments and blank/edge whitespace, preserving quoted strings.
// The result is in temporary storage; comments become spaces to keep tokens separate.
// ```jai
// assert(compact_css("  a { color: red; } /* note */\n") == "a { color: red; }\n");
// ```
compact_css :: (text: string) -> string {
    b: String_Builder;
    quote: u8;
    i := 0;

    while i < text.count {
        c := text[i];

        if quote != 0 {
            B.append(*b, c);

            if c == #char "\\" && i + 1 < text.count {
                i += 1;
                B.append(*b, text[i]);
            } else if c == quote quote = 0;
        } else if c == #char "\"" || c == #char "'" {
            quote = c;
            B.append(*b, c);
        } else if c == #char "/" && i + 1 < text.count && text[i + 1] == #char "*" {
            i += 2;
            while i + 1 < text.count && !(text[i] == #char "*" && text[i + 1] == #char "/") i += 1;
            i += 1;
            B.append(*b, #char " ");
        } else B.append(*b, c);

        i += 1;
    }

    return compact_lines(builder_to_string(*b,, temp), false);
} @TempAllocates

// Drops indentation, blank lines and whole-line // comments, preserving other JS bytes.
// Multiline strings, template literals and block comments are kept verbatim internally.
// ```jai
// assert(compact_js("  // note\n  let url = 'https://x';\n") == "let url = 'https://x';\n");
// ```
compact_js :: (text: string) -> string {
    return compact_lines(text, true);
} @TempAllocates

#scope_module

// Only lexical state that can continue onto the next line matters here.
compact_lines :: (text: string, js: bool) -> string {
    b: String_Builder;
    quote: u8;
    block := false;
    lines := split(text, "\n");

    defer array_free(lines);

    for line: lines {
        protected := quote != 0 || block;
        cleaned := line;

        if !protected {
            while cleaned.count && space(cleaned[0]) cleaned = advance(cleaned, 1);
            if cleaned.count == 0 || (js && begins_with(cleaned, "//")) continue;
        }

        i := 0;
        previous: u8 = #char "=";

        while i < line.count {
            c := line[i];
            next: u8;

            if i + 1 < line.count next = line[i + 1];

            if quote != 0 {
                if c == #char "\\" {
                    i += 2;
                    continue;
                }

                if c == quote quote = 0;
            } else if block {
                if c == #char "*" && next == #char "/" {
                    block = false;
                    i += 1;
                }
            } else if js && c == #char "/" && next == #char "/" {
                break;
            } else if js && c == #char "/" && next == #char "*" {
                block = true;
                i += 1;
            } else if js && c == #char "/" && (regex_prefix(previous) || regex_keyword(slice(line, 0, i))) {
                // Skip a single-line regex, including quoted bytes and // inside a class.
                j := i + 1;
                in_class := false;

                while j < line.count {
                    r := line[j];
                    if r == #char "\\" {
                        j += 2;
                        continue;
                    }

                    if r == #char "[" in_class = true;
                    else if r == #char "]" in_class = false;
                    else if r == #char "/" && !in_class {
                        i = j;
                        break;
                    }

                    j += 1;
                }
            } else if c == #char "\"" || c == #char "'" || (js && c == #char "`") quote = c;

            if !space(c) previous = c;

            i += 1;
        }

        // JS promises to keep trailing bytes. CSS trims only outside strings.
        if !js && quote == 0 {
            while cleaned.count && space(cleaned[cleaned.count - 1]) cleaned = slice(cleaned, 0, cleaned.count - 1);
        }

        if cleaned.count == 0 && !protected continue;

        B.append(*b, cleaned);
        B.append(*b, #char "\n");
    }

    return builder_to_string(*b,, temp);
} @TempAllocates

regex_prefix :: (c: u8) -> bool {
    for byte: "=(:,[!&|?;{}" if c == byte return true;
    return false;
}

regex_keyword :: (prefix: string) -> bool {
    before := trim(prefix);
    return ends_with(before, "return") || ends_with(before, "throw") || ends_with(before, "yield");
}
