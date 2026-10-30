// A bare `xx` argument picks the overload that takes one item over the one that takes a slice.
// rules: cast.37
#import "Basic";

Append :: inline (buf: *[] $T, item: T) {
    buf.count += 1;
    buf.*[buf.count - 1] = item;
}

Append :: inline (buf: *[] $T, items: [] T) {
    for items {
        Append(buf, it);
    }
}

main :: () {
    storage: [8] u32;
    view: [] u32 = storage;
    view.count = 0;
    small: u8 = 65;

    Append(*view, xx small);
    data: [] u32 = u32.[7, 8];

    Append(*view, data);
    assert(view.count == 3);
    assert(view[0] == 65);
    assert(view[2] == 8);
    print("ok\n");
}
