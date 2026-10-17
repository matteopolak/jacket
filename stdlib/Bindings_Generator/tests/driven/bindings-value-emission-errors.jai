#import "Basic";
B :: #import "Bindings_Generator";

errors: s64;

capture :: (message: string, data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

main :: () {
    for mode: 0..3 {
        destination: String_Builder;

        defer free_buffers(*destination);
        append(*destination, "keep");
        literal: B.Literal;
        literal.literal_kind = .ENUM;
        cyclic := B.Paren_Expression.{ kind = .PAREN_EXPRESSION };
        cyclic.child = cast(*B.Expression) *cyclic;
        bad_binary := B.Binary_Operator.{ kind = .BINARY_OPERATOR, op = "=" };

        push_context {
            context.logger = capture;
            if mode == 0 assert(!B.print_literal_to_builder(*destination, *literal));
            if mode == 1 assert(!B.print_expression_to_builder(*destination, cast(*B.Expression) *cyclic));

            if mode == 2 {
                literal.literal_kind = .FLOAT;
                bits: u64 = 0x7ff0000000000000;
                literal.float_value = (cast(*float64) *bits).*;
                assert(!B.print_literal_to_builder(*destination, *literal));
            }

            if mode == 3 assert(!B.print_expression_to_builder(*destination, cast(*B.Expression) *bad_binary));
        }

        assert(destination.failed && builder_string_length(*destination) == 4);
    }

    assert(errors == 4);
    print("ok\n");
}
