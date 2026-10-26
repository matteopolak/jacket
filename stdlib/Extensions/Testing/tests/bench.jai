#import "Extensions/Testing";

black_box_gives_the_value_back :: () {
    expect_equal(black_box(41) + 1, 42);
    expect_equal(black_box("text"), "text");

    point := black_box(Bench_Pair.{ 3, 4 });
    expect_equal(point.x + point.y, 7);
} @test

a_bench_sees_its_settings :: (t: *Test) {
    b: Bench;
    b.iterations = 10;
    set_bytes(*b, 64);
    set_items(*b, 2);

    expect_equal(b.bytes_per_iteration, 64);
    expect_equal(b.items_per_iteration, 2);
} @test

// Listed by `jacket test --list`; `jacket bench` measures it.
sums_a_range :: (b: *Bench) {
    set_items(b, 100);

    for 0..b.iterations - 1 {
        total := 0;
        for 0..99 total += black_box(it);
        black_box(total);
    }
} @bench @samples(5)

Bench_Pair :: struct {
    x: s64;
    y: s64;
}
