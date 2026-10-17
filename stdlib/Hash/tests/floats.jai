#import "Basic";
H :: #import "Hash";

main :: () {
    assert(H.sdbm_hash(null, 0) == 5381);
    assert(H.fnv1a_hash(null, 0) == H.FNV_64_OFFSET_BIAS);
    assert(H.djb2_hash("hello") == 261238937);
    assert(H.sdbm_hash("hello".data, 5) == 1000974957);
    assert(H.fnv1a_hash("hello".data, 5) == 0xa430d84680aabd0b);
    positive: float32 = 0;
    bits: u32 = 0x80000000;
    negative := (cast(*float32) *bits).*;

    assert(H.get_hash(positive) == H.sdbm_hash(*positive, 4));
    assert(H.get_hash(negative) == H.sdbm_hash(*negative, 4));
    assert(H.get_hash(positive) != H.get_hash(negative));
    assert(H.sdbm_float_hash(*positive, 1) == H.sdbm_float_hash(*negative, 1));
    wide: float64 = 1.25;

    assert(H.get_hash(wide, 7) == H.sdbm_hash(*wide, 8, 7));
    print("ok\n");
}
