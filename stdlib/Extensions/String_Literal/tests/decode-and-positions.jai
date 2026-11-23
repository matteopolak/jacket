// Decoding quoted literals at run time: escapes, their source offsets and the end of the literal, plus
// the line and column helpers that map byte offsets to source positions.
#import "Basic";
#import "String";
SL :: #import "Extensions/String_Literal";

failures := 0;

expect :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

main :: () {
    // The source text: "a\tb\x41é\n" with a backslash before each escape letter.
    text := "\"a\\tb\\x41\\u00e9\\n\"";

    value, sources, ok := SL.decode(text, 0);

    expect(ok, "decode of a quoted literal");
    expect(value.count == 7, tprint("value has % bytes, expected 7", value.count));

    expect(value[0] == #char "a" && value[2] == #char "b" && value[3] == #char "A", "plain bytes and \\x41");
    expect(value[1] == 9, "\\t is tab");
    expect(value[4] == 0xC3 && value[5] == 0xA9, "\\u00e9 is the two UTF-8 bytes of e-acute");
    expect(value[6] == 10, "\\n is newline");

    // Each byte maps to the offset of the source that produced it: the letter, or the backslash of an escape.
    expect(sources.count == 7, "one source per byte");
    expect(sources[0] == 1 && sources[1] == 2 && sources[2] == 4 && sources[3] == 5, "sources of a, tab, b, A");
    expect(sources[4] == 9 && sources[5] == 9 && sources[6] == 15, "sources of e-acute and newline");

    // The end of the literal is the offset of the closing quote.
    expect(SL.end_offset(text, 0, sources) == 17, tprint("end of the literal is %", SL.end_offset(text, 0, sources)));

    // A literal inside a longer text decodes from its own opening quote.
    inline_text := "say \"hi\" now";
    inline_value, inline_sources, inline_ok := SL.decode(inline_text, 4);

    expect(inline_ok && inline_value == "hi", "literal in the middle of a text");
    expect(inline_sources.count == 2 && inline_sources[0] == 5 && inline_sources[1] == 6, "sources of hi");
    expect(SL.end_offset(inline_text, 4, inline_sources) == 7, "closing quote of hi is at 7");

    // Malformed literals: unterminated, unknown escape, and a position that holds no literal.
    _, _, unterminated := SL.decode("\"abc", 0);

    expect(!unterminated, "unterminated literal is not ok");

    _, _, unknown := SL.decode("\"a\\qb\"", 0);

    expect(!unknown, "unknown escape is not ok");

    _, _, not_literal := SL.decode("xyz", 0);

    expect(!not_literal, "a position without a literal is not ok");

    // Here-strings: the body after the header line is decoded.
    here := "#string END\nhello\nEND\n";
    here_value, _, here_ok := SL.decode(here, 0);

    expect(here_ok && find_index_from_left(here_value, "hello") >= 0, "here-string body is decoded");

    // Offset and position helpers agree.
    lines := "first\nsecond\n  third";

    expect(SL.offset_of(lines, 1, 1) == 0, "line 1 column 1 is offset 0");
    expect(SL.offset_of(lines, 2, 1) == 6, "line 2 column 1 is offset 6");
    expect(SL.offset_of(lines, 3, 3) == 15, "line 3 column 3 is offset 15");
    expect(SL.offset_of(lines, 9, 1) == -1, "a line past the end has no offset");
    expect(SL.offset_of(lines, 2, 100) == -1, "a column past the end has no offset");

    line, column := SL.line_and_column(lines, 15);

    expect(line == 3 && column == 3, tprint("offset 15 is line % column %", line, column));

    line, column = SL.line_and_column(lines, 0);
    expect(line == 1 && column == 1, "offset 0 is line 1 column 1");

    line, column = SL.line_and_column(lines, 6);
    expect(line == 2 && column == 1, "offset 6 is line 2 column 1");

    if failures > 0 {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
