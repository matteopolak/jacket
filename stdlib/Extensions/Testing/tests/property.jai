// Property tests: what holds on generated inputs, and the counterexamples of what does not.
#import "Basic";
#import "Extensions/Testing";

Prop_Point :: struct {
    x, y: s32;
}

Prop_Color :: enum {
    RED;
    GREEN;
    BLUE;
}

Prop_Shape :: struct {
    name: string;
    corners: [] Prop_Point;
    color: Prop_Color;
    visible: bool;
}

reversed :: (items: [] s64) -> [] s64 {
    result := NewArray(items.count, s64,, temp);
    for items result[items.count - 1 - it_index] = it;

    return result;
}

reversing_twice_is_the_identity :: (items: [] s64) {
    expect_equal(reversed(reversed(items)), items);
} @test @property

addition_commutes :: (a: s32, b: s32) -> bool {
    return cast(s64) a + b == cast(s64) b + a;
} @test @property

tests_may_take_the_test :: (t: *Test, text: string, flag: bool) {
    expect(text.count >= 0);
    expect(flag || !flag);
} @test @property(20)

structs_enums_and_floats_are_generated :: (shape: Prop_Shape, ratio: float64, small: u8, wide: s64) {
    expect(shape.corners.count >= 0);
    expect(cast(s64) shape.color < 3);
    expect(ratio == ratio);
    expect(small <= 255);
    expect(wide >= -9223372036854775807 - 1);
} @test @property

fixed_arrays_and_nested_lists :: (cells: [4] s16, rows: [][] s8) {
    expect_equal(cells.count, 4);
    for rows expect(it.count >= 0);
} @test @property

assume_skips_inputs :: (n: s64) {
    assume(n > 0);
    expect(n > 0);
} @test @property

even_numbers :: (rng: *Rng) -> s64 {
    return rng_range(rng, -50, 50) * 2;
}

custom_generators_make_the_inputs :: (n: s64) {
    expect_equal(n % 2, 0);
} @test @property @generate(even_numbers)

called_by_hand :: (t: *Test) {
    check :: (n: s64) -> bool {
        return n * 2 == n + n;
    }

    run_property(t, check, 10);
} @test

// Failures: the counterexample is as small as the property allows.

integers_shrink_to_the_boundary :: (n: s64) {
    expect(n < 1000, "n is too big");
} @test @property @should_fail(integers_shrink_to_the_boundary(1000))

negative_integers_shrink_too :: (n: s32) {
    expect(n > -37);
} @test @property @should_fail(negative_integers_shrink_too(-37))

lists_shrink_to_one_element :: (items: [] s64) {
    for items expect(it < 100);
} @test @property @should_fail(lists_shrink_to_one_element([100]))

strings_shrink_to_the_character :: (text: string) -> bool {
    for text if it == #char "Q" return false;
    return true;
} @test @property(400) @should_fail(strings_shrink_to_the_character("Q"))

structs_shrink_member_by_member :: (point: Prop_Point, other: Prop_Point) {
    expect(point.x < 20 || other.y < 20);
} @test @property(400) @should_fail(.{x=20,)

failures_name_the_seed :: (n: bool) {
    expect(false, "always");
} @test @property @should_fail(seed)

failures_say_how_to_rerun :: (n: bool) {
    expect(false, "always");
} @test @property @should_fail(rerun:)

returning_false_fails :: (n: s64) -> bool {
    return n < 5;
} @test @property @should_fail(returned)
