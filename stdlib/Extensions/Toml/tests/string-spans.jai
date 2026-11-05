#import "Basic";
#import "String";
#import "Extensions/Toml";

check :: (input: string, expected: string) {
    doc, error := parse_toml(input);

    assert(error.kind == .NONE, error.message);
    value := toml_get(doc.root, "x");

    assert(value && value.kind == .STRING && value.text == expected, input);
    written, write_error := write_toml(doc.root);

    assert(write_error.kind == .NONE);
    again, again_error := parse_toml(written);

    assert(again_error.kind == .NONE && toml_get(again.root, "x").text == expected);
    free(written);
    free_toml(*again);
    free_toml(*doc);
}

main :: () {
    check("x = \"plain words\"", "plain words");
    check("x = \"a\\tb\\n\\\"c\\\\d\"", "a\tb\n\"c\\d");
    check("x = 'a\\b'", "a\\b");
    check("x = \"a£b λ\"", "a£b λ");
    check("x = \"\"\"\r\nalpha \"\" beta\r\nomega\"\"\"\"", "alpha \"\" beta\nomega\"");
    check("x = '''\n''a'\r\nb''''", "''a'\nb'");
    check("x = \"\"\"a \\\r\n \t b\"\"\"", "a b");
    check("x = \"\\u00a3 and \\U0001f642\"", "£ and 🙂");

    for limit: 0..4 {
        doc, error := parse_toml("\"\" = \"abcde\"", .{ max_string_bytes = limit });
        assert(error.kind == .LIMIT && error.span.start == 7 + limit);
        free_toml(*doc);
    }

    doc, error := parse_toml("\"abcd\" = 1", .{ max_string_bytes = 3 });

    assert(error.kind == .LIMIT && error.span.start == 5);
    free_toml(*doc);
    unicode, unicode_error := parse_toml("x = \"a£b\"", .{ max_string_bytes = 3 });

    assert(unicode_error.kind == .LIMIT && unicode_error.span.start == 9);
    free_toml(*unicode);
    check_decoded_limits();
    // The builder's first buffer is inline; exceed it to test redundant heap scratch.
    bytes: [8192] u8;
    for *bytes it.* = 97;

    expected := string.{ bytes.count, bytes.data };
    plain_allocations := parse_allocations(tprint("x = \"%\"", expected), expected);
    escaped_allocations := parse_allocations(tprint("x = \"\\u0061%\"", slice(expected, 1, expected.count - 1)), expected);

    assert(plain_allocations < escaped_allocations, tprint("plain=% escaped=%", plain_allocations, escaped_allocations));
    print("ok\n");
}

check_decoded_limits :: () {
    Escape :: struct {
        spelling: string;
        decoded: string;
    }

    for Escape.[
        .{ "\\u00a3", "£" },
        .{ "\\u20ac", "€" },
        .{ "\\U0001f642", "🙂" },
    ] {
        input := tprint("\"\" = \"%X\"", it.spelling);
        width := it.decoded.count;
        failed, error := parse_toml(input, .{ max_string_bytes = width - 1 });

        assert(error.kind == .LIMIT && error.span.start == 6 + it.spelling.count);
        free_toml(*failed);
        exact, exact_error := parse_toml(tprint("\"\" = \"%\"", it.spelling), .{ max_string_bytes = width });

        assert(exact_error.kind == .NONE && toml_get(exact.root, "").text == it.decoded);
        free_toml(*exact);
        followed, followed_error := parse_toml(input, .{ max_string_bytes = width });

        assert(followed_error.kind == .LIMIT && followed_error.span.start == 7 + it.spelling.count);
        free_toml(*followed);
    }

    for limit: 1..2 {
        doc, error := parse_toml("\"\" = \"\"\"x\"\"\"\"\"", .{ max_string_bytes = limit });
        assert(error.kind == .LIMIT && error.span.start == 9 + limit);
        free_toml(*doc);
    }
}

Parse_Stats :: struct {
    allocations: int;
}

count_parse :: (mode: Allocator_Mode, size: s64, old_size: s64, memory: *void, data: *void) -> *void {
    stats := cast(*Parse_Stats) data;
    if mode == .ALLOCATE stats.allocations += 1;
    backing := context.default_allocator;
    return backing.proc(mode, size, old_size, memory, backing.data);
}

parse_allocations :: (input: string, expected: string) -> int {
    stats: Parse_Stats;

    {
        push_allocator(count_parse, *stats);
        doc, error := parse_toml(input);
        assert(error.kind == .NONE && toml_get(doc.root, "x").text == expected);
        free_toml(*doc);
    }

    return stats.allocations;
}
