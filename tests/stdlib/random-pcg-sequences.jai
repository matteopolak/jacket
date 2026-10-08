// Random (128-bit multiplicative generator) and PCG (XSH-RR 64/32) at run time: exact
// sequences from an independent model of each generator, seeding, the context and global
// forms, ranges over many draws and a coarse uniformity check.
#import "Basic";
R :: #import "Random";
P :: #import "PCG";

main :: () {
    random_module();
    pcg_module();
    print("ok\n");
}

random_module :: () {
    // The context generator starts from the state {1, 0}.
    first_default := u64.[0x0fc94e3bf4e9ab32, 0x9f4c53132cb5b55a, 0x04f16bbaa6c209fe, 0x9c0827f89f0f242f, 0x5b5349ddf2ca0286];
    state := R.Random_State.{ 1, 0 };
    for first_default {
        got := R.random_get(*state);
        assert(got == it, "explicit state draw %: % vs %", it_index, got, it);
    }

    // Seeding is reproducible, through an explicit state or the context.
    seeded := u64.[0xccc794426ccad24f, 0xcdb628befa0086eb, 0x16f536a5274567d3, 0x8eaa48d282c65758, 0x2a563a0eb991d853];
    R.random_seed(*state, 12345);
    for seeded assert(R.random_get(*state) == it, "seed 12345 draw %", it_index);
    R.random_seed(12345);
    for seeded assert(R.random_get() == it, "context seed 12345 draw %", it_index);
    R.random_seed(12345);
    floats := u32.[0x3f4ad250, 0x3b06eb01, 0x3e8acfa7];
    for floats {
        value := R.random_get_zero_to_one();
        assert((cast(*u32) *value).* == it, "zero_to_one draw %: %", it_index, value);
    }

    // Seeding from a whole state forces the low word odd (an even state would decay to 0).
    R.random_seed(*state, R.Random_State.{ 4, 7 });
    assert(state.low == 5 && state.high == 7);
    R.random_seed(R.Random_State.{ 0, 99 });
    assert(context.random_state.low == 1 && context.random_state.high == 99);
    copy := context.random_state;
    a := R.random_get();
    assert(R.random_get(*copy) == a, "a copied state continues the same sequence");
    R.random_seed(1);
    one := R.random_get();
    R.random_seed(2);
    assert(R.random_get() != one, "different seeds differ");

    // Ranges over many draws.
    R.random_seed(777);
    low_closed, high_closed: float = 1, 0;
    sum: float64 = 0;
    buckets: [10] int;
    DRAWS :: 20000;
    for 1..DRAWS {
        closed := R.random_get_zero_to_one();
        open := R.random_get_zero_to_one_open();
        wide := R.random_get_zero_to_one_new();
        wide_open := R.random_get_zero_to_one_open_new();
        ranged := R.random_get_within_range(-3, 5);
        assert(closed >= 0 && closed <= 1 && open >= 0 && open < 1, "float draws in range: % %", closed, open);
        assert(wide >= 0 && wide <= 1 && wide_open >= 0 && wide_open < 1, "float64 draws in range: % %", wide, wide_open);
        assert(ranged >= -3 && ranged <= 5, "within_range: %", ranged);
        low_closed = min(low_closed, closed);
        high_closed = max(high_closed, closed);
        sum += wide;
        buckets[cast(int) (wide_open * 10)] += 1;
    }
    assert(low_closed < 0.001 && high_closed > 0.999, "draws cover [0, 1]: % %", low_closed, high_closed);
    mean := sum / DRAWS;
    assert(mean > 0.48 && mean < 0.52, "mean of uniform draws: %", mean);
    for buckets assert(it > DRAWS / 10 * 9 / 10 && it < DRAWS / 10 * 11 / 10, "bucket % has % draws", it_index, it);

    // The explicit-state forms match the context forms.
    s1: R.Random_State;
    R.random_seed(*s1, 5);
    R.random_seed(5);
    assert(R.random_get_zero_to_one(*s1) == R.random_get_zero_to_one());
    assert(R.random_get_zero_to_one_open(*s1) == R.random_get_zero_to_one_open());
    assert(R.random_get_zero_to_one_new(*s1) == R.random_get_zero_to_one_new());
    assert(R.random_get_zero_to_one_open_new(*s1) == R.random_get_zero_to_one_open_new());
    assert(R.random_get_within_range(*s1, 2, 4) == R.random_get_within_range(2, 4));
    fixed: R.Random_State;
    R.random_seed(*fixed, 9);
    assert(R.random_get_within_range(*fixed, 6, 6) == 6, "a one-point range");
}

pcg_module :: () {
    // The global state starts at the default increment.
    expected_default := u32.[0x602bf3fd, 0xe823a24e, 0x7a7ecbd9, 0x89fd6c06, 0xae646aa8];
    for expected_default {
        got := P.random_get();
        assert(got == it, "pcg default draw %: % vs %", it_index, got, it);
    }
    expected_42 := u32.[0xc2f57bd6, 0x6b07c4a9, 0x72b7b29b, 0x44215383, 0xf5af5ead, 0x68beb632];
    state: u64;
    P.random_seed(*state, 42);
    for expected_42 assert(P.random_get(*state) == it, "pcg seed 42 draw %", it_index);
    P.random_seed(42);
    for expected_42 assert(P.random_get() == it, "pcg global seed 42 draw %", it_index);
    assert(P.pcg32_state == state, "the global and explicit states agree");

    // Bounded draws: every value in range, all of them hit, roughly uniformly.
    P.random_seed(*state, 1234);
    counts: [7] int;
    DRAWS :: 14000;
    for 1..DRAWS {
        value := P.random_get_within_bound(*state, 7);
        assert(value < 7, "bound 7 gave %", value);
        counts[value] += 1;
        assert(P.random_get_within_bound(*state, 1) == 0);
        assert(P.random_get_within_bound2(*state, 3) < 3);
        big := P.random_get_within_bound(*state, 0x8000_0001);
        assert(big <= 0x8000_0000, "a bound just above 2^31 (most rejection): %", big);
    }
    for counts assert(it > DRAWS / 7 * 9 / 10 && it < DRAWS / 7 * 11 / 10, "value % drawn % times", it_index, it);
    P.random_seed(3);
    for 1..1000 assert(P.random_get_within_bound(10) < 10);

    // Floats.
    P.random_seed(*state, 99);
    lowest, highest: float = 1, 0;
    for 1..20000 {
        f := P.random_get_zero_to_one(*state);
        assert(f >= 0 && f < 1, "zero_to_one %", f);
        lowest = min(lowest, f);
        highest = max(highest, f);
        g := P.random_get_within_range(*state, -1, 1);
        assert(g >= -1 && g <= 1);
    }
    assert(lowest < 0.001 && highest > 0.999, "pcg floats cover [0, 1): % %", lowest, highest);
    P.random_seed(8);
    f1 := P.random_get_zero_to_one();
    g1 := P.random_get_within_range(10, 20);
    P.random_seed(*state, 8);
    assert(P.random_get_zero_to_one(*state) == f1 && P.random_get_within_range(*state, 10, 20) == g1, "global and explicit float forms agree");
    assert(g1 >= 10 && g1 <= 20);
}
