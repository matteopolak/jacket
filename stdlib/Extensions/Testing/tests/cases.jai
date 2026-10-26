// Tests with `@cases(LIST)`: one subtest per entry.
#import "Basic";
#import "Extensions/Testing";

Color :: enum {
    RED;
    GREEN;
    BLUE;
}

Length :: struct {
    input: string;
    length: s64;
}

LENGTHS :: Length.[
    .{ "", 0 },
    .{ "a", 1 },
    .{ "a/b", 3 },
    .{ "a/b", 3 },
    .{ "line\nbreak", 10 },
];

single_values :: (n: s64) {
    expect(n % 2 == 1);
} @test @cases(s64.[1,3,5])

several_arguments :: (input: string, length: s64) {
    expect_equal(input.count, length);
} @test @cases(LENGTHS)

with_the_test :: (t: *Test, input: string, length: s64) {
    expect(t.name.count > "with_the_test/".count);
    expect_equal(input.count, length);
} @test @cases(LENGTHS)

a_struct_argument :: (entry: Length) {
    expect_equal(entry.input.count, entry.length);
} @test @cases(LENGTHS)

every_enum_member :: (color: Color) {
    expect(color == .RED || color == .GREEN || color == .BLUE);
} @test @cases(all_values(Color))

floats_from_integers :: (got: float64, want: float64) {
    expect_near(got, want, 0.001);
} @test @cases(FLOATS)

Floats :: struct {
    got: float64;
    want: float64;
}

FLOATS :: Floats.[.{ 1, 1 }, .{ 2.5, 2.5 }];

require_returns_from_the_case :: (n: s64) {
    require(n != 2);
    expect(n != 2);
} @test @cases(s64.[1,2,3]) @should_fail

failing_cases_fail_the_test :: (n: s64) {
    expect(n < 3, "three_is_too_big");
} @test @cases(s64.[1,2,3,4]) @should_fail(three_is_too_big)

known_to_fail :: (n: s64) {
    expect(n < 0, "known");
} @test @cases(s64.[1,2]) @xfail(each_case_fails)

skips_per_case :: (t: *Test, n: s64) {
    if n == 2 skip(t, "two");
    expect(n != 2);
} @test @cases(s64.[1,2,3])

all_values_lists_in_order :: (t: *Test) {
    colors := all_values(Color);
    expect_equal(colors, Color.[.RED, .GREEN, .BLUE]);
} @test

seen_by_hand: [..] s64;

run_cases_by_hand :: (t: *Test) {
    seen_by_hand.allocator = temp;
    array_reset_keeping_memory(*seen_by_hand);

    passed := run_cases(t, s64.[1, 2, 3], (n: s64) {
        array_add(*seen_by_hand, n);
    });

    expect(passed);
    expect_equal(seen_by_hand, s64.[1, 2, 3]);
} @test

names_are_filterable :: (t: *Test) {
    // The names a list gets: no `/`, empty strings quoted, repeats numbered.
    names := case_names(LENGTHS, several_arguments);

    expect_equal(names.count, 5);
    expect_equal(names[0], "\"\",0");
    expect_equal(names[1], "a,1");
    expect_equal(names[2], "a|b,3");
    expect_equal(names[3], "a|b,3 #3");
    expect_equal(names[4], "line break,10");
} @test
