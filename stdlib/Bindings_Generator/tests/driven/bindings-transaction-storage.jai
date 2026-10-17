// Public builder defaults and generator fragment failure/storage regression.
// Provenance: reviewed Basic builder_to_string and Bindings_Generator printers.
#import "Basic";
B :: #import "Bindings_Generator";

errors: s64;

capture :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

main :: () {
    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, "preserve");
    consumed := builder_to_string(*builder);

    defer free(consumed);
    assert(consumed == "preserve");
    assert(builder_string_length(*builder) == 0);

    append(*builder, "preserve");
    cycle: B.Paren_Expression;
    cycle.kind = .PAREN_EXPRESSION;
    cycle.child = cast(*B.Expression) *cycle;
    assert(!B.append_cpp_expression(*builder, cast(*B.Expression) *cycle));
    assert(!builder.failed);
    assert(builder_string_length(*builder) == "preserve".count);
    unchanged := builder_to_string(*builder, do_reset = false);

    defer free(unchanged);
    assert(unchanged == "preserve");

    push_context {
        context.logger = capture;
        B.print_expression_to_builder(*builder, cast(*B.Expression) *cycle);
    }

    assert(builder.failed);
    assert(errors == 1);
    assert(builder_string_length(*builder) == "preserve".count);
    print("ok\n");
}
