// Reviewed hex helpers intentionally reject a 16-byte target.
#import "Basic";
B :: #import "Bindings_Generator";

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);
    wide := B.CType.{ size = 16 };
    builder: String_Builder;

    defer free_buffers(*builder);
    if args[1] == "signed" B.append_as_signed_integer_hex(*builder, 42, *wide);
    else B.append_as_unsigned_integer_hex(*builder, 42, *wide);
}
