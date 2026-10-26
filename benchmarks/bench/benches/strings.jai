// Allocation-heavy and throughput benches, to check what the report shows for them.
#import "Basic";
#import "String";
#import "Extensions/Testing";

join_words :: (b: *Bench) {
    words := string.["alpha", "beta", "gamma", "delta"];

    set_items(b, words.count);

    for 0..b.iterations - 1 {
        text := join(..words, ",");
        black_box(text.count);
        free(text);
    }
} @bench @min_time(300)

grow_array :: (b: *Bench) {
    for 0..b.iterations - 1 {
        numbers: [..] s64;

        for 0..15 array_add(*numbers, it);

        black_box(numbers.count);
        array_free(numbers);
    }
} @bench @samples(15)

empty_loop :: (b: *Bench) {
    for 0..b.iterations - 1 {
        black_box(it);
    }
} @bench

// Nothing to do in the body: the optimizer deletes the loop, so this reports about zero.
empty_body :: (b: *Bench) {
    for 0..b.iterations - 1 {}
} @bench
