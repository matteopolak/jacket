// Deterministic random source plus the per-case allocations made while filling values.
Rng :: struct {
    state: u64;

    // With `from_input`, every choice is read from `data` instead of the seeded sequence, as a
    // coverage-guided engine (libFuzzer) supplies it. Reads past the end give zero bytes, which
    // select the first alternative (empty, zero, the first member).
    from_input: bool;
    data: *u8;
    size: s64;
    position: s64;

    // Upper bound for generated lengths; the minimizer lowers it to shrink a failing case.
    max_len: s64 = 256;

    // Pointers are filled with null or a fresh allocation only when the target allows it.
    pointers: bool;

    // With `pointers`, a pointer may also be null; for procedures that document accepting null.
    nullable: bool;

    live: [..] *void;
}

rng_seed :: (r: *Rng, seed: u64) {
    r.state = seed;
    next(r);
}

next :: (r: *Rng) -> u64 {
    r.state += 0x9E3779B97F4A7C15;
    z := r.state;
    z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9;
    z = (z ^ (z >> 27)) * 0x94D049BB133111EB;

    return z ^ (z >> 31);
}

// The next `bytes` (1 to 8) bytes of the input, or the low bytes of the next seeded value.
take :: (r: *Rng, bytes: s64) -> u64 {
    if !r.from_input {
        value := next(r);
        if bytes >= 8 return value;
        return value & (((cast(u64) 1) << cast(u64) (8 * bytes)) - 1);
    }

    value: u64 = 0;

    for i: 0..bytes - 1 {
        if r.position >= r.size break;
        byte := cast(u64) r.data[r.position];
        value |= byte << cast(u64) (8 * i);
        r.position += 1;
    }

    return value;
}

// Uniform in [0, n); 0 when n <= 0. Reads only as many input bytes as n needs.
below :: (r: *Rng, n: s64) -> s64 {
    if n <= 1 return 0;

    bytes := 1;
    limit: u64 = 256;

    while bytes < 8 && cast(u64) n > limit {
        bytes += 1;
        limit <<= 8;
    }

    return cast(s64) (take(r, bytes) % cast(u64) n);
}

chance :: (r: *Rng, one_in: s64) -> bool {
    return below(r, one_in) == 0;
}

// Memory that lives until the end of the case; zeroed, so strings are NUL terminated.
case_alloc :: (r: *Rng, bytes: s64) -> *u8 {
    p := cast(*u8) alloc(bytes + 1);
    memset(p, 0, bytes + 1);
    array_add(*r.live, p);
    return p;
}

case_end :: (r: *Rng) {
    for r.live free(it);
    r.live.count = 0;
}
