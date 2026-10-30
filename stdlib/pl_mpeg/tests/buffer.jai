#import "Basic";
M :: #import "pl_mpeg";
F :: #import "File";
S :: #import "String";

Load_State :: struct {
    calls: s64;
}

load :: (buffer: *M.Buffer, user: *void) {
    state := cast(*Load_State) user;
    state.calls += 1;
    assert(M.buffer_write(buffer, u8.[0x98, 0x76]) == 2);
}

main :: () {
    assert((cast(*Type_Info_Struct) type_info(M.Buffer)).name == "Buffer");
    bytes := u8.[0b10110110, 0b01101001, 0, 0, 1, 0xb3, 0x5a];
    buffer := M.buffer_create_with_memory(bytes, false);

    assert(buffer != null && M.buffer_get_size(buffer) == bytes.count);
    assert(M.buffer_read(buffer, 3) == 5 && M.buffer_read(buffer, 5) == 22);
    assert(M.buffer_read(buffer, 4) == 6 && M.buffer_read(buffer, 4) == 9);
    assert(M.buffer_has_start_code(buffer, .SEQUENCE));
    assert(M.buffer_tell(buffer) == 2);
    assert(M.buffer_next_start_code(buffer) == .SEQUENCE);
    assert(M.buffer_tell(buffer) == 6 && M.buffer_read(buffer, 8) == 0x5a);
    assert(!M.buffer_has(buffer, 1) && buffer.has_ended);
    M.buffer_rewind(buffer);
    bytes[0] = 0xf0;
    assert(M.buffer_read(buffer, 8) == 0xf0);
    M.buffer_destroy(buffer);

    ring := M.buffer_create_with_capacity(2);

    assert(M.buffer_write(ring, u8.[1, 2]) == 2);
    assert(M.buffer_read(ring, 8) == 1);
    assert(M.buffer_write(ring, u8.[3, 4, 5]) == 3);
    assert(M.buffer_get_remaining(ring) == 4);
    assert(M.buffer_read(ring, 32) == 0x02030405);
    M.buffer_signal_end(ring);
    assert(!M.buffer_has(ring, 1) && ring.has_ended);
    M.buffer_destroy(ring);

    ring = M.buffer_create_with_capacity(4);
    assert(M.buffer_write(ring, u8.[1, 2, 3, 4]) == 4);
    M.buffer_signal_end(ring);
    assert(M.buffer_read(ring, 8) == 1);
    M.buffer_discard_read_bytes(ring);
    assert(M.buffer_read(ring, 24) == 0x020304);
    assert(!M.buffer_has(ring, 1) && ring.has_ended);
    M.buffer_destroy(ring);

    append := M.create_for_appending(1);

    assert(M.buffer_write(append, u8.[0, 0, 1, 0xe0, 0x33]) == 5);
    assert(M.buffer_find_start_code(append, .VIDEO_1));
    M.buffer_seek(append, 0);
    assert(M.buffer_tell(append) == 0 && M.buffer_get_size(append) == 5);
    M.buffer_destroy(append);

    data := u8.[0b01000000];
    vlc := M.Vlc.[.{ 2, 0 }, .{ 0, 8 }, .{ 0, -3 }, .{ 0, 7 }];

    buffer = M.buffer_create_with_memory(data, false);
    assert(M.buffer_read_vlc(buffer, vlc) == 7);
    assert(M.buffer_read_vlc(buffer, vlc) == -3);
    M.buffer_destroy(buffer);
    unsigned := M.Vlc_Uint.[.{ 0, 65535 }, .{ 0, 1 }];

    buffer = M.buffer_create_with_memory(data, false);
    assert(M.buffer_read_vlc_uint(buffer, unsigned) == 65535);
    M.buffer_destroy(buffer);
    state: Load_State;

    buffer = M.buffer_create_with_capacity(2);
    M.buffer_set_load_callback(buffer, load, *state);
    assert(M.buffer_read(buffer, 16) == 0x9876 && state.calls == 1);
    M.buffer_destroy(buffer);
    filename := sprint("%/../../../target/portable-tests/mpeg-buffer.bin", S.path_strip_filename(#file));

    defer free(filename);
    assert(F.write_entire_file(filename, "abcde"));
    buffer = M.buffer_create_with_filename(filename);
    assert(buffer != null && M.buffer_get_size(buffer) == 5);
    assert(M.buffer_read(buffer, 8) == 97 && M.buffer_tell(buffer) == 1);
    M.buffer_seek(buffer, 3);
    assert(M.buffer_read(buffer, 8) == 100 && M.buffer_tell(buffer) == 4);
    M.buffer_destroy(buffer);
    assert(F.file_delete(filename));
    print("ok\n");
}
