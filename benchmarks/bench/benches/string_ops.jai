// String helpers over a large text (find, split, replace, compare, case changes, join), searches
// over long inputs and the path helpers. The texts are made once, on the first call.
#import "Basic";
#import "String";
#import "Extensions/Testing";

ops_text: string;
scan_text: string;
scan_copy: string;

// Lines of lower-case words separated by spaces and commas.
make_ops_text :: () {
    builder: String_Builder;
    state: u32 = 3;

    for line: 0..20_000 - 1 {
        for word: 0..11 {
            if word append(*builder, ifx word % 4 == 0 then ", " else " ");

            length := 2 + (line + word * 7) % 9;
            for 0..length - 1 {
                state = state * 1664525 + 1013904223;
                append(*builder, cast(u8) (#char "a" + (state >> 24) % 26));
            }
        }

        append(*builder, "\n");
    }

    ops_text = builder_to_string(*builder);
}

// Lines of lower-case words, so upper-case probes always miss and "xyzq" is rare.
make_scan_text :: () {
    builder: String_Builder;
    state: u32 = 3;

    for line: 0..30_000 - 1 {
        for word: 0..11 {
            if word > 0 append(*builder, ifx word % 4 == 0 then ", " else " ");

            length := 2 + (line + word * 7) % 9;
            for 0..length - 1 {
                state = state * 1664525 + 1013904223;
                append(*builder, cast(u8) (#char "a" + (state >> 24) % 26));
            }
        }

        append(*builder, "\n");
    }

    scan_text = builder_to_string(*builder);
    scan_copy = copy_string(scan_text);
}

// One round of the text workload: byte and substring search, split, compare, join, case changes,
// replace and trim.
string_ops_round :: (b: *Bench) {
    if !ops_text.count make_ops_text();

    reset_timer(b);
    set_bytes(b, ops_text.count);

    for round: 0..b.iterations - 1 {
        checksum: u64 = 1469598103934665603;

        mix :: (value: int) #expand {
            checksum = (checksum ^ cast,no_check(u64) value) * 1099511628211;
        }

        text := ops_text;
        at := 0;
        found := 0;

        while true {
            at = find_index_from_left(text, #char "q", at);
            if at < 0 break;
            found += 1;
            at += 1;
        }

        mix(found);

        at = 0;
        found = 0;

        while true {
            at = find_index_from_left(text, "xyz", at);
            if at < 0 break;
            found += 1;
            at += 1;
        }

        mix(found);
        mix(find_index_from_right(text, "abcdefgh"));
        mix(xx contains(text, "zzzzzz"));

        lines := split(text, #char "\n");

        mix(lines.count);

        previous := "";

        for line: lines {
            words := split(line, ", ");

            mix(words.count);
            mix(compare(line, previous));
            if equal(line, previous) mix(1);
            if begins_with(line, "ab") mix(2);

            previous = line;
            array_free(words);
        }

        joined := join(..lines, separator = "|");

        mix(joined.count);

        upper := to_upper_copy(joined);
        lower := to_lower_copy(upper);

        mix(compare(lower, joined));
        mix(cast(int) upper[upper.count / 2]);

        replaced, count := replace(text, "e", "EE");

        mix(replaced.count + count);

        trimmed := trim(lines[round % lines.count]);

        mix(trimmed.count);

        black_box(checksum);
        free(joined);
        free(upper);
        free(lower);
        free(replaced);
        array_free(lines);
    }
} @bench @samples(10)

// Byte, any-of and substring search from both ends and compare over a long text, 40 shifted
// starts per iteration.
string_scan_long :: (b: *Bench) {
    if !scan_text.count make_scan_text();

    reset_timer(b);
    set_bytes(b, scan_text.count * 40);

    for 0..b.iterations - 1 {
        checksum: u64 = 1469598103934665603;

        mix :: (value: int) #expand {
            checksum = (checksum ^ cast,no_check(u64) value) * 1099511628211;
        }

        for round: 0..39 {
            // Each search starts a little further in, so no two are the same call.
            shifted := string.{ scan_text.count - round, scan_text.data + round };

            mix(find_index_from_left(shifted, #char "Q"));
            mix(find_index_from_right(shifted, #char "Q"));
            mix(find_index_of_any_from_left(shifted, "QXZ"));
            mix(find_index_of_any_from_right(shifted, "QX"));
            mix(find_index_from_left(shifted, "xyzqz"));
            mix(find_index_from_right(shifted, "xyzqz"));
            mix(compare(shifted, string.{ scan_copy.count - round, scan_copy.data + round }));
        }

        black_box(checksum);
    }
} @bench @samples(10)

// Short strings, where the vector searches must not cost more than the loop.
string_scan_short :: (b: *Bench) {
    if !scan_text.count make_scan_text();

    reset_timer(b);
    set_items(b, 2);

    line := string.{ 60, scan_text.data };
    checksum: u64 = 1469598103934665603;

    for 0..b.iterations - 1 {
        checksum = (checksum ^ cast,no_check(u64) find_index_from_left(line, #char "Q", it % 4)) * 1099511628211;
        checksum = (checksum ^ cast,no_check(u64) find_index_from_right(line, #char "Q")) * 1099511628211;
    }

    black_box(checksum);
} @bench

// Path helpers: file names, directories and extensions of five paths in turn.
path_helpers :: (b: *Bench) {
    checksum: u64 = 1469598103934665603;

    mix :: (value: int) #expand {
        checksum = (checksum ^ cast,no_check(u64) value) * 1099511628211;
    }

    paths := string.["/usr/local/lib/libthing.so.1", "relative/dir/file.tar.gz", "C:/Program Files/app/run.exe", "noext", "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p.txt"];

    for 0..b.iterations - 1 {
        path := paths[it % paths.count];

        mix(path_filename(path).count);
        mix(path_strip_filename(path).count);
        extension, found := path_extension(path);

        mix(extension.count + cast(int) found);
        mix(path_strip_extension(path).count);
    }

    black_box(checksum);
} @bench
