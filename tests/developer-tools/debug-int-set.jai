#import "Basic";
D :: #import "Debug_Info";

#assert D.Int_Set.BPW == 32;
#assert size_of(D.Int_Set) == size_of([] u32);

main :: () {
    words := u32.[0x80000001, 0, 0xffffffff, 0x10204080];
    set: D.Int_Set = .{ words = words };

    assert(D.cardinality(set) == 38);
    count: u64;

    for i: 0..words.count * 32 - 1 {
        expected := (words[i / 32] & ((cast(u32) 1) << (i % 32))) != 0;
        assert(D.is_set(set, i) == expected);
        if expected count += 1;
    }

    assert(count == D.cardinality(set));
    assert(!D.is_set(set, -1) && !D.is_set(set, 128));
    assert(!D.is_set(set, 0x7fffffffffffffff));
    words[1] = 0xffffffff;
    assert(D.cardinality(set) == 70 && D.is_set(set, 32) && D.is_set(set, 63));
    empty: D.Int_Set;

    assert(D.cardinality(empty) == 0 && !D.is_set(empty, 0));
    print("ok\n");
}
