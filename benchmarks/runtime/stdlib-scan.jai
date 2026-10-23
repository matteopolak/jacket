// String and array searches over long inputs: byte and substring search from both ends,
// find_index_of_any, compare, and array_find over integers and bytes.
#import "Basic";
#import "String";

main :: () {
    builder: String_Builder;
    state: u32 = 3;

    // Lines of lower-case words, so upper-case probes always miss and "xyzq" is rare.
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

    text := builder_to_string(*builder);
    copy := copy_string(text);
    checksum: u64 = 1469598103934665603;

    mix :: (value: int) #expand {
        checksum = (checksum ^ cast(u64) value) * 1099511628211;
    }

    for round: 0..39 {
        // Each search starts a little further in, so no two are the same call.
        shifted := string.{ text.count - round, text.data + round };

        mix(find_index_from_left(shifted, #char "Q"));
        mix(find_index_from_right(shifted, #char "Q"));
        mix(find_index_of_any_from_left(shifted, "QXZ"));
        mix(find_index_of_any_from_right(shifted, "QX"));
        mix(find_index_from_left(shifted, "xyzqz"));
        mix(find_index_from_right(shifted, "xyzqz"));
        mix(compare(shifted, string.{ copy.count - round, copy.data + round }));
    }

    // Short strings, where the vector searches must not cost more than the loop.
    line := string.{ 60, text.data };

    for 0..2_000_000 - 1 {
        mix(find_index_from_left(line, #char "Q", it % 4));
        mix(find_index_from_right(line, #char "Q"));
    }

    values: [..] s32;
    bytes: [..] u8;

    for 0..999_999 {
        array_add(*values, cast(s32) it);
        array_add(*bytes, cast(u8) (it % 251));
    }

    for 0..199 {
        found, index := array_find(values, cast(s32) (-1 - it));

        mix(index);
        found, index = array_find(bytes, cast(u8) 255);
        mix(index);
        found, index = array_find(values, cast(s32) (999_000 + it));
        mix(index);
    }

    print("% %\n", text.count, checksum);
}
