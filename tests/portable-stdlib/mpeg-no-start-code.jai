// Reviewed availability/refill and rounded cursor contract, not a scanning API.
#import "Basic";
M :: #import "pl_mpeg";

load :: (buffer: *M.Buffer, data: *void) {
    calls := cast(*s64) data;
    calls.* += 1;
    written := M.buffer_write(buffer, u8.[0, 0, 1, 0xb3, 0x55]);
    if written != 5 exit(2);
}

main :: () {
    bytes := u8.[0xff, 0, 0, 1, 0xb3, 0x55];

    for offset: 0..7 {
        buffer := M.buffer_create_with_memory(bytes, false);
        if !buffer exit(3);

        M.buffer_skip(buffer, offset);
        absent := M.buffer_no_start_code(buffer);
        if absent != (offset == 0) || buffer.bit_index != offset exit(4);

        M.buffer_destroy(buffer);
    }

    short := u8.[8, 9, 10, 11];
    buffer := M.buffer_create_with_memory(short, false);

    if M.buffer_no_start_code(buffer) || buffer.bit_index != 0 || !buffer.has_ended exit(5);

    M.buffer_destroy(buffer);
    calls: s64;

    buffer = M.buffer_create_with_capacity(2);
    M.buffer_set_load_callback(buffer, load, *calls);
    if M.buffer_no_start_code(buffer) || calls != 1 || buffer.bit_index != 0 exit(6);

    M.buffer_destroy(buffer);
    write_string("ok\n");
}
