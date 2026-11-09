// Run with jacket bench --path stdlib/Extensions/Uuid.
#import "Extensions/Testing";
#import "Basic";
#import "Extensions/Uuid";

TEXT :: "919108f7-52d1-4320-9bac-f847db4148a8";

uuid_parse :: (b: *Bench) {
    for 0..b.iterations - 1 {
        id, error := parse_uuid(black_box(TEXT));
        assert(error.kind == .NONE);
        black_box(id);
    }
} @bench

uuid_format :: (b: *Bench) {
    value, _ := parse_uuid(TEXT);

    reset_timer(b);

    for 0..b.iterations - 1 {
        text := format_uuid(black_box(value));
        black_box(text);
        free(text);
    }
} @bench

uuid_v4 :: (b: *Bench) {
    for 0..b.iterations - 1 {
        id, error := uuid_v4();
        assert(error.kind == .NONE);
        black_box(id);
    }
} @bench

uuid_v7 :: (b: *Bench) {
    for 0..b.iterations - 1 {
        id, error := uuid_v7();
        assert(error.kind == .NONE);
        black_box(id);
    }
} @bench
