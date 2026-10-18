// Fuzz generators are deterministic per seed, honour the length bound, and reach edge values.
#import "Basic";
#import "Fuzz";

Color :: enum u8 { RED; GREEN; BLUE; }
Pair :: struct { id: s32; name: string; color: Color; weights: [3] float; }

main :: () {
    r: Rng;
    rng_seed(*r, 7);
    first := gen(*r, s64);
    case_end(*r);
    rng_seed(*r, 7);
    assert(gen(*r, s64) == first);

    r.max_len = 8;
    saw_empty := false;
    saw_invalid_utf8 := false;
    saw_minimum := false;
    saw_maximum := false;
    saw_invalid_enum := false;
    saw_nan := false;
    for seed: 1..4000 {
        rng_seed(*r, cast(u64) seed);
        text := gen(*r, string);
        assert(text.count <= 8 * 4);
        if text.count == 0 saw_empty = true;
        if text.count > 0 && text[0] >= 0xF8 saw_invalid_utf8 = true;

        number := gen(*r, s8);
        if number == -128 saw_minimum = true;
        if number == 127 saw_maximum = true;

        pair := gen(*r, Pair);
        if cast(s64) pair.color > 2 saw_invalid_enum = true;
        for pair.weights if it != it saw_nan = true;

        view := gen(*r, [] u16);
        assert(view.count <= 8);
        case_end(*r);
    }
    assert(saw_empty && saw_invalid_utf8 && saw_minimum && saw_maximum && saw_invalid_enum && saw_nan);

    // Pointers are filled only when the target allows them, and then are never null by default.
    rng_seed(*r, 3);
    assert(gen(*r, *s64) == null);
    r.pointers = true;
    pointer := gen(*r, *s64);
    assert(pointer != null);
    case_end(*r);

    assert(can_fill(type_info(Pair)) && !can_fill(type_info(Any)));
    assert(has_pointer(type_info(*s64)) && !has_pointer(type_info(Pair)));
    print("ok\n");
}
