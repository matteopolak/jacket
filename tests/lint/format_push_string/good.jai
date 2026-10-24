#import "Basic";

describe :: (name: string, value: int) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);

    print_to_builder(*builder, "%: %\n", name, value);

    append(*builder, "literal");
    append(*builder, name);

    text := tprint("%", value);

    append(*builder, text);

    return builder_to_string(*builder);
}

main :: () {
    print("%\n", describe("x", 1));
}
