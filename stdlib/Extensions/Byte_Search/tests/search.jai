// Every search against a plain loop, on random texts, every start and end, and needles built to
// defeat the vector loop.
//
//     jaic run stdlib/Extensions/Byte_Search/tests/search.jai

#import "Basic";
#import "Random";
#import "String";
Byte_Search :: #import "Extensions/Byte_Search";

failures := 0;

check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    if failures < 20 print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

naive_find :: (text: string, needle: string, start: int) -> int {
    if start < 0 start = 0;

    i := start;
    while i + needle.count <= text.count {
        if memcmp(text.data + i, needle.data, needle.count) == 0 return i;
        i += 1;
    }

    return -1;
}

naive_find_last :: (text: string, needle: string, end: int) -> int {
    if end < 0 || end > text.count end = text.count;

    i := end - needle.count;
    while i >= 0 {
        if memcmp(text.data + i, needle.data, needle.count) == 0 return i;
        i -= 1;
    }

    return -1;
}

naive_find_any :: (text: string, set: [] u8, start: int) -> int {
    for i: max(start, 0)..text.count - 1 for set if text[i] == it return i;
    return -1;
}

naive_find_last_any :: (text: string, set: [] u8, end: int) -> int {
    if end < 0 || end > text.count end = text.count;

    i := end - 1;
    while i >= 0 {
        for set if text[i] == it return i;
        i -= 1;
    }

    return -1;
}

random_text :: (count: int, alphabet: string) -> string {
    text := alloc_string(count,, temp);
    for 0..count - 1 text[it] = alphabet[random_get() % cast(u64) alphabet.count];
    return text;
}

bytes :: () {
    for round: 0..100 {
        alphabet := ifx round % 3 == 0 then "ab" else ifx round % 3 == 1 then "abcdefgh\n" else "abcdefghijklmnopqrstuvwxyz0123456789 \n\x00\xFF";
        size := cast(int) (random_get() % 200);
        text := random_text(size, alphabet);
        a := alphabet[random_get() % cast(u64) alphabet.count];
        b := alphabet[random_get() % cast(u64) alphabet.count];
        c: u8 = #char "z";

        for start: -1..size + 1 {
            check(Byte_Search.find(text, a, start) == naive_find_any(text, .[a], start), tprint("find % in % from %", a, text, start));
            check(Byte_Search.find_any(text, a, b, start) == naive_find_any(text, .[a, b], start), tprint("find_any %/% in % from %", a, b, text, start));
            check(Byte_Search.find_any(text, a, b, c, start) == naive_find_any(text, .[a, b, c], start), tprint("find_any %/%/% in % from %", a, b, c, text, start));
            end := start;

            check(Byte_Search.find_last(text, a, end) == naive_find_last_any(text, .[a], end), tprint("find_last % in % before %", a, text, end));
            check(Byte_Search.find_last_any(text, a, b, end) == naive_find_last_any(text, .[a, b], end), tprint("find_last_any %/% in % before %", a, b, text, end));
            check(Byte_Search.find_last_any(text, a, b, c, end) == naive_find_last_any(text, .[a, b, c], end), tprint("find_last_any %/%/% in % before %", a, b, c, text, end));
        }

        expected := 0;
        for text if it == a expected += 1;

        check(Byte_Search.count(text, a) == expected, tprint("count % in %", a, text));
        reset_temporary_storage();
    }

    // Enough of one byte to empty the counting lanes several times.
    big := alloc_string(100_000,, temp);

    memset(big.data, #char "x", big.count);
    big[99_999] = #char "y";
    check(Byte_Search.count(big, #char "x") == 99_999, "count over 255 chunks");
    check(Byte_Search.find(big, #char "y") == 99_999 && Byte_Search.find_last(big, #char "y") == 99_999, "a byte at the very end");
}

byte_sets :: () {
    // Sets of every size, including ones spread over more than 8 high nibbles, against a
    // membership loop.
    for round: 0..80 {
        members: [..] u8;
        members.allocator = temp;
        size := cast(int) (random_get() % 40);
        if round % 5 == 0 size = 0;

        for 1..size {
            b := cast(u8) (random_get() % 256);
            if round % 2 b = cast(u8) (#char "a" + random_get() % 26);
            array_add(*members, b);
        }

        set := Byte_Search.make_byte_set(members);
        text := alloc_string(cast(int) (random_get() % 300),, temp);

        for *text it.* = ifx round % 3 then cast(u8) (#char "a" + random_get() % 26) else cast(u8) (random_get() % 256);

        for b: 0..255 {
            inside := false;
            for members if it == b inside = true;
            check(Byte_Search.contains(*set, cast(u8) b) == inside, tprint("contains % in %", b, members));
        }

        other := Byte_Search.make_byte_set(.[cast(u8) (random_get() % 256), #char "q", 0xC3]);
        // Every fourth round, sets of one byte each, which take the plain compare path.
        singles := round % 4 == 1;

        if singles {
            array_reset(*members);
            members.allocator = temp;
            array_add(*members, cast(u8) (#char "a" + random_get() % 26));
            set = Byte_Search.make_byte_set(members);
            other = Byte_Search.make_byte_set(.[cast(u8) (#char "a" + random_get() % 26)]);
        }

        gap := cast(int) (random_get() % 5);

        for start: -1..text.count + 1 {
            check(Byte_Search.find(*set, text, start) == naive_find_any(text, members, start), tprint("find set % from %", members, start));
            expected := -1;

            for i: max(start, 0)..text.count - 1 - gap {
                if Byte_Search.contains(*set, text[i]) && Byte_Search.contains(*other, text[i + gap]) {
                    expected = i;
                    break;
                }
            }

            check(Byte_Search.find_pair(*set, *other, gap, text, start) == expected, tprint("find_pair % gap % from %", members, gap, start));
            third := ifx singles then Byte_Search.make_byte_set(.[#char "e"]) else Byte_Search.make_byte_set(.[#char "a", #char "e", 0x80]);
            gap_3 := gap + 1 + cast(int) (random_get() % 3);

            expected = -1;

            for i: max(start, 0)..text.count - 1 - gap_3 {
                if Byte_Search.contains(*set, text[i]) && Byte_Search.contains(*other, text[i + gap]) && Byte_Search.contains(*third, text[i + gap_3]) {
                    expected = i;
                    break;
                }
            }

            check(Byte_Search.find_triple(*set, *other, *third, gap, gap_3, text, start) == expected, tprint("find_triple % gaps %/% from %", members, gap, gap_3, start));
        }

        reset_temporary_storage();
    }
}

substrings :: () {
    // Short alphabets make near misses common; long needles exercise both the vector loop and
    // Two-Way.
    for round: 0..250 {
        alphabet := ifx round % 4 == 0 then "a" else ifx round % 4 == 1 then "ab" else ifx round % 4 == 2 then "abc" else "abcdefghij ";
        size := cast(int) (random_get() % 150);
        text := random_text(size, alphabet);
        needle_size := cast(int) (random_get() % 12);
        needle: string;

        if size > 0 && random_get() % 2 == 0 {
            // Taken from the text, so it is there at least once.
            from := cast(int) (random_get() % cast(u64) size);
            needle = slice(text, from, min(needle_size, size - from));
        } else {
            needle = random_text(needle_size, alphabet);
        }

        finder := Byte_Search.make_finder(needle);

        for start: -1..size + 1 {
            expected := naive_find(text, needle, start);

            check(Byte_Search.find(*finder, text, start) == expected, tprint("find \"%\" in \"%\" from %: % expected %", needle, text, start, Byte_Search.find(*finder, text, start), expected));
            end := start;

            expected = naive_find_last(text, needle, end);
            check(Byte_Search.find_last(*finder, text, end) == expected, tprint("find_last \"%\" in \"%\" before %: % expected %", needle, text, end, Byte_Search.find_last(*finder, text, end), expected));
        }

        check(Byte_Search.find(text, needle) == naive_find(text, needle, 0), "find with a string");
        check(Byte_Search.find_last(text, needle) == naive_find_last(text, needle, -1), "find_last with a string");
        reset_temporary_storage();
    }

    // Needles that are periodic, nearly periodic, or have their rare bytes at the edges.
    NEEDLES :: string.["aaaaaaab", "baaaaaaa", "abababababc", "aabaabaabaab", "abcabcabd", "zaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaz", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab"];
    TEXTS :: string.["a", "ab", "aab", "abc"];

    for needle: NEEDLES for unit : TEXTS {
        builder: String_Builder;
        builder.allocator = temp;
        for 1..2000 / unit.count append(*builder, unit);

        text := builder_to_string(*builder,, temp);
        finder := Byte_Search.make_finder(needle);

        for start: 0..text.count {
            if start % 97 != 0 continue;
            check(Byte_Search.find(*finder, text, start) == naive_find(text, needle, start), tprint("find % in % from %", needle, unit, start));
            check(Byte_Search.find_last(*finder, text, text.count - start) == naive_find_last(text, needle, text.count - start), tprint("find_last % in % before %", needle, unit, text.count - start));
        }

        // With a match planted in the middle and at the end.
        planted := copy_string(text,, temp);
        middle := planted.count / 2;

        memcpy(planted.data + middle, needle.data, needle.count);
        check(Byte_Search.find(*finder, planted) == naive_find(planted, needle, 0), tprint("planted find %", needle));
        check(Byte_Search.find_last(*finder, planted) == naive_find_last(planted, needle, -1), tprint("planted find_last %", needle));
        reset_temporary_storage();
    }
}

// Two-Way keeps the search linear even when the vector loop's candidates nearly all fail.
linear_time :: () {
    text := alloc_string(2_000_000,, temp);

    memset(text.data, #char "a", text.count);
    needle := "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab";
    start := current_time_monotonic();

    check(Byte_Search.find(text, needle) == -1, "no match in all a's");
    check(Byte_Search.find_last(text, needle) == -1, "no match in all a's, backwards");
    seconds := to_float64_seconds(current_time_monotonic() - start);

    // A quadratic search would compare about 128M bytes; even the interpreter does the linear
    // one in well under this.
    check(seconds < 20, tprint("took % seconds", seconds));
}

main :: () {
    random_seed(20261012);
    bytes();
    byte_sets();
    substrings();
    linear_time();

    if failures {
        print("% failures\n", failures);
        exit(1);
    }
}
