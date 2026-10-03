// Operational failures must still report when optional Basic assertions are off.
report_time_failure :: (message: string, loc := #caller_location) {
    if context.handling_assertion_failure return;
    context.handling_assertion_failure = true;
    defer context.handling_assertion_failure = false;
    context.assertion_failed(loc, message);
}

time_failure_nan :: (message: string) -> float64 {
    report_time_failure(message);
    return time_quiet_nan();
}

time_quiet_nan :: () -> float64 {
    bits: u64 = 0x7ff8000000000000;
    result: float64;
    memcpy(*result, *bits, size_of(float64));
    return result;
}
