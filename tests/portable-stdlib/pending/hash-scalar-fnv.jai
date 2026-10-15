B :: #import "Basic";
H :: #import "Hash";

main :: () {
    value := H.fnv1a_hash(cast(u64) 1);
    B.assert(value != 0);
}
