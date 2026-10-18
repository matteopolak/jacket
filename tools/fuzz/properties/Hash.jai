// Hashing a buffer in two parts, feeding the first hash in as the seed, equals hashing it whole.
fuzz_hash_chaining :: (r: *Rng) {
    input := gen(r, string);
    split := cast(s64) 0;

    if input.count > 0 split = cast(s64) (gen(r, u64) % cast(u64) (input.count + 1));

    whole := Target.sdbm_hash(input.data, input.count);
    head := Target.sdbm_hash(input.data, split);
    chained := Target.sdbm_hash(input.data + split, input.count - split, head);

    assert(whole == chained, "sdbm chaining");

    whole64 := Target.fnv1a_hash(input.data, input.count);
    head64 := Target.fnv1a_hash(input.data, split);
    chained64 := Target.fnv1a_hash(input.data + split, input.count - split, head64);

    assert(whole64 == chained64, "fnv1a chaining");
}

fuzz_hash_deterministic :: (r: *Rng) {
    input := gen(r, string);
    seed := gen(r, u32);
    assert(Target.get_hash(input, seed) == Target.get_hash(input, seed), "get_hash is a function of its input");
    assert(Target.djb2_hash(input) == Target.djb2_hash(input), "djb2 is a function of its input");
}
