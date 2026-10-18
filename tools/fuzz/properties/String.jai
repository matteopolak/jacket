// Case conversion keeps the length of ASCII text, and splitting then joining returns the input.
fuzz_string_split_join :: (r: *Rng) {
    input := gen(r, string);
    separator := gen(r, string);

    if separator.count == 0 return;

    parts := Target.split(input, separator);
    joined := Target.join(..parts, separator = separator);

    assert(joined == input, "split then join");
}

fuzz_string_trim :: (r: *Rng) {
    input := gen(r, string);
    trimmed := Target.trim(input);
    assert(trimmed.count <= input.count, "trim never grows");
    assert(Target.trim(trimmed) == trimmed, "trim is idempotent");
}
