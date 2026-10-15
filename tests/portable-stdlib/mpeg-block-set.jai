// Reviewed caller di/si mutation, row strides and expression truncation.
#import "Basic";
M :: #import "pl_mpeg";

cell :: (source: *s32, index: s64, calls: *s64, extra: s32) -> s32 {
    calls.* += 1;
    return source[index] + extra;
}

main :: () {
    source: [40] s32;
    for *value, index: source value.* = cast(s32) index * 257 - 800;

    storage: [40] u8;

    memset(storage.data, 0x5a, storage.count);
    dest := storage.data;
    di: s64 = 5;
    si: s64 = 2;
    calls: s64;
    row: s32 = 37;

    M.BLOCK_SET(dest, 7, 9, 3, #code cell(source.data, si, *calls, row));
    if calls != 9 || di != 26 || si != 29 || row != 37 exit(2);

    for index: 0..39 {
        expected: u8 = 0x5a;

        for y: 0..2 for x : 0..2 {
            if index == 5 + y * 7 + x expected = cast,truncate(u8) (source[2 + y * 9 + x] + 37);
        }

        if storage[index] != expected exit(3);
    }

    M.BLOCK_SET(dest, 7, 9, 0, #code cell(source.data, si, *calls, row));
    if calls != 9 || di != 26 || si != 29 exit(4);

    write_string("ok\n");
}
