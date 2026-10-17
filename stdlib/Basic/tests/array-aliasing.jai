#import "Basic";

Moving_Storage :: struct {
    bytes: [4096] u8;
    used: s64;
    moves: s64;
}

moving_allocator :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    storage := cast(*Moving_Storage) allocator_data;

    if mode == .FREE return null;

    assert(mode == .ALLOCATE || mode == .RESIZE);
    offset := align_forward(storage.used, 16);

    assert(size >= 0 && offset + size <= storage.bytes.count);
    result := storage.bytes.data + offset;

    storage.used = offset + size;

    if old_memory {
        memcpy(result, old_memory, min(size, old_size));
        memset(old_memory, 0xdd, old_size);
        storage.moves += 1;
    }

    return result;
}

main :: () {
    storage: Moving_Storage;
    values: [..] s64;
    values.allocator = .{ moving_allocator, *storage };
    defer array_free(values);
    for index: 0..7 array_add(*values, index);

    before := values.data;

    array_add(*values, ..values);
    assert(values.data != before && storage.moves == 1);
    assert(values.count == 16);
    for index: 0..15 assert(values[index] == index % 8);

    suffix := array_view(values, 3, 6);

    array_add(*values, ..suffix);
    assert(values.count == 22 && storage.moves == 2);
    for index: 0..5 assert(values[16 + index] == (index + 3) % 8);

    array_add(*values, ..([] s64).{});
    assert(values.count == 22 && storage.moves == 2);
    array_copy(*values, array_view(values, 3, 6));
    assert(values.count == 6 && storage.moves == 2);
    for index: 0..5 assert(values[index] == (index + 3) % 8);

    array_copy(*values, values);
    assert(values.count == 6 && storage.moves == 2);
    print("ok\n");
}
