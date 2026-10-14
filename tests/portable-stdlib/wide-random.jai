#import "Basic";
R :: #import "Random";

main :: () {
    high, low := multiply_64bits_with_overflow(0xffffffffffffffff, 0xffffffffffffffff);

    assert(high == 0xfffffffffffffffe && low == 1);
    q, r, error := unsigned_divide_with_remainder(U128.{ 0, 1 }, U128.{ 3, 0 });

    assert(!error && q.high == 0 && q.low == 6148914691236517205 && r.low == 1 && r.high == 0);
    state: R.Random_State;

    R.random_seed(*state, cast(u64) 0);
    assert(R.random_get(*state) == 0x0fc94e3bf4e9ab32);
    assert(state.low == 0x866458cd56f5e605);
    R.random_seed(*state, R.Random_State.{ 2, 5 });
    assert(state.low == 3 && state.high == 5);

    for 0..99 {
        value := R.random_get_zero_to_one_open(*state);
        assert(value >= 0 && value < 1);
        d := R.random_get_zero_to_one_open_new(*state);
        assert(d >= 0 && d < 1);
    }

    print("ok\n");
}
