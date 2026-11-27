#import "Extensions/Testing";
#import "Basic";

// Busy-waits so the time is certainly spent, whatever the clock's resolution.
spin :: (milliseconds: s64) {
    start := current_time_monotonic();

    while true {
        nanoseconds, ok := to_nanoseconds(current_time_monotonic() - start);
        if !ok || nanoseconds >= milliseconds * 1_000_000 break;
    }
}

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

    for b {
        total := 0;
        for 0..99 total += black_box(it);
        black_box(total);
    }
} @bench @samples(5)

for_b_runs_the_body_once_per_iteration :: () {
    b: Bench;
    b.iterations = 7;

    count := 0;
    last := -1;

    for b {
        count += 1;
        last = it_index;
    }

    expect_equal(count, 7);
    expect_equal(last, 6);
} @test

for_b_supports_break_and_continue :: () {
    b: Bench;
    b.iterations = 10;

    seen: s64;

    for b {
        if it_index == 2 continue;
        if it_index == 5 break;
        seen += 1;
    }

    expect_equal(seen, 4);
} @test

for_b_leaves_setup_out_of_the_time :: () {
    b: Bench;
    b.iterations = 3;

    spin(20);

    for b black_box(it_index);

    expect(b.elapsed < 5_000_000);
    expect(!b.paused_clock);
} @test

pausing_the_clock_leaves_setup_and_teardown_out :: () {
    b: Bench;
    b.iterations = 3;

    // Preemption can land between any two reads of the clock, so no bound on `elapsed` alone holds
    // on a loaded machine. What always holds: every paused span is in `paused_nanoseconds`, and the
    // run's time is the timed span plus the paused one.
    wall_start := current_time_monotonic();

    for b {
        pause_timer(*b);
        spin(5);
        resume_timer(*b);

        black_box(it_index);

        pause_timer(*b);
        spin(5);
        resume_timer(*b);
    }

    wall, ok := to_nanoseconds(current_time_monotonic() - wall_start);

    expect(ok);
    expect(b.paused_nanoseconds >= 6 * 5_000_000);
    expect(b.elapsed + b.paused_nanoseconds <= wall);
    expect(b.paused_clock);
} @test

Bench_Pair :: struct {
    x: s64;
    y: s64;
}
