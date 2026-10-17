// Reviewed raw byte-domain luma and integer contributions; alpha is untouched.
#import "Basic";
M :: #import "pl_mpeg";

main :: () {
    samples := u8.[99, 16, 240, 0];
    frame: M.Frame;
    frame.y.data = samples.data;
    storage: [20] u8;

    memset(storage.data, 37, storage.count);
    dest := storage.data;
    y_index: s32 = 1;
    d_index: s32 = 2;
    r, g, b: s64;

    M.PUT_PIXEL(0, 1, 2, 0, 1);
    if storage[3] != 16 || storage[4] != 16 || storage[5] != 16 || storage[6] != 37 exit(2);
    if y_index != 1 || d_index != 2 exit(3);

    r = 1000;
    g = 1000;
    b = -1000;
    M.PUT_PIXEL(2, 1, 0, 1, 5);
    if storage[7] != 0 || storage[8] != 0 || storage[9] != 255 || storage[10] != 37 exit(4);

    r = 9223372036854775807;
    g = -9223372036854775807 - 1;
    b = -9223372036854775807 - 1;
    M.PUT_PIXEL(0, 1, 2, 0, 9);
    if storage[11] != 255 || storage[12] != 255 || storage[13] != 0 || storage[14] != 37 exit(5);

    write_string("ok\n");
}
