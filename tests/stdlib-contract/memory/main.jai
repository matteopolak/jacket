// Generic copy has memmove-style semantics for overlapping byte ranges.
#import "Basic";
Memory :: #import "Memory";

main :: () {
    bytes: [8] u8 = .[0, 1, 2, 3, 4, 5, 6, 7];

    Memory.copy(*bytes[2], *bytes[0], 6);
    assert(bytes[0] == 0 && bytes[1] == 1);
    assert(bytes[2] == 0 && bytes[3] == 1 && bytes[4] == 2);
    assert(bytes[7] == 5);

    Memory.copy(*bytes[0], *bytes[2], 6);
    assert(bytes[0] == 0 && bytes[1] == 1 && bytes[5] == 5 && bytes[7] == 5);

    before := bytes;

    Memory.copy(*bytes[1], *bytes[6], 0);
    for i: 0..bytes.count - 1 assert(bytes[i] == before[i]);

    print("ok\n");
}
