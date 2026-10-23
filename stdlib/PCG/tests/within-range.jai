// An ordered range never yields a value outside it; reversed bounds are not clamped.
#import "Basic";
PCG :: #import "PCG";

main :: () {
    state: u64;

    PCG.random_seed(*state, 7);

    for 0..9999 {
        value := PCG.random_get_within_range(*state, 0.1, 0.3);
        assert(value >= 0.1 && value <= 0.3);
        value = PCG.random_get_within_range(*state, -3, 3);
        assert(value >= -3 && value <= 3);
    }

    varied := false;

    for 0..99 {
        value := PCG.random_get_within_range(*state, 3, -3);
        assert(value >= -3 && value <= 3);
        if value != 3 varied = true;
    }

    assert(varied);
    assert(PCG.random_get_within_range(*state, 2, 2) == 2);
    print("ok\n");
}
