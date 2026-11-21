Block :: struct {
    a: u64;
    b: u64;
    c: [5] u32;
}

fill :: (bytes: *u8, count: s64) {
    for index: 0..count - 1 bytes[index] = cast,no_check(u8) (index * 7 + 1);
}

main :: () {
    source: [64] u8;
    target: [64] u8;

    fill(source.data, 64);

    memcpy(target.data, source.data, 64);
    print("copy %\n", memcmp(target.data, source.data, 64) == 0);

    // Overlapping moves, in both directions.
    memcpy(target.data, source.data, 64);
    memcpy(target.data + 3, target.data, 40);
    forward := true;
    for index: 0..39 if target[index + 3] != source[index] forward = false;

    print("move up %\n", forward);

    memcpy(target.data, source.data, 64);
    memcpy(target.data, target.data + 5, 40);
    backward := true;
    for index: 0..39 if target[index] != source[index + 5] backward = false;

    print("move down %\n", backward);

    memcpy(target.data, source.data, 64);
    memset(target.data, 0xab, 33);
    filled := true;
    for index: 0..32 if target[index] != 0xab filled = false;

    print("set % %\n", filled, target[33] == source[33]);

    one: Block;
    one.a = 1;
    one.b = 2;
    for index: 0..4 one.c[index] = cast(u32) index;

    two := one;

    print("struct % %\n", two.b, two.c[4]);

    a: [4] u8 = .[1, 2, 3, 4];
    b: [4] u8 = .[1, 2, 9, 4];

    print("cmp % %\n", memcmp(a.data, b.data, 4) < 0, memcmp(b.data, a.data, 4) > 0);
}
