// Deterministic random source plus the per-case allocations made while filling values.
Rng :: struct {
    state: u64;

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

// Uniform in [0, n); 0 when n <= 0.
below :: (r: *Rng, n: s64) -> s64 {
    if n <= 0 return 0;
    return cast(s64) (next(r) % cast(u64) n);
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
