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

// The five hashes over suffixes of the text of 47 to 78 bytes, and over an integer.
hash_mixed :: (b: *Bench) {
    checksum: u64 = 1469598103934665603;

    mix :: (value: u64) #expand {
        checksum = (checksum ^ value) * 1099511628211;
    }

    for 0..b.iterations - 1 {
        piece := string.{ TEXT.count - it % 32, TEXT.data + it % 32 };

        mix(fnv1a_hash(piece.data, piece.count));
        mix(sdbm_hash(piece.data, piece.count));
        mix(djb2_hash(piece));
        mix(get_hash(it));
        mix(get_hash(piece));
    }

    black_box(checksum);
} @bench
