// Searches and comparisons over strings long enough to take the vector paths, against plain loops.
#import "Basic";
#import "Extensions/Testing";

searches_agree_with_plain_loops :: () {
    random_seed(20261022);

    // Sizes either side of the 64-byte and 256-byte switches to the vector searches.
    for size: .[0, 1, 31, 63, 64, 65, 127, 128, 255, 256, 257, 300, 1000, 4097] {
        for round: 0..5 {
            alphabet := ifx round % 2 == 0 then "abcdefghij" else "ab";
            text := random_text(size, alphabet);
            a: u8 = alphabet[random_get() % cast(u64) alphabet.count];
            set := string.{ 1 + round % 4, "jaz?".data };

            for start: .[-3, 0, 1, 40, 63, 64, 65, 200, size - 1, size, size + 5] {
                what := tprint("size % round % start %", size, round, start);
                expect(find_index_from_left(text, a, start) == naive_left(text, a, start), tprint("byte left %", what));
                expect(find_index_of_any_from_left(text, set, start) == naive_any_left(text, set, start), tprint("any left %", what));
            }

            what := tprint("size % round %", size, round);

            expect(find_index_from_right(text, a) == naive_right(text, a), tprint("byte right %", what));
            expect(find_index_of_any_from_right(text, set) == naive_any_right(text, set), tprint("any right %", what));

            for length: .[1, 2, 3, 5, 9] {
                if length > size continue;

                at := cast(int) (random_get() % cast(u64) (size - length + 1));
                needle := ifx round % 3 == 0 then random_text(length, alphabet) else string.{ length, text.data + at };

                for start: .[0, 1, 70, 300] {
                    if start > size continue;
                    expect(find_index_from_left(text, needle, start) == naive_substring_left(text, needle, start), tprint("substring left % in % from %", needle, what, start));
                }

                expect(find_index_from_right(text, needle) == naive_substring_right(text, needle), tprint("substring right % in %", needle, what));
            }
        }
    }
} @test

comparison_agrees_with_a_plain_loop :: () {
    random_seed(20261022);

    for size: .[0, 1, 31, 32, 33, 100, 1000] {
        base := random_text(size, "abcxyz\x01\x80\xFF");
        copy := copy_string(base,, temp);

        expect(compare(base, copy) == 0, "equal strings compare equal");
        expect(equal(base, copy), "equal strings are equal");

        for at: .[0, 1, size / 2, size - 1] {
            if at < 0 || at >= size continue;

            // The first difference decides, whichever way the bytes are signed.
            for pair: u8.[0x01, 0x80, 0x7F, 0xFF] {
                for other: u8.[0x00, 0x80, 0xFF] {
                    if pair == other continue;

                    changed := copy_string(base,, temp);

                    base[at] = pair;
                    changed[at] = other;
                    expect(compare(base, changed) == naive_compare(base, changed), tprint("compare at % of %: % vs %", at, size, pair, other));
                    expect(compare(changed, base) == naive_compare(changed, base), tprint("compare reversed at % of %", at, size));
                }
            }
        }

        // A longer string with the same start is greater.
        longer := tprint("%x", base);

        expect(compare(base, longer) == -1 && compare(longer, base) == 1, tprint("prefix order at %", size));
    }
} @test

#scope_file

#import "Random";
#import "String";

// Searches and comparisons over strings long enough to take the vector paths, against plain loops.
random_text :: (count: int, alphabet: string) -> string {
    text := alloc_string(count,, temp);
    for 0..count - 1 text[it] = alphabet[random_get() % cast(u64) alphabet.count];
    return text;
}

naive_left :: (s: string, byte: u8, start: int) -> int {
    for i: max(start, 0)..s.count - 1 if s[i] == byte return i;
    return -1;
}

naive_right :: (s: string, byte: u8) -> int {
    i := s.count - 1;
    while i >= 0 {
        if s[i] == byte return i;
        i -= 1;
    }

    return -1;
}

naive_any_left :: (s: string, bytes: string, start: int) -> int {
    for i: max(start, 0)..s.count - 1 for bytes if s[i] == it return i;
    return -1;
}

naive_any_right :: (s: string, bytes: string) -> int {
    i := s.count - 1;
    while i >= 0 {
        for bytes if s[i] == it return i;
        i -= 1;
    }

    return -1;
}

naive_substring_left :: (s: string, needle: string, start: int) -> int {
    for i: start..s.count - needle.count if memcmp(s.data + i, needle.data, needle.count) == 0 return i;
    return -1;
}

naive_substring_right :: (s: string, needle: string) -> int {
    i := s.count - needle.count;
    while i >= 0 {
        if memcmp(s.data + i, needle.data, needle.count) == 0 return i;
        i -= 1;
    }

    return -1;
}

naive_compare :: (a: string, b: string) -> int {
    for i: 0..min(a.count, b.count) - 1 {
        if a[i] < b[i] return -1;
        if a[i] > b[i] return 1;
    }

    if a.count < b.count return -1;
    if a.count > b.count return 1;

    return 0;
}
