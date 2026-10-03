compile_time_debug_break :: () #compiler #no_context;
main :: () -> int {
    if #compile_time {
        compile_time_debug_break();
        return 1;
    } else {
        return 42;
    }
}
