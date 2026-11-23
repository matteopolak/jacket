#import "Basic";
#import "Extensions/Testing";
#import "slugs";

bad_matcher :: () {
    got := slugify("Top Ten");
    defer free(got);
    assert_that(got, all_of(starts_with("top"), has_length(5)));
} @test

COUNTS :: int.[1, 2, 3];

bad_case :: (n: int) {
    expect_equal(n * n, n + 2);
} @test @cases(COUNTS)

bad_property :: (n: u8) {
    expect(n < 10);
} @test @property
