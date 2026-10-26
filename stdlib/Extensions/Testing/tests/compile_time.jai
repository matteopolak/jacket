// Compile-time tests: they run inside the compiler, and a failure there is a failed test.
#import "Basic";
#import "Extensions/Testing";

triangle :: (n: s64) -> s64 {
    sum := 0;
    for 1..n sum += it;

    return sum;
}

arithmetic_holds_in_the_compiler :: () {
    expect_equal(triangle(10), 55);
    expect_equal(#run triangle(4), 10);
} @test @compile_time

ct_takes_the_test :: (t: *Test) {
    log(t, "computed % at compile time", triangle(100));
    expect_equal(triangle(100), 5050);
} @test @compile_time

ct_runs_in_the_compiler :: () {
    expect(at_compile_time(), "this should run in the compiler");
} @test @compile_time

runtime_tests_do_not :: () {
    expect(!at_compile_time(), "this should run in the program");
} @test

ct_skips :: (t: *Test) {
    skip(t, "not now");
    expect(false);
} @test @compile_time

ct_fails_with_its_check :: () {
    expect_equal(triangle(3), 7);
} @test @compile_time @should_fail(want:)

ct_fails_with_its_message :: () {
    expect(false, "compile_time_marker");
} @test @compile_time @should_fail(compile_time_marker)

ct_assertion_fails_the_test_not_the_build :: () {
    assert(triangle(2) == 4, "assertion_marker");
    expect(true);
} @test @compile_time @should_fail(assertion_marker)

ct_xfail :: () {
    expect_equal(1, 2);
} @test @compile_time @xfail(known)

ct_continues_after_a_failed_check :: () {
    expect(false, "first");
    expect(false, "second");
} @test @compile_time @should_fail(second)

ct_cleanups_run :: (t: *Test) {
    cleanup(t, (data: *void) {}, null);
    expect(true);
} @test @compile_time
