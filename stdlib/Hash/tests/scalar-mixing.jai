// Scalar hashing follows the reviewed one-value mixing contract.
#import "Basic";
Hash :: #import "Hash";

main :: () {
    value: u64 = 0x0123456789abcdef;
    seed: u32 = 0x50607080;
    mixed := Hash.knuth_hash(value ^ cast(u64) seed);

    assert(Hash.get_hash(value, seed) == cast(u32) (mixed >> 32));
    mixed_default := Hash.knuth_hash(value ^ cast(u64) 5381);

    assert(Hash.get_hash(value) == cast(u32) (mixed_default >> 32));

    fnv_seed: u64 = 0x1020304050607080;
    scalar_fnv := Hash.fnv1a_hash(value, fnv_seed);

    assert(scalar_fnv == (value ^ fnv_seed) * Hash.FNV_64_PRIME);
    assert(Hash.fnv1a_hash(value) == (value ^ Hash.FNV_64_OFFSET_BIAS) * Hash.FNV_64_PRIME);
    print("ok\n");
}
