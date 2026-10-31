// Hash functions for strings, bytes and scalars: `get_hash` for general use, `fnv1a_hash` and `sdbm_hash` for bytes, `djb2_hash` for strings.
//
// The functions are fast and well spread, not secure: don't use them for passwords or to guard against crafted input.
//
// ```jai
// assert(get_hash("jacket") == get_hash("jacket"));
// assert(get_hash(1) != get_hash(2));
// ```

sdbm_hash :: Impl.sdbm_hash;
sdbm_float_hash :: Impl.sdbm_float_hash;
djb2_hash :: Impl.djb2_hash;
FNV_64_PRIME :: Impl.FNV_64_PRIME;
FNV_64_OFFSET_BIAS :: Impl.FNV_64_OFFSET_BIAS;
fnv1a_hash :: Impl.fnv1a_hash;
knuth_hash :: Impl.knuth_hash;
get_hash :: Impl.get_hash;

#scope_file
Impl :: #import "internal/hash";
