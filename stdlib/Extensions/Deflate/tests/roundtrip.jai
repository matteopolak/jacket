// Every container and level on varied inputs: compress, decompress, compare. Also checks the
// stored-only level and that higher levels do not grow output on compressible data.
//
//     jaic stdlib/Extensions/Deflate/tests/roundtrip.jai -interp

#load "support/helpers.jai";

round_trip :: (data: [] u8, container: Deflate_Container, level: s32, label: string) -> int {
    packed := deflate(data, container, level);
    output, ok, error := inflate(packed, container);

    message := "";

    if !ok message = tprint("% level % %: inflate failed: %", label, level, container, error);
    else if !same_bytes(output, data) message = tprint("% level % %: output differs", label, level, container);

    check(message.count == 0, message);

    size := packed.count;

    // inflate_into must agree and fit exactly.
    if ok {
        exact := NewArray(data.count, u8);
        written, ok2 := inflate_into(packed, exact, container);

        check(ok2 && written == data.count && same_bytes(exact, data), tprint("% level % %: inflate_into", label, level, container));
        free(exact.data);
        free(output.data);
    }

    free(packed.data);

    return size;
}

run_input :: (data: [] u8, label: string, levels: [] s32) {
    for container: Deflate_Container.[.RAW, .ZLIB, .GZIP] {
        for level: levels {
            round_trip(data, container, level, label);
        }
    }
}

main :: () {
    all_levels := s32.[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    some_levels := s32.[0, 1, 4, 6, 9];

    run_input(.[], "empty", all_levels);
    run_input(.[42], "one byte", all_levels);
    run_input(all_byte_values(), "all byte values", all_levels);
    run_input(bytes_of("abababababababababababababababab"), "short repeat", all_levels);
    run_input(run_bytes(5000), "long runs", all_levels);
    run_input(text_bytes(20000, 1), "text", all_levels);
    run_input(random_bytes(3000, 2), "random", all_levels);
    run_input(image_bytes(12000, 3), "image rows", some_levels);

    // Past the 32 KiB window (matches reach back the full distance) and past one block of
    // tokens (65536 literals, or more matches than that).
    wide := text_bytes(40000, 4);
    far := NewArray(100000, u8);
    pattern := random_bytes(200, 5);

    for 0..far.count - 1 {
        far[it] = pattern[it % 200];
        if it >= 32000 && it < 33000 far[it] = cast,trunc(u8) (it * 7);
    }

    run_input(wide, "multi-window text", some_levels);
    run_input(far, "distance near the window size", some_levels);

    // The same 32768 random bytes twice: the second copy is one match at the largest distance.
    block := random_bytes(32768, 8);
    twice := NewArray(65536, u8);

    for 0..65535 twice[it] = block[it % 32768];

    run_input(twice, "period equal to the window", s32.[1, 6, 9]);

    for level: s32.[1, 6, 9] {
        packed := deflate(twice, .RAW, level);
        check(packed.count < 33500, tprint("level % finds the match at distance 32768 (% bytes)", level, packed.count));
    }

    big_random := random_bytes(70000, 6);

    run_input(big_random, "random past one block", s32.[1, 6]);

    // Many literals and few matches so that one block ends before the input does.
    mixed := NewArray(200000, u8);
    lcg := Lcg.{ state = 9 };

    for *mixed {
        it.* = cast,trunc(u8) next(*lcg);
        if (it_index / 3000) % 2 == 1 it.* = cast,trunc(u8) (it_index % 17);
    }

    run_input(mixed, "several blocks", s32.[1, 6]);

    // Out-of-range levels are clamped.
    round_trip(wide, .ZLIB, -5, "clamped low");
    round_trip(wide, .ZLIB, 99, "clamped high");

    // Level 0 stores; the others shrink compressible data, and 9 is no worse than 1.
    stored := deflate(wide, .RAW, 0);
    fast := deflate(wide, .RAW, 1);
    best := deflate(wide, .RAW, 9);

    check(stored.count > wide.count, "level 0 stores");
    check(fast.count < wide.count / 2, "level 1 compresses text");
    check(best.count <= fast.count, "level 9 is no larger than level 1");

    assert(failures == 0);
    print("ok\n");
}
