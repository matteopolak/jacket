to_hex_string :: (s: string) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);

    for byte: s print_to_builder(*builder, "%", fh2(byte));

    return builder_to_string(*builder);
}

to_hex_string :: (a: [8] $T) -> string {
    #assert type_info(T).type == .INTEGER;
    builder: String_Builder;

    defer free_buffers(*builder);

    for value: a print_to_builder(*builder, "%", fh(value, minimum_digits = 2 * size_of(T)));

    return builder_to_string(*builder);
}

#scope_file
using #import "Basic";
