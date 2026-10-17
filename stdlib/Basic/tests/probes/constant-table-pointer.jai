#import "Basic";

Entry :: struct {
    levels: u16;
    group: u8;
    bits: u8;
}

TABLE :: Entry.[.{ 3, 1, 5 }, .{ 5, 1, 7 }];

entry :: (index: s64) -> *Entry {
    return *TABLE[index];
}

clobber :: () -> s64 {
    bytes: [1024] u8;

    memset(bytes.data, 122, bytes.count);
    total: s64;
    for bytes total += it;

    return total;
}

main :: () {
    first := entry(0);
    second := entry(1);

    assert(clobber() == 1024 * 122);
    assert(first.levels == 3 && first.group == 1 && first.bits == 5);
    assert(second.levels == 5 && second.group == 1 && second.bits == 7);
    assert(first == entry(0) && second == entry(1));
    print("ok\n");
}
