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

// `slice` asserts that its index and count are not negative, so they are drawn here instead of
// generated: the result starts at the index and is cut short by the end of the input.
fuzz_string_slice :: (r: *Rng) {
    input := gen(r, string);
    index := below(r, input.count + 3);
    count := below(r, input.count + 3);
    part := Target.slice(input, index, count);

    if index >= input.count {
        assert(part.count == 0, "slice past the end is empty");
        return;
    }

    assert(part.count == min(count, input.count - index), "slice length");
    assert(part.data == input.data + index, "slice starts at the index");
}
