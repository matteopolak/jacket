// Checks. Each reports at the caller's line and returns whether it held; `jacket test` shows the
// line's source next to the message.

// ```jai
// expect(1 + 1 == 2);
// ```
expect :: (condition: bool, message := "", loc := #caller_location) -> bool {
    if condition return true;
    report_failure(loc, ifx message then message else "expected true, got false");
    return false;
}

// `got` and `want` are compared by value: numbers of any width, strings, enums, arrays element
// by element (fixed, view and resizable alike) and structs member by member. Strings and arrays
// that differ get a diff.
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
expect_near :: (got: float64, want: float64, tolerance: float64, loc := #caller_location) -> bool {
    difference := got - want;
    if difference < 0 difference = -difference;
    if difference <= tolerance return true;

    report_failure(loc, tprint("expected % within % of %, off by %", got, tolerance, want, difference));

    return false;
}

expect_null :: (pointer: *$T, loc := #caller_location) -> bool {
    if pointer == null return true;
    report_failure(loc, tprint("expected null, got %", cast(*void) pointer));
    return false;
}

expect_not_null :: (pointer: *$T, loc := #caller_location) -> bool {
    if pointer != null return true;
    report_failure(loc, "expected a pointer, got null");
    return false;
}

// The `require` forms check the same way and, when the check fails, return from the procedure
// they are written in (which must return nothing).
require :: (condition: bool, message := "", loc := #caller_location) #expand {
    if !expect(condition, message, loc) `return;
}

require_equal :: (got: $T, want: $U, loc := #caller_location) #expand {
    if !expect_equal(got, want, loc) `return;
}

require_near :: (got: float64, want: float64, tolerance: float64, loc := #caller_location) #expand {
    if !expect_near(got, want, tolerance, loc) `return;
}

require_not_null :: (pointer: *$T, loc := #caller_location) #expand {
    if !expect_not_null(pointer, loc) `return;
}
