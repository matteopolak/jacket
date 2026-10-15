// Positive baseline for the reviewed public Hash surface and privacy rejection.
#import "Basic";
H :: #import "Hash";

main :: () {
    assert(H.FNV_64_PRIME == 0x100000001b3);
    assert(H.FNV_64_OFFSET_BIAS == 0xcbf29ce484222325);
    assert(H.get_hash(cast(u64) 0) == cast(u32) (H.knuth_hash(5381) >> 32));
    print("ok\n");
}
