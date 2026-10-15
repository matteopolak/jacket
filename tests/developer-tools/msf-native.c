int original_debug_entry(void) {
    volatile int retained = 42;
    return retained;
}
