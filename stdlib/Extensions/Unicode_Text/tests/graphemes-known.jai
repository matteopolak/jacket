// Grapheme cluster counts and boundaries for combining marks, CR LF, regional indicator pairs, emoji
// modifiers and ZWJ sequences, Hangul jamo, and invalid input.
#import "Basic";
UT :: #import "Extensions/Unicode_Text";

failures := 0;

expect :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

// UTF-8 text for a list of code points.
cps :: (values: [] u32) -> string {
    out: [..] u8;

    for values {
        cp := it;
        if cp < 0x80 {
            array_add(*out, cast(u8) cp);
        } else if cp < 0x800 {
            array_add(*out, cast(u8) (0xC0 | (cp >> 6)));
            array_add(*out, cast(u8) (0x80 | (cp & 0x3F)));
        } else if cp < 0x10000 {
            array_add(*out, cast(u8) (0xE0 | (cp >> 12)));
            array_add(*out, cast(u8) (0x80 | ((cp >> 6) & 0x3F)));
            array_add(*out, cast(u8) (0x80 | (cp & 0x3F)));
        } else {
            array_add(*out, cast(u8) (0xF0 | (cp >> 18)));
            array_add(*out, cast(u8) (0x80 | ((cp >> 12) & 0x3F)));
            array_add(*out, cast(u8) (0x80 | ((cp >> 6) & 0x3F)));
            array_add(*out, cast(u8) (0x80 | (cp & 0x3F)));
        }
    }

    return .{ out.count, out.data };
}

count_of :: (text: string, what: string, expected: int) {
    count, error := UT.grapheme_count(text);
    expect(error.kind == .NONE && count == expected, tprint("% : % clusters, expected %", what, count, expected));
}

main :: () {
    count_of("", "empty text", 0);
    count_of("abc", "three ASCII letters", 3);
    count_of(cps(.[0x65, 0x301]), "e with combining acute", 1);
    count_of(cps(.[0x61, 0x308, 0x301]), "a with two combining marks", 1);
    count_of(cps(.[0x0D, 0x0A]), "CR LF is one cluster", 1);
    count_of(cps(.[0x1F1FA, 0x1F1F8, 0x1F1EB, 0x1F1F7]), "two regional indicator pairs (US, FR)", 2);
    count_of(cps(.[0x1F44D, 0x1F3FD]), "thumbs up with a skin tone modifier", 1);
    count_of(cps(.[0x1F468, 0x200D, 0x1F469, 0x200D, 0x1F467]), "family ZWJ sequence", 1);
    count_of(cps(.[0x1100, 0x1161, 0x11A8]), "Hangul L V T jamo", 1);
    count_of(cps(.[0x61, 0x200D, 0x62]), "a ZWJ b", 2);

    // Boundaries: next_grapheme goes forward from a boundary, previous_grapheme backward.
    text := cps(.[0x65, 0x301, 0x78]);
    end, next_error := UT.next_grapheme(text, 0);

    expect(next_error.kind == .NONE && end == 3, tprint("next_grapheme after e+acute is 3, got %", end));

    end_x, next_x_error := UT.next_grapheme(text, 3);

    expect(next_x_error.kind == .NONE && end_x == 4, tprint("next_grapheme after the e cluster is 4, got %", end_x));

    start, prev_error := UT.previous_grapheme(text, 3);

    expect(prev_error.kind == .NONE && start == 0, tprint("previous_grapheme before x is 0, got %", start));

    start_x, prev_x_error := UT.previous_grapheme(text, 4);

    expect(prev_x_error.kind == .NONE && start_x == 3, tprint("previous_grapheme at the end is 3, got %", start_x));

    // Invalid UTF-8 is reported, not counted.
    bad := string.{ 2, cast(*u8) alloc(2) };
    bad[0] = #char "a";
    bad[1] = 0xFF;
    _, bad_error := UT.grapheme_count(bad);

    expect(bad_error.kind == .INVALID_UTF8, "grapheme_count reports invalid UTF-8");

    if failures > 0 {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
