long independent_frame(long x) {
    volatile long result = x + 3;
    return result;
}
