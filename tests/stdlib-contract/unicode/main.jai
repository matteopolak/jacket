// Truncated UTF-8 input is replacement-decoded and its remaining bytes are consumed.
#import "Basic";
Unicode :: #import "Unicode";

#assert(is_constant(Unicode.offsetsFromUTF8));
#assert(is_constant(Unicode.firstByteMark));

main :: () {
    cursor := "A\xe2\x82";
    first, status := Unicode.utf8_next_character(*cursor);

    assert(status == .CONVERSION_OK);
    assert(first == 65);
    replacement, tail_status := Unicode.utf8_next_character(*cursor);

    assert(tail_status == .SOURCE_EXHAUSTED);
    assert(replacement == Unicode.UNI_REPLACEMENT_CHAR);
    assert(cursor.count == 0);

    assert(Unicode.offsetsFromUTF8[0] == 0);
    assert(Unicode.firstByteMark[0] == 0);
    print("ok\n");
}
