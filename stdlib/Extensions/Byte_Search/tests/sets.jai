// Byte sets of one, few and many bytes, and the pair and triple searches with fixed gaps.
#import "Basic";
BS :: #import "Extensions/Byte_Search";

failures := 0;

expect :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

as_text :: (bytes: [] u8) -> string {
    return cast(string) bytes;
}

main :: () {
    // Delimiters: the set answers membership and find.
    delimiters := BS.make_byte_set(.[#char " ", #char "\t", #char ",", #char ";"]);

    expect(BS.find(*delimiters, "one;two three") == 3, "find in a small set");
    expect(BS.contains(*delimiters, #char "\t"), "contains a member");
    expect(!BS.contains(*delimiters, #char "a"), "contains a non-member");
    expect(BS.find(*delimiters, "abc") == -1, "find with no member");

    // One byte: the single-byte fast path.
    only_z := BS.make_byte_set(.[#char "z"]);

    expect(BS.find(*only_z, "zzz", 1) == 1, "single-byte set with start");

    // Sixteen bytes, one per high nibble: the wide path, where a set has more high nibbles than
    // the vector tables hold.
    wide_bytes: [16] u8;
    for 0..15 wide_bytes[it] = cast(u8) (it * 16);

    wide := BS.make_byte_set(wide_bytes);

    expect(BS.contains(*wide, 0x50), "wide set contains 0x50");
    expect(!BS.contains(*wide, 0x51), "wide set rejects 0x51");
    expect(BS.find(*wide, as_text(.[#char "a", #char "b", #char "c", 0x51, 0x50])) == 4, "wide set finds the first member");

    // Every byte from 0x80 up, with a newline: the set has all high nibbles from 8 to F.
    high_bytes: [129] u8;
    for 0..127 high_bytes[it] = cast(u8) (0x80 + it);

    high_bytes[128] = #char "\n";
    high := BS.make_byte_set(high_bytes);

    expect(BS.find(*high, as_text(.[#char "a", #char "\n"])) == 1, "newline is in the high set");
    expect(BS.find(*high, as_text(.[#char "a", #char "b"])) == -1, "ASCII letters are not in the high set");
    expect(BS.find(*high, as_text(.[#char "a", 0xFF])) == 1, "0xFF is in the high set");

    // Pairs: text[i] in first, text[i + gap] in second.
    a := BS.make_byte_set(.[#char "a"]);
    b := BS.make_byte_set(.[#char "b"]);
    c := BS.make_byte_set(.[#char "c"]);

    // "xaabxab": the first match is at 2 ("ab" with a gap of 1).
    expect(BS.find_pair(*a, *b, 1, "xaabxab") == 2, "pair found at the first match");
    expect(BS.find_pair(*a, *b, 1, "xaabxab", 3) == 5, "pair found after start");
    // With gap 2, text[1] is a and text[3] is b.
    expect(BS.find_pair(*a, *b, 2, "xaabxab") == 1, "pair with gap 2");
    expect(BS.find_pair(*a, *b, 3, "xaabxab") == -1, "pair with a gap that never matches");

    // Triples: text[i] in first, text[i + 1] in second, text[i + 2] in third.
    expect(BS.find_triple(*a, *b, *c, 1, 2, "xabcabc") == 1, "triple found at the first match");
    expect(BS.find_triple(*a, *b, *c, 1, 2, "xabcabc", 2) == 4, "triple found after start");
    expect(BS.find_triple(*a, *b, *c, 1, 2, "xabcab") == 1, "triple whose last byte is the end of text");
    expect(BS.find_triple(*a, *b, *c, 1, 2, "xab") == -1, "triple cut short by the end of text");

    if failures > 0 {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
