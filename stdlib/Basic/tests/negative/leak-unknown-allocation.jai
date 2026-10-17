D :: #import "Basic"()(MEMORY_DEBUGGER = true, ENABLE_ASSERT = false, VISUALIZE_MEMORY_DEBUGGER = false);

main :: () {
    value: int;
    D.this_allocation_is_not_a_leak(*value);
}
