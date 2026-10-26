// Snapshots: what passes against a stored file, and what fails (and leaves a `.snap.new`).
#import "Basic";
#import "Extensions/Testing";

Snap_Point :: struct {
    x, y: s32;
}

Snap_Shape :: struct {
    name: string;
    corners: [] Snap_Point;
    sides: [3] s32;
    tags: [] string;
}

Snap_Color :: enum {
    RED;
    GREEN;
}

matches_a_text_snapshot :: (t: *Test) {
    expect_snapshot(t, "first line\nsecond line");
} @test

matches_a_struct_snapshot :: (t: *Test) {
    expect_snapshot(t, Snap_Point.{ 1, -2 });
} @test

prints_nested_values_a_line_each :: (t: *Test) {
    shape := Snap_Shape.{ "triangle", Snap_Point.[.{ 0, 0 }, .{ 4, 0 }, .{ 0, 3 }], .[3, 4, 5], string.["a", "b\n"] };
    expect_snapshot(t, shape);
} @test

prints_arrays_and_enums :: (t: *Test) {
    expect_snapshot(t, s64.[1, 2, 3], "numbers");
    expect_snapshot(t, Snap_Color.GREEN, "color");
    expect_snapshot(t, 3.5, "float");
    expect_snapshot(t, true, "bool");
} @test

numbers_snapshots_by_the_order_they_are_taken :: (t: *Test) {
    expect_snapshot(t, "one");
    expect_snapshot(t, "two");
} @test

line_endings_do_not_matter :: (t: *Test) {
    expect_snapshot(t, "carriage\r\nreturns\r\n");
} @test

Snap_Size :: struct {
    name: string;
    size: s64;
}

SNAP_SIZES :: Snap_Size.[.{ "small", 1 }, .{ "big/ger", 100 }, .{ "what?", 7 }];

each_case_has_its_own_file :: (t: *Test, name: string, size: s64) {
    expect_snapshot(t, tprint("% is %", name, size));
} @test @cases(SNAP_SIZES)

subtests_have_their_own_file :: (t: *Test) {
    run(t, "inner", (t: *Test) {
        expect_snapshot(t, "inside");
    });
} @test

differs_from_the_stored_snapshot :: (t: *Test) {
    expect_snapshot(t, "got this");
} @test @should_fail(differs)

has_no_snapshot_yet :: (t: *Test) {
    expect_snapshot(t, "nobody stored this");
} @test @should_fail(yet)
