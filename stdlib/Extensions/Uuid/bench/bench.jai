// jaic -O2 stdlib/Extensions/Uuid/bench/bench.jai -exe /tmp/uuid-bench
// Five rounds of 200000 operations; best ns/op includes ownership cleanup.
#import "Basic";
#import "Extensions/Uuid";

N :: 200000;
TEXT :: "919108f7-52d1-4320-9bac-f847db4148a8";
sink: u64;

main :: () {
    value, _ := parse_uuid(TEXT);

    for mode: 0..3 {
        best: float64 = 1.0e30;

        for 1..5 {
            started := current_time_monotonic();

            for 1..N {
                if mode == 0 {
                    id, error := parse_uuid(TEXT);
                    assert(error.kind == .NONE);
                    sink += id.bytes[0];
                } else if mode == 1 {
                    text := format_uuid(value);
                    sink += text[0];
                    free(text);
                } else if mode == 2 {
                    id, error := uuid_v4();
                    assert(error.kind == .NONE);
                    sink += id.bytes[0];
                } else {
                    id, error := uuid_v7();
                    assert(error.kind == .NONE);
                    sink += id.bytes[15];
                }
            }

            elapsed, _ := to_float64_seconds(current_time_monotonic() - started);

            best = min(best, elapsed * 1.0e9 / N);
        }

        print("%: % ns/op\n", string.["parse", "format", "v4", "v7"][mode], best);
    }

    print("checksum %\n", sink);
}
