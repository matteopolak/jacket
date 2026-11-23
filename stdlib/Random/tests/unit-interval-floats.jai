// The float helpers stay in their documented intervals, the open forms never return 1, and the 24-bit open form lands on multiples of 2^-24.
#import "Basic";
R :: #import "Random";

main :: () {
    state: R.Random_State;

    R.random_seed(*state, cast(u64) 11);

    open_new_sum: float64 = 0;
    count := 20000;

    for 1..count {
        r := R.random_get_zero_to_one(*state);

        assert(r >= 0 && r <= 1);

        o := R.random_get_zero_to_one_open(*state);

        assert(o >= 0 && o < 1);
        // o * 2^24 is an integer, because the draw is k / 2^24 for an integer k below 2^24.
        scaled := o * 16777216;

        assert(scaled == cast(float) cast(s64) scaled);

        n := R.random_get_zero_to_one_open_new(*state);

        assert(n >= 0 && n < 1);
        open_new_sum += n;

        z := R.random_get_zero_to_one_new(*state);

        assert(z >= 0 && z < 1);

        w := R.random_get_within_range(*state, -2.0, 3.0);

        assert(w >= -2 && w <= 3);
    }

    // The mean of uniform values in [0, 1) is 0.5; with this seed the sample mean is well within 0.01 of it.
    mean := open_new_sum / cast(float64) count;

    assert(mean > 0.49 && mean < 0.51);

    print("ok\n");
}
