// The matchers: what passes, and the message of what does not.
#import "Basic";
#import "Extensions/Testing";

Spot :: struct {
    x, y: s32;
}

Shape :: enum {
    SQUARE;
    CIRCLE;
}

passing :: (value: $T, matcher: Matcher, loc := #caller_location) {
    message := explain(value, matcher);
    if message expect(false, tprint("should have matched: %", message), loc);
}

failing :: (value: $T, matcher: Matcher, message: string, loc := #caller_location) {
    expect_equal(explain(value, matcher), message, loc);
}

assert_that_returns_whether_it_held :: () {
    expect(assert_that(1, is(1)));
} @test

failed_matcher_fails_the_test :: () {
    assert_that(4, greater_than(5));
} @test @should_fail(greater)

require_that_returns_when_it_fails :: () {
    require_that("abc", contains("x"));
    expect(false, "unreachable");
} @test @should_fail(contain)

require_that_goes_on_when_it_holds :: () {
    require_that("abc", contains("b"));
} @test

equality :: () {
    passing(5, is(5));
    passing(cast(u8) 7, is(7));
    passing("ab", is("ab"));
    passing(s32.[1, 2], is(s32.[1, 2]));
    passing(Spot.{ 1, 2 }, is(Spot.{ 1, 2 }));
    passing(Shape.CIRCLE, is(Shape.CIRCLE));
    failing(4, is(5), "expected 4 to equal 5");
    failing(5, not(is(5)), "expected 5 not to equal 5");
    failing("1", is(1), "expected \"1\" to equal 1");
} @test

equality_of_composites_shows_the_diff :: () {
    failing("abcd", is("abXd"), "got:  \"abcd\"\nwant: \"abXd\"\n         ^ first difference at byte 2");
    failing(Spot.{ 1, 2 }, is(Spot.{ 1, 3 }), "got:  {1, 2}\nwant: {1, 3}\nfirst difference at .y");
    assert_that(explain(s32.[1, 2, 3], is(s32.[1, 5, 3])), contains("[1]: got 2, want 5"));
    assert_that(explain("one\ntwo\n", is("one\n2\n")), contains("+++"));
    failing("abc", not(is("abc")), "expected \"abc\" not to equal \"abc\"");
} @test

comparisons :: () {
    passing(6, greater_than(5));
    passing(-1, greater_than(cast(s8) -2));
    passing(cast(u64) 5, greater_than(-1));
    passing(2.5, greater_than(2));
    passing(5, at_least(5));
    passing(4, less_than(5));
    passing(-3, less_than(-2));
    passing(5, at_most(5));
    passing(5, between(5, 6));
    passing(6, between(5, 6));
    passing("b", greater_than("a"));
    passing("ab", less_than("abc"));
    passing(0.5, less_than(1));

    failing(4, greater_than(5), "expected 4 to be greater than 5");
    failing(5, greater_than(5), "expected 5 to be greater than 5");
    failing(6, at_most(5), "expected 6 to be at most 5");
    failing(4, at_least(5), "expected 4 to be at least 5");
    failing(5, less_than(5), "expected 5 to be less than 5");
    failing(7, between(5, 6), "expected 7 to be between 5 and 6");
    failing(4, between(5, 6), "expected 4 to be between 5 and 6");
    failing("x", greater_than(1), "expected \"x\" to be greater than 1");
    failing(4, not(less_than(5)), "expected 4 not to be less than 5");
} @test

near_allows_a_tolerance :: () {
    passing(0.5, near(0.45, 0.1));
    passing(3, near(2.5, 0.5));
    failing(0.5, near(0.25, 0.1), "expected 0.5 to be within 0.1 of 0.25");
    // 0/0 is a negative NaN on x86-64; build a positive one so the message reads "NaN".
    bits: u64 = 0x7ff8_0000_0000_0000;
    nan := <<cast(*float64) *bits;

    failing(nan, near(1, 1000), "expected NaN to be within 1000 of 1");
} @test

string_matchers :: () {
    passing("hello world", contains("lo w"));
    passing("hello", contains(""));
    passing("hello", starts_with("he"));
    passing("hello", ends_with("llo"));
    passing("hello", starts_with("hello"));
    passing("", is_empty());
    passing("four", has_length(4));

    failing("hello", contains("hi"), "expected \"hello\" to contain \"hi\"");
    failing("hello", starts_with("lo"), "expected \"hello\" to start with \"lo\"");
    failing("hello", ends_with("he"), "expected \"hello\" to end with \"he\"");
    failing("hi", starts_with("hip"), "expected \"hi\" to start with \"hip\"");
    failing("hi", ends_with("ohi"), "expected \"hi\" to end with \"ohi\"");
    failing("hello", is_empty(), "expected \"hello\" to be empty");
    failing("hello", has_length(4), "expected \"hello\" to have length 4");
    failing("hello", not(contains("ell")), "expected \"hello\" not to contain \"ell\"");
} @test

array_matchers :: () {
    numbers := s32.[3, 1, 2];

    passing(numbers, contains(2));
    passing(numbers, contains(cast(u8) 3));
    passing(numbers, has_length(3));
    passing(numbers, starts_with(s32.[3, 1]));
    passing(numbers, ends_with(s32.[1, 2]));
    passing(numbers, not(is_empty()));
    passing(s32.[], is_empty());
    passing(string.["a", "b"], contains("b"));

    failing(numbers, contains(5), "expected [3, 1, 2] to contain 5");
    failing(numbers, has_length(2), "expected [3, 1, 2] to have length 2");
    failing(numbers, starts_with(s32.[1]), "expected [3, 1, 2] to start with [1]");
    failing(numbers, ends_with(s32.[1]), "expected [3, 1, 2] to end with [1]");
    failing(numbers, is_empty(), "expected [3, 1, 2] to be empty");
    failing(s32.[], not(is_empty()), "expected [] not to be empty");

    dynamic: [..] s32;
    dynamic.allocator = temp;
    array_add(*dynamic, 4, 5);
    passing(dynamic, contains(5));
    passing(dynamic, has_length(2));
} @test

item_matchers :: () {
    numbers := s32.[3, 1, 2];

    passing(numbers, has_item(greater_than(2)));
    passing(numbers, each(greater_than(0)));
    passing(s32.[], each(greater_than(0)));
    passing(numbers, has_item(is(1)));

    failing(numbers, has_item(greater_than(5)), "expected [3, 1, 2] to have an item that should be greater than 5");
    failing(numbers, each(greater_than(1)), "expected [3, 1, 2] to have every item that should be greater than 1\n  [1]: expected 1 to be greater than 1");
    failing(s32.[], has_item(is(1)), "expected [] to have an item that should equal 1");
    failing(numbers, not(has_item(is(1))), "expected [3, 1, 2] not to have an item that should equal 1");
    failing(5, each(is(5)), "expected 5 to have every item that should equal 5");
} @test

truthiness_and_pointers :: () {
    number := 4;
    pointer: *s32 = null;

    passing(true, is_true());
    passing(false, is_false());
    passing(pointer, is_null());
    passing(*number, not_null());

    failing(false, is_true(), "expected false to be true");
    failing(true, is_false(), "expected true to be false");
    failing(3, is_true(), "expected 3 to be true");
    failing(pointer, not_null(), "expected null not to be null");
    failing(3, is_null(), "expected 3 to be null");
} @test

type_checks :: () {
    passing(5, of_type(s64));
    passing(cast(u8) 5, of_type(u8));
    passing("x", of_type(string));
    passing(Spot.{ 1, 2 }, of_type(Spot));

    any: Any = 3.5;

    passing(any, of_type(float32));

    failing(5, of_type(string), "expected 5 to be of type string");
    failing(cast(u8) 5, not(of_type(u8)), "expected 5 not to be of type u8");
} @test

composition :: () {
    passing(5, all_of(greater_than(1), less_than(10), not(is(7))));
    passing(5, any_of(is(1), is(5)));
    passing("hello", all_of(starts_with("he"), ends_with("lo"), has_length(5)));
    passing(5, not(any_of(is(1), is(2))));
    passing(5, not(not(is(5))));
    passing(s32.[1, 2], all_of());
    passing(5, all_of(any_of(is(4), is(5)), not(is(6))));

    failing(5, all_of(greater_than(1), less_than(3), is(4)), "expected 5 to be less than 3");
    failing(5, any_of(is(1), is(2)), "expected 5 to satisfy any of: equal 1; equal 2");
    failing(5, any_of(), "expected 5 to satisfy any of: ");
    failing(5, not(any_of(is(1), is(5))), "expected 5 not to equal 5");
    failing(5, not(all_of(is(5), greater_than(1))), "expected 5 not to satisfy all of: equal 5; be greater than 1");
} @test

matchers_read_other_matchers_into_messages :: () {
    failing(5, not(is(5)), "expected 5 not to equal 5");
    failing(5, all_of(not(is(5))), "expected 5 not to equal 5");
    failing(5, any_of(not(is(5)), not(is(5))), "expected 5 to satisfy any of: not equal 5; not equal 5");
} @test

matchers_work_on_any_value :: () {
    values: [..] Any;
    values.allocator = temp;
    array_add(*values, 1, "two", 3.0);

    for values {
        passing(it, not(is_null()));
        passing(it, not(is_empty()));
    }
} @test
