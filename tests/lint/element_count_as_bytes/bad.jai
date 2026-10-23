#import "Basic";

Pair :: struct {
    a, b: int;
}

copy_values :: (dest: [] s32, values: [] s32) {
    memcpy(dest.data, values.data, values.count);
}

clear :: (weights: [] float) {
    memset(weights.data, 0, weights.count);
}

same :: (a: [4] Pair, b: [4] Pair) -> bool {
    return memcmp(a.data, b.data, a.count) == 0;
}

main :: () {
    a: [4] s32;
    b: [4] s32;

    copy_values(a, b);
    weights: [3] float;

    clear(weights);
    p: [4] Pair;

    print("%\n", same(p, p));
}
