#import "Basic";
#import "Compiler";
#import "Program_Print";

#run {
    values: [14] Code = .[
        (#code 0.1234567), (#code 0.12345678), (#code 0.123456789),
        (#code 1.0e39), (#code 1.0e-39), (#code -0.0),
        (#code 0h3f800000), (#code 0h3ff0000000000000),
        (#code 0h7ff8000000001234), (#code -0h3f800000),
        (#code 0xffff_ffff_ffff_ffff), (#code -0x7fff),
        (#code 0b101010), (#code -0b101010)
    ];
    for value, index: values {
        original := cast(*Code_Literal) compiler_get_nodes(value);
        if index == 0 assert(!(original.value_flags & (.DEFAULTS_TO_FLOAT64 | .REQUIRES_FLOAT64)));
        if index == 1 assert((original.value_flags & .DEFAULTS_TO_FLOAT64) != 0);
        if index == 2 assert((original.value_flags & .REQUIRES_FLOAT64) != 0);

        copy := cast(*Code_Literal) compiler_get_nodes(compiler_get_code(original, value));

        assert(copy.kind == .LITERAL && original.value_type == copy.value_type);
        assert(original.value_flags == copy.value_flags);
        assert(original._u64 == copy._u64, "literal bits must round trip exactly");
    }

    // Editing the public literal is authoritative, rather than a source-text fallback.
    original := #code 7;
    node := cast(*Code_Literal) compiler_get_nodes(original);
    node._s64 = 19;
    copy := cast(*Code_Literal) compiler_get_nodes(compiler_get_code(node, original));

    assert(copy._s64 == 19);
    node.kind = .FOR;
    builder: String_Builder;

    assert(!print_expression(*builder, node), "a record kind mismatch must be rejected before casting");
    free_buffers(*builder);
}

main :: () {}
