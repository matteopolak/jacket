#import "Basic";
using R :: #import "RadixSort";
P :: #import "Relative_Pointers";

main :: () {
    sorter: R.RadixSort;
    values: [5] u64 = .[9, 0, 0xffffffffffffffff, 4, 4];

    R.sort(*sorter, values);
    assert(sorter.valid_ranks && sorter.ranks.count == 5);
    for i: 1..4 assert(values[R.rank(*sorter, i - 1)] <= values[R.rank(*sorter, i)]);

    R.sort(*sorter, cast(*u64) null, 0);
    assert(sorter.valid_ranks && sorter.ranks.count == 5);
    floats: [4] float = .[2, -3, 0, 1];

    R.sort(*sorter, floats);
    assert(sorter[0] == 1 && sorter[3] == 0);
    R.free_resources(*sorter);

    Pair :: struct {
        p: P.Relative_Pointer(s32, s64);
        target: s64 = 91;
    }

    pair: Pair;

    P.encode(*pair.p, *pair.target);
    assert(P.decode(*pair.p) == *pair.target);
    copy := pair;

    assert(P.decode(*copy.p) == *copy.target && P.decode(*copy.p).* == 91);
    P.encode(*pair.p, null);
    assert(P.decode(*pair.p) == null);
    self: P.Relative_Pointer(s64, u8);

    P.encode(*self, cast(*u8) *self);
    assert(P.decode(*self) == cast(*u8) *self);
    print("ok\n");
}
