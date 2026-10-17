main :: () {
    write_string("runtime ");
    write_number(-9223372036854775808);
    write_string(" ");
    write_nonnegative_number(255, 16);
    write_string("\n");
    bytes: [6] u8 = .[7, 9, 0, 0, 0, 0];

    __element_duplicate(bytes.data, 3, 2);
    if bytes[4] != 7 || bytes[5] != 9 debug_break();

    memory := context.allocator.proc(.ALLOCATE, 32, 0, null, context.allocator.data);
    if !memory debug_break();

    memset(memory, 42, 32);
    memory = context.allocator.proc(.RESIZE, 64, 32, memory, context.allocator.data);
    if !memory || (cast(*u8) memory)[31] != 42 debug_break();

    context.allocator.proc(.FREE, 0, 64, memory, context.allocator.data);
}
