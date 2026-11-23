#module_parameters(ENABLE_ASSERT := true);

// The room, in bytes, a `String_Builder` has inside it and the default size of each buffer it adds later:
// a 4 KiB buffer, or 256 bytes where addresses are 16 bits or fewer and a builder on the stack must fit
// in a few KiB of RAM.
STRING_BUILDER_BYTES :: ifx size_of(*void) < 4 then 256 else 4096;
// The bytes of text a `String_Builder`'s own buffer holds: `STRING_BUILDER_BYTES` less the buffer's header.
STRING_BUILDER_BUFFER_SIZE :: STRING_BUILDER_BYTES - size_of(String_Builder.Buffer);

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
