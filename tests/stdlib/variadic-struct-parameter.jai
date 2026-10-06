// A struct's variadic parameter (`struct(types: ..Type)`) binds its arguments as one constant
// `[] T`, and the instance is keyed by the whole list.
#import "Basic";

Holder :: struct(types: ..Type) {
    COUNT :: types.count;
    bytes: [#run biggest(types)] u8;
}

Numbers :: struct(first: s64, rest: ..s64) {
    TOTAL :: #run sum(first, rest);
}

biggest :: (types: [] Type) -> s64 {
    result := 0;
    for types {
        size := (cast(*Type_Info) it).runtime_size;
        if size > result result = size;
    }
    return result;
}

sum :: (first: s64, rest: [] s64) -> s64 {
    total := first;
    for rest total += it;
    return total;
}

main :: () {
    H :: Holder(s64, string);
    assert(H.COUNT == 2);
    assert(size_of(H) == 16);
    assert(Holder(u8).COUNT == 1);
    assert(size_of(Holder(u8)) == 1);
    assert(Holder().COUNT == 0);
    assert(Holder(s64, string) == H);
    assert(Holder(s64) != Holder(string));
    assert(Holder(s64, string) != Holder(string, s64));

    x: Holder(s64, string);
    y: H = x;
    assert(y.bytes.count == 16);

    assert(Numbers(1).TOTAL == 1);
    assert(Numbers(1, 2, 3).TOTAL == 6);
    assert(Numbers(first = 4).TOTAL == 4);
    assert(Numbers(1, 2) != Numbers(1, 3));
    print("ok\n");
}
