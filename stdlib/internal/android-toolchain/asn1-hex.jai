// Returns the bytes of `s` as lowercase hex, two digits per byte.
to_hex_string :: (s: string) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);

    for byte: s print_to_builder(*builder, "%", fh2(byte));

    return builder_to_string(*builder);
}

// Returns the eight integers of `a` as hex strings, zero-padded to the size of the element, joined.
to_hex_string :: (a: [8] $T) -> string {
    #assert type_info(T).type == .INTEGER;
    builder: String_Builder;

    defer free_buffers(*builder);

    for value: a print_to_builder(*builder, "%", fh(value, minimum_digits = 2 * size_of(T)));

    return builder_to_string(*builder);
}

#scope_file
using #import "Basic";
