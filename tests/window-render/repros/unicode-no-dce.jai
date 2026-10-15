// Original production-import reproducer for the NEW producer dependency.
// Ordinary -check passes; -check -no_dce diagnoses the retained Unicode
// utf8_inital_byte_mask declaration: expected [6]u8, found s64.
// This worker does not change peer Unicode or rewrite staged imports.
#import "Basic";
U :: #import "Unicode";

main :: () {
    code, count, status := U.character_utf8_to_utf32("a".data, 1, true);
    assert(code == 97 && count == 1 && status == .CONVERSION_OK);
}
