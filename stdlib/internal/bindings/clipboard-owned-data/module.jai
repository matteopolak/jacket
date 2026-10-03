// The public buffer owns these allocations in its canonical Jai allocator
// domain. Its platform facade supplies the display/selection synchronization.
release :: (buffer: *$T) {
    if buffer.text_data.data Basic.free(buffer.text_data);
    if buffer.rgb_data Basic.free(buffer.rgb_data);

    buffer.text_data = .{};
    buffer.rgb_data = null;
    buffer.width = 0;
    buffer.height = 0;
    buffer.pitch = 0;
}

#scope_file
Basic :: #import "Basic";
