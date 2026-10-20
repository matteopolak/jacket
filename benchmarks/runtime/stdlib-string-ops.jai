// String helpers over a large text: find, split, replace, compare, case changes and join.
#import "Basic";
#import "String";

main :: () {
    builder: String_Builder;
    state: u32 = 3;

    // Lines of lower-case words separated by spaces and commas.
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

    text := builder_to_string(*builder);
    checksum: u64 = 1469598103934665603;

    mix :: (value: int) #expand {
        checksum = (checksum ^ cast(u64) value) * 1099511628211;
    }

    for round: 0..19 {
        // Byte and substring search.
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

        trimmed := trim(lines[round]);

        mix(trimmed.count);

        free(joined);
        free(upper);
        free(lower);
        free(replaced);
        array_free(lines);
    }

    print("% %\n", text.count, checksum);
}
