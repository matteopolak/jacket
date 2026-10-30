// test: skip_on(.WASM)
// The 16-byte and the 32-byte version of every dispatched search against a plain loop. The module
// is loaded into this program instead of imported, to reach both versions; the program calls
// them directly, so the 32-byte one runs here whatever the processor, and the table that picks
// one by processor is not involved.
//
//     jaic stdlib/Extensions/Byte_Search/tests/variants.jai -interp

#import "Basic";
#import "Random";
#load "../module.jai";

failures := 0;

// The 32-byte versions are compiled for AVX2: on x64 they only run where the processor has it.
wide_ok: bool;

check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    if failures < 20 print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
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

naive_find :: (text: string, needle: string, start: int) -> int {
    i := max(start, 0);

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

naive_count :: (text: string, byte: u8) -> int {
    total := 0;
    for text if it == byte total += 1;
    return total;
}

random_text :: (count: int, alphabet: string) -> string {
    text := alloc_string(count,, temp);
    for 0..count - 1 text[it] = alphabet[random_get() % cast(u64) alphabet.count];
    return text;
}

bytes :: () {
    for round: 0..60 {
        alphabet := ifx round % 3 == 0 then "ab" else ifx round % 3 == 1 then "abcdefgh\n" else "abcdefghijklmnopqrstuvwxyz0123456789 \n\x00\xFF";
        // Up to a few hundred bytes covers every mix of 128-byte rounds, single chunks and tails.
        size := cast(int) (random_get() % 400);
        text := random_text(size, alphabet);
        a := alphabet[random_get() % cast(u64) alphabet.count];
        b := alphabet[random_get() % cast(u64) alphabet.count];
        c: u8 = #char "z";

        for start: -1..size + 1 {
            what := tprint("in % from %", text, start);
            expected_one := naive_find_any(text, .[a], start);
            expected_two := naive_find_any(text, .[a, b], start);
            expected_three := naive_find_any(text, .[a, b, c], start);

            check(find_byte_narrow(text, a, start) == expected_one, tprint("find narrow %", what));
            check(!wide_ok || find_byte_wide(text, a, start) == expected_one, tprint("find wide %", what));
            check(find_two_narrow(text, a, b, start) == expected_two, tprint("find_any narrow %", what));
            check(!wide_ok || find_two_wide(text, a, b, start) == expected_two, tprint("find_any wide %", what));
            check(find_three_narrow(text, a, b, c, start) == expected_three, tprint("find_any3 narrow %", what));
            check(!wide_ok || find_three_wide(text, a, b, c, start) == expected_three, tprint("find_any3 wide %", what));

            end := start;
            last_one := naive_find_last_any(text, .[a], end);
            last_two := naive_find_last_any(text, .[a, b], end);
            last_three := naive_find_last_any(text, .[a, b, c], end);

            check(find_last_byte_narrow(text, a, end) == last_one, tprint("find_last narrow %", what));
            check(!wide_ok || find_last_byte_wide(text, a, end) == last_one, tprint("find_last wide %", what));
            check(find_last_two_narrow(text, a, b, end) == last_two, tprint("find_last_any narrow %", what));
            check(!wide_ok || find_last_two_wide(text, a, b, end) == last_two, tprint("find_last_any wide %", what));
            check(find_last_three_narrow(text, a, b, c, end) == last_three, tprint("find_last_any3 narrow %", what));
            check(!wide_ok || find_last_three_wide(text, a, b, c, end) == last_three, tprint("find_last_any3 wide %", what));
        }

        expected := naive_count(text, a);

        check(count_byte_narrow(text, a) == expected, tprint("count narrow in %", text));
        check(!wide_ok || count_byte_wide(text, a) == expected, tprint("count wide in %", text));
    }
}

// A run of more than 255 chunks of the same byte fills the u8 counters of the count kernels.
long_runs :: () {
    for size: s64.[255 * 32 + 5, 255 * 16 * 3, 70_000] {
        text := alloc_string(size,, temp);

        memset(text.data, #char "x", size);
        check(count_byte_narrow(text, #char "x") == size, "count narrow of a long run");
        check(!wide_ok || count_byte_wide(text, #char "x") == size, "count wide of a long run");
        check(!wide_ok || count_byte_wide(text, #char "y") == 0, "count wide, no match");
    }
}

substrings :: () {
    for round: 0..40 {
        alphabet := ifx round % 2 == 0 then "ab" else "abcdefghijklmnop";
        size := cast(int) (random_get() % 500);
        text := random_text(size, alphabet);

        for 0..30 {
            length := 2 + cast(int) (random_get() % 8);
            needle: string;

            if size >= length && random_get() % 2 == 0 {
                at := cast(int) (random_get() % cast(u64) (size - length + 1));
                needle = string.{ length, text.data + at };
            } else {
                needle = random_text(length, alphabet);
            }

            finder := make_finder(needle);

            for start: s64.[-1, 0, 1, 7, 33, size / 2, size - 1, size, size + 1] {
                what := tprint("% in % from %", needle, text, start);
                expected := naive_find(text, needle, start);

                check(find_finder_narrow(*finder, text, start) == expected, tprint("find narrow %", what));
                check(!wide_ok || find_finder_wide(*finder, text, start) == expected, tprint("find wide %", what));

                expected_last := naive_find_last(text, needle, start);

                check(find_last_finder_narrow(*finder, text, start) == expected_last, tprint("find_last narrow %", what));
                check(!wide_ok || find_last_finder_wide(*finder, text, start) == expected_last, tprint("find_last wide %", what));
            }
        }
    }
}

main :: () {
    wide_ok = CPU != .X64 || cpu_features().avx2;
    if !wide_ok print("no AVX2 here: the 32-byte versions are not run\n");

    random_seed(20261022);
    bytes();
    long_runs();
    substrings();

    if failures {
        print("% failures\n", failures);
        exit(1);
    }
}
