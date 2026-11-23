// copy writes exactly `count` bytes for every size from 0 to 300 and leaves the bytes around the range unchanged.
#import "Basic";
Memory :: #import "Memory";

SIZE :: 400;

main :: () {
    for count: 0..300 {
        source: [SIZE] u8;
        target: [SIZE] u8;

        for *b, i: source b.* = cast,trunc(u8) (i * 13 + 5);
        for *b, i: target b.* = 0xEE;

        Memory.copy(target.data + 50, source.data + 7, count);

        for i: 0..SIZE - 1 {
            expected: u8 = 0xEE;
            if i >= 50 && i < 50 + count expected = source[i - 50 + 7];
            assert(target[i] == expected);
        }
    }

    // A large copy between two distinct buffers.
    big_source: [4096] u8;
    big_target: [4096] u8;

    for *b, i: big_source b.* = cast,trunc(u8) (i * 7 + i / 256);

    Memory.copy(big_target.data, big_source.data, 4096);
    for i: 0..4095 assert(big_target[i] == big_source[i]);

    print("ok\n");
}
