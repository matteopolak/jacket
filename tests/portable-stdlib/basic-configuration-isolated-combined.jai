B :: #import "Basic"()(MEMORY_DEBUGGER = true, ENABLE_ASSERT = false, TEMP_ALLOCATOR_OVERWRITE_MEMORY = true, VISUALIZE_MEMORY_DEBUGGER = false);

check :: (condition: bool, loc := #caller_location) {
    if !condition context.assertion_failed(loc, "configuration regression failed");
}

main :: () {
    #assert !#exists(B.String_Builder.Buffer.ensured_count);
    check(size_of(B.String_Builder.Buffer) == 24);
    check_combined_configuration();
    B.print("ok\n");
}

check_combined_configuration :: () {
    check(size_of(B.String_Builder.Buffer) == 24);
    builder: B.String_Builder;
    bytes: [5000] u8;

    B.append(*builder, bytes.data, bytes.count);
    check(B.builder_string_length(*builder) == bytes.count);
    B.free_buffers(*builder);
    initial: [128] u8;
    storage: B.Temporary_Storage_Paged_Alloc;
    storage.data = initial.data;
    storage.size = initial.count;

    push_context {
        context.temporary_storage = *storage;
        mark := B.get_temporary_storage_mark();
        memory := cast(*u8) B.talloc(512);
        memory[0] = 42;
        check(storage.overflow_pages != null);
        B.set_temporary_storage_mark(mark);
        check(storage.overflow_pages == null && storage.total_bytes_occupied == 0);
        retained := cast(*u8) B.talloc(16);
        retained[0] = 42;
        B.reset_temporary_storage();
        check(retained[0] == B.RESET_STORAGE_STAMP);
    }
}
