// The checks: what passes, and the message of what does not.
#import "Basic";
#import "Extensions/Testing";

Point :: struct {
    x, y: s32;
}

true_passes :: () {
    expect(true);
    expect(expect(1 + 1 == 2), "expect returns whether it held");
} @test

false_fails_with_its_message :: () {
    expect(false, "custom_message");
} @test @should_fail(custom_message)

numbers_of_any_width_compare_by_value :: () {
    expect_equal(cast(u8) 7, 7);
    expect_equal(cast(s64) -1, cast(s8) -1);
    expect_equal(2.5, 2.5);
} @test

strings_arrays_and_structs_compare_by_value :: () {
    expect_equal(tprint("a%", "b"), "ab");
    expect_equal(s32.[1, 2, 3], s32.[1, 2, 3]);
    dynamic: [..] s32;
    dynamic.allocator = temp;
    array_add(*dynamic, 1, 2, 3);
    expect_equal(dynamic, s32.[1, 2, 3]);
    expect_equal(Point.{ 1, 2 }, Point.{ 1, 2 });
} @test

different_numbers_fail :: () {
    expect_equal(3, 4);
} @test @should_fail(want:)

different_strings_point_at_the_first_difference :: () {
    expect_equal("abcd", "abXd");
} @test @should_fail(byte)

different_lines_are_diffed :: () {
    expect_equal("one\ntwo\nthree\n", "one\n2\nthree\n");
} @test @should_fail(+++)

different_arrays_name_the_element :: () {
    expect_equal(s32.[1, 2, 3], s32.[1, 5, 3]);
} @test @should_fail([1])

different_lengths_say_so :: () {
    expect_equal(s32.[1, 2], s32.[1, 2, 3]);
} @test @should_fail(missing)

different_structs_name_the_member :: () {
    expect_equal(Point.{ 1, 2 }, Point.{ 1, 3 });
} @test @should_fail(.y)

different_types_say_so :: () {
    expect_equal("1", 1);
} @test @should_fail(string)

near_values_pass :: () {
    expect_near(0.1 + 0.2, 0.3, 1e-9);
} @test

far_values_fail :: () {
    expect_near(1.0, 2.0, 0.5);
} @test @should_fail(within)

pointers :: () {
    value := 1;
    expect_not_null(*value);
    nothing: *int;
    expect_null(nothing);
} @test

require_returns :: () {
    reached := false;
    inner :: (reached: *bool) {
        require(false, "stopped_here");
        reached.* = true;
    }

    inner(*reached);
    expect(!reached, "require went on");
} @test @should_fail(stopped_here)

an_assert_fails_the_test :: () {
    assert(false, "asserted");
} @test @should_fail(asserted)

a_known_failure :: () {
    expect(false);
} @test @xfail

narrow_negatives_widen_by_value :: () {
    expect_equal(cast(s8) -2, -2);
    expect_equal(cast(s16) -300, cast(s64) -300);
    expect_equal(cast(s32) 5, cast(u8) 5);
} @test

signed_and_unsigned_differ_when_negative :: () {
    expect_equal(cast(s8) -1, cast(u8) 255);
} @test @should_fail(want:)
