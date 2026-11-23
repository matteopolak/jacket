// Byte and substring searches at the start, the end, past the end, for absent bytes, for empty and
// oversized needles, and for a needle that defeats the vector loop.
#import "Basic";
BS :: #import "Extensions/Byte_Search";

failures := 0;

expect :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

repeat_byte :: (byte: u8, count: int) -> string {
    result := alloc_string(count,, temp);
    for i: 0..count - 1 result[i] = byte;
    return result;
}

main :: () {
    // Single bytes: first byte, last byte, absent byte, and a start past the end.
    expect(BS.find("abc", #char "a") == 0, "find at the start");
    expect(BS.find("abc", #char "c") == 2, "find at the end");
    expect(BS.find("abc", #char "z") == -1, "find an absent byte");
    expect(BS.find("abc", #char "c", 3) == -1, "find with start at the end");
    expect(BS.find("abc", #char "c", 2) == 2, "find with start on the byte");
    expect(BS.find("", #char "a") == -1, "find in an empty text");

    // Two and three byte sets.
    expect(BS.find_any("xyz;", .[#char ";", #char "q"]) == 3, "find_any, second byte at the end");
    expect(BS.find_any("abc", .[#char "q", #char "w"]) == -1, "find_any with no match");
    expect(BS.find_any("a|b,c", .[#char "|", #char ",", #char ";"]) == 1, "find_any with three bytes");
    expect(BS.find_any("a;b", .[#char "x", #char "y", #char "z"]) == -1, "find_any three bytes, absent");
    expect(BS.find_any("abc", .[]) == -1, "find_any with no bytes");
    expect(BS.find_any("a b\tc;d", .[#char ";", #char "\t", #char "!", #char "?"], 2) == 3, "find_any with four bytes");
    expect(BS.find_last_any("a;b,c|d", .[#char ";", #char ",", #char "|", #char "!"], 5) == 3, "find_last_any with four bytes");

    // Last occurrences. `end` is exclusive.
    expect(BS.find_last("a/b/c", #char "/") == 3, "find_last at the end");
    expect(BS.find_last("a/b/c", #char "/", 3) == 1, "find_last with end excludes the byte at end");
    expect(BS.find_last("a/b/c", #char "z") == -1, "find_last absent");
    expect(BS.find_last_any("dir\\file/name", .[#char "/", #char "\\"]) == 8, "find_last_any");

    // Counting.
    expect(BS.count("", #char "a") == 0, "count in empty text");
    expect(BS.count("aaaa", #char "a") == 4, "count every byte");

    // Substrings: at the start, at the end, absent, longer than the text.
    expect(BS.find("abcabc", "abc") == 0, "needle at the start");
    expect(BS.find("xxxyz", "yz") == 3, "needle at the end");
    expect(BS.find("abcabc", "abc", 1) == 3, "needle found after start");
    expect(BS.find("abc", "abcd") == -1, "needle longer than text");
    expect(BS.find("ab", "abc") == -1, "needle longer than text, no match");
    expect(BS.find("abc", "xyz") == -1, "absent needle");
    expect(BS.find("abc", "c") == 2, "one-byte needle");

    // An empty needle is found at start.
    expect(BS.find("abc", "") == 0, "empty needle at 0");
    expect(BS.find("abc", "", 2) == 2, "empty needle at start");

    // Last substring: `end` is the position the match must end at or before.
    expect(BS.find_last("abab", "ab") == 2, "find_last substring");
    expect(BS.find_last("abab", "ab", 3) == 0, "find_last substring with end");
    expect(BS.find_last("ab", "abc") == -1, "find_last needle longer than text");

    // A reusable finder.
    finder := BS.make_finder("ab");

    expect(BS.find(*finder, "xxabyyab", 3) == 6, "finder with start");
    expect(BS.find(*finder, "xxabyyab", 7) == -1, "finder past the last match");

    // A needle that makes the vector loop fail on most blocks: 40 'a' then 'b', in a text of 200
    // 'a' followed by the needle.
    needle := repeat_byte(#char "a", 40);
    needle = tprint("%b", needle);
    text := tprint("%%", repeat_byte(#char "a", 200), needle);

    expect(BS.find(text, needle) == 200, "long needle in repetitive text");
    expect(BS.find_last(text, needle) == 200, "long needle, last occurrence");
    expect(BS.find(text, needle, 201) == -1, "long needle after its only match");
    expect(BS.find(repeat_byte(#char "a", 300), needle) == -1, "long needle absent from repetitive text");

    if failures > 0 {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
