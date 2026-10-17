#import "Basic";

main :: () #no_aoc {
    state: u64 = 0x7e57a11c4b932f65;

    for iteration: 0..127 {
        state = state * 6364136223846793005 + 1442695040888963407;
        bits := state;
        if ((bits >> 52) & 0x7ff) == 0x7ff bits ^= 0x0010000000000000;

        value := (cast(*float64) *bits).*;
        text := sprint("%", formatFloat(value, mode = .SHORTEST));

        defer free(text);
        parsed, success, remainder := string_to_float64(text);

        assert(success && remainder.count == 0 && (cast(*u64) *parsed).* == bits);
        narrow := cast,truncate(u32) (state >> 17);
        if ((narrow >> 23) & 0xff) == 0xff narrow ^= 0x00800000;

        single := (cast(*float32) *narrow).*;
        single_text := sprint("%", formatFloat(single, mode = .SHORTEST));

        defer free(single_text);
        parsed_single, single_ok, single_rest := string_to_float(single_text);

        assert(single_ok && single_rest.count == 0 && (cast(*u32) *parsed_single).* == narrow);
    }

    singles: [5] u32 = .[1, 0x007fffff, 0x00800000, 0x7f7fffff, 0x80000000];
    for bits: singles {
        value := (cast(*float32) *bits).*;
        text := sprint("%", formatFloat(value, mode = .SHORTEST));

        defer free(text);
        parsed, ok, rest := string_to_float(text);

        assert(ok && rest.count == 0 && (cast(*u32) *parsed).* == bits);
    }

    print("ok\n");
}
