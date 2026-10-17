// Official native observation: explicit null descriptor is invalid (SEGV).
// Keep descriptor validation; this is distinct from a default Type value.
#import "Basic";

main :: () {
    builder: String_Builder;

    defer free_buffers(*builder);
    print_type_to_builder(*builder, null);
    print("%\n", builder_to_string(*builder));
    print("%\n", type_to_string(null));
}
