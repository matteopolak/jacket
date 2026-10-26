// The Hash module's functions over a string and over integers. The input goes through
// `black_box` inside the loop: otherwise the optimizer computes the (pure) hash once.
#import "Hash";
#import "Extensions/Testing";

TEXT :: "The quick brown fox jumps over the lazy dog, again and again and again and again.";

fnv1a :: (b: *Bench) {
    set_bytes(b, TEXT.count);

    for 0..b.iterations - 1 {
        text := black_box(TEXT);
        black_box(fnv1a_hash(text.data, text.count));
    }
} @bench

sdbm :: (b: *Bench) {
    set_bytes(b, TEXT.count);

    for 0..b.iterations - 1 {
        text := black_box(TEXT);
        black_box(sdbm_hash(text.data, text.count));
    }
} @bench

djb2 :: (b: *Bench) {
    set_bytes(b, TEXT.count);

    for 0..b.iterations - 1 {
        black_box(djb2_hash(black_box(TEXT)));
    }
} @bench

hash_string :: (b: *Bench) {
    set_bytes(b, TEXT.count);

    for 0..b.iterations - 1 {
        black_box(get_hash(black_box(TEXT)));
    }
} @bench

hash_integer :: (b: *Bench) {
    set_items(b, 1);

    for 0..b.iterations - 1 {
        black_box(get_hash(black_box(it)));
    }
} @bench @samples(20)
