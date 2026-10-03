callback: (value: s32) -> s32 = (value: s32) -> s32 {
    return value + 2;
};
main :: () -> int { return callback(40); }
