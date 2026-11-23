#import "Basic";
#import "Extensions/Testing";
#import "slugs";

breaks_on_purpose :: () {
    got := slugify("One Two");
    defer free(got);
    expect_equal(got, "one_two");
} @test

snapshot_of_slug :: (t: *Test) {
    got := slugify("Snapshot Me");
    defer free(got);
    expect_snapshot(t, got);
} @test
