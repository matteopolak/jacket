// Reviewed pl_mpeg buffer API; independent bit order and borrowed-storage checks.
#import "Basic";
M :: #import "pl_mpeg";

main :: () {
    bytes := u8.[0x96, 0x3c, 0xa5, 0x7e, 0x81, 0x00];
    buffer := M.buffer_create_with_memory(bytes, false);

    if !buffer exit(2);

    assert(buffer.mode == .FIXED_MEM && !buffer.free_when_done);
    assert(M.buffer_read(buffer, 0) == 0 && buffer.bit_index == 0);

    for bit: 0..47 {
        expected := cast(s32) ((bytes[bit / 8] >> (7 - bit % 8)) & 1);
        assert(M.buffer_read(buffer, 1) == expected);
    }

    assert(!M.buffer_has(buffer, 1) && buffer.has_ended);
    M.buffer_rewind(buffer);
    assert(!buffer.has_ended && M.buffer_tell(buffer) == 0);
    bytes[0] = 0x12;
    assert(M.buffer_read(buffer, 4) == 1);
    M.buffer_align(buffer);
    assert(M.buffer_tell(buffer) == 1);
    M.buffer_skip(buffer, 8);
    assert(M.buffer_read(buffer, 8) == 0xa5);
    assert(M.buffer_write(buffer, bytes) == 0);
    M.buffer_destroy(buffer);
    assert(bytes[0] == 0x12 && bytes[5] == 0);

    buffer = M.create_for_appending(2);
    if !buffer exit(3);

    prefix := u8.[0, 0, 0, 1, 0xb3, 0x55, 0, 0, 1, 0xe0, 0x77];
    written := M.buffer_write(buffer, prefix);

    if written != prefix.count exit(4);

    assert(buffer.mode == .APPEND && M.buffer_get_size(buffer) == prefix.count);
    position := M.buffer_tell(buffer);

    assert(M.buffer_has_start_code(buffer, .SEQUENCE));
    assert(M.buffer_tell(buffer) == position && !buffer.discard_read_bytes);
    assert(M.buffer_find_start_code(buffer, .VIDEO_1));
    assert(M.buffer_read(buffer, 8) == 0x77);
    M.buffer_seek(buffer, 0);
    assert(M.buffer_skip_bytes(buffer, 0) == 3);
    M.buffer_destroy(buffer);

    buffer = M.buffer_create_with_capacity(8);
    if !buffer exit(5);

    payload := u8.[0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc];

    written = M.buffer_write(buffer, payload);
    if written != payload.count exit(6);

    assert(M.buffer_read(buffer, 12) == 0x123);
    M.buffer_discard_read_bytes(buffer);
    assert(buffer.bit_index == 4 && buffer.length == 5);
    assert(M.buffer_read(buffer, 12) == 0x456);
    M.buffer_signal_end(buffer);
    assert(M.buffer_read(buffer, 24) == 0x789abc);
    assert(!M.buffer_has(buffer, 1) && buffer.has_ended);
    M.buffer_destroy(buffer);
    print("ok\n");
}
