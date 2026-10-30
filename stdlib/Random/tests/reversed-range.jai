// Range bounds can be given in either order and the result stays inside the range.
#import "Basic";
R :: #import "Random";

main :: () {
    state: R.Random_State;

    R.random_seed(*state, cast(u64) 5);

    varied := false;
    first := R.random_get_within_range(*state, 10, 2);

    for 0..9999 {
        value := R.random_get_within_range(*state, 10, 2);

        assert(value >= 2 && value <= 10);
        if value != first varied = true;

        value = R.random_get_within_range(*state, 2, 10);
        assert(value >= 2 && value <= 10);
    }

    assert(varied);
    assert(R.random_get_within_range(*state, 4, 4) == 4);
    print("ok\n");
}
