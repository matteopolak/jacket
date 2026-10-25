// The `*Test` procedures: subtests, skips, cleanups and the temporary directory.
#import "Basic";
#import "File";
#import "Extensions/Testing";

Case :: struct {
    input: string;
    length: s64;
}

CASES :: Case.[.{ "", 0 }, .{ "a", 1 }, .{ "abc", 3 }];

subtests_pass :: (t: *Test) {
    passed := run(t, "inner", (t: *Test) {
        expect(true);
    });

    expect(passed);

    for CASES {
        run(t, tprint("length_%", it.length), (t: *Test, c: *Case) {
            expect_equal(c.input.count, c.length);
        }, *it);
    }
} @test

a_failed_subtest_fails_its_test :: (t: *Test) {
    passed := run(t, "inner", (t: *Test) {
        expect(false, "inner_failed");
    });
    expect(!passed);
} @test @should_fail(inner_failed)

skipping :: (t: *Test) {
    skip(t, "not here");
    expect(false, "went on after skip");
} @test

failing_by_hand :: (t: *Test) {
    fail(t, "by_hand");
} @test @should_fail(by_hand)

cleanups_run_last_first :: (t: *Test) {
    order: [..] int;
    order.allocator = temp;
    // A subtest's cleanups run when it ends.
    run(t, "registers", (t: *Test, order: *[..] int) {
        cleanup(t, (data: *void) {
            array_add(cast(*[..] int) data, 1);
        }, order);
        cleanup(t, (data: *void) {
            array_add(cast(*[..] int) data, 2);
        }, order);
        expect_equal(order.count, 0);
    }, *order);
    expect_equal(order, int.[2, 1]);
} @test

each_test_gets_a_temporary_directory :: (t: *Test) {
    dir := temp_dir(t);

    require(dir != "");
    path := tprint("%/file.txt", dir);

    expect(write_entire_file(path, "hello"));
    text, ok := read_entire_file(path,, temp);

    expect(ok);
    expect_equal(text, "hello");
} @test

logging :: (t: *Test) {
    log(t, "value is %", 42);
} @test
