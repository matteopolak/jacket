// Chunking must never change the result. The llhttp fixtures, and mutations of them (a flipped,
// dropped, doubled or inserted byte, from a fixed random sequence), are fed whole, a byte at a
// time and cut in random places, with the vector and scalar scans: all of those runs must report
// the same heads, body bytes, trailers and error. Mutated input also has to leave the parser
// failed or waiting, never stuck or crashed.
//
//     jaic stdlib/Extensions/Http/tests/chunking.jai -interp

#load "lib/support.jai";
#load "data/llhttp_cases.jai";

MUTATIONS :: 24;

seed: u64 = 0x9e3779b97f4a7c15;

random :: (limit: int) -> int {
    seed = seed * 6364136223846793005 + 1442695040888963407;

    return cast(int) ((seed >> 33) % cast(u64) limit);
}

SPECIAL :: string.["\r", "\n", ":", " ", "\t", "\x00", ";", ",", "0", "9", "f", "\x7f", "\xff"];

mutate :: (input: string) -> string {
    bytes: [..] u8;

    defer array_reset(*bytes);

    for input array_add(*bytes, it);

    if bytes.count == 0 return input;

    where := random(bytes.count);
    which := random(4);

    if which == {
        case 0;
            special := SPECIAL[random(SPECIAL.count)];

            bytes[where] = special[0];

        case 1;
            array_ordered_remove_by_index(*bytes, where);

        case 2;
            array_insert_at(*bytes, bytes[where], where);

        case;
            special := SPECIAL[random(SPECIAL.count)];

            array_insert_at(*bytes, special[0], where);
    }

    return copy_string(as_string(bytes.data, bytes.count));
}

random_cuts :: (count: int, pieces: int) -> [..] int {
    cuts: [..] int;

    for 1..pieces - 1 array_add(*cuts, random(count + 1));

    // Ascending.
    for i: 1..cuts.count - 1 {
        j := i;

        while j > 0 && cuts[j - 1] > cuts[j] {
            swap := cuts[j];
            cuts[j] = cuts[j - 1];
            cuts[j - 1] = swap;
            j -= 1;
        }
    }

    return cuts;
}

compare_all_cuts :: (run: Run, input: string, name: string) {
    http_use_simd = true;
    whole := observe(run, input, .{});

    check(observe(run, input, .{ -1 }) == whole, tprint("% differs byte by byte", name));

    for 1..3 {
        cuts := random_cuts(input.count, 2 + random(5));

        defer array_reset(*cuts);

        if observe_at(run, input, cuts) != whole {
            check(false, tprint("% differs when cut at %", name, cuts));

            break;
        }
    }

    http_use_simd = false;
    scalar := observe(run, input, .{});

    http_use_simd = true;

    check(scalar == whole, tprint("% differs between the vector and scalar scans", name));
}

main :: () {
    inputs := 0;

    for fixture: LLHTTP_CASES {
        run := default_run(ifx fixture.response then Parser_Kind.RESPONSE else Parser_Kind.REQUEST);

        compare_all_cuts(run, fixture.input, fixture.id);
        inputs += 1;

        for 1..MUTATIONS {
            mutated := mutate(fixture.input);
            compare_all_cuts(run, mutated, tprint("% (mutated: %)", fixture.id, mutated));
            inputs += 1;
        }
    }

    // Two messages back to back: every cut across the seam.
    first := LLHTTP_CASES[0].input;
    second := "POST /two HTTP/1.1\r\nHost: a\r\nContent-Length: 3\r\n\r\nabc";
    joined := tprint("%%", first, second);

    inputs += 1;
    compare_all_cuts(default_run(.REQUEST), joined, "two messages");

    print("% inputs, each cut in many ways\n", inputs);
    finish();
}
