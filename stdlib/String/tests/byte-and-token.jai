// Byte-oriented search, split, token parsing, and line-ending behavior.
#import "Basic";
#import "Extensions/Testing";

contains_returns_the_text_after_the_match :: () {
    found, tail := String.contains("copper/blue", "per/");

    expect(found && tail == "blue");
} @test

split_keeps_empty_fields :: () {
    fields := String.split(";a;;", #char ";");

    defer array_free(fields);

    expect_equal(fields.count, 4);
    expect(fields[0] == "" && fields[1] == "a" && fields[2] == "" && fields[3] == "");

    empty := String.split("", #char ",");

    defer array_free(empty);

    expect_equal(empty.count, 1);
} @test

default_delimiters_and_spaces :: () {
    expect(String.DEFAULT_DELIMITERS == ";=()" && String.DEFAULT_SPACES == " \t\r\n");
} @test

lowering_changes_the_bytes_in_place :: () {
    letters: [5] u8 = .[77, 105, 88, 101, 68];
    mutable: string;
    mutable.data = letters.data;
    mutable.count = letters.count;
    String.to_lower_in_place(mutable);
    expect_equal(mutable, "mixed");
} @test

parse_token_splits_names_and_delimiters :: () {
    input := "   name  =value";
    token, ok := String.parse_token(*input);

    expect(ok && token == "name" && input == "  =value");
    token, ok = String.parse_token(*input);
    expect(ok && token == "=" && input == "value");
} @test

normalize_line_endings_turns_crlf_into_lf :: () {
    normalized := String.normalize_line_endings("one\r\ntwo\rthree");
    defer free(normalized);

    expect_equal(normalized, "one\ntwo\rthree");
} @test

#scope_file

String :: #import "String";
