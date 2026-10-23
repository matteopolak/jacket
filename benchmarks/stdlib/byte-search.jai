// Byte_Search: the 16-byte version, the 32-byte (AVX2) version and the one the processor picks,
// against a plain loop, over a large text. Prints GB/s for each.
//
//     jaic benchmarks/stdlib/byte-search.jai -o -exe byte-search-bench && ./byte-search-bench
//
// The two versions are reached by loading the module into the program instead of importing it,
// so that nothing but the version differs. On x64 the 32-byte version runs only with AVX2.
#import "Basic";
#load "../../stdlib/Extensions/Byte_Search/module.jai";

ROUNDS :: 21;
TEXT_BYTES :: 8 * 1024 * 1024;

sink: u64;
offset: int;
text: string;
needle: string;

now :: () -> float64 {
    return to_float64_seconds(current_time_monotonic());
}

// The best of ROUNDS runs, in GB/s over the text.
speed :: (name: string, body: Code) #expand {
    best: float64 = 1e18;

    for round: 1..ROUNDS {
        offset = round % 7;
        started := now();

        #insert body;
        elapsed := now() - started;
        if elapsed < best best = elapsed;
    }

    print("  %: % GB/s\n", name, formatFloat(cast(float64) text.count / best / 1e9, trailing_width = 1));
}

scalar_find :: (text: string, byte: u8, start: int) -> int {
    for i: start..text.count - 1 if text[i] == byte return i;
    return -1;
}

main :: () {
    text = alloc_string(TEXT_BYTES);
    state: u32 = 7;

    for *text {
        state = state * 1664525 + 1013904223;
        it.* = cast(u8) (#char "a" + (state >> 24) % 26);
    }

    needle = "xyzqzx";
    wide_ok := CPU != .X64 || cpu_features().avx2;

    print("best tier: %, 32-byte version %\n", best_vector_tier(), ifx wide_ok then "run" else "skipped");

    print("find (miss)\n");
    speed("scalar loop", #code {
        sink += cast(u64) scalar_find(text, #char "Q", offset);
    });
    speed("16-byte", #code {
        sink += cast(u64) find_byte_narrow(text, #char "Q", offset);
    });
    if wide_ok speed("32-byte", #code {
        sink += cast(u64) find_byte_wide(text, #char "Q", offset);
    });

    speed("picked", #code {
        sink += cast(u64) find(text, #char "Q", offset);
    });

    print("find_any, 3 bytes (miss)\n");
    speed("16-byte", #code {
        sink += cast(u64) find_three_narrow(text, #char "Q", #char "X", #char "Z", offset);
    });
    if wide_ok speed("32-byte", #code {
        sink += cast(u64) find_three_wide(text, #char "Q", #char "X", #char "Z", offset);
    });

    print("find_last (miss)\n");
    speed("16-byte", #code {
        sink += cast(u64) find_last_byte_narrow(text, #char "Q", text.count - offset);
    });
    if wide_ok speed("32-byte", #code {
        sink += cast(u64) find_last_byte_wide(text, #char "Q", text.count - offset);
    });

    print("count\n");
    speed("16-byte", #code {
        sink += cast(u64) count_byte_narrow(text, #char "a");
    });
    if wide_ok speed("32-byte", #code {
        sink += cast(u64) count_byte_wide(text, #char "a");
    });

    print("substring (miss)\n");
    finder := make_finder(needle);

    speed("16-byte", #code {
        sink += cast(u64) find_finder_narrow(*finder, text, offset);
    });
    if wide_ok speed("32-byte", #code {
        sink += cast(u64) find_finder_wide(*finder, text, offset);
    });

    print("%\n", sink);
}
