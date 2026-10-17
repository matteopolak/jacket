#import "Basic";
B :: #import "Bindings_Generator";

main :: () {
    builder: String_Builder;
    last: B.Location;
    defer free_buffers(*builder);
    B.maybe_add_spacing(*builder, *last, .{}, 0, B.Mimic_Spacing_Flags.STRUCT);
}
