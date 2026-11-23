// Seeding from a u64, from a state or from a time value gives the documented starting state, and the context generator matches a seeded state.
#import "Basic";
R :: #import "Random";

Time :: struct {
    low: u64;
    high: s64;
}

main :: () {
    // A u64 seed gives the state {low = 1, high = seed}.
    a: R.Random_State;

    R.random_seed(*a, cast(u64) 5);
    assert(a.low == 1 && a.high == 5);

    b := R.Random_State.{ low = 1, high = 5 };
    for 0..99 assert(R.random_get(*a) == R.random_get(*b));

    // A state seed copies high and forces low odd: an even low becomes low | 1.
    c: R.Random_State;

    R.random_seed(*c, R.Random_State.{ low = 4, high = 9 });
    assert(c.low == 5 && c.high == 9);

    d: R.Random_State;

    R.random_seed(*d, R.Random_State.{ low = 5, high = 9 });
    assert(d.low == 5 && d.high == 9);
    for 0..99 assert(R.random_get(*c) == R.random_get(*d));

    // A time value is copied the same way.
    t: R.Random_State;

    R.random_seed(*t, Time.{ 4, 9 });
    assert(t.low == 5 && t.high == 9);

    // The context generator, seeded with a u64, produces the same first value as a state with that seed.
    R.random_seed(cast(u64) 42);
    first := R.random_get();

    R.random_seed(*a, cast(u64) 42);
    assert(R.random_get(*a) == first);

    // An all-zero state is a fixed point.
    z: R.Random_State;
    for 0..9 assert(R.random_get(*z) == 0);

    print("ok\n");
}
