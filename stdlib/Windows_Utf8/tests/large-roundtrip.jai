// test: only(.WINDOWS) timeout(30)
// Round trips a 600 KB string of ASCII, two-byte, three-byte and four-byte sequences, and
// checks the UTF-16 length of each prefix against the code-point widths.
#import "Basic";
U :: #import "Windows_Utf8";

main :: () {
    units := string.["a", "\xc3\xa9", "\xe2\x82\xac", "\xf0\x9f\x99\x82"];
    unit_widths := s64.[1, 1, 1, 2];
    builder: String_Builder;
    expected_units := 0;
    for 0..99999 {
        index := it % 4;
        append(*builder, units[index]);
        expected_units += unit_widths[index];
    }
    text := builder_to_string(*builder);
    defer free(text);

    wide, success, bytes := U.utf8_to_wide(text);
    assert(success && wide != null);
    assert(bytes == cast(s32) (expected_units * 2));
    assert(wide[expected_units] == 0);

    back, valid := U.wide_to_utf8(wide);
    assert(valid && back == text);
    free(back);
    free(wide);
    print("round trip: ok\n");
    print("utf-16 units: %\n", expected_units);
}
