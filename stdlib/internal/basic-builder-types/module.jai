#module_parameters(ENABLE_ASSERT := true);

STRING_BUILDER_BUFFER_SIZE :: 4096 - size_of(String_Builder.Buffer);

String_Builder :: struct {
    Buffer :: struct {
        count: s64;
        allocated: s64;
        #if ENABLE_ASSERT ensured_count: s64;
        next: *Buffer;
    }
    initialized := false;
    failed := false;
    subsequent_buffer_size := -1;
    allocator: Allocator;
    current_buffer: *Buffer;
    initial_bytes: [STRING_BUILDER_BUFFER_SIZE] u8;
}
