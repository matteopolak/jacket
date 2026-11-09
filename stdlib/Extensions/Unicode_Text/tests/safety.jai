#import "Basic";
#import "Extensions/Unicode_Text";
#load "../../test-support/counting-allocator.jai";

invalid_utf8_and_limits :: () {
    for text: string.["\x80", "\xC0\xAF", "\xED\xA0\x80", "\xF4\x90\x80\x80", "\xF0\x90", "ok\xFF"] {
        b: String_Builder;

        defer free_buffers(*b);
        append(*b, "prefix");
        error := normalize(text, .NFC, *b);

        assert(error.kind == .INVALID_UTF8);
        assert(builder_string_length(*b) == 6);
        _, error = is_normalized(text, .NFC);
        assert(error.kind == .INVALID_UTF8);

        for kind: 0..2 {
            iter := segments(text, cast(Segmentation) kind);

            while true {
                _, _, more := next_segment(*iter);
                if !more break;
            }

            assert(iter.error.kind == .INVALID_UTF8);
        }
    }

    b: String_Builder;

    defer free_buffers(*b);
    error := normalize("abc", .NFC, *b, .{ max_size = 2 });

    assert(error.kind == .LIMIT);
    error = normalize("ﬃ", .NFKD, *b, .{ max_output = 2 });
    assert(error.kind == .LIMIT && builder_string_length(*b) == 0);
    iter := segments("abc", .WORD, .{ max_size = 2 });

    assert(iter.error.kind == .LIMIT);
} @test

long_sequences_and_ownership :: () {
    source: String_Builder;

    defer free_buffers(*source);
    append(*source, "a");
    for 1..10000 append(*source, "́");

    append(*source, "̣");
    text := builder_to_string(*source);

    defer free(text);
    tally: Tally;

    {
        push_allocator(tally_proc, *tally);
        b: String_Builder;

        defer free_buffers(*b);
        error := normalize(text, .NFD, *b);

        assert(error.kind == .NONE);
        out := builder_to_string(*b);

        defer free(out);
        assert(out[1] == 0xCC && out[2] == 0xA3);
        error = normalize(text, .NFC, *b, .{ stream_safe = true });
        assert(error.kind == .NONE);
        safe := builder_to_string(*b);

        defer free(safe);
        assert(safe.count > text.count);
        again: String_Builder;

        defer free_buffers(*again);
        error = normalize(safe, .NFC, *again, .{ stream_safe = true });
        assert(error.kind == .NONE);
        same := builder_to_string(*again);

        defer free(same);
        assert(same == safe);
        yes, qc_error := is_normalized(out, .NFD);

        assert(yes && qc_error.kind == .NONE);
        iterator := segments(text, .GRAPHEME);
        start, end, more := next_segment(*iterator);

        assert(more && start == 0 && end == text.count);
    }

    assert(tally.outstanding == 0);
} @test

quick_check_maybe_and_reordering :: () {
    yes, error := is_normalized("Ḍ̇", .NFC);

    assert(!yes && error.kind == .NONE);
    b: String_Builder;

    defer free_buffers(*b);
    error = normalize("ạ́́", .NFD, *b);
    assert(error.kind == .NONE);
    text := builder_to_string(*b);

    defer free(text);
    assert(text == "ạ́́");
} @test

sinhala_composition_after_reordering :: () {
    b: String_Builder;

    defer free_buffers(*b);
    error := normalize("ෝ̴", .NFC, *b);

    assert(error.kind == .NONE);
    text := builder_to_string(*b);

    defer free(text);
    assert(text == "ෝ̴");
} @test

stream_safe_threshold_and_helper_limits :: () {
    b: String_Builder;

    defer free_buffers(*b);
    for 1..30 append(*b, "́");

    thirty := builder_to_string(*b);

    defer free(thirty);
    append(*b, thirty);
    append(*b, "̈́"); // Canonically expands to two nonstarters.
    source := builder_to_string(*b);

    defer free(source);
    error := normalize(source, .NFD, *b, .{ stream_safe = true });

    assert(error.kind == .NONE);
    safe := builder_to_string(*b);

    defer free(safe);
    assert(safe.count == 66);
    assert(safe[60] == 0xCD && safe[61] == 0x8F); // CGJ, then diaeresis and acute.
    _, error = next_grapheme("x", 0, .{ max_size = 0 });
    assert(error.kind == .LIMIT);
    _, error = previous_grapheme("x", 0, .{ max_size = 0 });
    assert(error.kind == .LIMIT);
    _, error = is_normalized("x", cast(Normalization_Form) 255);
    assert(error.kind == .INVALID_ARGUMENT);
} @test

allocation_balance_on_errors_and_maybe :: () {
    tally: Tally;

    {
        push_allocator(tally_proc, *tally);

        for input: string.["Ḍ̇", "a\xFF", "ﬃ", "각", "x"] {
            for f: 0..3 {
                b: String_Builder;

                defer free_buffers(*b);
                form := cast(Normalization_Form) f;

                normalize(input, form, *b);
                normalized := builder_to_string(*b);

                free(normalized);
                normalize(input, form, *b, .{ max_output = 1 });
                is_normalized(input, form);
            }
        }
    }

    assert(tally.outstanding == 0);
} @test
