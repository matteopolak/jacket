#import "Basic";

main :: () {
    builder: String_Builder;
    defer free_buffers(*builder);
    ensure_contiguous_space(*builder, 1);
    advance_through_ensured_space(*builder, 2);
}
