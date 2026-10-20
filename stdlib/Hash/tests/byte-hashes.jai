// sdbm_hash and djb2_hash fold several bytes per step; every length must give what one byte per step gives.
#import "Basic";
H :: #import "Hash";

main :: () #no_aoc {
    text := "Sphinx of black quartz, judge my vow! 0123456789";

    for length: 0..40 {
        for start: 0..3 {
            piece := string.{ length, text.data + start };

            sdbm: u32 = 5381;
            djb2: u32 = 5381;

            for byte: piece {
                sdbm = sdbm * 65599 + byte;
                djb2 = djb2 * 33 + byte;
            }

            assert(H.sdbm_hash(piece.data, piece.count) == sdbm);
            assert(H.djb2_hash(piece) == djb2);
        }
    }

    print("ok\n");
}
