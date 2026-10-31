// Checks `condition`, reports a failure at the caller's line and returns whether it held. The
// other `expect*` checks work the same way; `jacket test` shows the line's source next to the
// message.
//
// ```jai
// expect(1 + 1 == 2);
// expect(1 + 1 == 2, "the arithmetic is off"); // the message replaces the default one
// ```
//
// - condition: what must be true.
// - message: what to report instead of the default.
// - loc: where the check is written, shown when it fails. Leave the default.
expect :: (condition: bool, message := "", loc := #caller_location) -> bool {
    if condition return true;
    report_failure(loc, ifx message then message else "expected true, got false");
    return false;
}

// `got` and `want` are compared by value: numbers of any width, strings, enums, arrays element
// by element (fixed, view and resizable alike) and structs member by member. Strings and arrays
// that differ get a diff.
//
// ```jai
// expect_equal(2 + 2, 4);
// expect_equal("abc", "abc");
// expect_equal(s32.[1, 2, 3], s32.[1, 2, 3]);
// ```
//
// - got: the value under test.
// - want: the value it must equal.
// - loc: where the check is written, shown when it fails. Leave the default.
expect_equal :: (got: $T, want: $U, loc := #caller_location) -> bool {
    if values_equal(got, want) return true;
    report_failure(loc, describe_difference(got, want));
    return false;
}

// `|got - want| <= tolerance`. NaN is never near anything.
//
// ```jai,should_fail
// expect_near(0.5, 0.25, 0.1);
// ```
//
// ```jai,compile_fail
// expect_near("half", 0.5, 0.1);
// ```
//
// - got: the value under test.
// - want: the value it should be close to.
// - tolerance: the largest allowed difference.
// - loc: where the check is written, shown when it fails. Leave the default.
expect_near :: (got: float64, want: float64, tolerance: float64, loc := #caller_location) -> bool {
    difference := got - want;
    if difference < 0 difference = -difference;
    if difference <= tolerance return true;

    report_failure(loc, tprint("expected % within % of %, off by %", got, tolerance, want, difference));

    return false;
}

// Checks that `pointer` is null.
//
// ```jai
// p: *int;
// expect_null(p);
// ```
//
// - pointer: the pointer that must be null.
// - loc: where the check is written, shown when it fails. Leave the default.
expect_null :: (pointer: *$T, loc := #caller_location) -> bool {
    if pointer == null return true;
    report_failure(loc, tprint("expected null, got %", cast(*void) pointer));
    return false;
}

// Checks that `pointer` is not null.
//
// ```jai
// x := 1;
// expect_not_null(*x);
// ```
//
// - pointer: the pointer that must not be null.
// - loc: where the check is written, shown when it fails. Leave the default.
expect_not_null :: (pointer: *$T, loc := #caller_location) -> bool {
    if pointer != null return true;
    report_failure(loc, "expected a pointer, got null");
    return false;
}

// The `require` forms check the same way and, when the check fails, return from the procedure
// they are written in (which must return nothing). Use them when the rest of the test needs the
// check to hold.
//
// ```jai
// items: [..] int;
// array_add(*items, 1);
// require(items.count == 1);
// expect_equal(items[0], 1);
// ```
//
// - condition: what must be true.
// - message: what to report instead of the default.
// - loc: where the check is written, shown when it fails. Leave the default.
require :: (condition: bool, message := "", loc := #caller_location) #expand {
    if !expect(condition, message, loc) `return;
}

// `expect_equal`, then returns from the calling procedure on a failure.
//
// ```jai
// require_equal(1 + 1, 2);
// ```
//
// - got: the value under test.
// - want: the value it must equal.
// - loc: where the check is written, shown when it fails. Leave the default.
require_equal :: (got: $T, want: $U, loc := #caller_location) #expand {
    if !expect_equal(got, want, loc) `return;
}

// `expect_near`, then returns from the calling procedure on a failure.
//
// ```jai
// require_near(0.1 + 0.2, 0.3, 0.0001);
// ```
//
// - got: the value under test.
// - want: the value it should be close to.
// - tolerance: the largest allowed difference.
// - loc: where the check is written, shown when it fails. Leave the default.
require_near :: (got: float64, want: float64, tolerance: float64, loc := #caller_location) #expand {
    if !expect_near(got, want, tolerance, loc) `return;
}

// `expect_not_null`, then returns from the calling procedure on a failure.
//
// ```jai
// x := 1;
// require_not_null(*x);
// ```
//
// - pointer: the pointer that must not be null.
// - loc: where the check is written, shown when it fails. Leave the default.
require_not_null :: (pointer: *$T, loc := #caller_location) #expand {
    if !expect_not_null(pointer, loc) `return;
}
