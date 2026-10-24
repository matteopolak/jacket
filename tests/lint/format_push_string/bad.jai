#import "Basic";

describe :: (name: string, value: int) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);

    append(*builder, tprint("%: %\n", name, value));

    text := sprint("%", value + 1);

    append(*builder, text);

    append(*builder, sprint("[%]", name));

    return builder_to_string(*builder);
}

main :: () {
    print("%\n", describe("x", 1));
}
