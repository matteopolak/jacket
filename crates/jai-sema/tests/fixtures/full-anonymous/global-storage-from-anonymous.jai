offset :: 2;
total: s32 = 40;
main :: () -> int {
    callback := () -> s32 {
        total += offset;
        return total;
    };
    return callback();
}
