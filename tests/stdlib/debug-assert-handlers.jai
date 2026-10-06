// Debug's assertion handlers report and return false (meaning "do not break"); trace_assert also
// lists the call stack, which backtrace exposes as one string per frame.
Debug :: #import "Debug"(USE_GRAPHICS = false);

main :: () -> s64 {
    loc := #location();
    if Debug.no_assert(loc, "silent") return 1;
    if Debug.print_assert(loc, "printed only") return 2;
    if Debug.trace_assert(loc, "printed with a trace") return 3;
    frames := Debug.backtrace();
    count := frames.count;
    Debug.free_backtrace(frames);
    if count == 0 return 4;
    return 0;
}
