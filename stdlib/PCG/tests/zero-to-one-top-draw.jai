// The largest possible draw must stay below 1, and reversed range bounds must stay inside the range.
#import "Basic";
PCG :: #import "PCG";

main :: () {
    // The first draw from this state is 0xffffffff.
    state: u64 = 0x07fffe0000000000;
    copy := state;

    assert(PCG.random_get(*copy) == 0xffffffff);

    value := PCG.random_get_zero_to_one(*state);

    assert(value < 1 && value > 0.99999);

    PCG.random_seed(*state, 11);

    for 0..9999 {
        value = PCG.random_get_zero_to_one(*state);
        assert(value >= 0 && value < 1);
    }

    for 0..9999 {
        value = PCG.random_get_within_range(*state, 3, -3);
        assert(value >= -3 && value <= 3);
    }

    print("ok\n");
}
