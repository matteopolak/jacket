#import "Basic";
H :: #import "Hash";

main :: () {
    assert(H.fnv1a_hash(cast(u64) 0) == 0xaf63bd4c8601b7df);
    assert(H.fnv1a_hash(cast(u64) 1) == 0xaf63bc4c8601b62c);
    assert(H.fnv1a_hash(cast(u64) 0xffffffffffffffff) == 0x509c41b379fe466e);
    assert(H.fnv1a_hash(val = cast(u64) 0x0102030405060708, h = cast(u64) 0) == 0xbc7227d4893df298);
    bytes: [8] u8 = .[8, 7, 6, 5, 4, 3, 2, 1];

    assert(H.fnv1a_hash(bytes.data, bytes.count, h = cast(u64) 0) != H.fnv1a_hash(cast(u64) 0x0102030405060708, h = cast(u64) 0));
    assert(H.fnv1a_hash(null, 0, h = cast(u64) 17) == 17);
    print("ok\n");
}
