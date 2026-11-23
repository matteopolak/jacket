#import "Basic";
#import "Extensions/Testing";
#import "slugs";

lowercases_and_joins :: () {
    got := slugify("Hello World");
    defer free(got);
    expect_equal(got, "hello-world");
} @test

drops_punctuation :: (t: *Test) {
    got := slugify("Rock & Roll!");
    defer free(got);
    log(t, "slug: %", got);
    expect_equal(got, "rock-roll");
} @test

keeps_digits :: () {
    got := slugify("Top 10 Tips");
    defer free(got);
    assert_that(got, starts_with("top-10"));
} @test
