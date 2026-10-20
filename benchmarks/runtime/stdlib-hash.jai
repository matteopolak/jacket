// The Hash module's functions over strings of varying length and over integers.
#import "Basic";
#import "Hash";

main :: () {
    checksum: u64 = 1469598103934665603;

    mix :: (value: u64) #expand {
        checksum = (checksum ^ value) * 1099511628211;
    }

    text := "The quick brown fox jumps over the lazy dog, again and again and again and again.";

    for 0..2_000_000 - 1 {
        piece := string.{ text.count - it % 32, text.data + it % 32 };

        mix(fnv1a_hash(piece.data, piece.count));
        mix(sdbm_hash(piece.data, piece.count));
        mix(djb2_hash(piece));
        mix(get_hash(it));
        mix(get_hash(piece));
    }

    print("%\n", checksum);
}
